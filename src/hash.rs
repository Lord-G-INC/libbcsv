use std::{path::Path, collections::HashMap, num::Wrapping};
/// Preforms an accurate recreation of MR::calcHash from SMG2.
pub fn calchash(text: &str) -> u32 {
    let mut output = Wrapping(0u32);
    for char in text.bytes() {
        output = Wrapping(char as u32) + (output * Wrapping(0x1f));
    }
    output.0
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