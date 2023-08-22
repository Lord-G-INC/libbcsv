use std::{io::Cursor, ops::Deref};
use wasm_bindgen::prelude::*;
use js_sys::*;
use crate::*;

#[wasm_bindgen]
pub fn bcsv_to_csv(path: &str, data: &Uint8Array, endian: u8) -> Result<JsString, JsError> {
    let data = data.to_vec();
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let mut stream = Cursor::new(data);
    let bcsv = types::BCSV::read_options(&mut stream, endian, ())?;
    let hashes = hash::read_hashes(path)?;
    Ok(JsString::from(bcsv.convert_to_csv(hashes)))
}

#[wasm_bindgen]
pub fn csv_to_bcsv(path: &str, endian: u8, mask: u32) -> Result<Uint8Array, JsError> {
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let csv = csv::CSV::new(path)?;
    let data = csv.convert_to_bcsv(endian, mask)?;
    Ok(Uint8Array::from(data.deref()))
}