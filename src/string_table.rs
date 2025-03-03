use std::collections::HashMap;
use binrw::NullString;

use crate::*;

#[derive(Clone, Debug, Default)]
/// A string table containing unique, null terminated strings.
pub struct StringTable {
    table: HashMap<String, u32>,
    off: u32
}

impl StringTable {
    /// Creates a brand new table. Does not allocate until one item is pushed.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        StringTable { table: HashMap::new(), off: 0 }
    }
    /// Pushes a string to the Table, if it's already in the table, nothing will happen.
    #[inline]
    pub fn push<A: AsRef<str>>(&mut self, item: A) -> &mut Self {
        let str = String::from(item.as_ref());
        let len = str.len() as u32 + 1;
        if !self.table.contains_key(&str) {
            self.table.insert(str, self.off);
            self.off += len;
        }
        self
    }
    /// Attempts to find the offset via string.
    #[inline]
    pub fn find(&self, key: &String) -> Option<&u32> {
        self.table.get(key)
    }
    /// The byte data of the Table.
    pub fn data(&self) -> Vec<NullString> {
        self.table.keys().map(|x| x.clone().into())
        .fold(vec![], |mut v, x| {v.push(x); v})
    }
    /// Updates all [`types::Value::STRINGOFF`] entries to have a matching offset with the table.
    #[inline]
    pub fn update_offs(&mut self, entries: &mut Vec<types::Value>) {
        for entry in entries {
            if let types::Value::STRINGOFF((off, str)) = entry {
                self.push(&str);
                if let Some(o) = self.find(str) {
                    *off = *o;
                }
            }
        }
    }
}