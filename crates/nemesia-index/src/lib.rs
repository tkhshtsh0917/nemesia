//! Candidate input and shared normalization.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

/// Maximum Unicode scalar count per raw or normalized query variant.
pub const MAX_QUERY: usize = 128;
/// Maximum Unicode scalar count per raw or normalized candidate key.
pub const MAX_KEY: usize = 256;
/// Maximum number of alternate keys supplied for each candidate.
pub const MAX_ALIASES: usize = 8;
/// Maximum number of candidates in a single index.
pub const MAX_CANDIDATES: usize = 200_000;
/// Maximum decoded index size, in bytes.
pub const MAX_INDEX_BYTES: usize = 32 * 1024 * 1024;

/// One row in the candidate JSONL or an in-memory build.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    /// Unique identifier in this index generation.
    pub id: u32,
    /// Display text and source of the canonical match key.
    pub text: String,
    #[serde(default)]
    /// Static tie breaker applied after match quality. Defaults to zero.
    pub weight: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Additional match keys that lead to this same candidate ID.
    pub aliases: Vec<String>,
}

/// Normalizes a bounded field while optionally retaining a trailing space.
///
/// Use [`MAX_QUERY`] for queries and [`MAX_KEY`] for keys. Applies NFKC,
/// per-scalar lowercase and whitespace folding. Rejects unsupported
/// controls and fields that exceed the byte or scalar limit.
pub fn normalize(raw: &str, max: usize, trailing: bool) -> Result<String, &'static str> {
    if raw.len() > max * 4 {
        return Err("input exceeds byte limit");
    }
    if raw.chars().count() > max {
        return Err("input exceeds scalar limit");
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

pub mod binary;
