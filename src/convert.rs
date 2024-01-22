use std::collections::HashMap;
use crate::*;
use crate::types::*;
use std::io::{Cursor, Write};
use xlsxwriter::prelude::*;

pub fn convert_to_csv(bcsv: BCSV, hashes: HashMap<u32, String>) -> String {
    let mut text = String::new();
    let mut names = vec![];
    for field in &bcsv.fields {
        let hash = field.hash;
        if hashes.contains_key(&hash) {
            names.push(hashes[&hash].clone());
        } else {
            names.push(format!("0x{:x}", hash));
        }
    }
    for i in 0..names.len() {
        let last = i == names.len() - 1;
        text.push_str(&names[i]);
        text.push(':');
        text.push_str(&bcsv.fields[i].datatype.to_string());
        if !last {
            text.push(',');
        } else {
            text.push('\n');
        }
    }
    let mut v = 0;
    while v < bcsv.values.len() {
        for i in 0..bcsv.fields.len() {
            let last = i == bcsv.fields.len() - 1;
            let shift = bcsv.fields[i].shift;
            let mask = bcsv.fields[i].mask;
            let term = match last { false => ',', true => '\n' };
            match &bcsv.values[v] {
                Value::LONG(l) => {
                    let mut l = *l;
                    l = (l & mask as i32) >> shift as i32;
                    let txt = format!("{}{}", l, term);
                    text.push_str(&txt);
                }
                Value::STRING(s) => {
                    text.push_str(&String::from(String::from_utf8_lossy(s)));
                    text.push(term);
                }
                Value::FLOAT(f) => {
                    let txt = format!("{}{}", f, term);
                    text.push_str(&txt);
                }
                Value::ULONG(u) => {
                    let mut u = *u;
                    u = (u & mask) >> shift;
                    let txt = format!("{}{}", u, term);
                    text.push_str(&txt);
                }
                Value::SHORT(sh) => {
                    let mut sh = *sh;
                    sh = (sh & mask as u16) >> shift as u16;
                    let txt = format!("{}{}", sh, term);
                    text.push_str(&txt);
                }
                Value::CHAR(c) => {
                    let mut c = *c;
                    c = (c & mask as u8) >> shift as u8;
                    let txt = format!("{}{}", c, term);
                    text.push_str(&txt);
                }
                Value::STRINGOFF(st) => {
                    text.push_str(st);
                    text.push(term);
                }
                Value::NULL => {}
            }
            v += 1;
        }
    }
    text
}

pub fn convert_to_bcsv(mut csv: csv::CSV, endian: Endian, mask: u32) -> BinResult<Vec<u8>> {
    let mut bcsv = csv.generate_bcsv();
    let mut buffer = Cursor::new(vec![]);
    let table = csv.create_stringtable();
    csv.create_values(&mut bcsv);
    for field in &mut bcsv.fields {
        field.mask = mask;
    }
    bcsv.write_options(&mut buffer, endian, ())?;
    let stroff = bcsv.header.entrydataoff as usize + bcsv.header.entrysize as usize *
        bcsv.header.entrycount as usize;
    let mut size = table.len() + stroff;
    size += (size + 31 & !31) - size;
    buffer.get_mut().resize(size, 0x40);
    buffer.write_all(table.as_bytes())?;
    Ok(buffer.into_inner())
}

pub fn convert_to_xlsx(bcsv: BCSV, hashes: HashMap<u32, String>, outpath: String) 
    -> Result<(), BcsvError> {
    let csv_data = bcsv.convert_to_csv(hashes);
    let split = csv_data.split('\n').collect::<Vec<_>>();
    let mut idx = 0;
    let mut lines = vec![vec![String::new(); 0]; split.len()];
    for line in split {
        lines[idx] = line.split(',').map(|x| x.to_string()).collect();
        idx += 1;
    }
    let writer = Workbook::new(&outpath)?;
    let mut worksheet = writer.add_worksheet(None)?;
    for i in 0..lines.len() {
        let line = &lines[i];
        for j in 0..line.len() {
            worksheet.write_string(i as u32, j as u16, &line[j], None)?;
        }
    }
    Ok(())
}