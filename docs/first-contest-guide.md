# 初回AHC: 手を動かす手順書

この文書は、初めて新しいAHC問題へ取り組むときの一本道である。各段階で「何を編集するか」「何のためか」「実行するコマンド」「成功なら次に何をするか」「失敗ならどこを見るか」を決める。

最初の目標は高得点ではない。**公式toolsに受理され、可視化した内容を自分で説明できる解を一つ提出できるようにすること**である。

## 主に使う場所

| 場所 | 役割 | 自分で編集するか |
| --- | --- | --- |
| `src/problem.rs` | 問題固有の入力、提出出力、表示、検証、採点 | 編集する |
| `src/bin/valid.rs` | 最小の合法解だけを出す練習用solver | 編集する |
| `src/bin/a.rs` | 普段の提出対象solver | 編集するか、`greedy.rs`へのラッパーとして使う |
| `src/constructive_state.rs` | 解を一手ずつ組み立てる貪欲・ビーム用の状態 | 構築型の道を選んだ時だけ編集する |
| `src/bin/greedy.rs` | `constructive_state.rs`を貪欲ループで動かす入口 | 通常は編集しない |
| `src/framework/` | 貪欲・ビーム・局所探索の共通ループ | 編集しない |
| `tools/` | AtCoder公式の生成器・採点器・可視化器 | 編集しない |
| `out/`, `results/`, `visualizations/`, `target/` | 実行時の出力・ログ・可視化・ビルド成果物 | 編集しない |
| `submit/` | `export` が作る提出用単一ファイル | 編集しない |

`valid.rs` はAtCoder提出の必須要件ではない。しかし初回は、最小の合法解を通す経験を省かないために使う。

## 0. 問題用ディレクトリを作る

リポジトリ直下で実行する。

```sh
./ahc new <contest_id>
cd contests/<contest_id>
./ahc doctor
```

`new` は問題ごとのRustプロジェクトと公式toolsを用意する。`doctor` は必須コマンドの存在や、関連ファイルの配置を確認する。コンパイル・入力生成・採点はまだ実行していない。

`Ready.` は必須コマンドが見つかった印であり、toolsや入力の準備完了を保証しない。`[missing]` に加え `[not found]` も確認する。開始後に公式toolsのURLだけを見つけられない場合は、表示されたURL指定の `./ahc tools --url <URL>` を実行する。実際に動くことは、段階3の `run` で確認する。

## 1. コードを書く前に問題文からメモする

問題文と公式toolsのREADMEを開き、次の5行だけを書き出す。まだRustは書かない。

```text
入力: 何が与えられるか
出力: 何を何個出すか
合法性: 値域・重複・順序・操作可能条件
得点: 大きいほど良いか、小さいほど良いか
可視化: 何を見れば意図どおりか
```

次へ進んでよい基準は、「一番弱くてよいので合法な出力を日本語で説明できる」こと。説明できないときは実装せず、問題文とtoolsの `README.md` / `src/lib.rs` を読む。

## 2. `src/problem.rs` を問題文の言葉に直す

### 編集するもの

`src/problem.rs` の次の4つを編集する。

1. `Input`: 入力で固定される値をフィールドにする。
2. `Output`: 提出する解をフィールドにする。
3. `read_input`: 標準入力から `Input` を作る。
4. `print_answer`: `Output` を問題文の出力形式で表示する。

最初のテンプレートの `raw: String` と `lines: Vec<String>` は仮置きなので、そのまま使い続ける前提ではない。内部では0-indexedにそろえ、提出するときだけ必要なら `+ 1` する。

### 何のためか

solver、検証、採点で同じ出力形式を共有するためである。ここが曖昧だと、解法以前に「何を出しているか」がずれる。

### この段階では書かなくてよいもの

`validate_output` と `calculate_score` は、最初は `Ok(())` と `0` のままでよい。次の最小解を作ることを優先する。

## 3. `src/bin/valid.rs` に最小の合法解を書く

### 編集するもの

`valid.rs` のこの行を、問題文で説明できる最小の合法な `Output` を作る処理へ置き換える。

```rust
let output = Output { /* 問題固有の値 */ };
```

例: 「N個の値を出す」ならN個を用意する。「操作列を出す」なら、初期状態から確実にできる操作だけを並べる。「何も選ばない」が合法なら、空出力でもよい。

