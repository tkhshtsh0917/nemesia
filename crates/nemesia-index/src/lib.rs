//! Candidate input and shared normalization.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

pub const MAX_QUERY: usize = 128;
pub const MAX_KEY: usize = 256;
pub const MAX_ALIASES: usize = 8;
pub const MAX_CANDIDATES: usize = 200_000;
pub const MAX_INDEX_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub id: u32,
    pub text: String,
    #[serde(default)]
    pub weight: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

/// Normalizes a bounded field while keeping a query's trailing word boundary.
pub fn normalize(raw: &str, max: usize, trailing: bool) -> Result<String, &'static str> {
    if raw.len() > max * 4 {
        return Err("input exceeds byte limit");
    }
    let mut value = String::new();
    let mut space = false;
    for c in raw.nfkc().flat_map(char::to_lowercase) {
        if c.is_whitespace() {
            space = !value.is_empty();
            continue;
        }
        if c.is_control() {
            return Err("unsupported control character");
        }
        if space {
            value.push(' ');
            space = false;
        }
        value.push(c);
        if value.len() > max * 4 {
            return Err("normalized input exceeds byte limit");
        }
    }
    if trailing && space {
        value.push(' ');
    }
    if value.chars().count() > max {
        return Err("normalized input exceeds scalar limit");
    }
    Ok(value)
}
