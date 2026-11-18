#!/usr/bin/env bash
set -euo pipefail
export ASAN_OPTIONS="${ASAN_OPTIONS:-quarantine_size_mb=32:thread_local_quarantine_size_kb=64}"
mkdir -p fuzz/corpus/index
seed=fuzz/corpus/index/initial.nmsi
rm -f "$seed"
cargo run --locked -p nemesia-cli -- build --input fuzz/seed.jsonl --output "$seed"
cargo +nightly-2026-10-20 fuzz run index -- -max_total_time="${1:-20}" -max_len=65536 -rss_limit_mb=512
