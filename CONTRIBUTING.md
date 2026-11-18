# Contributing

[日本語](CONTRIBUTING.ja.md)

Run `rustup show` and `mise run check` from this checkout. Rustup selects the pinned toolchain, and mise runs the same checks as CI. Keep browser changes in nemesia-demo and update the English and Japanese documentation together.

Compile an index and query it locally with the CLI:

```sh
cargo build --locked --release -p nemesia-cli
printf '%s\n' '{"id":1,"text":"Tokyo Station"}' > candidates.jsonl
./target/release/nemesia build --input candidates.jsonl --output suggestions.nmsi
./target/release/nemesia suggest --index suggestions.nmsi --query 'toky st'
```

Use a fresh output path for each invocation of `build` or `extract`. See [corpus preparation](docs/corpus.md) for the native Sudachi step. Build the read-only module with `cargo build --locked --release -p nemesia-wasm --target wasm32-unknown-unknown`. Bindings should be generated with wasm-bindgen 0.2.129 in the consuming project.
