use std::{io::Cursor, pin::Pin};
use crate::*;
use cxx::*;


#[cxx::bridge(namespace = "libbcsv")]
mod ffi {
    extern "Rust" {
        fn bcsv_to_csv(path: Pin<&mut CxxString>, data: &CxxVector<u8>, endian: u8);
        fn csv_to_bcsv(path: &CxxString, endian: u8, buffer: Pin<&mut CxxVector<u8>>, mask: u32);
    }
}

pub fn bcsv_to_csv(mut path: Pin<&mut CxxString>, data: &CxxVector<u8>, endian: u8)  {
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
    let result = bcsv.convert_to_csv(hash_data);
    path.as_mut().clear();
    path.push_str(&result);
}

pub fn csv_to_bcsv(path: &CxxString, endian: u8, mut buffer: Pin<&mut CxxVector<u8>>, mask: u32) {
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let path = path.to_string_lossy().to_string();
    let csv = csv::CSV::new(path).unwrap_or_default();
    let data = csv.convert_to_bcsv(endian, mask).unwrap_or_default();
    // For some reason clear isn't exposed, do this trick instead
    while let Some(_) = buffer.as_mut().pop() {
        continue;
    }
    for byte in data {
        buffer.as_mut().push(byte);
    }
}