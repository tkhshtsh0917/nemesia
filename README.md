# Nemesia

[日本語](README.ja.md)

Fast, typo-tolerant query autocompletion powered by Rust and WebAssembly. Nemesia suggests registered completions from the beginning of your input. Use it to fill a search box before sending the completed query to your existing search service.

## Quick start

Download the matching CLI from [GitHub Releases](https://github.com/tkhshtsh0917/nemesia/releases): `nemesia-macos-aarch64` for Apple Silicon or `nemesia-linux-x86_64` for Linux x86-64. In v0.1.0, these require macOS 11 or later and glibc 2.34 or later, respectively. For other environments, see [building from source](docs/usage.md#install-the-cli). Compare its SHA-256 with the matching entry in `SHA256SUMS`. Run these commands in a directory containing the downloaded binary:

```sh
case "$(uname -sm)" in
  'Darwin arm64') cli=./nemesia-macos-aarch64 ;;
  'Linux x86_64') cli=./nemesia-linux-x86_64 ;;
esac
chmod +x "$cli"
printf '%s\n' '{"id":1,"text":"Tokyo Station","weight":10}' > candidates.jsonl
"$cli" build --input candidates.jsonl --output suggestions.nmsi
"$cli" suggest --index suggestions.nmsi --query 'toky st'
```

```json
[{ "id": 1, "text": "Tokyo Station", "distance": 1 }]
```

Use a fresh output path if you run `build` again. This example uses a supplied candidate list. The [usage guide](docs/usage.md) also covers building from source, Japanese candidate extraction, and embedding Nemesia in a Rust application.

## Use it in a browser

Use the CLI output `suggestions.nmsi` and the WASM from the same release. The [browser quick start](https://github.com/tkhshtsh0917/nemesia-demo/blob/main/docs/browser-quickstart.md) shows how to generate JavaScript bindings, load the index, and display suggestions in an existing page. See the [live demo](https://tkhshtsh0917.github.io/nemesia-demo/).

## Documentation

- [Usage guide](docs/usage.md) for installation, CLI commands, and Rust integration.
- [Rust API reference](https://tkhshtsh0917.github.io/nemesia/) for the current published release.
- [Corpus preparation](docs/corpus.md) and [matching specification](docs/matching.md) for candidate data, normalization, and ranking.
- [Contributing](CONTRIBUTING.md) and [design and project documents](docs/README.md) for development, verification, and architecture.
- [Performance and ranking evaluation](https://github.com/tkhshtsh0917/nemesia-demo/blob/main/docs/performance.md) for measured results and reproducible workloads.

The name comes from the nemesia flower's association with tolerance. [Flower meaning](https://ozaki-flowerpark.co.jp/dictionary/7568/).

Code is available under the [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) license. See [NOTICE](NOTICE) for third-party notices.
