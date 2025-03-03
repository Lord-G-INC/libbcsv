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
        let info = self.get_string(true);
        info.serialize(serializer)
    }
}

impl Serialize for BCSV {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer {
        let mut map = serializer.serialize_map(Some(self.dictonary.len()))?;
        for (k, value) in &self.dictonary {
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
            bcsv.fields.push(field);
            let mut vec = Vec::with_capacity(vaules.len());
            for v in vaules {
                let mut value = Value::new(field);
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
            bcsv.dictonary.insert(field, vec);
        }
        bcsv.header.fieldcount = bcsv.fields.len() as _;
        let mut doff = 0;
        let sorted = bcsv.sort_fields();
        for f in sorted {
            if let Some(og) = bcsv.fields.iter_mut().find(|x| x.hash == f.hash) {
                if let Some(values) = bcsv.dictonary.remove(og) {
                    if bcsv.header.entrycount == 0 {
                        bcsv.header.entrycount = values.len() as _;
                    }
                    og.dataoff = doff;
                    doff += og.get_field_type().size();
                    bcsv.dictonary.insert(*og, values);
                }
            }
        }
        bcsv.header.entrysize = doff as _;
        bcsv.header.entrydataoff = 16 + (12 * bcsv.header.fieldcount);
        let mut table = string_table::StringTable::new();
        table.update_offs(&mut bcsv.values);
        for (_, vals) in &mut bcsv.dictonary {
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
        let mut i = 0;
        for _ in 0..self.header.entrycount {
            for _ in 0..self.fields.len() {
                writer.write_field(self.values[i].get_string(signed))?;
                i += 1;
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
            bcsv.dictonary.insert(field, vec![]);
        }
        let items = reader.records().collect::<Vec<_>>();
        for i in 0..items.len() {
            let record = &items[i];
            match record.as_ref() {
                Ok(record) => {
                    for j in 0..bcsv.fields.len() {
                        let item = &record[j];
                        let field = &bcsv.fields[j];
                        let mut value = Value::new(*field);
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
                        bcsv.values.push(value.clone());
                        if let Some(entries) = bcsv.dictonary.get_mut(field) {
                            entries.push(value);
                        }
                    }
                },
                Err(e) => return Err(csv::Error::custom(e))
            }
        }
        bcsv.header.fieldcount = bcsv.fields.len() as _;
        let mut doff = 0;
        let sorted = bcsv.sort_fields();
        for f in sorted {
            if let Some(og) = bcsv.fields.iter_mut().find(|x| x.hash == f.hash) {
                if let Some(values) = bcsv.dictonary.remove(og) {
                    if bcsv.header.entrycount == 0 {
                        bcsv.header.entrycount = values.len() as _;
                    }
                    og.dataoff = doff;
                    doff += og.get_field_type().size();
                    bcsv.dictonary.insert(*og, values);
                }
            }
        }
        bcsv.header.entrysize = doff as _;
        bcsv.header.entrydataoff = 16 + (12 * bcsv.header.fieldcount);
        let mut table = string_table::StringTable::new();
        table.update_offs(&mut bcsv.values);
        for (_, vals) in &mut bcsv.dictonary {
            table.update_offs(vals);
        }
        Ok(bcsv)
    }
}