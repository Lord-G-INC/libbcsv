use crate::*;
use crate::types::*;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Default, Clone)]
/// A CSV file. Used to convert from CSV to BCSV.
pub struct CSV {
    /// The BCSV header calculated from the CSV.
    pub header: Header,
    /// The fields calculated from the CSV.
    pub fields: Vec<Field>,
    pub(crate) entries: Vec<Value>,
    pub(crate) dict: HashMap<Field, Vec<Value>>
}

impl CSV {
    /// Produces a [`Vec`] of Fields Sorted by their [`FieldType`]'s order.
    /// Refer to [`FieldType::order`] for more.
    pub fn get_sorted_fields(&self) -> Vec<Field> {
        let mut fields = self.fields.clone();
        fields.sort();
        fields
    }
    /// Parses a [`CSV`] from the path, using a delimeter to split the text.
    /// Empty values will result in [`Default::default`] being used.
    #[cfg(not(feature = "serde"))]
    pub fn from_path<P: AsRef<Path>>(path: P, delim: char) -> Result<BCSV, BcsvError> {
        let mut result = Self::default();
        let text = std::fs::read_to_string(path)?.replace('\r', "");
        let lines = text.split('\n').collect::<Vec<_>>();
        for i in 0..lines.len() {
            let line = lines[i];
            let info = line.split(delim).collect::<Vec<_>>();
            if i == 0 {
                for j in 0..info.len() {
                    let split = info[j].split(':').collect::<Vec<_>>();
                    let name = split[0];
                    let dt = split[1];
                    let mut field = Field::default();
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
            } else {
                for j in 0..info.len() {
                    let v = info[j];
                    let field = result.fields[j];
                    let mut value = Value::new(field);
                    match &mut value {
                        Value::LONG(l) => {
                            *l = v.parse().unwrap_or_default();
                        },
                        Value::STRING(st) => {
                            *st = v.as_bytes().try_into().unwrap_or_default();
                        },
                        Value::FLOAT(f) => {
                            *f = v.parse().unwrap_or_default();
                        },
                        Value::ULONG(ul) => {
                            *ul = v.parse().unwrap_or_default();
                        },
                        Value::SHORT(s) => {
                            *s = v.parse().unwrap_or_default();
                        },
                        Value::CHAR(c) => {
                            *c = v.parse().unwrap_or_default();
                        },
                        Value::STRINGOFF((_, data)) => {
                            *data = String::from(v);
                        },
                        Value::NULL => {}
                    }
                    result.entries.push(value.clone());
                    if let Some(vec) = result.dict.get_mut(&field) {
                        vec.push(value);
                    }
                }
            }
        }
        result.header.fieldcount = result.fields.len() as _;
        let mut doff = 0;
        let sorted = result.get_sorted_fields();
        for f in sorted {
            if let Some(og) = result.fields.iter_mut().find(|x| x.hash == f.hash) {
                if let Some(values) = result.dict.remove(og) {
                    if result.header.entrycount == 0 {
                        result.header.entrycount = values.len() as _;
                    }
                    og.dataoff = doff;
                    doff += og.get_field_type().size();
                    result.dict.insert(*og, values);
                }
            }
        }
        result.header.entrysize = doff as _;
        result.header.entrydataoff = 16 + (12 * result.header.fieldcount);
        let mut table = string_table::StringTable::new();
        table.update_offs(&mut result.entries);
        for (_, vals) in &mut result.dict {
            table.update_offs(vals);
        }
        Ok(result.create_bcsv())
    }
    #[cfg(feature = "serde")]
    /// Parses a [`CSV`] from the path, using a delimeter to split the text.
    /// Empty values will result in [`Default::default`] being used.
    pub fn from_path<P: AsRef<Path>>(path: P, delim: char) -> Result<BCSV, BcsvError> {
        let text = std::fs::read_to_string(path)?;
        Ok(BCSV::from_csv_serde(text, delim)?)
    }
    /// Creates a BCSV using the internal info stored
    pub fn create_bcsv(self) -> BCSV {
        BCSV {header: self.header, fields: self.fields, values: self.entries, dictonary: self.dict, ..Default::default()}
    }
}