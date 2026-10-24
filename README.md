# AHC Rust Workspace

短期AHCの実装・ローカル採点・比較・保存を行うworkspaceです。競技中の操作はAIや配信環境に依存しません。

テンプレートや開発用コードの整備にはAI支援を利用しています。実行スクリプトと生成される解法には、AIサービスへ問い合わせる機能はありません。各コンテストの保存コードと、終了後に整備した汎用テンプレートは別のファイルとして管理しています。

## 最初に開くもの

VS Codeでは **ahc-rustフォルダ単体** を開きます。コンテスト中に編集するのは `contests/<ID>/src/bin/a.rs`。入出力・State・探索関数を一枚にまとめています。探索部分も自由に変更できます。

- [当日の手順](docs/contest-day.md): コマンドと実装の順番
- [初回ガイド](docs/first-contest-guide.md): 動く例から問題固有コードへの置き換え
- [探索ガイド](docs/strategy-guide.md): ビーム・乱択・焼きなましの選び方
- [環境と検証](docs/environment.md): エディタ・依存・開発用チェック

## 普段使うコマンド

日常操作は `./scripts/...` に統一します。`ahc` はスクリプトが呼ぶ内部CLIで、同じ操作を二通り覚える必要はありません。

リポジトリ直下で:

```sh
./scripts/doctor.sh
./scripts/new.sh ahcNNN
```

問題公開後に作成すると、公式toolsを取得し、可視化・入力生成に必要なものを準備します。生成する解法は **intro-heuristics Aで実際に動く貪欲の例** なので、新しい問題では `a.rs` の「変更1〜6」を置き換えます。新規問題の解法が自動で完成するわけではありません。

コンテスト内で:

```sh
cd contests/ahcNNN
./scripts/run-one.sh a 0          # releaseで1ケース・採点・可視化
./scripts/debug.sh a 0            # 検算とバックトレース
./scripts/run-all.sh a 10         # seed 0〜9を逐次計測
./scripts/save.sh a greedy-v1 10  # 採点し、その時点のソースと結果を保存
./scripts/export.sh a            # submit/a.rsへ保存し、クリップボードにもコピー
```

引数は **bin名 → seed / 件数**。`build.sh a` でビルドだけもできます。実行時にも自動ビルドするため、毎回buildを挟む必要はありません。並列比較は同じ入口の `./scripts/run-all.sh a 10 --parallel --threads 4` で選べます（事前に `./scripts/install-pahcer.sh` が必要）。

保存は同期実行です。成功表示を確認してから次の案へ進みます。単なるrunではソースの版は保存されません。一覧・比較・復元は `list.sh`・`compare.sh <左ID> <右ID>`・`restore.sh <ID>`。復元前のソースは自動退避します。

## 貪欲から別の探索へ

```sh
cp -n src/bin/a.rs src/bin/beam.rs
# samples/beam.rsをbeam.rs末尾へ貼り、コメントに従ってmainを変更する
./scripts/run-all.sh beam 10
./scripts/export.sh beam
```

Cargoが `src/bin/<名前>.rs` を自動認識します。登録コマンドは不要です。生成後のソースはビルド・実行時に再生成しません。

`samples/` は自分の実装に貼る追加部分、`samples/complete/` はintroの入出力とStateも含む動く完成例です。貪欲の自作Stateを維持したいときは、追加部分だけを使います。焼きなましの近傍は問題ごとに書き換えます。

## 動く環境をそのまま復習する

通信せずに、生成から採点まで試せるintro用の練習環境も作れます。

```sh
./scripts/new.sh intro-practice --example
cd contests/intro-practice
./scripts/run-one.sh a 0
```

この例では既存のintro公式toolsをコピーします。既存の `contests/intro-heuristics` の解法は変更しません。

## 過去コードと共通lib

過去のAHCは `contests/`、共通libとABCは `references/` から `⌘⇧F` で検索できます。別端末での初回だけ `python3 scripts/setup-references.py` でリンクを作ります。元の変更はそのまま反映され、日常の同期は不要です。

共通libの実体はABC側の `atcoder-rust/src/lib/src` 一つだけです。solverのCargo依存にはせず、必要な実装をコピーします。スニペットは `⌃Space` から挿入でき、共通lib更新後だけ `./scripts/sync-vscode-snippets.sh` で再生成します。

新しいcontestはrust-analyzerへ自動登録します。手動削除後の整理は `./scripts/vscode-sync.sh`。生成物、計測結果、作業snapshotはGit管理せず、保存した解の検索用断面は `solutions/` に残します。

配信環境は別repoの `ahc-studio`。競技環境は確定した `results/latest.json` を出力し、配信環境が読みます。
