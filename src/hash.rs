use std::{collections::HashMap, path::Path};
/// Preforms an accurate recreation of MR::calcHash from SMG2.
#[inline]
pub const fn calchash(text: &str) -> u32 {
    let mut output = 0u32;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i != bytes.len() {
        output = (bytes[i] as u32).wrapping_add(output.wrapping_mul(0x1f));
        i += 1;
    }
    output
}
#[inline]
pub const fn calc_old_hash(text: &str) -> u32 {
    let mut output = 0u32;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i != bytes.len() {
        output = ((bytes[i] as u32).wrapping_shl(8) & u32::MAX).wrapping_add(output) % 33554393;
        i += 1;
    }
    output
}

/// Reads a HashMap of BCSV hashes and Strings from a path.
pub fn read_hashes<P: AsRef<Path>>(path: P) -> std::io::Result<HashMap<u32, String>> {
    let text = std::fs::read_to_string(path)?;
    let mut result = HashMap::new();
    for line in text.split('\n') {
        if line.starts_with('#') {
            continue;
        }
        let hash = calchash(line);
        result.insert(hash, String::from(line));
    }
    Ok(result)
}

pub fn read_old_hashes<P: AsRef<Path>>(path: P) -> std::io::Result<HashMap<u32, String>> {
    let text = std::fs::read_to_string(path)?;
    let mut result = HashMap::new();
    for line in text.split('\n') {
        if line.starts_with('#') {
            continue;
        }
        let hash = calc_old_hash(line);
        result.insert(hash, String::from(line));
    }
    Ok(result)
}