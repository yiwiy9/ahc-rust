# 短期AHC 当日手順

## 開始直後

```sh
cd ahc-rust
./scripts/install-pahcer.sh # 初回だけ。事前に実行する
./ahc new ahc071
cd contests/ahc071
./ahc doctor
```

公式ツールURLを自動検出できない場合だけ、問題ページのZIP URLを指定します。

```sh
./ahc tools --url 'https://img.atcoder.jp/.../tools.zip'
```

提出は `./ahc export --clipboard` の後、AtCoderのWeb画面へ貼り付けます。ログイン情報をCLIへ渡しません。

## 実装順

1. 問題文を読み、制約・得点の向き・出力の自由度を `ahc.toml` とメモへ書く
2. Input / Output / `read_input` / `print_answer` を0-indexedで実装する
3. `valid.rs` で形式上有効な最小解を出す
4. `validate_output` と、遅くても正しい `calculate_score` を書く
5. `ConstructiveState` を埋め、`greedy` で正の得点を取る
6. seed 0だけでビジュアライザを見て、方針のズレを直す
7. seed 0〜9を固定してbaselineを保存する
8. 問題の形から、乱択・ビーム・局所探索の一つを追加する
9. 同じseed集合で比較し、改善した断面だけ `save` する
10. debug検証を通してからreleaseを提出する

必要なときだけ `../../scripts/check-linux.sh ahc071 a` でRust 1.89/Linux上のコンパイルも確認します。通常の試行錯誤はMacネイティブの方が軽く、Dockerは必須ではありません。

## コマンドの短いループ

```sh
./ahc run 0 --solver a
./ahc vis 0 --solver a --open
./ahc bench --solver a --cases 10 --threads 4
./ahc history --rank
./ahc save greedy --solver a --cases 10 --threads 4
./ahc compare 1 2
./ahc export --solver a --clipboard
./ahc web --open
```

`run` は毎回 `cargo build` を呼びますが、Cargoの差分ビルドを使います。コードを変えていない連続確認では `--no-build` を付けられます。

`bench` はpahcerを使って並列実行します。配信と同時なら `--threads 4` など明示的に抑え、速度だけを測りたい時は `--threads 1` にします。pahcerは0点をWAとして扱うため、0点・負値が正常な問題や設定が合わない時は `--builtin` へ切り替えます。

相対スコアの基準を固定してA/B比較する場合は `--freeze-best-scores`、保存済みの解同士を順位スコアで比べる場合は `./ahc history --rank` を使います。これはローカル履歴内の順位で、AtCoder順位ではありません。

履歴を比較する時は `cases` と `threads` が同じ行だけを比べます。特に2ケースの試運転と100ケースの本計測を順位順に混ぜて解釈しないようにします。

## 時間探索を再現可能にする

通常は時間制限で探索します。デバッグや変更前後の比較では、局所探索テンプレートに用意した固定反復へ切り替えます。

```sh
AHC_ITERATIONS=10000 ./ahc run 0 --solver local_search_rebuild --debug
```

乱数seed、入力seed、反復回数を固定すれば、バグの再現と二分探索がしやすくなります。

## 試行錯誤を残す

`save <name>` の名前だけ人が決めます。候補は次からコピーできます。

```text
valid
greedy
random
beam
beam-fast
hill-1change
hill-swap
sa-1change
sa-swap
sa-mixed
rebuild-baseline
delta-update
final
```

完全な作業状態は無視対象の `snapshots/`、検索・Git管理する自己完結コードは `solutions/` に自動保存されます。コンテスト中にGit操作は不要です。
