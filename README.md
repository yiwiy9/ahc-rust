# AHC Rust Workspace

短期AtCoder Heuristic Contestの実装、ローカル計測、過去コード検索、復習配信を支える専用workspaceです。

競技環境はAI、OBS、AtCoderログインへ依存しません。配信環境とは確定したローカル計測 `results/latest.json` を介して連携します。競技側の履歴は別途 `results/events.jsonl` に残します。

## 基本操作

```bash
./ahc doctor
./ahc new ahc071
cd contests/ahc071
./ahc run 0 --solver greedy
./ahc bench --solver greedy --cases 10
./ahc save greedy --solver greedy --cases 10
./ahc add beam
./ahc export --solver greedy --clipboard
./ahc web --open
```

`cargo compete` と `oj` は使用しません。提出は `export` した単一RustファイルをWeb画面へコピーします。

## ディレクトリ

- `crates/ahc-cli`: コンテスト生成、実行、採点、保存、検索、export
- `templates`: 問題固有コードへ追加する探索テンプレート
- `contests`: Git管理する各AHCのコードと振り返り
- `docs`: 設計判断、操作方法、安全性の記録

生成物、計測結果、作業中snapshotはGit管理しません。残したい提出版や方針の異なる解だけ `solutions/` へ昇格します。

## 共通lib

共通libはAHC solverのCargo依存ではなく、検索・コピペ対象です。AHC workspaceと同じ親ディレクトリへcloneし、`workspace.toml` から参照します。

```text
ahc-workspace/
├── ahc-rust/
├── ahc-studio/
└── atcoder-lib/
```

現在のclone元は `https://github.com/yiwiy9/practice-algorithm-rust-snippets.git` です。競技前に `./ahc doctor` がcloneのrevisionとdirty状態を表示します。

詳しい当日手順は `docs/contest-day.md`、方式選択は `docs/strategy-guide.md`、バグ調査は `docs/debugging.md` を参照します。

## グローバルへの影響

このリポジトリはグローバルインストール、PATH変更、シェル設定変更を要求しません。`./ahc` はリポジトリ内のCLIをCargoで実行します。
