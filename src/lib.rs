#![cfg_attr(doc, feature(doc_cfg))]

pub mod types;
pub mod bcsv;
pub use bcsv::BCSV;
pub use binrw::Endian;

use binrw::prelude::*;

/// Calculates a BCSV hash from input string.
/// 
/// NOTE: This is the newer variant of the hash method found in "newer" JSystem games like SMG1/2.
pub const fn calc_hash(field_name: &str) -> u32 {
    let mut hash = 0u32;
    let bytes = field_name.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let mut ch = bytes[i];
        // Signed byte check, inspired from pyjmap
        if ch & 0x80 != 0 {ch |= !0xFF;}
        // Use wrapping_mul and wrapping_add to avoid debug runtime
        // panics
        hash = hash.wrapping_mul(31).wrapping_add(ch as _);
        i += 1;
    }
    hash
}