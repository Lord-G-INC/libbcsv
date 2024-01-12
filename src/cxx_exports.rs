use std::io::Cursor;
use crate::*;
use cxx::*;


#[cxx::bridge(namespace = "libbcsv")]
mod ffi {
    extern "Rust" {
        fn bcsv_to_csv(path: &CxxString, data: &CxxVector<u8>, endian: u8) -> UniquePtr<CxxVector<u8>>;
        fn csv_to_bcsv(path: &CxxString, endian: u8, mask: u32) -> UniquePtr<CxxVector<u8>>;
        fn bcsv_to_xlsx(path: &CxxString, data: &CxxVector<u8>, output: &CxxString, endian: u8);
    }
}

pub fn bcsv_to_csv(path: &CxxString, data: &CxxVector<u8>, endian: u8) -> UniquePtr<CxxVector<u8>> {
    let hash_path = path.to_string_lossy().to_string();
    let data = data.as_slice().to_vec();
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let mut stream = Cursor::new(data);
    let bcsv = types::BCSV::read_options(&mut stream, endian, ())
    .unwrap_or_default();
    let hash_data = hash::read_hashes(hash_path).unwrap_or_default();
    let csv = bcsv.convert_to_csv(hash_data);
    let bytes = csv.as_bytes();
    let mut result = CxxVector::new();
    let mut pin = result.pin_mut();
    for byte in bytes {
        pin.as_mut().push(*byte);
    }
    return result;
}

pub fn csv_to_bcsv(path: &CxxString, endian: u8, mask: u32) -> UniquePtr<CxxVector<u8>> {
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let path = path.to_string_lossy().to_string();
    let csv = csv::CSV::new(path).unwrap_or_default();
    let data = csv.convert_to_bcsv(endian, mask).unwrap_or_default();
    let mut result = CxxVector::new();
    let mut pin = result.pin_mut();
    for byte in data {
        pin.as_mut().push(byte);
    }
    return result;
}

pub fn bcsv_to_xlsx(path: &CxxString, data: &CxxVector<u8>, output: &CxxString, endian: u8) {
    let hash_path = path.to_string_lossy().to_string();
    let data = data.as_slice().to_vec();
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let mut stream = Cursor::new(data);
    let bcsv = types::BCSV::read_options(&mut stream, endian, ())
    .unwrap_or_default();
    let hashes = hash::read_hashes(hash_path).unwrap_or_default();
    let output_path = output.to_string_lossy().to_string();
    bcsv.convert_to_xlsx(hashes, output_path).unwrap_or_default();
}