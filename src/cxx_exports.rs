use std::io::Cursor;
use cxx::*;

use crate::*;

#[cxx::bridge(namespace = "libbcsv")]
mod ffi {
    extern "Rust" {
        fn bcsv_to_csv(path: &CxxString, data: &CxxVector<u8>, endian: u8) -> UniquePtr<CxxVector<u8>>;
        fn bcsv_to_xlsx(path: &CxxString, output: &CxxString, data: &CxxVector<u8>, endian: u8);
        fn csv_to_bcsv(path: &CxxString, endian: u8) -> UniquePtr<CxxVector<u8>>;
    }
}


pub fn bcsv_to_csv(path: &CxxString, data: &CxxVector<u8>, endian: u8) -> UniquePtr<CxxVector<u8>> {
    let path = path.to_string_lossy().to_string();
    let data = data.as_slice().to_vec();
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let mut reader = Cursor::new(data);
    let hashes = hash::read_hashes(path).unwrap_or_default();
    let mut bcsv = types::BCSV::new();
    bcsv.read(&mut reader, endian).unwrap_or_default();
    let text = bcsv.convert_to_csv(&hashes, false);
    let bytes = text.as_bytes();
    let mut result = CxxVector::new();
    let mut pin = result.pin_mut();
    for byte in bytes {
        pin.as_mut().push(*byte);
    }
    result
}

pub fn bcsv_to_xlsx(path: &CxxString, output: &CxxString, data: &CxxVector<u8>, endian: u8) {
    let hash_path = path.to_string_lossy().to_string();
    let data = data.as_slice().to_vec();
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let mut reader = Cursor::new(data);
    let mut bcsv = types::BCSV::new();
    bcsv.read(&mut reader, endian).unwrap_or_default();
    let hashes = hash::read_hashes(hash_path).unwrap_or_default();
    let output_path = output.to_string_lossy();
    bcsv.convert_to_xlsx(output_path, &hashes, false).unwrap_or_default();
}

pub fn csv_to_bcsv(path: &CxxString, endian: u8) -> UniquePtr<CxxVector<u8>> {
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let path = path.to_string_lossy().to_string();
    let csv = csv_parse::CSV::from_path(path).unwrap_or_default();
    let data = csv.create_bcsv().to_bytes(endian).unwrap_or_default();
    let mut result = CxxVector::new();
    let mut pin = result.pin_mut();
    for byte in data {
        pin.as_mut().push(byte);
    }
    result
}