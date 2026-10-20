use crate::types::*;
use super::*;
use std::collections::{BTreeMap, HashMap};
use std::io::SeekFrom;
use encoding_rs::SHIFT_JIS;

#[derive(Debug, Default, Clone)]
pub struct BCSV {
    pub header: Header,
    pub values: Vec<(Field, Vec<Value>)>,
}

fn sort_by_data_off(items: &[(Field, Vec<Value>)]) -> Vec<Field> {
    let mut vec = items.iter()
    .map(|(f, _)| *f).collect::<Vec<_>>();
    vec.sort_by(|a, b| 
        a.data_off.cmp(&b.data_off));
    vec
}

// Writing and Reading
impl BCSV {
    pub const fn new() -> Self {
        Self {header: Header::new(), values: Vec::new()}
    }
    pub fn read<R: BinReaderExt>(&mut self, reader: &mut R, endian: Endian) -> BinResult<()> {
        let Self {header, values} 
            = self;
        // BCSV has no endian specific magic, so we have to have it
        // manually entered. :\
        *header = reader.read_type(endian)?;
        values.reserve_exact(header.field_count as _);
        for _ in 0..header.field_count {
            let field: Field = reader.read_type(endian)?;
            // This makes it so every value in this vec is the correct variant
            // and the correct length too.
            let vec = vec![Value::new(field.data_type); 
                header.entry_count as _];
            values.push((field, vec));
        }
        // We should be here position wise, but always good to be safe.
        reader.seek(SeekFrom::Start(header.entry_data_off as _))?;
        let sorted = sort_by_data_off(values);
        // This loop is a bit complex so here's a small tldr:
        // Because values are sorted by field, we read one "row" of values,
        // then increment `i` to read the next, rinse and repeat till we
        // reach header.entry_count
        let mut i = 0;
        while i < header.entry_count as usize {
            for f in &sorted {
                let (_, vals) = values.iter_mut()
                .find(|(fi, _)| f == fi).unwrap();
                vals[i].read(reader, endian)?;
            }
            i += 1;
        }
        // Should already be here, but its for saftey.
        let pos = reader.seek(SeekFrom::Start(header.string_table_off()))?;
        for (f, vals) in values {
            // Skip any field/value(s) that isn't a str_off
            if f.data_type != DataType::STRINGOFF {
                continue;
            }
            for val in vals {
                // Saftey: guaranteed to be STRINGOFF by previous if check
                if let Value::STRINGOFF((o, s)) = val {
                    reader.seek(SeekFrom::Current(*o as _))?;
                    let str = binrw::NullString::read_ne(reader)?;
                    // Some bcsvs contain kanji, thus shjis is needed.
                    let (dec, _, _) = SHIFT_JIS.decode(&str.0);
                    *s = dec.into_owned();
                    reader.seek(SeekFrom::Start(pos))?;
                }
            }
        }
        Ok(())
    }
    pub fn write<W: BinWriterExt>(&mut self, writer: &mut W, endian: Endian) -> BinResult<()> {
        self.calculate_offsets();
        let Self {header, values} = &self;
        writer.write_type(header, endian)?;
        for (f, _) in values {
            writer.write_type(f, endian)?;
        }
        writer.seek(SeekFrom::Start(header.entry_data_off as _))?;
        let sorted = sort_by_data_off(values);
        for i in 0..header.entry_count as usize {
            for f in &sorted {
                let (_, vals) = values.iter().find(|(fi, _)| fi == f).unwrap();
                writer.write_type(&vals[i], endian)?;
            }
        }
        let mut pos = writer.seek(SeekFrom::End(0))?;
        for i in 0..header.entry_size as usize {
            for (f, vals) in values {
                if f.data_type != DataType::STRINGOFF {
                    continue;
                }
                if let Value::STRINGOFF((o, s)) = &vals[i] {
                    writer.seek_relative(*o as _)?;
                    let (enc, _, _) = SHIFT_JIS.encode(s);
                    writer.write_all(&enc)?;
                    writer.write_ne(&0u8)?;
                    writer.seek(SeekFrom::Start(pos))?;
                }
            }
        }
        pos = writer.seek(SeekFrom::End(0))?;
        let end = pos.next_multiple_of(32);
        let size = end - pos;
        let buf = vec![0x40; size as _];
        writer.write_all(&buf)?;
        Ok(())
    }
    pub fn to_bytes(&mut self, endian: Endian) -> BinResult<Vec<u8>> {
        let mut cursor = std::io::Cursor::new(vec![]);
        self.write(&mut cursor, endian)?;
        Ok(cursor.into_inner())
    }
}

