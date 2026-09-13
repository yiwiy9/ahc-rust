# 短期AHC 当日手順

コンテスト固有の問題名・時刻・規則は、毎回公式ページで確認してこの手順の `<contest_id>` を置き換える。これは開始前に用意する常設の操作メモであり、コンテスト終了後も削除しない。

## 開始前

```sh
cd /Users/yiwiy/Codes/atcoder/ahc-workspace/ahc-rust
./ahc doctor
```

`Ready.` と出ること、`pahcer` が `[ok]` であることを確認する。VS Codeは、共通libも検索するため [ahc-rust.code-workspace](../ahc-rust.code-workspace) を開く。Explorerに `ahc-rust` と `atcoder-lib` が並ぶことを確認する。

共通libを探すときは `⌘⇧F` で両方を検索し、必要ならRustファイルでprefix（例: `bfs`）を書いて `⌃Space` からスニペットを挿入する。共通libを更新した場合だけ、開始前に `./scripts/sync-vscode-snippets.sh` を実行する。

短期AHCでは生成AI利用規則を必ず公式ページで確認する。生成AIが原則禁止の回では、開始後に対話型生成AIを問題理解・方針・実装・デバッグ・実行結果の分析に使わない。事前に公開したコードテンプレートを使う場合だけ、提出コード内の対応するURL注記を残す。事前に作った操作メモ・学習ノートへはこの注記は不要である。

## 開始直後

ahc-rust直下から実行する。pahcerの初回導入は開始前に済ませる。

