use crate::*;

#[derive(Debug, Default, Clone, Copy, BinWrite, PartialEq, Eq)]
#[brw(repr(u8))]
pub enum DataType {
    LONG,
    STRING,
    FLOAT,
    ULONG,
    SHORT,
    CHAR,
    STRINGOFF,
    #[default]
    NULL
}

impl BinRead for DataType {
    type Args<'a> = ();
    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        _: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self>
    {
        let byte = u8::read_ne(reader)?;
        if byte <= 6 {
            Ok(unsafe {std::mem::transmute(byte)})
        } else {
            Ok(DataType::NULL)
        }
    }
}

impl DataType {
    pub const fn size(&self) -> u16 {
        match self {
            DataType::ULONG | DataType::FLOAT | DataType::LONG 
                | DataType::STRINGOFF => 4,
            DataType::SHORT => 2,
            DataType::CHAR => 1,
            DataType::STRING => 32,
            DataType::NULL => 0
        }
    }
    pub const fn order(&self) -> u8 {
        match self {
            DataType::STRING => 0,
            DataType::FLOAT => 1,
            DataType::LONG => 2,
            DataType::ULONG => 3,
            DataType::SHORT => 4,
            DataType::CHAR => 5,
            DataType::STRINGOFF => 6,
            DataType::NULL => 7
        }
    }
    pub const fn mask(&self) -> u32 {
        match self {
            DataType::LONG | DataType::ULONG | DataType::STRINGOFF => u32::MAX,
            DataType::SHORT => 0xFFFF,
            DataType::CHAR => 0xFF,
            _ => 0,
        }
    }
}

impl PartialOrd for DataType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order().partial_cmp(&other.order())
    }
}

impl Ord for DataType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}