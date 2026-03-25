use std::io::Cursor;

use wasm_bindgen::prelude::*;
use js_sys::*;
use crate::types::BCSV;

#[cfg(not(feature = "serde"))]
compile_error!("Serde feature flag must be on for wasm bindings!");

#[wasm_bindgen]
pub fn read_bcsv(data: &[u8]) -> Option<Array> {
    let mut cursor = Cursor::new(data);
    let mut bcsv = BCSV::new();
    if let Ok(_) = bcsv.read(&mut cursor, binrw::Endian::Big) {
        let fields = Array::new();
        let mut f = 0;
        for field in &bcsv.fields {
            let fobject = Object::new();
            Reflect::set(&fobject, &"Hash".into(), &field.hash.into()).unwrap();
            Reflect::set(&fobject, &"Mask".into(), &field.mask.into()).unwrap();
            Reflect::set(&fobject, &"Shift".into(), &field.shift.into()).unwrap();
            let ft = format!("{:?}", field.get_field_type());
            Reflect::set(&fobject, &"DataType".into(), &ft.into()).unwrap();
            let values = Array::new();
            let mut v = 0;
            for value in &bcsv.values[field] {
                let str = value.get_string(true);
                values.set(v, str.into());
                v += 1;
            }
            Reflect::set(&fobject, &"Values".into(), &values).unwrap();
            fields.set(f, fobject.into());
            f += 1;
        }
        return Some(fields);
    }
    None
}