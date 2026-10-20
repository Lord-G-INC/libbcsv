use crate::*;

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
pub struct Header {
    pub entry_count: u32,
    pub field_count: u32,
    pub entry_data_off: u32,
    pub entry_size: u32
}

impl Header {
    pub const fn string_table_off(&self) -> u64 {
        self.entry_data_off as u64 +
         (self.entry_size as u64 * self.entry_count as u64)
    }
    pub const fn new() -> Self {
        Self {entry_count: 0, field_count: 0, entry_data_off: 0, entry_size: 0}
    }
}