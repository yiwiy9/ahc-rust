# AHC Rust Workspace

短期AtCoder Heuristic Contestの実装、ローカル計測、過去コード検索、復習配信を支える専用workspaceです。

競技環境はAI、OBS、AtCoderログインへ依存しません。配信環境とは確定したローカル計測 `results/latest.json` を介して連携します。競技側の履歴は別途 `results/events.jsonl` に残します。

## いつもの操作

VS Codeでは **ahc-rustフォルダ単体** を開く。過去コードは `contests/`、共通libとABCコードは `references/` から読める。`⌘⇧F` でまとめて検索できる。

```sh
# 初回だけリンクを作成（この環境では設定済み。以降の更新操作は不要）
python3 scripts/setup-references.py
# コンテスト内で使う基本の4つ。bin名は省略するとa。
cd contests/ahc071
./scripts/build.sh a
./scripts/run-one.sh a 0
./scripts/debug.sh a 0
./scripts/run-all.sh a 10
./ahc export --solver a --clipboard
```

`run-all` は追加ツールなしの逐次計測。並列比較が欲しくなったときだけ `bench` / pahcerを使う。[エディタ・実行環境](docs/environment.md)に設定・依存・InputとStateの扱いをまとめている。

## コンテスト作成・追加操作

```bash
./ahc doctor
./scripts/install-pahcer.sh # 並列計測を使う場合だけ、事前に実行
./ahc new ahc071
cd contests/ahc071
# ここで問題固有の入出力・解の構築を実装する（下記の初回ガイド参照）。
./ahc run 0 --solver greedy
./ahc bench --solver greedy --cases 10 --threads 4
./ahc history --rank
./ahc save greedy --solver greedy --cases 10 --threads 4
./ahc add beam
./ahc export --solver greedy --clipboard
./ahc web --open
```

`run` は1 seedの実行・採点で、既定はreleaseビルドです。検算やpanicの調査には `run 0 --solver greedy --debug` を使います。`bench` はworkspace-localのpahcerによる並列比較です。pahcerが扱えない得点形式や障害時には `bench --builtin` で逐次runnerへ戻れます。生成直後のテンプレートは未実装なので、上記をそのまま順に実行しても合法な解にはなりません。

`cargo compete` と `oj` は使用しません。提出は `export` した単一RustファイルをWeb画面へコピーします。

## VS Code

`rust-analyzer` と CodeLLDB を推奨拡張として登録しています。`./ahc new <contest_id>` は、そのコンテストの `Cargo.toml` を rust-analyzer の対象へ自動追加します。コンテストを手動で削除した後は、ルートで次を一度実行して古い登録を掃除します。

```bash
./ahc vscode sync
```

Rustでは保存時に `rustfmt`、保存後に `cargo check` を実行します。補完候補は自動表示しない設定で、必要なときだけ `Ctrl+Space`（macOSでは `⌃Space`）で表示します。

本番はahc-rustフォルダ単体で開く。共通libは `references/atcoder-lib/`、ABCの過去実装は `references/abc/` のシンボリックリンクから読む。元の変更がそのまま見えるため同期操作は不要。VS Codeでは参照部分を読み取り専用にしている。

`atcoder-lib` の `#[snippet]` 定義は `.vscode/rust.code-snippets` へ生成済みである。Rustファイルで `bfs` などのprefixを書き、`⌃Space` から選んで挿入する。共通libを更新した後だけ、次で再生成する。

```bash
./scripts/sync-vscode-snippets.sh
```

このスクリプトはリポジトリ内の `.tools/bin/cargo-snippet` を使い、共通libは変更しない。初回だけ `.tools` へローカル導入が必要な場合は、表示されるコマンドに従う。

## ディレクトリ

- `crates/ahc-cli`: コンテスト生成、実行、採点、保存、検索、export
- `templates`: 問題固有コードへ追加する探索テンプレート
- `contests`: Git管理する各AHCのコードと振り返り
- `docs`: 設計判断、操作方法、安全性の記録

生成物、計測結果、作業中snapshotはGit管理しません。残したい提出版や方針の異なる解だけ `solutions/` へ昇格します。

## 共通lib

共通libの実体はABC側の `atcoder-rust/src/lib/src` の一つだけです。AHC側は `references/atcoder-lib` から検索・スニペット生成に使い、別cloneは持ちません。solverのCargo依存にはしません。

```text
ahc-workspace/
├── ahc-rust/
└── ahc-studio/
```

`./ahc doctor` はリンク先であるABC側libのrevisionとdirty状態を表示します。ABC側でメンテした内容は検索へそのまま反映されます。スニペットを再生成するときも同じ実体を読みます。

初めて問題を解くときは `docs/first-contest-guide.md`、当日の短い手順は `docs/contest-day.md`、方式選択は `docs/strategy-guide.md`、バグ調査は `docs/debugging.md` を参照します。

## グローバルへの影響

このリポジトリはグローバルインストール、PATH変更、シェル設定変更を要求しません。`./ahc` はリポジトリ内のCLIをCargoで実行し、pahcerも `.tools/` 内だけへ固定versionを入れます。
