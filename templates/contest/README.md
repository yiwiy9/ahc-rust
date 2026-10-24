# __CONTEST_ID__

**編集するのは `src/bin/a.rs` 一枚です。** 入出力・構築State・貪欲の探索が入っています。生成時だけ結合しており、buildやrunで上書きしません。

最初のコードはintro-heuristics Aで動く例です。今回の問題に使うときは、コメントの「変更1〜6」を問題文に合わせます。最初から今回の問題への合法解にはなっていません。

採点設定は必要に応じて `ahc.toml` を変更します。公式得点が小さいほどよい場合の `score_direction` は `minimize`。これは探索内の評価値の符号とは別の設定です。

## いつもの操作

```sh
./scripts/run-one.sh a 0
./scripts/debug.sh a 0
./scripts/run-all.sh a 10
./scripts/save.sh a greedy-v1 10
./scripts/export.sh a
```

bin名→seed / 件数の順。run時にビルドするため、事前のbuildは不要です。`./scripts/build.sh a` はビルドだけしたいときに使います。

`save` は採点と保存が終わるまで待ちます。成功表示後に次の案へ進みます。`run-one` / `run-all` だけではソースの版は保存しません。`export` は提出ファイルを保存し、クリップボードにもコピーします。ファイル保存だけなら `./scripts/export.sh a --file-only`。

## 別案を試す

```sh
cp -n src/bin/a.rs src/bin/beam.rs
# samples/beam.rsを末尾へ貼り、mainをコメントどおり変更
./scripts/run-all.sh beam 10
```

手で複製してもCargoが認識するので登録不要です。自作の入出力・Stateは引き継ぎます。[サンプルの使い方](samples/README.md) に乱択・ビーム・焼きなましの手順があります。`samples/complete/` にはintroで動く完成例もあります。

## 保存・可視化

```sh
./scripts/list.sh
./scripts/compare.sh 1 2
./scripts/restore.sh 1
./scripts/vis.sh a 0
./scripts/web.sh
```

復元前のソースは自動退避します。可視化・ログのパスは実行結果にも表示します。並列比較は `./scripts/run-all.sh a 10 --parallel --threads 4`（pahcerの事前準備が必要）。

詳しくは[当日手順](../../docs/contest-day.md)、[初回ガイド](../../docs/first-contest-guide.md)、[探索ガイド](../../docs/strategy-guide.md)。
