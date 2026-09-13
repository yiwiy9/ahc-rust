# AHC071 当日手順（AIなし）

この文書は開始前に用意した操作メモである。開始後は生成AIを使わず、この文書・公式問題文・公式ツール・ローカルのソースコードだけを参照する。

## 境界

- 開始: 2026-09-13 19:00 JST
- 終了: 2026-09-13 23:00 JST
- 提出間隔: 前回提出から5分以上
- 開始後は、対話型生成AIによる問題理解、方針検討、実装、デバッグ、実行結果・得点の分析をしない。検索結果のAI要約も見ない。
- `templates/` を使って提出する場合は、出力コードにある開始前公開URLを消さない。

公式: <https://atcoder.jp/contests/ahc071> / <https://info.atcoder.jp/entry/short-ahc-llm-rules-en>

## 0. 開始前（今だけ）

```sh
cd /Users/yiwiy/Codes/atcoder/ahc-workspace/ahc-rust
./ahc doctor
```

`Ready.` と出ることを確認する。`pahcer` も `[ok]` なら複数seed比較をそのまま使える。VS Code はこの `ahc-rust` ディレクトリを開く。

## 1. 問題用ディレクトリを作る

```sh
cd /Users/yiwiy/Codes/atcoder/ahc-workspace/ahc-rust
./ahc new ahc071
cd contests/ahc071
./ahc doctor
```

`new` は問題専用のRustプロジェクトを作り、公式ツールのURLを見つけられれば `tools/` へ入れる。`doctor` で `Cargo.toml`、`tools/Cargo.toml`、`tools/in/0000.txt` がすべて `[ok]` かを見る。

公式ツールを自動取得できなければ、問題ページから公式tools.zipのURLをコピーして実行する。

```sh
./ahc tools --url 'https://img.atcoder.jp/.../tools.zip'
./ahc doctor
```

## 2. 最初の実装（まず合法解）

1. 問題文を読み、入力、出力、制約、得点の向き、時間制限を `ahc.toml` と紙のメモへ書く。
2. `src/problem.rs` に `Input`、`Output`、`read_input`、`print_answer` を実装する。内部表現は0-index、入出力の境界だけで変換する。
3. `validate_output` に、行数・値域・個数・制約違反を検出する処理を書く。
4. `src/constructive_state.rs` と `src/bin/greedy.rs` を埋め、遅くても合法な解を出す。

この段階の目的は高得点ではない。「公式ツールが受理できる出力を、自分のコードが作る」こと。

## 3. 1ケース実行とエラー調査

```sh
./ahc run 0 --solver greedy --debug
```

これはseed 0で、debugビルド、solver実行、公式採点、HTML可視化までを一度に行う。失敗したら、まず表示されたメッセージを読み、次にログを読む。

```sh
less results/logs/greedy/debug/0000.log
```

panicの再現時は次を使う。

```sh
RUST_BACKTRACE=1 ./ahc run 0 --solver greedy --debug --no-vis
```

デバッグ表示は `println!` ではなく `eprintln!` に出す。標準出力は提出する解そのものなので、混ぜると出力が壊れる。

## 4. 「得点」と「意図した解」を別々に確認する

得点が合っても、意図した行動列・盤面・派生データが合っているとは限らない。debug時に次を順に確認する。

1. `validate_output` が通る。
2. 出力した各行動を先頭から再生し、各手が合法である。
3. 再生後の位置・盤面がStateの保持値と一致する。
4. Parametersから `rebuild` したOutputが、StateのOutputと一致する。
5. Stateの評価値／差分評価が、素直な `calculate_score` と一致する。
6. `apply_move` の後に `undo_move` すると、State全体が元に戻る。

重い検算には `debug_assert_eq!`、観察には `eprintln!` を使う。探索バグは、入力seed・乱数seed・反復回数を固定して再現する。時間探索を固定反復に切り替えられるテンプレートでは、例えば次のようにする。

```sh
AHC_ITERATIONS=10000 ./ahc run 0 --solver local_search_rebuild --debug --no-vis
```

## 5. 可視化

`run` が成功するとHTMLが作られる。再表示するには次を使う。

```sh
./ahc vis 0 --solver greedy --debug --open
```

見るのは得点だけではない。初期位置、各行動の軌跡、制約違反、想定した構造が本当に出力されているかを確認する。表示が意図と違えば、得点が良くても実装を直す。

## 6. 複数seedで比較する

```sh
./ahc bench --solver greedy --cases 10 --threads 4
./ahc history --rank
```

`bench` はseed 0〜9を同条件で測る。1ケースの偶然の良さを避け、平均・最小値・失敗seedを見る。比較する候補同士では `cases` と `threads` をそろえる。

`pahcer` が0点や負の得点をWA扱いする問題、またはpahcer連携が合わない問題では、逐次runnerへ切り替える。

```sh
./ahc bench --solver greedy --cases 10 --threads 4 --builtin
```

## 7. 保存する

```sh
./ahc save greedy --solver greedy --cases 10 --threads 4
./ahc history --rank
```

`save` は、その時点の `src/` と測定結果を保存する。名前は「何を変えたか」が分かるものにする（例: `greedy-distance`, `beam-100`, `rebuild-baseline`）。改善の判断には、同じseed集合の前後比較だけを使う。

## 8. 貪欲から探索へ進む判断

最初は必ず貪欲で、合法性・評価・Stateを確認する。その後は問題の構造で決める。

- 途中状態から次の合法手を列挙でき、早い選択を後から直しにくい: 構築型。まず乱択構築、次にビームサーチを試す。
- 完成解の一部を変えても合法性を保て、変更後を十分速く評価できる: 改善型。山登り、次に焼きなましを試す。
- どちらも成り立つ: 貪欲・ビームで初期解を作り、局所探索で改善する。

探索へ進む前提は、Moveを1種類に絞り、上の再生・再構築・undo検証をdebugで通すこと。1 Moveごとに全体再計算が重くて反復できないなら、まず正しい全再構築を保ったままMoveを粗くするか、影響範囲を限定する。差分計算は最後に導入する。

## 9. 提出用コードを出す

```sh
./ahc run 0 --solver a --debug
./ahc export --solver a --clipboard
```

最終候補をdebugで1回通してからexportする。`export` は `include!` を展開した単一ファイルを `submit/a.rs` に保存し、`--clipboard` でコピーもする。AtCoderの提出画面へ貼り付ける前に、先頭の開始前テンプレートURLが残っていること、`eprintln!` は残っていても `println!` のデバッグ出力が混ざっていないことを確認する。
