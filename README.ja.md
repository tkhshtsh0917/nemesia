# Nemesia

[English](README.md)

RustとWebAssemblyで動く、高速で誤入力に強いクエリ補完。

Nemesiaは、最初の数文字に小さな誤入力があっても、後段の検索対象から抽出した語や、別途登録した補完候補を提示するためのライブラリです。ネイティブCLIで補完索引を生成し、ブラウザで読み込んで候補を端末内で計算する設計です。このリポジトリではRustエンジン、CLI、WASMを設計します。TypeScriptアダプター、デモ、ブラウザテストは [nemesia-demo](https://github.com/tkhshtsh0917/nemesia-demo) が担当します。

**現在の状況**は設計段階です。エンジン、リリース成果物、性能目標は、これから実装・計測します。

- [アーキテクチャ](docs/ja/architecture.md)では、両リポジトリ、索引、実行時上限、成果物の契約を説明します。
- [照合仕様](docs/ja/matching.md)では、候補のデータモデル、接頭辞距離、順位を定義します。
- [コーパスの作り方](docs/ja/corpus.md)では、Sudachiでの抽出と順位付けのメタデータを定義します。
- [流用評価](docs/ja/reuse.md)では、Asearchとそのデモから引き継げるものを整理します。
- [ロードマップ](docs/ja/roadmap.md)では、マイルストーン、完了条件、ベンチマーク目標を定義します。

名前はネメシアの花言葉の一つである「包容力」に由来します。少しの打ち間違いを受け止める補完を表しています。[花言葉の出典](https://ozaki-flowerpark.co.jp/dictionary/7568/)。
