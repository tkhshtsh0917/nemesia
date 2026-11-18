# コントリビュート

[English](CONTRIBUTING.md)

このチェックアウトで `rustup show` と `mise run check` を実行します。Rustupが固定したツールチェーンを選び、miseがCIと同じ検証を実行します。ブラウザ側の変更はnemesia-demoで行い、日英の文書を同時に更新してください。

CLIでローカルの索引を作り、照合できます。

```sh
cargo build --locked --release -p nemesia-cli
printf '%s\n' '{"id":1,"text":"Tokyo Station"}' > candidates.jsonl
./target/release/nemesia build --input candidates.jsonl --output suggestions.nmsi
./target/release/nemesia suggest --index suggestions.nmsi --query 'toky st'
```

`build` や `extract` を呼び出すたびに新しい出力パスを指定します。ネイティブのSudachi工程は[コーパスの作り方](docs/ja/corpus.md)を参照してください。読み取り用モジュールは `cargo build --locked --release -p nemesia-wasm --target wasm32-unknown-unknown` でビルドできます。利用側のプロジェクトでwasm-bindgen 0.2.129を使ってバインディングを生成してください。
