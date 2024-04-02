use std::{ffi::*, io::Cursor};
use crate::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PtrInfo {
    pub ptr: *mut u8,
    pub len: usize
}

#[no_mangle]
pub unsafe extern "C" fn free_PtrInfo(info: PtrInfo) {
    let PtrInfo { ptr, len } = info;
    let slice = std::slice::from_raw_parts_mut(ptr, len);
    drop(Box::<[u8]>::from_raw(slice));
}

#[no_mangle]
pub unsafe extern "C" fn bcsv_to_csv(hash_path: *const i8, data: *const u8, len: usize, endian: u8) -> PtrInfo {
    let buffer = std::slice::from_raw_parts(data, len).to_vec();
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let mut reader = Cursor::new(buffer);
    let mut bcsv = types::BCSV::new();
    bcsv.read(&mut reader, endian).unwrap_or_default();
    let hash_path = CStr::from_ptr(hash_path).to_string_lossy().into_owned();
    let hashes = hash::read_hashes(hash_path).unwrap_or_default();
    let text = bcsv.convert_to_csv(&hashes);
    let bx = Box::<[u8]>::from(text.as_bytes());
    let len = bx.len();
    PtrInfo {ptr: Box::into_raw(bx).cast(), len}
}

#[no_mangle]
pub unsafe extern "C" fn bcsv_to_xlsx(hash_path: *const i8, output_path: *const i8, data: *const u8, len: usize, endian: u8) {
    let buffer = std::slice::from_raw_parts(data, len).to_vec();
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let mut reader = Cursor::new(buffer);
    let mut bcsv = types::BCSV::new();
    bcsv.read(&mut reader, endian).unwrap_or_default();
    let hash_path = CStr::from_ptr(hash_path).to_string_lossy().into_owned();
    let hashes = hash::read_hashes(hash_path).unwrap_or_default();
    bcsv.convert_to_xlsx(CStr::from_ptr(output_path).to_string_lossy(), &hashes).unwrap_or_default();
}

#[no_mangle]
pub unsafe extern "C" fn csv_to_bcsv(path: *const i8, endian: u8) -> PtrInfo {
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let path = CStr::from_ptr(path).to_string_lossy().to_string();
    let csv = csv_parse::CSV::from_path(path).unwrap_or_default();
    let data = csv.create_bcsv().to_bytes(endian).unwrap_or_default();
    let len = data.len();
    PtrInfo { ptr: Box::into_raw(data.into_boxed_slice()).cast(), len }
}