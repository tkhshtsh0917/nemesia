#!/usr/bin/env bash
set -euo pipefail
platform="${1:?expected platform}"
tag="${2:?expected release tag}"
case "$platform:$(uname -sm)" in
  'linux-x86_64:Linux x86_64'|'macos-aarch64:Darwin arm64') ;;
  *) echo "unsupported release platform" >&2; exit 1 ;;
esac

cargo build --release --locked -p nemesia-cli
cli=target/release/nemesia
version="$("$cli" --version)"
[[ "$version" == "nemesia ${tag#v}" ]]
mkdir -p dist
cp "$cli" "dist/nemesia-$platform"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
printf '%s\n' '{"id":1,"text":"Tokyo Station"}' > "$work/input.jsonl"
"$cli" build --input "$work/input.jsonl" --output "$work/index.nmsi"
"$cli" inspect --index "$work/index.nmsi"
"$cli" suggest --index "$work/index.nmsi" --query 'toky st' | grep -F '"distance":1'

if [[ "$platform" == linux-x86_64 || "${NEMESIA_PACKAGE_WASM:-}" == 1 ]]; then
  cargo build --release --locked --target wasm32-unknown-unknown -p nemesia-wasm
  cp target/wasm32-unknown-unknown/release/nemesia_wasm.wasm dist/
  printf '%s\n' '{"abi":1,"index":1,"normalization":1,"wasmBindgen":"0.2.129"}' > dist/nemesia-abi.json
  cargo metadata --locked --format-version 1 > "$work/metadata.json"
  python3 tools/licenses.py "$work/metadata.json" "$work/licenses"
  tar -czf dist/nemesia-licenses.tar.gz -C "$work" licenses
fi
