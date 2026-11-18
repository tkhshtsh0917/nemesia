//! Compares normalized flat lookups and the compiled trie for a supplied workload.
#![forbid(unsafe_code)]

use nemesia_builder::build;
use nemesia_engine::{prefix_distance, suggest};
use nemesia_index::{Candidate, MAX_KEY, binary::Index, normalize};
use std::{cmp::Reverse, fs, time::Instant};

fn flat(keys: &[(String, u32, u32)], query: &str) -> Vec<(u32, u8)> {
    let exact = query.chars().count() <= 2;
    let mut ranked = Vec::new();
    if exact {
        let start = keys.partition_point(|(key, _, _)| key.as_str() < query);
        for (key, weight, id) in &keys[start..] {
            if !key.starts_with(query) {
                break;
            }
            ranked.push((0, Reverse(*weight), *id));
        }
    } else {
        let q: Vec<_> = query.chars().collect();
        for (key, weight, id) in keys {
            let Some(distance) = prefix_distance(&q, key) else {
                continue;
            };
            if distance <= 2 {
                ranked.push((distance, Reverse(*weight), *id));
            }
        }
    }
    ranked.sort_unstable();
    ranked
        .into_iter()
        .take(8)
        .map(|(distance, _, id)| (id, distance))
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let source = fs::read_to_string(args.next().ok_or("expected candidate JSONL")?)?;
    let queries = fs::read_to_string(args.next().ok_or("expected query text lines")?)?;
    let rows: Vec<Candidate> = source
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    if rows.iter().any(|row| !row.aliases.is_empty()) {
        return Err("benchmark expects no aliases".into());
    }
    let mut keys: Vec<_> = rows
        .iter()
        .map(|row| {
            (
                normalize(&row.text, MAX_KEY, false).unwrap(),
                row.weight,
                row.id,
            )
        })
        .collect();
    keys.sort_unstable();
    let index = Index::open(build(&rows)?)?;
    for class in ["exact", "fuzzy"] {
        let inputs: Vec<_> = queries
            .lines()
            .filter(|query| (query.len() <= 2) == (class == "exact"))
            .collect();
        let run_trie = |q: &str| -> Vec<_> {
            suggest(&index, q, None, 8)
                .unwrap()
                .into_iter()
                .map(|hit| (hit.id, hit.distance))
                .collect()
        };
        for q in &inputs {
            assert_eq!(flat(&keys, q), run_trie(q), "{q}");
        }
        let time = |apply: &dyn Fn(&str) -> Vec<(u32, u8)>| {
            let now = Instant::now();
            for q in &inputs {
                std::hint::black_box(apply(q));
            }
            now.elapsed().as_secs_f64() * 1000.0 / inputs.len() as f64
        };

        for run in 0..4 {
            let (flat, trie) = if run % 2 == 0 {
                (time(&|q| flat(&keys, q)), time(&run_trie))
            } else {
                let trie = time(&run_trie);
                (time(&|q| flat(&keys, q)), trie)
            };
            println!("{},{class},{run},{flat:.6},{trie:.6}", rows.len());
        }
    }
    Ok(())
}
