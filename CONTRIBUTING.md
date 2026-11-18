# Contributing

[日本語](CONTRIBUTING.ja.md) · [Using Nemesia](docs/usage.md)

This repository owns Rust matching, index construction, native Sudachi extraction, the CLI, and the WASM ABI. The JavaScript integration, demo UI, and browser tests live in [nemesia-demo](https://github.com/tkhshtsh0917/nemesia-demo).

## Set up and check a change

Install Rustup and mise, then run these commands from the checkout. Rustup selects the version and components pinned in `rust-toolchain.toml`.

```sh
rustup show
mise run check
mise run docs
```

`check` runs rustfmt, Clippy and the locked workspace tests. `docs` builds the public Rust API reference and treats rustdoc warnings as errors. Open `target/doc/nemesia_engine/index.html` to preview the engine reference. The [document index](docs/README.md) maps changes to the relevant specification.

For a WASM change, check the target separately and test the artifact in the demo:

```sh
cargo clippy --locked -p nemesia-wasm --target wasm32-unknown-unknown -- -D warnings
cargo build --locked --release -p nemesia-wasm --target wasm32-unknown-unknown
```

For parser or layout changes, run `bash tools/fuzz-index.sh 20` with the nightly toolchain and cargo-fuzz version pinned in the Rust CI workflow.

## Prepare a pull request

Keep the tests focused on observable behavior. Matching changes should verify output order against the reference implementation. Format changes should update the [NMSI layout](docs/format.md) and cover invalid input. Update the English source and its Japanese translation together. Keep API comments focused on behavior, inputs, ownership, limits, and errors. Record any checks you ran and their results in the pull request.
