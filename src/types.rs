use std::{collections::HashMap, io::{Cursor, Read, Seek, SeekFrom, Write}};

use crate::*;
use encoding_rs::SHIFT_JIS;

#[derive(Clone, Copy, Debug, Default, BinRead, BinWrite)]
pub struct Header {
    pub entrycount: u32,
    pub fieldcount: u32,
    pub entrydataoff: u32,
    pub entrysize: u32
}

impl Header {
    /// The offset of the string table.
    pub const fn stringoffset(&self) -> u64 {
        (self.entrydataoff + (self.entrycount * self.entrysize)) as u64
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum FieldType {
    LONG,
    STRING,
    FLOAT,
    ULONG,
    SHORT,
    CHAR,
    STRINGOFF,
    NULL
}

impl From<u8> for FieldType {
    fn from(value: u8) -> Self {
        if value > 6 {
            Self::NULL
        } else {
            unsafe {std::mem::transmute(value)}
        }
    }
}

impl FieldType {
    /// The byte size of the type, STRINGOFF is 4 because it writes a offset to the Values Section.
    pub const fn size(&self) -> u16 {
        match self {
            Self::NULL => 0,
            Self::LONG | Self::ULONG | Self::FLOAT | Self::STRINGOFF => 4,
            Self::SHORT => 2,
            Self::CHAR => 1,
            Self::STRING => 32
        }
    }
    /// The bit mask to use for calculations, only works on the integral types.
    pub const fn mask(&self) -> u32 {
        match self {
            Self::NULL | Self::STRING | Self::FLOAT => 0,
            Self::LONG | Self::ULONG | Self::STRINGOFF => u32::MAX,
            Self::SHORT => 0xFFFF,
            Self::CHAR => 0xFF
        }
    }
    /// The order of which the Field should be written, used in the Ord/PartialOrd impl.
    pub const fn order(&self) -> i32 {
        match self {
            Self::NULL => -1,
            Self::LONG => 2,
            Self::STRING => 0,
            Self::FLOAT => 1,
            Self::ULONG => 3,
            Self::SHORT => 4,
            Self::CHAR => 5,
            Self::STRINGOFF => 6
        }
    }
}

#[derive(Clone, Copy, Debug, Default, BinRead, BinWrite, Hash, PartialEq, Eq)]
pub struct Field {
    pub hash: u32,
    pub mask: u32,
    pub dataoff: u16,
    pub shift: u8,
    pub datatype: u8
}

impl Field {
    /// Gets the FieldType for this Field
    pub fn get_field_type(&self) -> FieldType {
        self.datatype.into()
    }
    /// Attempts to get the name of this field using the Hash Table, will return a formated hex string if not present.
    pub fn get_name(&self, hashes: &HashMap<u32, String>) -> String {
        if let Some(val) = hashes.get(&self.hash) {
            val.clone()
        } else {
            format!("0x{:X}", self.hash)
        }
    }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_field_type().order().partial_cmp(&other.get_field_type().order())
    }
}

impl Ord for Field {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_field_type().order().cmp(&other.get_field_type().order())
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    LONG(i32),
    STRING([u8; 32]),
    FLOAT(f32),
    ULONG(u32),
    SHORT(i16),
    CHAR(i8),
    STRINGOFF((u32, String)),
    NULL
}

impl Value {
    /// Creates a new Value based off the FieldType of the Field.
    pub fn new(field: Field) -> Self {
        match field.get_field_type() {
            FieldType::LONG => Self::LONG(0),
            FieldType::STRING => Self::STRING([0u8; 32]),
            FieldType::FLOAT => Self::FLOAT(0.0),
            FieldType::ULONG => Self::ULONG(0),
            FieldType::SHORT => Self::SHORT(0),
            FieldType::CHAR => Self::CHAR(0),
            FieldType::STRINGOFF => Self::STRINGOFF(Default::default()),
            FieldType::NULL => Self::NULL
        }
    }

