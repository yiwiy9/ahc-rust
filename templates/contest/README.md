# __CONTEST_ID__

初めてこの問題を解くときは、[初回AHC: 手を動かす手順書](../../docs/first-contest-guide.md) を開き、「編集 → 実行 → 確認 → 修正」の順に進める。

## 初回の最短ルート

最初から探索テンプレートを埋める必要はありません。まず次だけ行います。

1. `src/problem.rs` の `Input` / `Output` / `read_input` / `print_answer` を問題文に合わせる
2. `src/bin/valid.rs` で、とても弱くても**合法な出力**を一つ作る
3. `./ahc run 0 --solver valid --debug` と可視化で、公式toolsがその出力を受け取ることを確認する
4. 本命の解を、下の道Aまたは道Bの片方で作る

`tools/`、`target/`、`out/`、`results/`、`submit/` は編集しません。公式toolsまたは自動生成物です。

`valid.rs` はAtCoderへの提出そのものには必須ではありません。ただし初回は、最小の合法解を公式toolsへ渡す練習として必ず通します。

## 本命の解は二択

### 道A: `a.rs` に直接書く

一つのファイルで解を作りたい場合は、`src/bin/a.rs` を自分の `main` と `solve` に書き換えます。`constructive_state.rs`、`greedy.rs`、`framework/` は不要です。最後は `./ahc export --solver a --clipboard` で提出用を作ります。

### 道B: 貪欲テンプレートを使う

解を一手ずつ組み立てる問題では、`src/constructive_state.rs` を編集します。ここへ「途中の解」「次に選べる候補」「1手進める処理」「候補比較用の値」を書きます。

| ファイル | 編集するか |
| --- | --- |
| `src/constructive_state.rs` | 編集する |
| `src/bin/greedy.rs` | 通常は編集しない |
| `src/framework/constructive.rs` | 編集しない |
| `src/bin/a.rs` | 通常は編集しない。`greedy.rs` を提出入口にしている |

この道でも実行・提出は `--solver a` でよいです。

## 後回しでよいもの

- `validate_output`: 最小解が通った後に、個数・値域などから追加する
- `calculate_score`: 探索で候補を比べる前に実装する
- `./ahc add beam` 等の探索: 貪欲や単純解を可視化して、改善したい失敗が見えてから使う
- `bench` / `save`: 1ケースが分かってから使う

## 構築テンプレートを選んだ場合の実装順

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
./ahc bench --solver greedy --cases 10 --threads 4
./ahc history --rank
./ahc save greedy --solver greedy --cases 10 --threads 4
./ahc export --solver greedy --clipboard
```

`bench` はpahcerで並列実行し、seedごとのローカルbestに対する相対値も保存します。0点・負値が正常な問題やpahcerで扱えない公式toolsでは `--builtin` を付けます。

`src/bin/a.rs` は最初の提出入口です。試行錯誤が増えたら名前つきbinを追加し、`solutions/` とスナップショットを検索可能な履歴として残します。
