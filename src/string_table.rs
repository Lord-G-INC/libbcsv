use std::collections::HashMap;
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct StringTable {
    table: HashMap<String, u32>,
    off: u32
}

impl StringTable {
    pub fn new() -> Self {
        StringTable { table: HashMap::new(), off: 0 }
    }
    /// Pushes a String to the Table, if it's already in the table, nothing will happen.
    pub fn push<A: AsRef<str>>(&mut self, item: A) -> &mut Self {
        let str = String::from(item.as_ref());
        let len = str.len() as u32 + 1;
        if !self.table.contains_key(&str) {
            self.table.insert(str, self.off);
            self.off += len;
        }
        self
    }
    /// Attempts to find the offset via string
    pub fn find(&self, key: &String) -> Option<&u32> {
        self.table.get(key)
    }
    /// The byte data of the Table.
    pub fn data(&self) -> Vec<u8> {
        self.table.keys().map(|x| x.clone().into_bytes())
        .fold(vec![], |mut v, mut x| {v.append(&mut x); v.push(0); v})
    }
    /// Updates all `Value::STRINGOFF` entries to have a matching offset with the table
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