    pub(crate) fn recalc(&mut self, field: Field) {
        match self {
            Self::LONG(lng) => {
                *lng &= FieldType::LONG.mask() as i32;
                *lng >>= field.shift as i32;
            },
            Self::ULONG(ulng) => {
                *ulng &= FieldType::ULONG.mask();
                *ulng >>= field.shift as u32;
            },
            Self::SHORT(ust) => {
                *ust &= FieldType::SHORT.mask() as i16;
                *ust >>= field.shift as u16;
            },
            Self::CHAR(b) => {
                *b &= FieldType::CHAR.mask() as i8;
                *b >>= field.shift;
            }
            _ => {}
        }
    }
    /// Reads the value based off row, header, and field info.
    pub fn read<R: Read + Seek>(&mut self, reader: &mut R, endian: Endian,
        row: i64, header: Header, field: Field) -> BinResult<()> {
        let oldpos = reader.seek(SeekFrom::Current(0))?;
        let off = row * header.entrysize as i64 + field.dataoff as i64;
        reader.seek(SeekFrom::Current(off))?;
        match self {
            Self::LONG(l) => {
                *l = reader.read_type(endian)?;
            },
            Self::STRING(s) => {
                *s = reader.read_ne()?;
            },
            Self::FLOAT(f) => {
                *f = reader.read_type(endian)?;
            },
            Self::ULONG(ul) => {
                *ul = reader.read_type(endian)?;
            },
            Self::SHORT(sh) => {
                *sh = reader.read_type(endian)?
            },
            Self::CHAR(c) => {
                *c = reader.read_ne()?;
            },
            Self::STRINGOFF((o, _)) => {
                *o = reader.read_type(endian)?;
            }
            Self::NULL => {},
        }
        reader.seek(SeekFrom::Start(oldpos))?;
        self.recalc(field);
        self.calc_stringoff(reader, header)?;
        Ok(())
    }

