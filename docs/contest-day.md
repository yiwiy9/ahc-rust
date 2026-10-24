# 当日の手順

新規生成は **ConstructiveStateの貪欲法が入ったa.rs一枚**。日常操作は `./scripts/...` に統一する。

## 開始

リポジトリ直下で、問題公開後に:

```sh
./scripts/new.sh ahcNNN
cd contests/ahcNNN
```

公式toolsの取得に失敗したら、問題ページのtools.zipを指定する。

```sh
./scripts/tools.sh --url <公式tools.zipのURL>
```

## 実装

編集するのは `src/bin/a.rs`。「変更1〜6」のコメントを順に置き換える。

1. `problem`: proconioの入力、Input・Output、出力、制約確認、素直な得点計算。
2. `constructive_state`: 部分解、Action、完了条件。
3. `legal_actions`: 次に試す候補。
4. `advance`: 出力・状態・評価キャッシュを同じ1手で更新。
5. `value_after` / `evaluated_value_after` / `evaluated_value`: 大きいほどよい評価。最小化なら-cost等。
6. `main`: 初めはsolve_greedy。

初期コードはintroで動く例。新しい問題に対しては入出力から置き換える。最小の合法解を作り、一つの性質を使った貪欲へ進める。探索方法を増やす前に、出力と状態更新を確認する。

採点設定は `ahc.toml`。小さい公式得点がよい場合は `score_direction = "minimize"` にする。探索用評価を-costにすることとは別に設定する。

```sh
./scripts/debug.sh a 0
./scripts/run-one.sh a 0
./scripts/run-all.sh a 10
./scripts/save.sh a greedy-v1 10
```

debugは軽量評価・更新・キャッシュを全体再計算と照合する。速度確認はreleaseのrun-oneで行う。saveの成功表示を待ってから次へ進む。保存は非同期ではない。

## ビーム・乱択・焼きなましへ

```sh
cp -n src/bin/a.rs src/bin/beam.rs
# samples/beam.rsを末尾へ貼り、mainの探索呼び出しを変更
./scripts/run-one.sh beam 0
./scripts/run-all.sh beam 10
./scripts/save.sh beam beam-v1 10
```

手で複製したbinも登録不要。自分の入出力・Stateを維持したまま探索だけ追加する。`samples/README.md` に各探索の貼り方、`samples/complete/` にintroの完成例がある。

ビームは親・手・評価値で候補を比較し、上位だけStateをコピーする。`evaluated_value_after` までcloneに戻すと評価用のコピーは残る。候補数は幅だけでなく経路長等にも依存するので、複数ケースの時間を確認する。既存の探索関数もその場で変更してよい。

焼きなましは貪欲の完成解を初期解とし、問題固有の近傍を追加する。ConstructiveStateから自動変換するものではない。[探索ガイド](strategy-guide.md) を参照。

## 失敗時・復元・提出

```sh
./scripts/debug.sh beam 0
./scripts/list.sh
./scripts/compare.sh 1 2
./scripts/restore.sh 1
./scripts/export.sh beam
```

restoreは現在のsrcを退避してから復元する。exportは `submit/beam.rs` に提出コードを保存し、クリップボードへコピーする。標準出力にデバッグ文字列が混じっていないことを確認し、Webで提出する。

ログ・出力・可視化のパスは実行結果に表示する。既存の可視化を開き直すなら `./scripts/vis.sh beam 0`。並列比較は同じ `run-all.sh beam 10 --parallel --threads 4` で行うが、制限時間の確認は単独実行でも行う。
