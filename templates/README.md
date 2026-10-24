# テンプレートの生成元

ここは保守用に分割したソースです。競技中に貼るコードは、新規生成したコンテスト内の `samples/*.rs` を使います。そちらはincludeを展開済みなので、一枚のbinへそのまま貼れます。

```sh
./scripts/new.sh intro-practice --example
```

- `contest/src/bin/a.rs` + `contest/src/base.rs`: 初期の貪欲版の組み立て。
- `contest/src/problem.rs`: proconio・型付き入出力・introの正確な得点。
- `contest/src/constructive_state.rs`: 動く構築状態と軽量評価。
- `framework/`: ビーム・乱択・山登り / 焼きなましの探索ループ。
- `states/`: introで動く局所探索の状態・近傍・undo。
- `snippets/`: a.rsの複製へ貼る追加部分の組み立て。
- `solvers/`: introの入出力・Stateも含む完成例の組み立て。

newは初期a.rs、貼る部分、完成例を一度だけ展開します。その後は各binを直接編集し、元テンプレートから再生成しません。

変更後は `python3 scripts/check-templates.py` で、実際の生成・コピペ・完成例・公式tools採点・export後のコンパイルを確認します。