    pub(crate) fn calc_stringoff<R: Read + Seek>(&mut self, reader: &mut R, header: Header) -> BinResult<()> {
        if let Self::STRINGOFF((n, str)) = self {
            let stringoff = header.stringoffset();
            let oldpos = reader.seek(SeekFrom::Current(0))?;
            reader.seek(SeekFrom::Start(stringoff))?;
            reader.seek(SeekFrom::Current(*n as i64))?;
            let mut bytes = vec![0u8; 0];
            let mut byte: u8 = reader.read_ne()?;
            while byte != 0 {
                bytes.push(byte);
                byte = reader.read_ne()?;
            }
            let (dec, _, _) = SHIFT_JIS.decode(&bytes);
            *str = dec.into();
            reader.seek(SeekFrom::Start(oldpos))?;
        }
        Ok(())
    }
    /// Gets a formatted string based off the Value's inner data.
    pub fn get_string(&self, signed: bool) -> String {
        match self {
            Self::LONG(l) => {
                format!("{}", l)
            },
            Self::STRING(s) => {
                String::from(String::from_utf8_lossy(s))
            },
            Self::FLOAT(f) => {
                format!("{}", f)
            },
            Self::ULONG(ul) => {
                format!("{}", ul)
            },
            Self::SHORT(sh) => {    
                match signed {
                    true => format!("{}", sh),
                    false => format!("{}", *sh as u16)
                }
            },
            Self::CHAR(c) => {
                match signed {
                    true => format!("{}", c),
                    false => format!("{}", *c as u8)
                }
            },
            Self::STRINGOFF((_, st)) => {
                st.clone()
            }
            Self::NULL => String::from("NULL")
        }
    }
    /// Writes to the writer based off the Value's inner data.
    pub fn write<W: Write + Seek>(&self, writer: &mut W, endian: Endian) -> BinResult<()> {
        match self {
            Self::LONG(l) => writer.write_type(l, endian),
            Self::STRING(s) => writer.write_ne(s),
            Self::FLOAT(f) => writer.write_type(f, endian),
            Self::ULONG(ul) => writer.write_type(ul, endian),
            Self::SHORT(sh) => writer.write_type(sh, endian),
            Self::CHAR(c) => writer.write_ne(c),
            Self::STRINGOFF((off, _)) => writer.write_type(off, endian),
            Self::NULL => Ok(())
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BCSV {
    pub header: Header,
    pub fields: Vec<Field>,
    pub(crate) values: Vec<Value>,
    pub(crate) dictonary: HashMap<Field, Vec<Value>>
}

impl BCSV {
    pub fn new() -> Self {
        Self::default()
    }
    /// Reads the BCSV info off the reader.
    pub fn read<R: Read + Seek>(&mut self, reader: &mut R, endian: Endian) -> BinResult<()> {
        let Self {header, fields, values, dictonary} = self;
        *header = reader.read_type(endian)?;
        *fields = vec![Field::default(); header.fieldcount as usize];
        for field in fields.iter_mut() {
            *field = reader.read_type(endian)?;
            dictonary.insert(*field, vec![]);
        }
        reader.seek(SeekFrom::Start(header.entrydataoff as u64))?;
        let entrysize = header.entrycount as usize * fields.len();
        let mut v = 0;
        let mut row = 0;
        while v != entrysize {
            if v >= entrysize {
                break;
            }
            for field in fields.iter() {
                let mut value = Value::new(*field);
                value.read(reader, endian, row, *header, *field)?;
                values.push(value.clone());
                if let Some(entries) = dictonary.get_mut(field) {
                    entries.push(value);
                }
                v += 1;
            }
            row += 1;
        }
        Ok(())
    }
    /// Converts all data to readable CSV data.
    pub fn convert_to_csv(&self, hashes: &HashMap<u32, String>, signed: bool, delim: char) -> String {
        let mut result = String::new();
        for i in 0..self.fields.len() {
            let last = i == self.fields.len() - 1;
            let term = match last { true => '\n', false => delim };
            result += &format!("{}:{}{}", self.fields[i].get_name(hashes), self.fields[i].datatype, term);
        }
        let mut v = 0;
        while v < self.values.len() {
            for i in 0..self.fields.len() {
                let last = i == self.fields.len() - 1;
                let term = match last { false => delim, true => '\n' };
                result += &format!("{}{}", self.values[v].get_string(signed), term);
                v += 1;
            }
        }
        result
    }
    /// Converts all data to a Excel Worksheet.
    pub fn convert_to_xlsx<S: AsRef<str>>(&self, name: S, hashes: &HashMap<u32, String>, signed: bool) -> Result<(), BcsvError> {
        let book = xlsxwriter::Workbook::new(name.as_ref())?;
        let mut sheet = book.add_worksheet(None)?;
        for i in 0..self.fields.len() {
            let text = format!("{}:{}", self.fields[i].get_name(hashes), self.fields[i].datatype);
            sheet.write_string(0 as u32, i as u16, &text, None)?;
        }
        for i in 0..self.fields.len() {
            let values = &self.dictonary[&self.fields[i]];
            for j in 0..values.len() {
                sheet.write_string((j + 1) as u32, i as u16, &values[j].get_string(signed), None)?;
            }
        }
        book.close()?;
        Ok(())
    }
    /// Sorts the Fields off their order.
    /// Refer to `FieldType::Order` for more.
    pub fn sort_fields(&self) -> Vec<Field> {
        let mut result = self.fields.clone();
        result.sort();
        result
    }
    /// Writes all data to the writer, this function makes various size/length checks during writing.
    pub fn write<W: Write + Seek>(&self, writer: &mut W, endian: Endian) -> BinResult<()> {
        {
            let Self {header, fields, ..} = self;
            writer.write_type(header, endian)?;
            for field in fields {
                writer.write_type(field, endian)?;
            }
        }
        let sorted = self.sort_fields();
        for i in 0..self.header.entrycount as usize {
            for f in &sorted {
                if let Some(entries) = self.dictonary.get(f) {
                    let val = &entries[i];
                    val.write(writer, endian)?;
                }
            }
        }
        let stringoff = self.header.stringoffset();
        let mut end = writer.seek(SeekFrom::End(0))?;
        if end != stringoff {
           let ioerr = std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof, "End and StrOff don't match");
           return Err(ioerr.into())
        }
        for value in &self.values {
            if let Value::STRINGOFF((off, str)) = value {
                let curoff = writer.seek(SeekFrom::Current(0))?;
                let realoff = *off as i64;
                writer.seek(SeekFrom::Current(realoff))?;
                let (data, _, _) = SHIFT_JIS.encode(str);
                writer.write_all(&data)?;
                writer.write_ne(&0u8)?;
                writer.seek(SeekFrom::Start(curoff))?;
            }
        }
        end = writer.seek(SeekFrom::End(0))?;
        let padded = end + ((end + 31 & !31) - end);
        let dist = padded - end;
        let buffer = vec![0x40u8; dist as usize];
        writer.write_all(&buffer)?;
        Ok(())
    }
    /// Alais to `write` using a `Cursor<Vec<u8>>`.
    pub fn to_bytes(&self, endian: Endian) -> BinResult<Vec<u8>> {
        let mut stream = Cursor::new(vec![]);
        self.write(&mut stream, endian)?;
        Ok(stream.into_inner())
    }
}