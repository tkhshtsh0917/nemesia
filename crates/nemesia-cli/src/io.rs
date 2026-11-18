//! Bounded local inputs and create-only outputs.
use nemesia_analyzer::Fallible;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        value.push(char::from(DIGITS[usize::from(byte >> 4)]));
        value.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    value
}

pub fn digest(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

pub fn file_digest(path: &str) -> Fallible<String> {
    let mut hash = Sha256::new();
    let mut file = File::open(path)?;
    if file.metadata()?.len() > 512 * 1024 * 1024 {
        return Err("dictionary byte limit exceeded".into());
    }
    let mut bounded = (&mut file).take(512 * 1024 * 1024 + 1);
    let mut count = 0;
    let mut buffer = [0; 16 * 1024];
    loop {
        let n = bounded.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
        count += n;
    }
    if count > 512 * 1024 * 1024 {
        return Err("dictionary byte limit exceeded".into());
    }
    Ok(hex(&hash.finalize()))
}

pub fn read(path: &str, max: usize) -> Fallible<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take((max + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err("input file exceeds byte limit".into());
    }
    Ok(bytes)
}

pub fn rows<T: DeserializeOwned>(
    path: &str,
    max_line: usize,
    max_rows: usize,
) -> Fallible<(Vec<T>, String)> {
    let mut input = BufReader::new(File::open(path)?);
    let mut hash = Sha256::new();
    let mut line = Vec::new();
    let mut records = Vec::new();
    let mut total = 0;
    loop {
        line.clear();
        let count = (&mut input)
            .take((max_line + 1) as u64)
            .read_until(b'\n', &mut line)?;
        if count == 0 {
            break;
        }
        total += count;
        if count > max_line || records.len() >= max_rows || total > 64 * 1024 * 1024 {
            return Err("JSONL input exceeds line, row or file limit".into());
        }
        hash.update(&line);
        let value = serde_json::from_slice(&line)
            .map_err(|error| format!("invalid JSONL at line {}: {error}", records.len() + 1))?;
        records.push(value);
    }
    Ok((records, hex(&hash.finalize())))
}

fn output(path: &str) -> Fallible<BufWriter<File>> {
    Ok(BufWriter::new(
        OpenOptions::new().create_new(true).write(true).open(path)?,
    ))
}

pub fn bytes(path: &str, data: &[u8]) -> Fallible<()> {
    let mut out = output(path)?;
    out.write_all(data)?;
    out.flush()?;
    Ok(())
}

pub fn jsonl(path: &str, rows: &[impl Serialize]) -> Fallible<()> {
    let mut file = output(path)?;
    for row in rows {
        serde_json::to_writer(&mut file, row)?;
        file.write_all(b"\n")?;
    }
    file.flush()?;
    Ok(())
}
