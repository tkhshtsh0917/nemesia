# Use Nemesia

[日本語](ja/usage.md) · [Quick start](../README.md#quick-start)

## Install the CLI

Use the binary for your system from [GitHub Releases](https://github.com/tkhshtsh0917/nemesia/releases). Obtain `SHA256SUMS` from the same tag. For example, on Apple Silicon these commands display the actual hash and the expected row:

```sh
shasum -a 256 nemesia-macos-aarch64
awk '$2 == "nemesia-macos-aarch64"' SHA256SUMS
```

Check that the hashes match, give the binary execute permission, and run it by path. To build from source, check out a release tag and use the repository's pinned Rust toolchain:

```sh
git clone --branch v0.1.0 https://github.com/tkhshtsh0917/nemesia.git
cd nemesia
cargo build --locked --release -p nemesia-cli
./target/release/nemesia --version
```

The following examples use `nemesia` as a command on your PATH. You can also substitute the full path to the release binary or `./target/release/nemesia`.

## Build and query an index

Choose one of two inputs. With a candidate list, supply UTF-8 JSONL with one `id` and display `text` per line, optionally with `weight` and `aliases`. Examples and ordering rules are in the [matching specification](matching.md). With Japanese source documents, supply one `{"id":"doc-1","text":"りんごを食べた。"}` per line and an existing Sudachi system dictionary. See [corpus preparation](corpus.md#cli-and-demo-preparation) for dictionary and policy details.

```sh
nemesia extract --input documents.jsonl --dictionary system.dic --output candidates.jsonl
nemesia build --input candidates.jsonl --output suggestions.nmsi
nemesia inspect --index suggestions.nmsi
nemesia suggest --index suggestions.nmsi --query 'たべ' --limit 8
```

Skip `extract` when providing candidates yourself. The builder creates a new output file. `suggest` returns an array of `{id, text, distance}` objects, ordered by rank. Use `text` as the proposed completion. Blank input returns an empty array. Invalid input, incompatible indexes or exhausted work limits return an error. To supply your own alternate query, pass `--derived`, such as `--query 'びーt' --derived bi-t`.

## Embed in Rust

The published CLI and WASM are ready-to-use artifacts. To call the engine directly from Rust, use a source dependency pinned to the same release tag. The workspace crates are distributed through GitHub rather than crates.io.

```toml
[dependencies]
nemesia-engine = { git = "https://github.com/tkhshtsh0917/nemesia", tag = "v0.1.0" }
nemesia-index = { git = "https://github.com/tkhshtsh0917/nemesia", tag = "v0.1.0" }
```

```rust
use nemesia_engine::suggest;
use nemesia_index::binary::Index;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Index::open(std::fs::read("suggestions.nmsi")?)?;

    for hit in suggest(&index, "toky st", None, 8)? {
        println!("{}", hit.text);
    }
    Ok(())
}
```

Open a validated index once and reuse it across queries. See the [Rust API reference](https://tkhshtsh0917.github.io/nemesia/) for crate and boundary details. For a web application instead, follow the [browser quick start](https://github.com/tkhshtsh0917/nemesia-demo/blob/main/docs/browser-quickstart.md).