impl BCSV {
    pub fn convert_to_csv(&self, hashes: Option<BTreeMap<u32, String>>) -> String {
        let hashes = hashes.unwrap_or(BTreeMap::new());
        let Self {header, values,} = self;
        let mut csv_string = String::new();
        let keys = values.iter().map(|(f, _)| f).collect::<Vec<_>>();
        let valss = values.iter().map(|(_, v)| v).collect::<Vec<_>>();
        for i in 0..keys.len() {
            let last = i == keys.len() - 1;
            let field = keys[i];
            let hash = match hashes.get(&field.hash) {
                Some(h) => h.clone(),
                None => format!("0x{:X}", field.hash)
            };
            let f_str = format!("{}:{:X}:{}:{:?}", 
                hash, field.bitmask, field.shift, field.data_type);
            csv_string.push_str(&f_str);
            match last {
                false => csv_string.push(','),
                true => csv_string.push('\n')
            }
        }
        let mut i = 0;
        while i < header.entry_count as usize {
            let full_last = i == header.entry_count as usize - 1;
            for j in 0..valss.len() {
                let last = j == valss.len() - 1;
                let vals = valss[j];
                let val = &vals[i];
                let val_str: String = match val {
                    Value::LONG(l) => l.to_string(),
                    Value::STRING(s) => 
                        String::from_utf8_lossy(s).to_string(),
                    Value::FLOAT(f) => f.to_string(),
                    Value::SHORT(sh) => sh.to_string(),
                    Value::CHAR(c) => c.to_string(),
                    Value::STRINGOFF((_, str)) => str.clone(),
                    _ => String::from(" ")
                };
                csv_string.push_str(&val_str);
                match last {
                    true => match full_last {
                        true => {},
                        false => csv_string.push('\n')
                    },
                    false => csv_string.push(',')
                }
                continue;
            }
            i += 1;
        }
        csv_string
    }
    pub fn from_csv(text: &str) -> Self {
        let mut result = Self::new();
        let Self {header, values, } = &mut result; 
        let lines = text.split('\n')
        .map(|x| x.split(',').collect::<Vec<_>>())
        .collect::<Vec<_>>();
        let fields = &lines[0];
        header.field_count = fields.len() as _;
        header.entry_count = (lines.len() - 1) as _;
        values.reserve_exact(header.field_count as _);
        for f in fields {
            let f_split = f.split(':').collect::<Vec<_>>();
            let name = f_split[0];
            let bitmask = u32::from_str_radix(f_split[1], 16).unwrap_or_default();
            let shift = f_split[2].parse::<u8>().unwrap_or_default();
            let dt = f_split[3];
            let hash = if name.starts_with("0x") {
                u32::from_str_radix(&name[2..], 16).unwrap_or_default()
            } else {
                crate::calc_hash(name)
            };
            let data_type = match dt {
                "LONG" => DataType::LONG,
                "STRING" => DataType::STRING,
                "FLOAT" => DataType::FLOAT,
                "ULONG" => DataType::ULONG,
                "SHORT" => DataType::SHORT,
                "CHAR" => DataType::CHAR,
                "STRINGOFF" => DataType::STRINGOFF,
                _ => DataType::NULL
            };
            let field = Field {hash, bitmask, data_off: 0, 
                shift, data_type};
            header.entry_size += data_type.size() as u32;
            let vec = vec![Value::new(data_type); header.entry_count as _];
            values.push((field, vec));
        }
        header.entry_data_off = 16 + (12 * header.field_count);
        header.entry_size = header.entry_size.next_multiple_of(4);
        for i in 0..header.entry_count as usize {
            let line = &lines[1 + i];
            for j in 0..values.len() {
                let (_, vals) = &mut values[j];
                let v = line[j];
                match &mut vals[i] {
                    Value::LONG(l) => *l = v.parse().unwrap_or_default(),
                    Value::STRING(s) => *s = v.as_bytes().try_into().unwrap_or_default(),
                    Value::FLOAT(f) => *f = v.parse().unwrap_or_default(),
                    Value::ULONG(u) => *u = v.parse().unwrap_or_default(),
                    Value::SHORT(sh) => *sh = v.parse().unwrap_or_default(),
                    Value::CHAR(c) => *c = v.parse().unwrap_or_default(),
                    Value::STRINGOFF((_, st)) => *st = String::from(v),
                    _ => {}
                }
            }
        }
        result.calculate_offsets();
        result
    }
}

impl BCSV {
    fn calculate_offsets(&mut self) {
        let Self {values, ..} = self;
        let mut fields = values.iter().map(|(f, _)| *f).collect::<Vec<_>>();
        fields.sort_by(|a, b| a.data_type.cmp(&b.data_type));
        let mut d_off = 0;
        for f in fields {
            let (og, _) = values.iter_mut().find(|(x, _)| *x == f).unwrap();
            og.data_off = d_off;
            d_off += f.data_type.size();
        }
        self.calculate_str_offs();
    }
    fn calculate_str_offs(&mut self) {
        let mut table = HashMap::<String, u32>::new();
        let entry_count = self.header.entry_count as usize;
        let mut offset = 0u32;
        for i in 0..entry_count {
            for (field, vals) in &mut self.values {
                if field.data_type != DataType::STRINGOFF {
                    continue;
                }
                match &mut vals[i] {
                    Value::STRINGOFF((o, s)) => {
                        if let Some(off) = table.get(s) {
                            *o = *off;
                        } else {
                            table.insert(s.clone(), offset);
                            *o = offset;
                            offset += s.len() as u32 + 1;
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}

