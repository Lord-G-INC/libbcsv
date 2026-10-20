use crate::*;
use super::datatype::DataType;

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite, PartialEq, Eq)]
pub struct Field {
    pub hash: u32,
    pub bitmask: u32,
    pub data_off: u16,
    pub shift: u8,
    pub data_type: DataType
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.data_type.partial_cmp(&other.data_type)
    }
}

impl Ord for Field {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data_type.cmp(&other.data_type)
    }
}