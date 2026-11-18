# Nemesia

[English](README.md)

RustとWebAssemblyで動く、高速で誤入力に強いクエリ補完。入力の先頭から登録済みの候補を探します。検索ボックスの入力を補完し、確定したクエリを既存の検索サービスへ渡す用途に使えます。

## クイックスタート

[GitHub Releases](https://github.com/tkhshtsh0917/nemesia/releases)から対応するCLIをダウンロードします。Apple Siliconは `nemesia-macos-aarch64`、Linux x86-64は `nemesia-linux-x86_64` です。ダウンロードしたファイルのSHA-256を `SHA256SUMS` の該当行と照合してください。取得したバイナリがあるディレクトリで次を実行します。

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

`build` をもう一度実行する場合は、新しい出力パスを指定してください。この例では、あらかじめ用意した候補リストを使います。[利用ガイド](docs/ja/usage.md)では、ソースからのビルド、日本語の候補抽出、Rustアプリケーションへの組み込みも説明します。

## ブラウザで使う

CLIが出力した `suggestions.nmsi` と、同じリリースのWASMを使います。[ブラウザのクイックスタート](https://github.com/tkhshtsh0917/nemesia-demo/blob/main/docs/ja/browser-quickstart.md)では、JavaScriptバインディングを生成し、索引を読み込んで、既存のページに候補を表示する方法を説明します。リリースの配信後は[デモ](https://tkhshtsh0917.github.io/nemesia-demo/)も利用できます。

## ドキュメント

- インストール、CLI、Rust連携は[利用ガイド](docs/ja/usage.md)。
- 現在の公開版の[Rust APIリファレンス](https://tkhshtsh0917.github.io/nemesia/)。
- 候補データ、正規化、順位付けは[コーパス準備](docs/ja/corpus.md)と[照合仕様](docs/ja/matching.md)。
- 開発・検証・設計は[コントリビュート](CONTRIBUTING.ja.md)と[設計・プロジェクト文書](docs/ja/README.md)。
- 実測値と再現可能なワークロードは[性能と順位付けの評価](https://github.com/tkhshtsh0917/nemesia-demo/blob/main/docs/ja/performance.md)。

名前はネメシアの花言葉の一つである「包容力」に由来します。[花言葉の出典](https://ozaki-flowerpark.co.jp/dictionary/7568/)。

コードは[MIT](LICENSE-MIT)または[Apache-2.0](LICENSE-APACHE)ライセンスで利用できます。第三者のライセンス表記は[NOTICE](NOTICE)を参照してください。
