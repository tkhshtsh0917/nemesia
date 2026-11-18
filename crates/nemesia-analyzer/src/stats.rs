use crate::{Analyzer, Fallible, language::Exclusions};
use nemesia_index::{Candidate, MAX_ALIASES, MAX_CANDIDATES};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_DOCUMENTS: usize = 200_000;
const MAX_ELIGIBLE_TOKENS: usize = 2_000_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: String,
    pub text: String,
}

#[derive(Default, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ranking {
    #[default]
    Df,
    Bm25Sum,
}

#[derive(Debug, Serialize)]
pub struct TermStats {
    pub id: u32,
    pub df: u32,
    pub cf: u32,
    pub bm25_sum: u32,
}

#[derive(Serialize)]
pub struct Summary {
    pub documents: usize,
    pub eligible_occurrences: usize,
    pub candidates: usize,
    pub aliases: usize,
    pub exclusions: Exclusions,
    pub ranking: Ranking,
    pub k1: f64,
    pub b: f64,
    pub scale: f64,
}

pub struct Extraction {
    pub rows: Vec<Candidate>,
    pub stats: Vec<TermStats>,
    pub summary: Summary,
}

struct Term {
    display: String,
    aliases: BTreeSet<String>,
    df: u32,
    cf: u32,
    bm25: f64,
}

const K1: f64 = 1.2;
const B: f64 = 0.75;
const SCALE: f64 = 1000.0;

/// Aggregates term counts per document before deriving corpus-wide weights.
pub fn extract(
    analyzer: &mut Analyzer,
    mut documents: Vec<Document>,
    ranking: Ranking,
) -> Fallible<Extraction> {
    if documents.len() > MAX_DOCUMENTS {
        return Err("document count exceeds limit".into());
    }
    documents.sort_unstable_by(|a, b| a.id.cmp(&b.id));
    if documents.windows(2).any(|rows| rows[0].id == rows[1].id)
        || documents
            .iter()
            .any(|row| row.id.trim().is_empty() || row.id.len() > 256)
    {
        return Err("duplicate or invalid document id".into());
    }

    let n = documents.len();
    let mut terms: BTreeMap<String, usize> = BTreeMap::new();
    let mut entries: Vec<Term> = Vec::new();
    let mut per_doc = Vec::with_capacity(n);
    let mut total = 0;
    let mut exclusions = Exclusions::default();
    for doc in documents {
        let tokens = analyzer.tokens(&doc.text, &mut exclusions)?;
        let len = tokens.len();
        total += len;
        if total > MAX_ELIGIBLE_TOKENS {
            return Err("eligible token limit exceeded".into());
        }

        let mut counts = BTreeMap::<usize, u32>::new();
        for token in tokens {
            let i = *terms.entry(token.key).or_insert_with(|| {
                let i = entries.len();
                entries.push(Term {
                    display: token.display.clone(),
                    aliases: BTreeSet::new(),
                    df: 0,
                    cf: 0,
                    bm25: 0.0,
                });
                i
            });
            if entries.len() > MAX_CANDIDATES {
                return Err("extracted candidate limit exceeded".into());
            }
            let entry = &mut entries[i];
            if token.display < entry.display {
                entry.display = token.display;
            }
            for alias in token.aliases {
                entry.aliases.insert(alias);
                if entry.aliases.len() > MAX_ALIASES {
                    entry.aliases.pop_last();
                }
            }
            *counts.entry(i).or_default() += 1;
        }
        for (&i, &count) in &counts {
            entries[i].df += 1;
            entries[i].cf += count;
        }
        per_doc.push((len, counts));
    }

    if total > 0 {
        let average = total as f64 / n as f64;
        for (len, counts) in per_doc {
            for (i, tf) in counts {
                let term = &mut entries[i];
                term.bm25 += bm25(n, term.df, tf, len, average);
            }
        }
    }

    let mut rows = Vec::with_capacity(entries.len());
    let mut stats = Vec::with_capacity(entries.len());
    let mut aliases = 0;
    for (position, (_, i)) in terms.into_iter().enumerate() {
        let id = (position + 1) as u32;
        let term = &mut entries[i];
        let scaled = (term.bm25 * SCALE).round();
        if !scaled.is_finite() || !(0.0..=f64::from(u32::MAX)).contains(&scaled) {
            return Err("BM25 weight out of range".into());
        }
        let bm25_sum = scaled as u32;
        let weight = match ranking {
            Ranking::Df => term.df,
            Ranking::Bm25Sum => bm25_sum,
        };
        aliases += term.aliases.len();
        rows.push(Candidate {
            id,
            text: std::mem::take(&mut term.display),
            weight,
            aliases: std::mem::take(&mut term.aliases).into_iter().collect(),
        });
        stats.push(TermStats {
            id,
            df: term.df,
            cf: term.cf,
            bm25_sum,
        });
    }

    let summary = Summary {
        documents: n,
        eligible_occurrences: total,
        candidates: rows.len(),
        aliases,
        exclusions,
        ranking,
        k1: K1,
        b: B,
        scale: SCALE,
    };
    Ok(Extraction {
        rows,
        stats,
        summary,
    })
}

fn bm25(n: usize, df: u32, tf: u32, dl: usize, average: f64) -> f64 {
    let (df, tf) = (f64::from(df), f64::from(tf));
    let idf = ((n as f64 - df + 0.5) / (df + 0.5)).ln_1p();
    idf * tf * (K1 + 1.0) / (tf + K1 * (1.0 - B + B * dl as f64 / average))
}
