# 0001: Workspace境界

## 決定

- AHC frameworkと各contestは同じ `ahc-rust` リポジトリで管理する。
- 配信画面、順位取得、AI連携は `ahc-studio` リポジトリへ分離する。
- 両者はJSONファイルの契約だけで接続する。
- 既存のABC環境は変更しない。
- 共通libはABC側の実体だけを持ち、AHCの `references/atcoder-lib` からリンクで検索・スニペット生成に使う。solverのビルド依存にはしない。
- Macネイティブ実行を標準とし、DockerはLinux互換確認用の任意機能とする。
- `cargo compete`、`oj`、AtCoderログイン情報へ依存しない。

## 理由

過去AHCの実装とコメントを同一リポジトリ内で検索し、問題固有コードを必要な単位で再利用するため。配信やAIが停止しても競技を継続できるよう、競技側にはネットワーク連携を持ち込まない。

## 分離可能性

`framework` と `templates` はcontestを参照しない。`ahc-studio` は現在 `results/latest.json` だけを読み、solverやrunnerを直接操作しない。競技側の `events.jsonl` は、必要になれば同じデータ境界のまま連携へ追加できる。
