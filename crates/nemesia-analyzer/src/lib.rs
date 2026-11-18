//! Native Sudachi extraction and corpus statistics.
#![forbid(unsafe_code)]

mod language;
mod stats;

pub use language::{Analyzer, Policy, query_rules};
pub use stats::{Document, Extraction, Ranking, TermStats, extract};

pub type Fallible<T> = Result<T, Box<dyn std::error::Error>>;
