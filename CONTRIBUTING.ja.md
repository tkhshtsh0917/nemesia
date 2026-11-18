# コントリビュート

[English](CONTRIBUTING.md) · [Nemesiaの利用ガイド](docs/ja/usage.md)

このリポジトリはRustの照合、索引生成、ネイティブのSudachi抽出、CLI、WASM ABIを担当します。JavaScript統合、デモUI、ブラウザテストは [nemesia-demo](https://github.com/tkhshtsh0917/nemesia-demo) が担当します。

## 準備と変更の検証

Rustupとmiseをインストールし、チェックアウトしたディレクトリで次を実行します。Rustupは `rust-toolchain.toml` のバージョンとコンポーネントを使います。

```sh
rustup show
mise run check
mise run docs
```

`check` はrustfmt、Clippy、ロック済みのワークスペーステストを実行します。`docs` は公開Rust APIリファレンスを生成し、rustdocの警告があればエラーにします。エンジンのリファレンスは `target/doc/nemesia_engine/index.html` から確認できます。変更に関係する仕様は[ドキュメント一覧](docs/ja/README.md)から探せます。

WASMの変更では、ターゲット別に検査し、デモ側で成果物をテストします。

```sh
cargo clippy --locked -p nemesia-wasm --target wasm32-unknown-unknown -- -D warnings
cargo build --locked --release -p nemesia-wasm --target wasm32-unknown-unknown
```

パーサーやファイル配置を変更した場合は、Rust CIのワークフローで固定したnightlyツールチェーンとcargo-fuzzを使い、`bash tools/fuzz-index.sh 20` を実行します。

## プルリクエストの準備

テストは観察できる挙動を対象にします。照合の変更は参照実装と結果の順序を比較します。形式の変更は[NMSIのバイト配置](docs/ja/format.md)を更新し、不正な入力も検証します。英語版と日本語訳は同じ変更で更新してください。APIコメントには挙動、入力、所有権、上限、エラーを書きます。プルリクエストには実行した検証と結果を記載してください。
