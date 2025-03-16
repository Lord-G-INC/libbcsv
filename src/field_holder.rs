use std::ops::{Deref, DerefMut, Drop};

use crate::types::*;

#[derive(Debug)]
pub struct FieldHolder<'a> {
    original: Field,
    index: usize,
    bcsv: &'a mut BCSV
}

impl<'a> FieldHolder<'a> {
    #[inline]
    pub fn from_bcsv(bcsv: &'a mut BCSV, index: usize) -> Self {
        let original = bcsv.fields[index];
        Self {original, index, bcsv}
    }
    #[inline]
    pub fn field(&self) -> Field {
        self.bcsv.fields[self.index]
    }
    #[inline]
    pub fn field_ref(&self) -> &Field {
        self.deref()
    }
    #[inline]
    pub fn field_mut_ref(&mut self) -> &mut Field {
        self.deref_mut()
    }
    #[inline]
    pub fn values(&mut self) -> Option<&mut Vec<Value>> {
        self.bcsv.values.get_mut(&self.field())
    }
    /// Creates a new value based off the held field's datatype.
    #[inline]
    pub fn new_value(&mut self) -> Option<&mut Value> {
        let dt = self.original.get_field_type();
        if let Some(values) = self.values() {
            values.push(Value::new(dt));
            let pos = values.len() - 1;
            Some(&mut values[pos])
        } else {
            None
        }
    }
}

impl<'a> Deref for FieldHolder<'a> {
    type Target = Field;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.bcsv.fields[self.index]
    }
}

impl<'a> DerefMut for FieldHolder<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bcsv.fields[self.index]
    }
}

impl<'a> Drop for FieldHolder<'a> {
    #[inline]
    fn drop(&mut self) {
        let og = self.original;
        let new = self.field();
        if let Some(values) = self.bcsv.values.remove(&og) {
            self.bcsv.values.insert(new, values);
        }
    }
}