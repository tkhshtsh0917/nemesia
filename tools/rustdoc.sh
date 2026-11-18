#!/usr/bin/env bash
set -euo pipefail

output="${1:?expected output archive}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --exclude nemesia-cli --no-deps --target-dir "$work/target"
docs="$work/target/doc"
for crate in nemesia_analyzer nemesia_builder nemesia_engine nemesia_index nemesia_wasm; do
  test -s "$docs/$crate/index.html"
done
rm -f "$docs/.lock"
cp tools/rustdoc-index.html "$docs/index.html"
tar -czf "$output" -C "$docs" .
