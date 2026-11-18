//! Ranked prefix completions over a validated Nemesia index.
//!
//! Use [`suggest`] for ordinary queries. The [`reference()`] implementation
//! scans every candidate and is useful for verifying search behavior.
#![forbid(unsafe_code)]

use nemesia_index::{Candidate, MAX_KEY, MAX_QUERY, normalize};
use serde::Serialize;

/// Default number of suggestions in the CLI and demo.
pub const DEFAULT_LIMIT: usize = 8;
/// Largest accepted number of suggestions per query.
pub const MAX_LIMIT: usize = 20;

/// An owned, ranked completion. It survives later queries and index disposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Suggestion {
    /// Candidate identifier supplied at build time.
    pub id: u32,
    /// Original display text to insert into the search box.
    pub text: String,
    /// Minimum edit distance of the path that ranked this candidate.
    pub distance: u8,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Quality(u8, u8, u8);

#[derive(Clone, Copy)]
struct Rank {
    quality: Quality,
    weight: u32,
    id: u32,
}

impl Rank {
    fn key(self) -> (Quality, std::cmp::Reverse<u32>, u32) {
        (self.quality, std::cmp::Reverse(self.weight), self.id)
    }
}

fn queries(raw: &str, derived: Option<&str>, limit: usize) -> Result<Vec<String>, &'static str> {
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err("invalid result limit");
    }
    let raw = normalize(raw, MAX_QUERY, true)?;
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let mut variants = vec![raw];
    if let Some(alternative) = derived {
        let value = normalize(alternative, MAX_QUERY, true)?;
        if !value.is_empty() && value != variants[0] {
            variants.push(value);
        }
    }
    Ok(variants)
}

fn budget(query: &str) -> u8 {
    if query.ends_with(' ') {
        return 0;
    }
    match query.chars().count() {
        0..=2 => 0,
        3..=5 => 1,
        _ => 2,
    }
}

/// Computes the minimum edit distance over nonempty leading prefixes.
///
/// Pass normalized text within the query and key limits. An empty key
/// returns `None`. This reference helper is intended for testing.
pub fn prefix_distance(query: &[char], value: &str) -> Option<u8> {
    let mut previous: Vec<u16> = (0..=query.len() as u16).collect();
    let mut row = previous.clone();
    let mut best = None;

    for (depth, c) in value.chars().enumerate() {
        row[0] = (depth + 1) as u16;
        for (i, q) in query.iter().enumerate() {
            row[i + 1] = (previous[i] + u16::from(*q != c))
                .min(previous[i + 1] + 1)
                .min(row[i] + 1);
        }
        let distance = row[query.len()];
        best = Some(best.map_or(distance, |old: u16| old.min(distance)));
        std::mem::swap(&mut previous, &mut row);
    }

    best.map(|distance| u8::try_from(distance).unwrap_or(u8::MAX))
}

/// Scans every supplied key and ranks the strongest path for each candidate.
pub fn reference(
    rows: &[Candidate],
    raw: &str,
    derived: Option<&str>,
    limit: usize,
) -> Result<Vec<Suggestion>, &'static str> {
    let variants: Vec<_> = queries(raw, derived, limit)?
        .into_iter()
        .map(|value| {
            let allowance = budget(&value);
            (value.chars().collect::<Vec<_>>(), allowance)
        })
        .collect();
    let mut found = Vec::new();
    for row in rows {
        let mut best = None;
        for (key_index, text) in std::iter::once(&row.text).chain(&row.aliases).enumerate() {
            let key = normalize(text, MAX_KEY, false)?;
            for (variant, (query, allowance)) in variants.iter().enumerate() {
                if let Some(distance) = prefix_distance(query, &key)
                    && distance <= *allowance
                {
                    let quality = Quality(distance, variant as u8, u8::from(key_index != 0));
                    best = Some(best.map_or(quality, |old: Quality| old.min(quality)));
                }
            }
        }
        if let Some(quality) = best {
            found.push((
                Rank {
                    quality,
                    weight: row.weight,
                    id: row.id,
                },
                row,
            ));
        }
    }
    found.sort_unstable_by_key(|(rank, _)| rank.key());
    Ok(found
        .into_iter()
        .take(limit)
        .map(|(rank, row)| Suggestion {
            id: row.id,
            text: row.text.clone(),
            distance: rank.quality.0,
        })
        .collect())
}

mod trie;
pub use trie::{WORK_LIMIT, suggest, suggest_bounded};
