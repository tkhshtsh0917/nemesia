#!/usr/bin/env bash
set -euo pipefail

output="${1:?expected site directory}"
tools="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
if [[ -e "$output" || -L "$output" ]]; then
  echo "Rustdoc output already exists: $output" >&2
  exit 1
fi
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --exclude nemesia-cli --no-deps --target-dir "$work/target"
docs="$work/target/doc"
for crate in nemesia_analyzer nemesia_builder nemesia_engine nemesia_index nemesia_wasm; do
  test -s "$docs/$crate/index.html"
done
rm -f "$docs/.lock"

rustdoc_asset() {
  local stem="$1" extension="$2" paths
  paths=("$docs/static.files/$stem-"*".$extension")
  if [[ ${#paths[@]} -ne 1 || ! -s "${paths[0]}" ]]; then
    echo "Expected one rustdoc asset matching $stem-*.$extension" >&2
    return 1
  fi
  basename "${paths[0]}"
}

favicon="$(rustdoc_asset favicon svg)"
normalize_css="$(rustdoc_asset normalize css)"
rustdoc_css="$(rustdoc_asset rustdoc css)"
storage_js="$(rustdoc_asset storage js)"
sed -e "s|@FAVICON@|$favicon|g" \
    -e "s|@NORMALIZE_CSS@|$normalize_css|g" \
    -e "s|@RUSTDOC_CSS@|$rustdoc_css|g" \
    -e "s|@STORAGE_JS@|$storage_js|g" \
    "$tools/rustdoc-index.html" > "$docs/index.html"

mkdir -p "$(dirname "$output")"
mv "$docs" "$output"
