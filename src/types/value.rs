use crate::*;
use super::datatype::DataType;

#[derive(Debug, Clone, Default)]
pub enum Value {
    LONG(i32),
    STRING([u8; 32]),
    FLOAT(f32),
    ULONG(u32),
    SHORT(u16),
    CHAR(u8),
    STRINGOFF((u32, String)),
    #[default]
    NULL
}

impl Value {
    pub const fn new(datatype: DataType) -> Self {
        match datatype {
            DataType::LONG => Self::LONG(0),
            DataType::STRING => Self::STRING([0u8; 32]),
            DataType::FLOAT => Self::FLOAT(0.0),
            DataType::ULONG => Self::ULONG(0),
            DataType::SHORT => Self::SHORT(0),
            DataType::CHAR => Self::CHAR(0),
            DataType::STRINGOFF => Self::STRINGOFF((0, String::new())),
            DataType::NULL => Self::NULL
        }
    }
    pub fn read<R: BinReaderExt>(&mut self, reader: &mut R, endian: Endian) -> BinResult<()> {
        match self {
            Value::LONG(l) => *l = reader.read_type(endian)?,
            Value::STRING(s) => reader.read_exact(s)?,
            Value::FLOAT(f) => *f = reader.read_type(endian)?,
            Value::ULONG(u) => *u = reader.read_type(endian)?,
            Value::SHORT(sh) => *sh = reader.read_type(endian)?,
            Value::CHAR(c) => *c = reader.read_type(endian)?,
            Value::STRINGOFF((o, _)) => *o = reader.read_type(endian)?,
            _ => {}
        }
        Ok(())
    }
}

impl BinWrite for Value {
    type Args<'a> = ();
    fn write_options<W: std::io::prelude::Write + std::io::prelude::Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()>
    {
        match self {
            Self::LONG(l) => writer.write_type(l, endian)?,
            Self::STRING(s) => writer.write_all(s)?,
            Self::FLOAT(f) => writer.write_type(f, endian)?,
            Self::ULONG(u) => writer.write_type(u, endian)?,
            Self::SHORT(sh) => writer.write_type(sh, endian)?,
            Self::CHAR(c) => writer.write_ne(c)?,
            Self::STRINGOFF((o, _)) => writer.write_type(o, endian)?,
            _ => {}
        }
        Ok(())
    }
}