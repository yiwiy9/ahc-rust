# AHC071 復習

編集するファイルは `src/bin/a.rs` の1つ。整理前の書き方を維持し、貪欲・ビームサーチ・State・入出力を同じファイルに置いている。実行・採点には新版のコマンドを使う。

## 実行コマンド

VS Codeでahc-rustを開き、ターミナルをこのディレクトリに移動する。

```sh
cd /Users/yiwiy/Codes/atcoder/ahc-workspace/ahc-rust/contests/ahc071
```

| やりたいこと | コマンド |
| --- | --- |
| seed 0を実行・採点・可視化 | `./scripts/run-one.sh a 0` |
| 検算つきで実行 | `./scripts/debug.sh a 0` |
| seed 0〜9をまとめて採点 | `./scripts/run-all.sh a 10` |
| 現在の解を名前付きで保存・採点 | `./ahc save greedy-v1 --solver a --cases 10 --builtin` |
| 保存した解の一覧 | `./ahc list` |
| 提出用の単一ファイルをコピー | `./ahc export --solver a --clipboard` |

実行時に差分ビルドするので、毎回buildを別に呼ぶ必要はない。`a` はbin名、末尾はseedまたは件数。run-allはpahcer不要。全体の使い方は [環境ガイド](../../docs/environment.md)。

## ビジュアライザ・結果を見る

```sh
./ahc vis 0 --solver a --open
```

先にrun-oneを実行する。debugで生成したHTMLなら `./ahc vis 0 --solver a --debug --open`。

- 解の出力: `out/a/release/0000.txt`（debug時は `out/a/debug/0000.txt`）
- stderr・panic: `results/logs/a/release/0000.log` または `results/logs/a/debug/0000.log`
- 一括計測: ターミナルの合計・平均・最小値と `results/latest.json`
- 提出用コード: `submit/a.rs`。編集せず、毎回exportで生成する。

## 編集する場所

- `src/bin/a.rs` の上側: `solve_greedy` / `solve_beam`。探索の共通ループ。
- 同ファイルの `main`: 呼び出す探索の切り替え。
- `main` より下: Input / Output・入出力・得点計算・State・行内の最小コスト計算。方針を変える中心はState。

入力は `&Input` で渡し、Stateには出力と更新されるキャッシュだけを持たせる。貪欲とビーム用のStateは共通。ビームを試すときは `main` の貪欲の呼び出しをコメントアウトし、既存の次の行を有効にする。新しいbinの追加は不要。

```rust
let state = solve_beam(&input, State::new(&input), BEAM_WIDTH);
```

現状は各行の候補を1つだけ返すため、単にbeamへ切り替えても探索の選択肢は増えない。今回は候補や評価方法の改善は入れていない。

`calculate_score` は提出版の内部検算式を保持している。debugではState内の得点と完成解の全体再計算を照合する。出力の有効性と公式得点は実行コマンドが呼ぶ公式toolsで確認する。

## 開始地点・保存物

- `solutions/submitted.rs`: 整理前の `submit/a.rs` をそのまま保存した比較用コード。AtCoder上の提出履歴との照合はしていない。
- 新しいaと保存版はseed 0〜9で出力が完全一致。
- 公式toolsで10 / 10ケースが有効。合計 **98,111**、平均 **9,811.10**、seed 0 **9,506**。ローカル10ケースの値であり、順位表の得点ではない。
- 旧未完成のvalid/greedy binとStateは作業対象から除去済み。整理前のソース・README・出力・ログ・HTML・submitは `../../.tools/ahc071-before-review-61sBIC/` に退避している。
- 公式tools、入力ケース、Cargo依存とビルドキャッシュは、すぐ実行できるよう残している。