### 実行する

```sh
./ahc run 0 --solver valid --debug
```

これは、debug buildで `valid.rs` をコンパイルし、seed 0の入力を作り、solverを実行し、公式toolsで採点し、可視化ファイルも作る操作である。

### 成功したら見るもの

- `score=...` に公式toolsから取得した得点が出る。
- `output=...` のファイルが作られる。
- 公式toolsが `vis.html` を生成する場合、`visualization=...` のパスが出る。生成しない形式では公式Webビジュアライザ等を使う。

続けて次を実行する。

```sh
./ahc vis 0 --solver valid --debug --open
```

可視化で「自分が出した最小解がどう扱われたか」を確認する。高得点である必要はない。合法で、期待した位置・順序・個数になっていれば成功である。

### 失敗したら、ここを見る

| 表示された症状 | 最初に見る場所 | 直す内容 |
| --- | --- | --- |
| Rustのコンパイルエラー | VS CodeのProblems、`valid.rs`、`problem.rs` | 型、フィールド名、`use`、括弧を直す |
| `invalid output` | `validate_output` のメッセージ | まだ仮の検証なら一旦条件を見直す。公式制約に合わせる |
| solver failed / panic | `results/logs/valid/debug/0000.log` | panic行と入力の読み方を確認する |
| 公式toolsのエラー・WA | `out/valid/debug/0000.txt` と問題文 | 行数、値域、1-index/0-index、操作順を確認する |
| 可視化が想定と違う | `out/valid/debug/0000.txt` と可視化 | 出力を先頭から手で追い、`print_answer`か解の作り方を直す |

ここで大切なのは、得点だけで成功としないこと。可視化と出力ファイルを見て、**意図した行動・配置・順序が実際に出ているか**を言葉で説明する。

## 4. 最小限の検証を足す

`valid` が公式toolsを通った後、`src/problem.rs` の `validate_output` に軽い検査を足す。

```text
まず追加する: 出力個数、各値の範囲、重複禁止
操作列なら追加する: 初期状態から先頭順に再生して各手が合法か
探索を始める前に追加する: 再生後の状態が保持している状態と一致するか
```

`validate_output` は得点を計算する関数ではない。「この出力は提出可能か」を調べる関数である。公式toolsを置き換えるものではなく、失敗を早く、分かりやすく止めるための安全網である。

## 5. 本命solverは一つだけ選ぶ

ここから二択である。二つを混ぜない。

### A. `a.rs`へ直接書く — 初回の推奨

次のどれかならこちらを選ぶ。

- 単純な規則で出力を作れる。
- 一つの解法をまず完成させたい。
- `State` / `Action` の分割がまだ負担に感じる。

#### 編集するもの

`src/bin/a.rs` の `include!("greedy.rs");` を、通常の `main` と `solve` を持つコードへ**置き換える**。先頭の事前公開テンプレートURLコメントは残す。

形は次でよい。

```rust
mod problem {
    include!("../problem.rs");
}
use problem::{print_answer, read_input, validate_output, Input, Output};

fn solve(input: &Input) -> Output {
    // 問題固有の規則で解を一つ作る
    todo!()
}

fn main() {
    let input = read_input();
    let output = solve(&input);
    #[cfg(debug_assertions)]
    validate_output(&input, &output).expect("invalid output");
    print_answer(&output);
}
```

#### 実行と確認

```sh
./ahc run 0 --solver a --debug
./ahc vis 0 --solver a --debug --open
```

`valid` と比べ、出力がどう変わったかを可視化で説明する。意図と違えば、最初に直すのは `solve` の規則であり、探索を増やすことではない。

### B. `constructive_state.rs`を使う — 一手ずつ組み立てる問題向け

次の全てに当てはまるときだけ選ぶ。

- 空の解から、操作・頂点・割当などを一手ずつ足して完成させる。
- 今の途中状態から、次に選べる候補を列挙できる。
- 候補を一つ選んだ後の良さを比べられる。

#### 編集するもの

`src/constructive_state.rs` だけをこの順に埋める。

1. `State` に、途中の出力と次の一手に必要な情報を入れる。
2. `Action` を「一手」で表す型にする。
3. `is_complete` に完成条件を書く。
4. `legal_actions` に今可能な候補を書く。
5. `advance` で出力と補助情報を**必ず同時に**更新する。
6. `evaluated_value` に「大きいほどよい」候補比較値を書く。
7. `debug_validate` で、必要なら保持情報を素直な再計算と照合する。

