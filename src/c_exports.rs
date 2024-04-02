use std::ffi::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PtrInfo {
    pub ptr: *mut c_uchar,
    pub len: usize
}

