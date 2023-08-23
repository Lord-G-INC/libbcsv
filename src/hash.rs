use std::{path::Path, collections::HashMap};

pub fn calchash(text: &str) -> u32 {
    let mut output = 0u32;
    for char in text.bytes() {
        output = output.wrapping_mul(3);
        output = output.wrapping_add(char as u32);
    }
    output
}

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