この道では `src/bin/greedy.rs` と `src/bin/a.rs` は通常編集しない。`a.rs` は既に `greedy.rs` を読み込むため、実行も提出も `--solver a` のままでよい。

#### 実行と確認

```sh
./ahc run 0 --solver a --debug
./ahc vis 0 --solver a --debug --open
```

止まらない、または `non-complete state must have a legal action` が出る場合は、`is_complete` と `legal_actions` の組を直す。可視化が途中からおかしい場合は、`advance` で出力だけ・補助情報だけを更新していないか確認する。

## 6. `calculate_score` はいつ書くか

最小解だけなら後回しでよい。次のいずれかを始める直前には、`src/problem.rs` に遅くても正しい `calculate_score` を書く。

- 複数の完成解のどちらが良いか自前で比べる。
- 局所探索で解を少し変更する。
- 差分更新で得点をキャッシュする。
- 「公式得点は合うが意図した行動か不明」を検査する。

最初は出力を先頭から再生する素直な実装にする。高速化や差分更新は後でよい。デバッグでは、Stateに保持した得点と `calculate_score(input, output)` が一致するかを確認する。

## 7. 1ケースが説明できてから測る

### 複数seed比較

```sh
./ahc bench --solver a --cases 10 --threads 4
./ahc history --rank
```

`bench` はseed 0から9を同じsolverで実行して記録する。これは「seed 0だけの偶然」ではないかを調べる操作である。`history --rank` はローカル履歴内での順位であり、AtCoder順位ではない。

### 良かった版を保存

```sh
./ahc save greedy-1 --solver a --cases 10 --threads 4
```

`save` は現在の `src/` を保存し、同じ条件で測る。名前には方式と変更点を入れる。例: `greedy-nearest`、`beam-100`、`hill-swap`。

同じseed数・同じthreadsの結果だけを比較する。方式・評価・近傍を一度に変えない。

## 8. ビームを追加する分岐

### 進んでよい条件

- 道Bの貪欲が合法な解を最後まで作る。
- `legal_actions`、`advance`、`evaluated_value` が動いている。
- 可視化で「一手だけ最良を選ぶと後で困る」場面を見つけた。

### 追加・編集・実行

```sh
./ahc add beam
```

このコマンドは `src/bin/beam.rs` と `src/framework/beam.rs` を追加する。既存ファイルは上書きしない。

編集するのはまず `src/bin/beam.rs` の `BEAM_WIDTH` だけ。`constructive_state.rs` は貪欲と共通で使う。`framework/beam.rs` は編集しない。

```sh
./ahc run 0 --solver beam --debug
./ahc vis 0 --solver beam --debug --open
./ahc bench --solver beam --cases 10 --threads 4
```

成功条件は、貪欲より毎回勝つことではなく、同じseed集合で改善する傾向があり、時間内に終わること。候補がなくなるpanicなら `legal_actions` / `is_complete` を直す。遅いなら、まず `BEAM_WIDTH` を小さくする。

提出するsolverをbeamにする場合は、`a.rs`を触らずに次を使う。

```sh
./ahc export --solver beam --clipboard
```

## 9. 局所探索・焼きなましを追加する分岐

### 進んでよい条件

- 完成解を一つ作れる（通常は貪欲・ビーム）。
- 変更したい一部分を言葉で説明できる。例: 「一日の割当を別の種類へ変える」「二つの順序を交換する」。
- 変更後の解を `calculate_score` で評価できる。
- applyしてundoすると元へ戻ることを確認する意思がある。

この条件がないなら、局所探索ではなく貪欲の状態更新や評価を直す。

### 直接変更する場合: `local-search-direct`

完成した出力配列・順列・割当そのものを少し変えるなら選ぶ。

```sh
./ahc add local-search-direct
```

追加される主なファイルは次。

| ファイル | 編集内容 |
| --- | --- |
| `src/local_search_state_direct.rs` | `Move`、`propose_move`、`apply_move`、`undo_move`、評価値 |
| `src/bin/local_search_direct.rs` | 時間予算を確認し、下記の初期解生成を接続する |
| `src/framework/local_search.rs` | 編集しない |

