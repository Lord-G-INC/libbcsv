use std::{collections::*, io::{Cursor, Read, Seek, SeekFrom, Write}, ops::Index};
use std::collections::hash_map::*;

use crate::*;
use crate::field_holder::FieldHolder;
use encoding_rs::SHIFT_JIS;

#[derive(Clone, Copy, Debug, Default, BinRead, BinWrite)]
/// The header information of a BCSV file. There is no magic to the format.
pub struct Header {
    /// The count of entries owned by a **single** field. Should **always** be the same for all fields.
    pub entrycount: u32,
    /// The total count of fields.
    pub fieldcount: u32,
    /// Absolute offset to the Entry section of the format. Explained further in the Entry docs.
    pub entrydataoff: u32,
    /// The bytesize of a **single** row. Should **always** be the sum of all Fields DataType Size.
    pub entrysize: u32
}

impl Header {
    /// The absolute offset of the string table.
    #[inline]
    pub const fn stringoffset(&self) -> u64 {
        (self.entrydataoff + (self.entrycount * self.entrysize)) as u64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
/// The possible types a Field can be. Anything >= 7 is unknown/null.
pub enum FieldType {
    /// A *signed* 32 bit integer. Equivalent to [`i32`].
    LONG,
    /// A fixed 32 length  string. Unused on the Wii.
    STRING,
    /// A 32 bit floating point number. Equivalent to [`f32`].
    FLOAT,
    /// A *unsigned* 32 bit integer. Equivalent to [`u32`].
    ULONG,
    /// A 16 bit integer. Can be signed/unsigned. Equivalent to [`i16`]/[`u16`]
    SHORT,
    /// A 8 bit integer. Can be signed/unsigned. Equivalent to [`i8`]/[`u8`]
    CHAR,
    /// A 32 bit offset to a string in a table. Is always unsigned.
    STRINGOFF,
    /// A unknown type. This value exists only to denote a invalid type.
    NULL
}

impl From<u8> for FieldType {
    #[inline]
    fn from(value: u8) -> Self {
        if value > 6 {
            Self::NULL
        } else {
            unsafe {std::mem::transmute(value)}
        }
    }
}

impl FieldType {
    /// The byte size of the type, [`FieldType::STRINGOFF`] is 4 because it writes a offset to the Values Section.
    #[inline]
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
    #[inline]
    pub const fn mask(&self) -> u32 {
        match self {
            Self::NULL | Self::STRING | Self::FLOAT => 0,
            Self::LONG | Self::ULONG | Self::STRINGOFF => u32::MAX,
            Self::SHORT => 0xFFFF,
            Self::CHAR => 0xFF
        }
    }
    /// The order of which the Field should be written, used in the Ord/PartialOrd impl.
    #[inline]
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

impl PartialOrd for FieldType {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order().partial_cmp(&other.order())
    }
}

impl Ord for FieldType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}

#[derive(Clone, Copy, Debug, Default, BinRead, BinWrite, Hash, PartialEq, Eq)]
/// A BCSV Field. Contains information regarding the values owned by this Field.
pub struct Field {
    /// The hash name of the field. Calculated by [`hash::calchash`].
    pub hash: u32,
    /// The bitmask for some calculations. Usually [`FieldType::mask`], but can be different.
    pub mask: u32,
    /// Relative offset to the entries owned by this Field.
    pub dataoff: u16,
    /// The bitshift for some calculations. Usually 0, but can be different.
    pub shift: u8,
    /// The [`FieldType`] of this Field.
    pub datatype: u8
}

impl Field {
    /// Gets the [`FieldType`] for this Field
    #[inline]
    pub const fn get_field_type(&self) -> FieldType {
        if self.datatype > 6 {
            FieldType::NULL
        } else {
            unsafe {std::mem::transmute(self.datatype)}
        }
    }
    /// Attempts to get the name of this field using the Hash Table, will return a formated hex string if not present.
    pub fn get_name(&self, hashes: &HashMap<u32, String>) -> String {
        if let Some(val) = hashes.get(&self.hash) {
            val.clone()
        } else {
            format!("0x{:X}", self.hash)
        }
    }
    #[inline]
    pub const fn new_with_type(name: &str, datatype: FieldType) -> Self {
        let hash = hash::calchash(name);
        let mask = datatype.mask();
        Self {hash, mask, dataoff: 0, shift: 0, datatype: datatype as _}
    }
    #[inline]
    pub const fn new(name: &str, mask: u32, shift: u8, datatype: FieldType) -> Self {
        Self {hash: hash::calchash(name), mask, dataoff: 0, shift, datatype: datatype as _}
    }
}

impl PartialOrd for Field {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_field_type().partial_cmp(&other.get_field_type())
    }
}

