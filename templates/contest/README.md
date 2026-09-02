# __CONTEST_ID__

## 最初に埋める順番

1. `src/problem.rs` の `Input` / `Output` / `read_input` / `print_answer`
2. `validate_output` と、可能なら小さい入力で照合できる `calculate_score`
3. `src/constructive_state.rs` の合法手・1手進める処理・評価値
4. `src/bin/greedy.rs` で正の得点を作る
5. 問題の形を見て `./ahc add random-search`、`beam`、`local-search-direct`、`local-search-rebuild` を追加する

`evaluated_value` は常に「大きいほどよい」値にそろえる。公式得点が小さいほどよい問題なら符号を反転する。

## よく使うコマンド

```sh
./ahc doctor
./ahc run 0 --solver greedy
./ahc bench --solver greedy --cases 10
./ahc save greedy --solver greedy --cases 10
./ahc export --solver greedy --clipboard
```

`src/bin/a.rs` は最初の提出入口です。試行錯誤が増えたら名前つきbinを追加し、`solutions/` とスナップショットを検索可能な履歴として残します。
