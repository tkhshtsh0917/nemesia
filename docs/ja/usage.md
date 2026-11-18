# Nemesiaの利用ガイド

[English](../usage.md) · [クイックスタート](../../README.ja.md#クイックスタート)

## CLIを用意する

[GitHub Releases](https://github.com/tkhshtsh0917/nemesia/releases)から利用環境に対応するバイナリを取得します。同じタグから `SHA256SUMS` も取得してください。例えばApple Siliconでは、次のコマンドで実際のハッシュと期待値の行を表示できます。

```sh
shasum -a 256 nemesia-macos-aarch64
awk '$2 == "nemesia-macos-aarch64"' SHA256SUMS
```

ハッシュが一致することを確認し、バイナリに実行権限を付け、パスを指定して実行します。ソースからビルドする場合は、リリースタグをcheckoutし、リポジトリが固定したRustツールチェーンを使います。

```sh
git clone --branch v0.1.0 https://github.com/tkhshtsh0917/nemesia.git
cd nemesia
cargo build --locked --release -p nemesia-cli
./target/release/nemesia --version
```

以下ではPATH上のコマンドを `nemesia` とします。代わりにリリースバイナリのフルパスや `./target/release/nemesia` を指定できます。

## 索引を作って照合する

入力は二つの方法から選べます。候補リストを渡す場合はUTF-8のJSONLを使い、各行に `id` と表示文字列 `text`、必要に応じて `weight` と `aliases` を含めます。例と順位規則は[照合仕様](matching.md)を参照してください。日本語の文書から抽出する場合は、各行に `{"id":"doc-1","text":"りんごを食べた。"}` のように記載し、既存のSudachiシステム辞書を指定します。辞書と設定の詳細は[コーパス準備](corpus.md#cliとデモの準備)を参照してください。

```sh
nemesia extract --input documents.jsonl --dictionary system.dic --output candidates.jsonl
nemesia build --input candidates.jsonl --output suggestions.nmsi
nemesia inspect --index suggestions.nmsi
nemesia suggest --index suggestions.nmsi --query 'たべ' --limit 8
```

候補を直接用意する場合は `extract` を省略します。ビルダーは新しい出力ファイルを作ります。`suggest` は `{id, text, distance}` のオブジェクトを順位順の配列で返します。`text` が提示する補完文字列です。空の入力には空配列を返します。不正な入力、互換性のない索引、処理量上限の超過はエラーになります。独自の変換入力を渡すには、`--query 'びーt' --derived bi-t` のように `--derived` を使います。

## Rustに組み込む

配布済みのCLIとWASMはそのまま利用できます。Rustからエンジンを直接呼び出す場合は、同じリリースタグに固定したソース依存を使います。ワークスペースのcrateはGitHub経由で配布しています。

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

索引を一度開いて検証し、複数のクエリに再利用してください。crateとAPIの詳細は[Rust APIリファレンス](https://tkhshtsh0917.github.io/nemesia/)を参照してください。Webアプリケーションでは[ブラウザのクイックスタート](https://github.com/tkhshtsh0917/nemesia-demo/blob/main/docs/ja/browser-quickstart.md)を使えます。
