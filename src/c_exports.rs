use crate::*;

use std::ffi::*;
use std::io::Cursor;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PtrInfo {
    pub ptr: *mut c_uchar,
    pub len: usize
}

#[no_mangle]
pub unsafe extern "C" fn bcsv_to_csv(hash_path: *const c_char, data: *mut c_uchar, len: usize,
    endian: c_uchar) -> *const c_char {
    let buffer = std::slice::from_raw_parts(data, len).to_vec();
    let mut stream = Cursor::new(buffer);
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let bcsv = types::BCSV::read_options(&mut stream, endian, ())
    .unwrap_or_default();
    let hash_pth = CStr::from_ptr(hash_path).to_string_lossy().to_string();
    let hashes = hash::read_hashes(hash_pth).unwrap_or_default();
    let text = bcsv.convert_to_csv(hashes);
    let result = CString::new(text).unwrap_or_default();
    result.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn csv_to_bcsv(path: *const c_char, endian: c_uchar, mask: c_uint) -> PtrInfo {
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let path = CStr::from_ptr(path).to_string_lossy().to_string();
    let csv = csv::CSV::new(path).unwrap_or_default();
    let data = csv.convert_to_bcsv(endian, mask).unwrap_or_default();
    let len = data.len();
    let bx = data.into_boxed_slice();
    PtrInfo { ptr: Box::into_raw(bx).cast(), len }
}

#[no_mangle]
pub unsafe extern "C" fn free_PtrInfo(info: PtrInfo) {
    let PtrInfo { ptr, len } = info;
    let slice = std::slice::from_raw_parts_mut(ptr, len);
    drop(Box::<[u8]>::from_raw(slice));
}