```sh
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

## 迷ったときの進行ガイド

「焼きなましまで到達する」ことを必須にしない。有効な提出を確保し、改善と判断できた解を積み重ねる。以下のパスは作業中の `contests/<contest_id>/` を基準にする。テンプレートは未実装の土台であり、コンパイル成功だけでは解けたことにならない。

### 1. 問題を読む：何を自由に決められるか

最初にメモするのは、入力・提出するもの・守る制約・得点の向き・実行時間制限。小さな例で「この出力をしたら何が起こるか」を手で追う。最大化か最小化かは `ahc.toml` に反映する。

次へ進む目安は「とても弱くてよいので、合法な出力を一つ説明できる」。最初から探索方式を決めなくてよい。インタラクティブ形式なら入出力とテスターの使い方が通常の一括出力と違うので、先に確認する。

### 2. 入出力と最小解：まず実際に通す

編集するのは `src/problem.rs` の `Input`、`Output`、`read_input`、`print_answer` と、`src/bin/valid.rs` の出力生成。内部の添字は0-indexに統一し、入出力の境界だけ変換する。Outputの型を変えたら、利用するbinとState側の仮の初期化も合わせて直す。

`validate_output` に個数・値域・重複禁止などを足す。初期状態の空出力や常にOkを返す検証は、合法性の保証ではない。

```sh
./ahc run 0 --solver valid --debug
./ahc vis 0 --solver valid --debug --open
```

次へ進む目安は「公式ツールが出力を受け取り、可視化も意図どおり」。0点が合法な問題では0点でもこの段階は成功。pahcerで0点を扱えない場合はbuiltinで確認する。独自採点や高度な初期解がまだなくても、提出可能な解を先に確保する。

### 3. シミュレーションと採点：正しさの基準を作る

`src/problem.rs` の `calculate_score` を素直に実装する。出力を最初から再生し、位置・訪問済み集合・残り資源などを更新する。最初から差分更新にしない。

小さな手計算例と公式採点で照合する。公式ツールが報告する最終得点と、途中評価・生の目的関数・丸め前の値は別の場合があるので、比較対象を揃える。独自採点を使わない方針なら後回しにできるが、未実装の0を探索評価として使わない。

次へ進む目安は「なぜその点数か説明でき、合法性と意図した動作の両方を確認できる」。得点が一致しても、作りたかった行動列になっているとは限らない。

### 4. 貪欲：一手を選ぶ基準を一つ作る

`src/constructive_state.rs` の `State` と `Action` を実装する。必要な入力もStateから参照・保持できるようにする。`src/bin/greedy.rs` が入口で、基本の探索ループは `src/framework/constructive.rs` にある。

考える順は次のとおり。

1. 何を1手とするか。
2. 終了条件 `is_complete` と合法手 `legal_actions` は何か。
3. `advance` で、出力と盤面・残量などの保持値を一緒に更新する。
4. その1手の後の状態を `evaluated_value` で比較する。「大きいほどよい」に統一する。
5. 完成した状態から `into_output` で実際の提出解を取り出す。

最初の基準は、今増える利益、短い移動、資源消費の少なさなど一つで十分。未完成状態では公式最終得点をそのまま使えないこともある。途中評価と完成解の採点は区別する。

まず1ケースで「意図した手を選んでいるか」を見る。その後、固定した複数seedで測り、baselineを保存する。弱くても有効な解を早めに提出する。未完成なのに合法手が空になる、終了しない、といった構築の不具合を先に直す。

評価を変えるときは「近いものばかり選んで遠方を取り残す」など観察した失敗に対し、一つずつ仮説を試す。最初から重い未来予測や複雑な重み調整を入れない。

### 5. 次の改善を選ぶ：貪欲の失敗を見て決める

- 同点の選択や構築のばらつきで良し悪しが変わり、作り直しが安い → 乱択構築。
- 一手の評価は使えそうだが、一択に絞ると後悔する → ビーム。
- 完成解の一部を変更し、良し悪しを判定できる → 局所探索。
- まだ不正出力がある、評価が怪しい → 探索を追加せず正しさを直す。

追加コマンドは `./ahc add random-search`、`./ahc add beam`、`./ahc add local-search-direct`、`./ahc add local-search-rebuild`。構築系は同じconstructive_stateを使う。ただしビームは現実装が想定する終了深さや、候補の途切れがないことも確認する。cloneの負担が重い場合、まず幅を小さくして測る。各binの試行数・ビーム幅・時間予算は問題ごとに確認する。

### 6. 局所探索：何を変え、何を作り直すかを先に決める

最初に「変更するもの」「固定するもの」「変更に伴って再計算するもの」を1行ずつ書く。

- 提出解の配列・割当を直接変える → `src/local_search_state_direct.rs`。
- 少数のパラメータを変え、その条件で行動列などを生成し直す → `src/local_search_state_rebuild.rs`。

後者では `Parameters` と `rebuild` が重要。例えば移動パターンを変更して、そのパターンで貪欲に行動列も作り直す設計なら、rebuildは行動選択からやり直す。「古い行動列を新しい条件で再生して採点」するだけでは別の探索になる。

`rebuild` の仮実装はOutputのコピーなので、名前があるだけで自動再構築されるわけではない。同じParametersから同じOutputを作れるようにし、乱数を使うならそのseedも固定・管理する。再構築検算で一致しても、そのrebuild自体が意図を満たすかは小さな具体例で確認する。

近傍は一種類から始める。自由な割当なら1か所変更、個数を保つ必要があれば2か所交換、順序なら挿入や反転を候補にする。変更後に制約を破らないかを先に考える。詳しくは[Moveの選び方](strategy-guide.md#moveの武器)。

### 7. 山登り：変更・採点・取り消しを確かめる

編集するのは選んだStateの `Move`、`propose_move`、`apply_move`、`undo_move`、`debug_validate`。初期状態の `propose_move = None` では探索は始まらない。入口は `src/bin/local_search_direct.rs` または `local_search_rebuild.rs` で、最初は `Acceptance::HillClimbing` のまま試す。

- 変更後のOutputが合法で、意図した箇所・派生データが変わる。
- 評価が素直な全再計算と一致する。
- apply→undoでState全体が元に戻る。
- 少数の固定反復を最後まで実行できる。

```sh
AHC_ITERATIONS=100 ./ahc run 0 --solver local_search_rebuild --debug --no-vis
```

スコアが伸びないことだけでバグと決めない。そもそも変更しているか、同じ変更ばかりか、採用されるか、反復回数が少なすぎないかをログで見る。検算の詳しい手順は[デバッグガイド](debugging.md)。

### 8. 焼きなまし：正しい近傍はそのまま、採用条件を変える

山登りで変更・取り消しが確認できたら、binの `Acceptance::HillClimbing` を `Acceptance::SimulatedAnnealing { start_temperature, end_temperature }` へ切り替える。温度は得点の総額でなく1 Moveの改悪幅を基準にする。[測定コマンド・倍率候補](strategy-guide.md#温度の決め方)を参照する。

現在解が途中で悪化するのは正常。共通ループが返す最良解と、同じseed集合の貪欲・山登りを比較する。まず近傍を固定して温度を試す。近傍・温度・評価を同時に変えると、何が効いたか分からなくなる。

差分更新は最後。1回の再計算が重く反復できないと分かってから、基準となる全再計算を残して高速化する。時間がなければ、正しく動く山登りや貪欲を提出してよい。

### 9. 最後：新方式より提出の確実さ

残り時間が減ったら新しい探索方式の導入を止める。保存済みの有効な解を基準に、同じseedで改善を確認できたものを選ぶ。提出間隔と終了時刻は大会ルールに従い、再提出できる余裕を残す。

- `--solver` が意図した解法か。最後に編集したファイルと提出対象が一致するか。
- 時間予算を問題に合わせたか。テンプレートの1,900msは汎用の正解ではない。
- 固定反復・温度調査用の環境変数を解除したか。AHC_ITERATIONSは時間制限の代替になる。
- releaseでも合法か。実測時間に余裕があるか。ローカルと提出サーバーの速度は同一ではない。
- `export` 後の単一ファイル、事前公開URL注記、実際の提出結果を確認したか。

**困ったら「最後に正しく動いた解へ戻る → 小さい入力で一つだけ確かめる」。高度な探索を完成させることより、有効な提出と改善の根拠を優先する。**

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
