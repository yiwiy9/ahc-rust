# Introduction to Heuristics Contest — 参照用の完成例

旧atcoder-rust/src/contest/intro-heuristicsから解答5ファイルをコピーしたものです。理解済みの実装を新しい問題と照らし合わせるため、Rustコードとコメントは変更していません。旧環境もそのまま残しています。

## 読むファイル

| ファイル | 内容 |
| --- | --- |
| `src/bin/beam_search.rs` | A問題。貪欲・ビームサーチと問題固有のState・評価 |
| `src/bin/local_search.rs` | A問題。全体再計算で採点する局所探索 |
| `src/bin/local_search_fast.rs` | A問題。得点の差分更新を加えた局所探索 |
| `src/bin/b.rs` | B問題。開催予定の採点 |
| `src/bin/c.rs` | C問題。開催予定変更後の得点更新 |

汎用テンプレートはリポジトリ直下のtemplatesを使用してください。このディレクトリはintro固有の入力・解・採点・近傍の完成例です。新テンプレートとはインターフェースが異なるため、処理の意味を確認して必要部分を利用します。

## 実行・比較・保存

他のコンテストと同じく、このディレクトリへ移動して操作します。

```sh
cd contests/intro-heuristics
./ahc run 0 --solver beam_search
./ahc run 0 --solver local_search
./ahc run 0 --solver local_search_fast
./ahc bench --solver local_search --cases 10 --threads 4
./ahc history --rank
./ahc save local_search --solver local_search --cases 10 --threads 4
./ahc export --solver local_search --clipboard
./ahc web --open
```

`--solver`にはA問題の3つのファイル名（拡張子なし）を指定します。出力・ログ・比較結果・保存先は他のコンテストと同じです。pahcer未導入時は `./ahc bench --builtin --solver local_search --cases 10` も使えます。

公式toolsのソース・依存ロック・seed一覧は旧環境から引き継ぎ、実行に必要なケースは新ハーネスが初回に生成します。cargo-compete設定、旧scripts、ahc-kit、過去の計測結果は持ち込んでいません。tools/Cargo.tomlに独立workspaceの宣言を追加した以外、公式ツールのコードは変更していません。

B・Cは入力形式が異なるため、A用ハーネスの採点対象ではありません。参照・単体実行用です。

```sh
cargo build --release --bins
cargo run --release --bin b < /path/to/b-input.txt
cargo run --release --bin c < /path/to/c-input.txt
```

提出用に使う場合は、問題ごとの規則・依存ライブラリと、各コードの資料参照コメントを確認してください。