**初期解の接続を確認する。** 追加したbinは、既定では `solve_greedy(ConstructiveState::new(&input)).into_output()` で初期解を作る。道Bの貪欲はそのまま利用できるが、道Aで書いた `a.rs` の `solve` やビームの結果は自動では利用されない。

道Aから進む場合は、解を作る `solve` をmainとは別の共有ファイルへ切り出し、a側と局所探索側から同じ `problem::Input` / `Output` で呼ぶ。局所探索binの `initial_output` をその関数の戻り値に置き換え、使わなくなったconstructive_state関連のmodule宣言・useを外す。mainを含む `a.rs` 全体をそのままincludeしない。ビームを初期解にする場合も、その生成処理を明示的に接続する。この確認はrebuild版にも必要。

最初は近傍を一種類だけにする。debug buildで `apply_move` → `undo_move` が元のStateに戻ること、評価値と `calculate_score` が一致することを確認する。

```sh
AHC_ITERATIONS=100 ./ahc run 0 --solver local_search_direct --debug --no-vis
./ahc run 0 --solver local_search_direct --debug
```

`AHC_ITERATIONS=100` は時間ではなく100回だけ試す再現用モードである。バグ調査中だけ使い、本番測定では外す。

### 再構築する場合: `local-search-rebuild`

少数のパラメータを変え、そのパラメータから行動列全体を作り直すなら選ぶ。

```sh
./ahc add local-search-rebuild
```

編集する `src/local_search_state_rebuild.rs` では、次を守る。

1. `Parameters` は探索が直接変更する少数の決定だけを持つ。
2. `rebuild` はParametersからOutputを**新しく構築する**。
3. `apply_move` と `undo_move` の後は必ず `rebuild` する。

古いOutputをコピーするだけの `rebuild` では、再構築探索にならない。例えば「優先順位」をParametersで変えるなら、その順位で貪欲選択からやり直してOutputを作る。

テンプレートのrebuild版は最初から焼きなましを選んでいる。まず `src/bin/local_search_rebuild.rs` の `let acceptance = ...;` を `let acceptance = Acceptance::HillClimbing;` に変更し、固定反復でMove・再構築・undoが正しいことを確認する。初期解生成も前節の接続手順に従う。

```sh
AHC_ITERATIONS=100 ./ahc run 0 --solver local_search_rebuild --debug --no-vis
```

### 焼きなましの温度を決める

局所探索の正しさを確認してから、`src/bin/local_search_direct.rs` または `local_search_rebuild.rs` の `Acceptance` を確認する。

- `HillClimbing`: 評価が改善するMoveと同点のMoveを採用する（`delta >= 0`）。まずはこちらで近傍の正しさを確かめる。
- `SimulatedAnnealing`: 悪化Moveも確率で採用する。山越えが必要なときだけ使う。

温度は得点全体の大きさではなく、1 Moveでどれくらい悪化するかに合わせる。まずは `AHC_SAMPLE_DELTAS=1` で悪化幅を観察し、表示された候補から試す。

```sh
AHC_ITERATIONS=1000 AHC_SAMPLE_DELTAS=1 ./ahc run 0 --solver local_search_direct --debug --no-vis
```

温度、近傍、評価を同時に変えない。どれが効いたか分からなくなる。

## 10. 提出直前

最後に選んだsolver名を明示して実行する。

```sh
./ahc run 0 --solver a --debug
./ahc run 0 --solver a
./ahc export --solver a --clipboard
```

`export` は `include!` を展開し、`submit/a.rs` に単一のRustファイルを作ってクリップボードへコピーする。AtCoderの提出ページへ貼り付ける。CLIは自動提出しない。

貼り付け前に `submit/a.rs` を開き、次を確認する。

- 実際に提出したいsolverの内容か。
- 開始前テンプレートのURL注記が残っているか。
- `println!` によるデバッグ出力が混ざっていないか。
- 固定反復の環境変数は付けず、時間内に動くか。

## 迷ったときの順番

1. 最後に公式toolsを通ったsolverへ戻る。
2. seed 0の出力と可視化を見る。
3. 出力を先頭から再生し、意図と違う最初の箇所を探す。
4. その箇所を作った `solve` または `advance` だけを直す。
5. 1ケースが直ってから複数seedで比べる。

「得点が同じ」だけでは、意図した行動列・状態を作れている証拠にならない。出力、再生、可視化、得点を別々に確認する。
