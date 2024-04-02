use crate::*;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Default, Clone)]
pub struct CSV {
    pub header: types::Header,
    pub fields: Vec<types::Field>,
    pub(crate) entries: Vec<types::Value>,
    pub(crate) dict: HashMap<types::Field, Vec<types::Value>>
}

impl CSV {
    pub fn get_sorted_fields(&self) -> Vec<types::Field> {
        let mut clone = self.fields.clone();
        clone.sort_by(|x, y| x.cmp(&y));
        clone
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, BcsvError> {
        let mut result = Self::default();
        let mut rdr = csv::Reader::from_path(path)?;
        for header in rdr.headers()?.iter() {
            let split = header.split(':').collect::<Vec<_>>();
            let name = split[0];
            let dt = split[1];
            let mut field = types::Field::default();
            field.datatype = dt.parse()?;
            field.mask = field.get_field_type().mask();
            if !name.starts_with("0x") {
                field.hash = hash::calchash(name);
            } else {
                field.hash = u32::from_str_radix(&name[2..], 16)?;
            }
            result.fields.push(field);
            result.dict.insert(field, vec![]);
        }
        result.header.fieldcount = result.fields.len() as u32;
        for record in rdr.records() {
            let record = record?;
            for i in 0..record.len() {
                let entry = &record[i];
                let field = result.fields[i];
                let mut value = types::Value::new(field);
                match &mut value {
                    types::Value::LONG(l) => {
                        *l = entry.parse()?;
                    },
                    types::Value::STRING(st) => {
                        *st = entry.as_bytes().try_into()?;
                    },
                    types::Value::FLOAT(f) => {
                        *f = entry.parse()?;
                    },
                    types::Value::ULONG(ul) => {
                        *ul = entry.parse()?;
                    },
                    types::Value::SHORT(s) => {
                        *s = entry.parse()?;
                    },
                    types::Value::CHAR(c) => {
                        *c = entry.parse()?;
                    },
                    types::Value::STRINGOFF((_, data)) => {
                        *data = String::from(entry);
                    }
                    types::Value::NULL => {}
                }
                result.entries.push(value.clone());
                if let Some(vec) = result.dict.get_mut(&field) {
                    vec.push(value);
                }
            }
        }
        let mut doff = 0;
        let sorted = result.get_sorted_fields();
        for f in sorted {
            if let Some(og) = result.fields.iter_mut().find(|x| x.hash == f.hash) {
                if let Some(values) = result.dict.remove(og) {
                    if result.header.entrycount == 0 {
                        result.header.entrycount = values.len() as u32;
                    }
                    og.dataoff = doff;
                    doff += og.get_field_type().size();
                    result.dict.insert(*og, values);
                }

            }
        }
        result.header.entrysize = result.fields.iter().map(|x| x.get_field_type().size() as u32).sum();
        result.header.entrydataoff = 16 + (12 * result.header.fieldcount);
        result.calc_stringoffs();
        Ok(result)
    }

    pub (crate) fn calc_stringoffs(&mut self) {
        let mut offs = HashMap::new();
        let mut num = 0;
        for value in &mut self.entries {
            if let types::Value::STRINGOFF((off, str)) = value {
                if !offs.contains_key(str) {
                    *off = num as u32;
                    offs.insert(str.clone(), num as u32);
                    num += str.len() + 1;
                }
            }
        }
        for (f, v) in &mut self.dict {
            if f.datatype != 6 {
                continue;
            }
            for value in v {
                if let types::Value::STRINGOFF((off, str)) = value {
                    *off = offs[str];
                }
            }
        }
    }

    pub fn create_bcsv(self) -> types::BCSV {
        types::BCSV {header: self.header, fields: self.fields, values: self.entries, dictonary: self.dict}
    }
}