# 構成と境界

```text
ahc-workspace/
├── ahc-rust/                 競技だけで完結するGitリポジトリ
│   ├── crates/ahc-cli/       生成・1 seed実行・計測adapter・保存・export
│   ├── templates/            新しい問題へコピーする型
│   ├── contests/ahc070/      問題固有コードと検索用の過去解
│   └── docs/
└── ahc-studio/               OBS画面・順位・AI操作のGitリポジトリ
```

`ahc-rust` は単独でコンテスト参加できます。`ahc-studio` は競技コードを起動・変更せず、現在は `results/latest.json` だけを読みます。競技側の `results/events.jsonl` は将来の分析や演出に使える履歴として残します。

各contestも独立したCargo workspaceです。これによりcontestごとの `Cargo.lock` と `target` が分かれ、将来 `contests` や `templates` を別リポジトリへ移してもコードの依存方向は変わりません。

共通libは `references/atcoder-lib` → ABC側の `atcoder-rust/src/lib/src` を参照する。AHC側に別cloneは置かない。

## 問題ごとに編集する場所

- `problem.rs`: Input / Output / 読み書き / 正確な得点 / 出力検証
- `constructive_state.rs`: 構築型のState / Action / 合法手 / 1手 / 評価
- `local_search_state_*.rs`: 完成解のParameters / Move / 再構築または差分更新
- `src/bin/*.rs` 冒頭の定数: 幅、反復数、温度、時間

## 原則として編集しない場所

- `framework/constructive.rs`: 貪欲の進行
- `framework/random_search.rs`: 複数の乱択構築
- `framework/beam*.rs`: 候補の展開と枝刈り
- `framework/local_search.rs`: 山登り・焼きなましの採否と時間管理
- 各binのI/Oから探索エンジンを呼ぶ流れ

多数seedの実行・相対評価・履歴はpahcerへ委譲し、`ahc-cli` は問題ごとの設定生成と共通JSONへの変換を担当します。pahcerに適合しない問題でもbuiltin runnerへ戻れるため、外部ツールが競技継続の必須条件にはなりません。

抽象化の目的は全問題を同じStateで解くことではありません。探索ループのバグと問題固有ロジックのバグを分け、配信中に読む範囲を狭めることです。
