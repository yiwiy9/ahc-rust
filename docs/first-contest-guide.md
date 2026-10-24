# 初回AHC: 動くコードから始める

普段編集するのは `src/bin/a.rs` 一枚。入出力も探索も同じファイルで読める。生成元は分割管理しているが、newで一度だけ結合し、その後のbuild/runでは生成し直さない。

## まずintroの例を一周する

リポジトリ直下で:

```sh
./scripts/new.sh intro-practice --example
cd contests/intro-practice
./scripts/run-one.sh a 0
./scripts/debug.sh a 0
./scripts/run-all.sh a 10
./scripts/save.sh a greedy-v1 10
```

通信せず、同梱intro toolsを使って入力生成・採点・可視化まで進められる。新規環境で依存がキャッシュになければ、事前のCargo依存取得は必要。

この例では、毎日26種類から開催するコンテストを一つ選ぶ。Stateには途中の日程・最後の開催日・満足度を持つ。各候補の「次の日までの満足度」を計算して最大を採用する。`proconio::input!` で読んだ数値や配列をそのまま使い、日程も `Vec<usize>` で保持する。

## 新しい問題で置き換える場所

問題公開後に `./scripts/new.sh ahcNNN` で作り、`a.rs` 内の「変更1〜6」を書き換える。初期コードが動くのはintroに対してであり、今回の入力形式へ合わせる必要がある。

| 箇所 | introの実装 | 別問題へ変える例 |
| --- | --- | --- |
| Input・read_input | 日数・減衰係数・満足度 | 盤面・座標・要求列 |
| Output・print_answer | 開催種類の配列、出力時に+1 | 座標の配列・操作enum |
| State | 日程・最終開催日・満足度 | 現在位置・盤面・操作履歴 |
| Action | 次に開催する種類 | 移動先・配置場所 |
| legal_actions | 26種類すべて | 合法な移動・配置候補 |
| value_after / evaluated_value_after | 次の1日分を加えた満足度 | 次の状態の見込み価値 |
| advance | 日程・開催日・満足度を更新 | 操作履歴・状態・評価を同時更新 |

Inputはmainで一つ保持し、Stateのメソッドへ `&Input` を渡す。Stateのcloneで入力全体をコピーする必要はない。print以外で文字列化しない。

評価は「大きいほどよい」。コスト最小化は-cost等へ変換する。公式の得点と途中の評価を分けてよいが、軽量評価の結果とadvance後の評価は一致させる。introでは公式得点の0点丸め前の満足度を探索用に使っている。

introでは評価式を `value_after` にまとめ、advanceと軽量評価から共用する。軽量評価の `evaluated_value_after` を省略すると、cloneしてadvanceするtraitの既定実装になる。advanceからtraitの軽量評価へ依存させないことで、既定実装でも動くようにしている。

採点・保存の比較方向は別途 `ahc.toml` の `score_direction` で指定する。公式得点が小さいほどよいなら `minimize`。得点の表示形式が異なる場合は同ファイルの `score_pattern`、制限時間は `total_time_limit_seconds` も問題に合わせる。

## 貪欲を動かしてから次へ

合法性確認用の解を確保し、問題の性質を一つ使った貪欲を作る。可視化で狙った動きか確認し、同じ10ケースで比較する。`save.sh a <名前> 10` の成功表示後に次の案へ進む。

```sh
cp -n src/bin/a.rs src/bin/beam.rs
# samples/beam.rsをbeam.rs末尾へ貼る
# mainのsolve_greedyをbeam::solve_beamへ変更する
./scripts/run-one.sh beam 0
./scripts/run-all.sh beam 10
```

Cargoは追加したファイル名を自動認識する。別ファイルへ分割し直す必要も、登録操作もない。ソート方法・候補数・重複排除を変えたければ探索関数を直接直してよい。

乱択や局所探索も `samples/README.md` の手順で追加する。自分のStateを残したいときは追加部分だけを貼る。`samples/complete/` はintro用の完成例なので、別問題のコードを丸ごと置き換えない。

## 保存と提出

```sh
./scripts/list.sh
./scripts/compare.sh 1 2
./scripts/restore.sh 1
./scripts/export.sh beam
```

saveはソース一式と採点結果を保存する。runだけではソースの版は残らない。restoreは復元前のソースを退避する。exportは単一ファイルをsubmitへ保存し、クリップボードへコピーする。貼り付けてWeb提出する。

困ったときは[当日手順](contest-day.md)、[探索ガイド](strategy-guide.md)、[デバッグ](debugging.md)を参照。
