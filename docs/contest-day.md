# 短期AHC 当日手順

コンテスト固有の問題名・時刻・規則は、毎回公式ページで確認してこの手順の `<contest_id>` を置き換える。これは開始前に用意する常設の操作メモであり、コンテスト終了後も削除しない。

## 開始前

```sh
cd /Users/yiwiy/Codes/atcoder/ahc-workspace/ahc-rust
./ahc doctor
```

`Ready.` と出ること、`pahcer` が `[ok]` であることを確認する。VS Code はこの `ahc-rust` ディレクトリを開く。

短期AHCでは生成AI利用規則を必ず公式ページで確認する。生成AIが原則禁止の回では、開始後に対話型生成AIを問題理解・方針・実装・デバッグ・実行結果の分析に使わない。事前に公開したコードテンプレートを使う場合だけ、提出コード内の対応するURL注記を残す。事前に作った操作メモ・学習ノートへはこの注記は不要である。

## 開始直後

```sh
cd ahc-rust
./scripts/install-pahcer.sh # 初回だけ。事前に実行する
./ahc new <contest_id>
cd contests/<contest_id>
./ahc doctor
```

開始前に公式ツールURLを自動検出できなければ、`new` はディレクトリを残さず失敗する。開始後にURL自動検出だけが失敗した場合はディレクトリを作り、`cd contests/<contest_id> && ./ahc tools --url <URL>` を案内する。URLを明示する場合は、開始時刻に関係なく使える。

```sh
./ahc tools --url 'https://img.atcoder.jp/.../tools.zip'
# または、作成時に明示する:
./ahc new <contest_id> --tools-url 'https://img.atcoder.jp/.../tools.zip'
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

必要なときだけ `../../scripts/check-linux.sh <contest_id> a` でRust 1.89/Linux上のコンパイルも確認します。通常の試行錯誤はMacネイティブの方が軽く、Dockerは必須ではありません。

## 1ケース・可視化・エラー調査

```sh
./ahc run 0 --solver greedy --debug
./ahc vis 0 --solver greedy --debug --open
```

`run` はseed 0について、debugビルド、solver実行、公式採点、HTML可視化を行う。可視化では得点だけでなく、行動の軌跡・盤面・制約・意図した構造を確認する。

失敗時は標準エラーログを見る。デバッグ表示は提出出力を壊さない `eprintln!` を使い、`println!` には解だけを出す。

```sh
less results/logs/greedy/debug/0000.log
RUST_BACKTRACE=1 ./ahc run 0 --solver greedy --debug --no-vis
```

## 得点と「意図した解」を分けて検証する

得点一致だけでは不十分である。debug時に、次を別々に確認する。

1. `validate_output` が出力形式・値域・個数・制約を検証する
2. 出力した行動を先頭から再生して、各手が合法である
3. 再生後の位置・盤面がStateの保持値と一致する
4. Parametersから再構築したOutputが、保持中のOutputと一致する
5. Stateの評価値または差分評価が、素直な `calculate_score` と一致する
6. `apply_move` の直後に `undo_move` するとState全体が戻る

重い検算は `debug_assert_eq!`、観察は `eprintln!` にする。探索バグは入力seed・乱数seed・反復回数を固定して再現する。

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

## 提出前

```sh
./ahc run 0 --solver a --debug
./ahc export --solver a --clipboard
```

`export` は `include!` を展開した単一ファイルを `submit/a.rs` に保存してクリップボードへコピーする。貼り付け前に、開始前テンプレートのURL注記が残っていること、デバッグ用の `println!` が混ざっていないことを確認する。
