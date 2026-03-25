use std::slice;
use std::ptr;
use std::mem::*;
use std::{ffi::*, io::Cursor};
use crate::*;
use crate::types::*;


/// A managed buffer for a C caller to use in a *read-only* context. For methods in this module that return a `ManagedBuffer`,
/// ***IT MUST BE FREED USING [`free_managed_buffer`]!***
#[repr(C)]
pub struct ManagedBuffer {
    buffer: *mut u8,
    len: usize,
    capacity: usize
}

impl ManagedBuffer {
    /// Creates a new buffer from the byte data.
    pub fn new<A: AsRef<[u8]>>(item: A) -> Self {
        let vec = item.as_ref().to_vec();
        let mut me = ManuallyDrop::new(vec);
        let buffer = me.as_mut_ptr();
        let len = me.len();
        let capacity = me.capacity();
        Self { buffer, len, capacity }
    }
}

impl Drop for ManagedBuffer {
    fn drop(&mut self) {
        let Self {buffer, len, capacity} = *self;
        unsafe {
            drop(Vec::from_raw_parts(buffer,len, capacity))
        }
    }
}

#[no_mangle]
/// Reclaims the buffer pointer from FFI and drops it. 
/// ***MUST BE USED TO FREE ANY [`ManagedBuffer`] POINTERS RETURNED BY THIS MODULE***
pub extern "C" fn free_managed_buffer(buffer: ManagedBuffer) {
    drop(buffer)
}

#[no_mangle]
/// Converts a BCSV to CSV. Will return NULL if anything fails.
/// # Arguments
/// * `data` -The raw BCSV data. **Must be valid for `len`**.
/// * `len` - The exact length of data.
/// * `hash_path` - C string path to the hash table.
/// * `is_signed` - Makes [`Value::SHORT`] and [`Value::CHAR`] print in signed form.
/// * `endian` - The endian to use. 0 is Big, 1 is Little. Anything else will resolve to [`Endian::NATIVE`].
pub unsafe extern "C" fn bcsv_to_csv(data: *const u8, len: usize, hash_path: *const i8, is_signed: bool, endian: u8, delim: u8) -> ManagedBuffer {
    let slice = slice::from_raw_parts(data, len);
    let mut reader = Cursor::new(slice);
    let mut bcsv = BCSV::new();
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    if let Ok(_) = bcsv.read(&mut reader, endian) {
        let hash_path = CStr::from_ptr(hash_path).to_string_lossy().into_owned();
        let hashes = hash::read_hashes(hash_path).unwrap_or_default();
        bcsv.hash_table = hashes;
        let csv = bcsv.convert_to_csv(is_signed, delim as char).unwrap_or_default();
        let buffer = ManagedBuffer::new(csv);
        buffer
    } else {
        ManagedBuffer {buffer: ptr::null_mut(), len: 0, capacity: 0}
    }
}

#[no_mangle]
/// Converts a BCSV to a Excel Worksheet. Will output a empty worksheet if anything fails.
/// # Arguments
/// * `hash_path`: C string path to the hashtable.
/// * `output_path`: C string path for the worksheet to output to.
/// * `data`: The raw BCSV data. **Must be valid for `len`**.
/// * `len`: The length of the raw data.
/// * `is_signed` - Makes [`Value::SHORT`] and [`Value::CHAR`] print in signed form.
/// * `endian` - The endian to use. 0 is Big, 1 is Little. Anything else will resolve to [`Endian::NATIVE`].
#[cfg(not(target_arch = "wasm32"))]
pub unsafe extern "C" fn bcsv_to_xlsx(hash_path: *const i8, output_path: *const i8, data: *const u8, len: usize, is_signed: bool, endian: u8) {
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
    bcsv.hash_table = hashes;
    bcsv.convert_to_xlsx(CStr::from_ptr(output_path).to_string_lossy(), is_signed).unwrap_or_default();
}

#[no_mangle]
/// Converts a CSV to BCSV. Will return an empty buffer if anything goes wrong.
pub unsafe extern "C" fn csv_to_bcsv(path: *const i8, endian: u8, delim: u8) -> ManagedBuffer {
    let endian = match endian {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => Endian::NATIVE
    };
    let path = CStr::from_ptr(path).to_string_lossy().to_string();
    let bcsv = csv_parse::CSV::from_path(path, delim as char).unwrap_or_default();
    let data = bcsv.to_bytes(endian).unwrap_or_default();
    let buffer = ManagedBuffer::new(data);
    buffer
}