impl Ord for Field {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_field_type().cmp(&other.get_field_type())
    }
}

#[derive(Clone, Debug)]
/// A BCSV value. Heavily dependant on the [`Field`] that owns this Value.
pub enum Value {
    /// A signed 32 bit integer.
    LONG(i32),
    /// A 32 length string. Not used on the Wii.
    STRING([u8; 32]),
    /// A 32 bit floating point number.
    FLOAT(f32),
    /// A unsigned 32 bit integer.
    ULONG(u32),
    /// A 16 bit integer.
    SHORT(i16),
    /// A 8 bit integer.
    CHAR(i8),
    /// A unsigned 32 bit integer to hold the offset, and the String itself.
    STRINGOFF((u32, String)),
    /// A unknown type. This value only exists to denote invalid entries.
    NULL
}

impl Value {
    /// Creates a new Value based off the FieldType.
    #[inline]
    pub const fn new(fieldtype: FieldType) -> Self {
        match fieldtype {
            FieldType::LONG => Self::LONG(0),
            FieldType::STRING => Self::STRING([0u8; 32]),
            FieldType::FLOAT => Self::FLOAT(0.0),
            FieldType::ULONG => Self::ULONG(0),
            FieldType::SHORT => Self::SHORT(0),
            FieldType::CHAR => Self::CHAR(0),
            FieldType::STRINGOFF => Self::STRINGOFF((0, String::new())),
            FieldType::NULL => Self::NULL
        }
    }
    #[doc(hidden)]
    #[inline]
    pub(crate) const fn recalc(&mut self, field: Field) {
        match self {
            Self::LONG(lng) => {
                *lng &= field.mask as i32;
                *lng >>= field.shift as i32;
            },
            Self::ULONG(ulng) => {
                *ulng &= field.mask;
                *ulng >>= field.shift as u32;
            },
            Self::SHORT(ust) => {
                *ust &= field.mask as i16;
                *ust >>= field.shift as u16;
            },
            Self::CHAR(b) => {
                *b &= field.mask as i8;
                *b >>= field.shift;
            }
            _ => {}
        }
    }
    /// Reads the value based off row, header, and field info.
    pub fn read<R: Read + Seek>(&mut self, reader: &mut R, endian: Endian,
        row: i64, header: Header, field: Field) -> BinResult<()> {
        let oldpos = reader.stream_position()?;
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
    #[doc(hidden)]
    pub(crate) fn calc_stringoff<R: Read + Seek>(&mut self, reader: &mut R, header: Header) -> BinResult<()> {
        if let Self::STRINGOFF((n, str)) = self {
            let stringoff = header.stringoffset();
            let oldpos = reader.seek(SeekFrom::Current(0))?;
            reader.seek(SeekFrom::Start(stringoff))?;
            reader.seek(SeekFrom::Current(*n as i64))?;
            let info = binrw::NullString::read_ne(reader)?;
            let (dec, _, _) = SHIFT_JIS.decode(&info);
            *str = dec.into();
            reader.seek(SeekFrom::Start(oldpos))?;
        }
        Ok(())
    }
    /// Gets a formatted string based off the Value's inner data. Can make signed 16 or 8 bit integers.
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
    #[doc(hidden)]
    #[inline]
    pub(crate) const fn calc_write(&mut self, field: Field) {
        match self {
            Self::LONG(lng) => {
                *lng <<= field.shift as i32;
                *lng &= field.mask as i32;
            },
            Self::ULONG(ulng) => {
                *ulng <<= field.shift;
                *ulng &= field.mask;
            },
            Self::SHORT(sh) => {
                *sh <<= field.shift as i16;
                *sh &= field.mask as i16;
            },
            Self::CHAR(ch) => {
                *ch <<= field.shift;
                *ch &= field.mask as i8;
            }
            _ => {}
        }
    }
    #[inline]
    pub const fn fieldtype(&self) -> FieldType {
        match self {
            Self::LONG(_) => FieldType::LONG,
            Self::CHAR(_) => FieldType::CHAR,
            Self::FLOAT(_) => FieldType::FLOAT,
            Self::SHORT(_) => FieldType::SHORT,
            Self::STRING(_) => FieldType::STRING,
            Self::STRINGOFF(_) => FieldType::STRINGOFF,
            Self::ULONG(_) => FieldType::ULONG,
            _ => FieldType::NULL
        }
    }
}

#[derive(Clone, Debug, Default)]
/// The BCSV file format. The struct most users will end up making use of.
pub struct BCSV {
    /// The header of the file.
    pub header: Header,
    /// The hash table to use, preferablly loaded by [`hash::read_hashes`].
    pub hash_table: HashMap<u32, String>,
    /// The fields of the BCSV.
    pub fields: Vec<Field>,
    /// The fields and values of the BCSV.
    pub values: HashMap<Field, Vec<Value>>
}

impl BCSV {
    /// Returns a default BCSV.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    /// Reads the BCSV info off the reader.
    pub fn read<R: Read + Seek>(&mut self, reader: &mut R, endian: Endian) -> BinResult<()> {
        let Self {header, values, fields, ..} = self;
        *header = reader.read_type(endian)?;
        fields.reserve_exact(header.fieldcount as _);
        values.reserve(header.fieldcount as _);
        for _ in 0..header.fieldcount as usize {
            let field = reader.read_type(endian)?;
            fields.push(field);
            values.insert(field, Vec::with_capacity(header.entrycount as _));
        }
        reader.seek(SeekFrom::Start(header.entrydataoff as u64))?;
        let entrysize = header.entrycount as usize * values.len();
        let mut v = 0;
        let mut row = 0;
        while v != entrysize {
            if v >= entrysize {
                break;
            }
            for (field, values) in values.iter_mut() {
                let mut value = Value::new(field.get_field_type());
                value.read(reader, endian, row, *header, *field)?;
                values.push(value);
                v += 1;
            }
            row += 1;
        }
        Ok(())
    }
    /// Converts all data to readable CSV data.
    #[cfg(not(feature = "serde"))]
    pub fn convert_to_csv(&self, signed: bool, delim: char) -> Result<String, BcsvError> {
        use std::fmt::Write;
        let mut result = String::new();
        let mut i = 0;
        for (field, _) in &self.values {
            let last = i == self.values.len() - 1;
            let term = match last { true => '\n', false => delim };
            write!(&mut result, "{}:{}{}", field.get_name(&self.hash_table), field.datatype, term)?;
        }
        i = 0;
        for (_, values) in &self.values {
            let last = i == values.len() - 1;
            let term = match last { true => '\n', false => delim };
            write!(&mut result, "{}{}", values[i].get_string(signed), term)?;
            i += 1;
            if last { i = 0; }
        }
        Ok(result)
    }
    #[cfg(feature = "serde")]
    /// Converts all data to readable CSV data.
    pub fn convert_to_csv(&self, signed: bool, delim: char) -> Result<String, BcsvError> {
        Ok(self.to_csv_serde(signed, delim)?)
    }
    /// Converts all data to a Excel Worksheet.
    pub fn convert_to_xlsx<S: AsRef<str>>(&self, name: S, signed: bool) -> Result<(), BcsvError> {
        let book = xlsxwriter::Workbook::new(name.as_ref())?;
        let mut sheet = book.add_worksheet(None)?;
        let mut i = 0;
        for field in &self.fields {
            let text = format!("{}:{}", field.get_name(&self.hash_table), field.datatype);
            sheet.write_string(0 as u32, i as u16, &text, None)?;
            i += 1;
        }
        i = 0;
        for (field, _) in &self.values {
            let values = &self.values[field];
            for j in 0..values.len() {
                sheet.write_string((j + 1) as u32, i as u16, &values[j].get_string(signed), None)?;
            }
            i += 1;
        }
        book.close()?;
        Ok(())
    }
    /// Sorts the Fields off their order.
    /// Refer to [`FieldType::order`] for more.
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
                if let Some(entries) = self.values.get(f) {
                    let mut val = entries[i].clone();
                    val.calc_write(*f);
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
        for value in self.values.values().flatten() {
            if let Value::STRINGOFF((off, str)) = value {
                let curoff = writer.stream_position()?;
                let realoff = *off as i64;
                writer.seek(SeekFrom::Current(realoff))?;
                let (data, _, _) = SHIFT_JIS.encode(str);
                let ns = binrw::NullString(data.into());
                writer.write_ne(&ns)?;
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
    /// Alais to [`BCSV::write`] using a [`Cursor<Vec<u8>>`].
    pub fn to_bytes(&self, endian: Endian) -> BinResult<Vec<u8>> {
        let mut stream = Cursor::new(vec![]);
        self.write(&mut stream, endian)?;
        Ok(stream.into_inner())
    }
    /// Creates a new field for the BCSV. Also creates a new entry in the dictonary.
    #[inline]
    pub fn new_field(&mut self, field: Field) {
        let ft = field.get_field_type();
        self.fields.push(field);
        self.values.insert(field, vec![Value::new(ft); self.header.entrycount as _]);
    }
    /// Attempts to find the values owned by this field. Refer to [`HashMap::get_mut`] for more.
    #[inline]
    pub fn get_values(&mut self, field: Field) -> Option<&mut Vec<Value>> {
        self.values.get_mut(&field)
    }
    /// Adds a new value to the BCSV using the field as the info of where it belongs.
    /// If value's datatype doesn't match field's datatype nothing will happen.
    /// This method will also do nothing if it cannot find the field within the dictonary.
    /// Fields should always be added via [`BCSV::new_field`].
    #[inline]
    pub fn add_value(&mut self, field: Field, value: Value) {
        if field.get_field_type() != value.fieldtype() {return;}
        if let Some(values) = self.values.get_mut(&field) {
            values.push(value);
        }
    }
    #[inline]
    pub fn field_holder<'a>(&'a mut self, index: usize) -> FieldHolder<'a> {
        FieldHolder::from_bcsv(self, index)
    }
}

impl IntoIterator for BCSV {
    type IntoIter = IntoIter<Field, Vec<Value>>;
    type Item = (Field, Vec<Value>);
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a> IntoIterator for &'a BCSV {
    type IntoIter = Iter<'a, Field, Vec<Value>>;
    type Item = (&'a Field, &'a Vec<Value>);
    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl<'a> IntoIterator for &'a mut BCSV {
    type IntoIter = IterMut<'a, Field, Vec<Value>>;
    type Item = (&'a Field, &'a mut Vec<Value>);
    fn into_iter(self) -> Self::IntoIter {
        self.values.iter_mut()
    }
}

impl Index<Field> for BCSV {
    type Output = Vec<Value>;
    #[inline]
    fn index(&self, index: Field) -> &Self::Output {
        &self.values.index(&index)
    }
}