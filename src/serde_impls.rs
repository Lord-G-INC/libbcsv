//! Note: [`Serialize`] and [`Deserialize`] is only implemented on a few types.
//! [`Header`] will be automatically calculated when Deserialized.

use std::collections::HashMap;
use crate::*;
use crate::types::*;
use serde::*;
use serde::ser::*;
use serde::de::*;
use serde::ser::Error;

impl Serialize for FieldType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer {
        serializer.serialize_str(&format!("{:?}", self))
    }
}

struct FieldTypeVisitor;

impl<'de> Visitor<'de> for FieldTypeVisitor {
    type Value = FieldType;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "either LONG, STRING, FLOAT, ULONG, SHORT, CHAR, STRINGOFF")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error, {
        match v {
            "LONG" => Ok(FieldType::LONG),
            "STRING" => Ok(FieldType::STRING),
            "FLOAT" => Ok(FieldType::FLOAT),
            "ULONG" => Ok(FieldType::ULONG),
            "SHORT" => Ok(FieldType::SHORT),
            "CHAR" => Ok(FieldType::CHAR),
            "STRINGOFF" => Ok(FieldType::STRINGOFF),
            _ => Ok(FieldType::NULL)
        }
    }
}

impl<'de> Deserialize<'de> for FieldType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de> {
        deserializer.deserialize_str(FieldTypeVisitor)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer {
        match self {
            Value::LONG(l) => serializer.serialize_i32(*l),
            Value::STRING(st) => serializer.serialize_str(&String::from(String::from_utf8_lossy(st))),
            Value::FLOAT(f) => serializer.serialize_f32(*f),
            Value::ULONG(ul) => serializer.serialize_u32(*ul),
            Value::SHORT(sh) => serializer.serialize_i16(*sh),
            Value::CHAR(ch) => serializer.serialize_i8(*ch),
            Value::STRINGOFF((_, s)) => serializer.serialize_str(s),
            _ => serializer.serialize_str("None")
        }
    }
}

impl Serialize for BCSV {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer {
        let mut map = serializer.serialize_map(Some(self.values.len()))?;
        for (k, value) in &self.values {
            let key = format!("{}:{}:{}:{:?}", k.get_name(&self.hash_table), k.mask, k.shift,
                k.get_field_type());
            map.serialize_entry(&key, value)?;
        }
        map.end()
    }
}

#[inline]
fn str_to_field_type(str: &str) -> FieldType {
    match str {
        "LONG" => FieldType::LONG,
        "STRING" => FieldType::STRING,
        "FLOAT" => FieldType::FLOAT,
        "ULONG" => FieldType::ULONG,
        "SHORT" => FieldType::SHORT,
        "CHAR" => FieldType::CHAR,
        "STRINGOFF" => FieldType::STRINGOFF,
        _ => FieldType::NULL
    }
}

impl<'de> Deserialize<'de> for BCSV {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de> {
        let mut bcsv = BCSV::new();
        let items: HashMap<String, Vec<String>> = HashMap::deserialize(deserializer)?;
        for (k, vaules) in items {
            let split = k.split(':').collect::<Vec<_>>();
            let name = split[0];
            let hash;
            if !name.starts_with("0x") {
                hash = hash::calchash(name);
            } else {
                hash = u32::from_str_radix(&name[2..], 16).unwrap_or_default()
            }
            let mask: u32 = split[1].parse().unwrap_or_default();
            let shift: u8 = split[2].parse().unwrap_or_default();
            let datatype = str_to_field_type(split[3]) as u8;
            let field = Field { hash, mask, dataoff: 0, shift, datatype };
            let mut vec = Vec::with_capacity(vaules.len());
            for v in vaules {
                let mut value = Value::new(field.get_field_type());
                match &mut value {
                    Value::LONG(l) => *l = v.parse().unwrap_or_default(),
                    Value::STRING(st) => *st = v.as_bytes().try_into().unwrap_or_default(),
                    Value::FLOAT(f) => *f = v.parse().unwrap_or_default(),
                    Value::ULONG(ul) => *ul = v.parse().unwrap_or_default(),
                    Value::SHORT(s) => *s = v.parse().unwrap_or_default(),
                    Value::CHAR(c) => *c = v.parse().unwrap_or_default(),
                    Value::STRINGOFF((_, data)) => *data = v.clone(),
                    _ => {}
                }
                vec.push(value);
            }
            bcsv.fields.push(field);
            bcsv.values.insert(field, vec);
        }
        bcsv.header.fieldcount = bcsv.values.len() as _;
        let mut doff = 0;
        let sorted = bcsv.sort_fields();
        for f in sorted {
            if let Some(og) = bcsv.fields.iter_mut().find(|x| x.hash == f.hash) {
                if let Some(values) = bcsv.values.remove(og) {
                    if bcsv.header.entrycount == 0 {
                        bcsv.header.entrycount = values.len() as _;
                    }
                    og.dataoff = doff;
                    doff += og.get_field_type().size();
                    bcsv.values.insert(*og, values);
                }
            }
        }
        bcsv.header.entrysize = doff as _;
        bcsv.header.entrydataoff = 16 + (12 * bcsv.header.fieldcount);
        let mut table = string_table::StringTable::new();
        for (_, vals) in &mut bcsv.values {
            table.update_offs(vals);
        }
        Ok(bcsv)
    }
}

