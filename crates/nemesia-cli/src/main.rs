//! Native extraction, index compilation and inspection commands.
#![forbid(unsafe_code)]

mod io;

use nemesia_analyzer::{Analyzer, Document, Fallible, Policy, extract, query_rules};
use nemesia_engine::{DEFAULT_LIMIT, suggest};
use nemesia_index::{MAX_CANDIDATES, MAX_INDEX_BYTES, binary::Index};
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

struct Options(BTreeMap<String, String>);

impl Options {
    fn parse() -> Fallible<(String, Self)> {
        let mut args = std::env::args().skip(1);
        let command = args
            .next()
            .ok_or("expected extract, build, inspect, suggest or --version")?;
        let mut options = BTreeMap::new();
        while let Some(key) = args.next() {
            if !key.starts_with("--") || options.contains_key(&key) {
                return Err(format!("unexpected option: {key}").into());
            }
            options.insert(key, args.next().ok_or("option requires a value")?);
        }
        Ok((command, Self(options)))
    }

    fn optional(&mut self, key: &str) -> Option<String> {
        self.0.remove(key)
    }

    fn required(&mut self, key: &str) -> Fallible<String> {
        self.optional(key)
            .ok_or_else(|| format!("missing {key}").into())
    }

    fn finish(self) -> Fallible<()> {
        if let Some((key, _)) = self.0.first_key_value() {
            return Err(format!("unknown option: {key}").into());
        }
        Ok(())
    }
}

fn extract_command(mut opts: Options) -> Fallible<()> {
    let input = opts.required("--input")?;
    let dictionary = opts.required("--dictionary")?;
    let policy = opts.optional("--policy");
    let output = opts.required("--output")?;
    let stats = opts.optional("--stats");
    let rules = opts.optional("--query-rules");
    let provenance = opts.optional("--provenance");
    let generation = opts.optional("--snapshot");
    opts.finish()?;

    let policy: Policy = policy
        .as_deref()
        .map(|path| io::read(path, 1024 * 1024))
        .transpose()?
        .map(|data| serde_json::from_slice(&data))
        .transpose()?
        .unwrap_or_default();
    let (documents, input_sha256) = io::rows::<Document>(&input, 300_000, 200_000)?;
    let mut native = Analyzer::open(Path::new(&dictionary), &policy)?;
    let extracted = extract(&mut native, documents, policy.ranking)?;

    let provenance = provenance.map(|path| -> Fallible<_> {
        let snapshot = generation.as_deref().unwrap_or(&input_sha256);
        if snapshot.len() > 128 || !snapshot.bytes().all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c)) {
            return Err("invalid snapshot identifier".into());
        }
        Ok((path, json!({
            "format": "nemesia-extraction/1", "snapshot": snapshot, "inputSha256": input_sha256,
            "dictionarySha256": io::file_digest(&dictionary)?, "policySha256": io::digest(&serde_json::to_vec(&policy)?),
            "policy": policy, "summary": extracted.summary
        })))
    }).transpose()?;
    io::jsonl(&output, &extracted.rows)?;
    if let Some(path) = stats {
        io::jsonl(&path, &extracted.stats)?;
    }
    if let Some(path) = rules {
        io::bytes(&path, &serde_json::to_vec(&query_rules())?)?;
    }
    if let Some((path, mut value)) = provenance {
        value["candidateSha256"] = io::file_digest(&output)?.into();
        io::bytes(&path, &serde_json::to_vec_pretty(&value)?)?;
    }
    Ok(())
}

fn run() -> Fallible<()> {
    let (command, mut opts) = Options::parse()?;
    match command.as_str() {
        "--version" => {
            opts.finish()?;
            println!("nemesia {}", env!("CARGO_PKG_VERSION"));
        }
        "extract" => extract_command(opts)?,
        "build" => {
            let input = opts.required("--input")?;
            let output = opts.required("--output")?;
            opts.finish()?;
            let (rows, _) = io::rows(&input, 64 * 1024, MAX_CANDIDATES)?;
            let compiled = nemesia_builder::build(&rows)?;
            io::bytes(&output, &compiled)?;
        }
        "inspect" | "suggest" => {
            let path = opts.required("--index")?;
            let raw = if command == "suggest" {
                Some(opts.required("--query")?)
            } else {
                None
            };
            let derived = opts.optional("--derived");
            let limit = opts
                .optional("--limit")
                .map(|value| value.parse())
                .transpose()?
                .unwrap_or(DEFAULT_LIMIT);
            if raw.is_none() && (derived.is_some() || limit != DEFAULT_LIMIT) {
                return Err("inspect does not accept query options".into());
            }
            opts.finish()?;
            let bytes = io::read(&path, MAX_INDEX_BYTES)?;
            let size = bytes.len();
            let index = Index::open(bytes)?;
            if let Some(query) = raw {
                println!(
                    "{}",
                    serde_json::to_string(&suggest(&index, &query, derived.as_deref(), limit)?)?
                );
            } else {
                println!(
                    "{}",
                    json!({"format": "NMSI", "version": 1, "candidates": index.len(), "bytes": size})
                );
            }
        }
        _ => return Err(format!("unknown command: {command}").into()),
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nemesia: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
