# Nemesia

[日本語](README.ja.md)

Fast, typo-tolerant query autocompletion powered by Rust and WebAssembly.

Nemesia is designed to suggest words extracted from the downstream search corpus, or other registered completions from the first few characters, even when the input contains a small typo. A native CLI builds the suggestion index. The browser loads it and computes suggestions locally. This repository contains the Rust engine, CLI and WASM. The TypeScript adapter, demo and browser tests belong to [nemesia-demo](https://github.com/tkhshtsh0917/nemesia-demo).

**Status:** v0.1 implementation and local verification. See the roadmap for hosted publication status.

- [Architecture](docs/architecture.md) describes the two repositories, index, runtime limits and artifact contract.
- [Matching specification](docs/matching.md) defines the candidate model, prefix distance and ranking.
- [Corpus preparation](docs/corpus.md) defines Sudachi extraction and the ranking metadata.
- [Reuse assessment](docs/reuse.md) records what can carry over from Asearch and its demo.
- [Contributing](CONTRIBUTING.md) explains the development and quick-start commands.
- [Roadmap](docs/roadmap.md) defines milestones, acceptance checks and benchmark targets.

The name comes from the nemesia flower's association with tolerance, an apt description of giving users room for a typo. [Flower meaning](https://ozaki-flowerpark.co.jp/dictionary/7568/).