#[inline]
fn format_field(x: Field, bcsv: &BCSV) -> String {
    format!("{}:{}:{}:{:?}", x.get_name(&bcsv.hash_table), x.mask, x.shift, x.get_field_type())
}

impl BCSV {
    pub fn to_csv_serde(&self, signed: bool, delim: char) -> Result<String, csv::Error> {
        let mut writer = csv::WriterBuilder::new()
        .delimiter(delim as u8)
        .from_writer(vec![]);
        let fields = self.fields.iter()
        .map(|x| format_field(*x, self)).collect::<Vec<_>>();
        writer.write_record(fields)?;
        for i in 0..self.header.entrycount {
            for j in 0..self.fields.len() {
                let f = self.fields[j];
                let vals = &self.values[&f];
                let value = &vals[i as usize];
                writer.write_field(value.get_string(signed))?;
            }
            writer.write_record(None::<&[u8]>)?;
        }
        writer.flush()?;
        let str = String::from_utf8(writer.into_inner().unwrap_or_default())
        .unwrap_or_default();
        Ok(str)
    }
    pub fn from_csv_serde<A: AsRef<[u8]>>(csv: A, delim: char) -> Result<Self, csv::Error> {
        let mut bcsv = BCSV::new();
        let mut reader = csv::ReaderBuilder::new().delimiter(delim as u8)
        .from_reader(csv.as_ref());
        let header = reader.headers()?;
        for record in header {
            let split = record.split(':').collect::<Vec<_>>();
            let name = split[0];
            let hash;
            if !name.starts_with("0x") {
                hash = hash::calchash(name);
            } else {
                hash = u32::from_str_radix(&name[2..], 16).unwrap_or_default()
            }
            let mask: u32 = split[1].parse().unwrap_or_default();
            let shift: u8 = split[2].parse().unwrap_or_default();
            let datatype = str_to_field_type(split[3]) as u8;
            let field = Field { hash, mask, dataoff: 0, shift, datatype };
            bcsv.fields.push(field);
            bcsv.values.insert(field, vec![]);
        }
        let items = reader.records().collect::<Vec<_>>();
        for i in 0..items.len() {
            let record = &items[i];
            match record.as_ref() {
                Ok(record) => {
                    let mut j = 0;
                    for (field, values) in &mut bcsv.values {
                        let item = &record[j];
                        let mut value = Value::new(field.get_field_type());
                        match &mut value {
                            Value::LONG(l) => *l = item.parse().unwrap_or_default(),
                            Value::STRING(st) => *st = item.as_bytes().try_into().unwrap_or_default(),
                            Value::FLOAT(f) => *f = item.parse().unwrap_or_default(),
                            Value::ULONG(ul) => *ul = item.parse().unwrap_or_default(),
                            Value::SHORT(sh) => *sh = item.parse().unwrap_or_default(),
                            Value::CHAR(c) => *c = item.parse().unwrap_or_default(),
                            Value::STRINGOFF((_, s)) => *s = item.into(),
                            _ => {}
                        }
                        values.push(value);
                        j += 1;
                    }
                },
                Err(e) => return Err(csv::Error::custom(e))
            }
        }
        bcsv.header.fieldcount = bcsv.values.len() as _;
        let mut doff = 0;
        let sorted = bcsv.sort_fields();
        for f in sorted {
            if let Some(og) = bcsv.fields.iter_mut().find(|x| x.hash == f.hash) {
                if let Some(values) = bcsv.values.remove(og) {
                    if bcsv.header.entrycount == 0 {
                        bcsv.header.entrycount = values.len() as _;
                    }
                    og.dataoff = doff;
                    doff += og.get_field_type().size();
                    bcsv.values.insert(*og, values);
                }
            }
        }
        bcsv.header.entrysize = doff as _;
        bcsv.header.entrydataoff = 16 + (12 * bcsv.header.fieldcount);
        let mut table = string_table::StringTable::new();
        for (_, vals) in &mut bcsv.values {
            table.update_offs(vals);
        }
        Ok(bcsv)
    }
}