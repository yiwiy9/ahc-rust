# AHC073 復習

編集するのは `src/bin/a.rs`。本番で使った入出力・State・探索関数を含むコードを維持しています。新規テンプレートへの自動移行はしていません。

今回のテンプレート改修はAHC073終了後の環境整備です。このディレクトリの保存コードと、改修後の `templates/` は別に管理しています。

## 日常の操作

```sh
./scripts/run-one.sh a 0
./scripts/debug.sh a 0
./scripts/run-all.sh a 10
./scripts/save.sh a baseline 10
./scripts/export.sh a
```

実行時にビルドします。引数はbin名→seed / 件数。別案を `src/bin/beam2.rs` へ手で複製したら、`./scripts/run-all.sh beam2 10` で実行できます。登録は不要です。

保存一覧は `./scripts/list.sh`、比較は `./scripts/compare.sh 1 2`、復元は `./scripts/restore.sh 1`。保存は採点も含む同期実行です。成功表示後に次の案へ進んでください。

可視化を開き直す場合は `./scripts/vis.sh a 0`。ログ・出力・HTMLの保存パスは実行結果にも表示します。

## 新しいテンプレートとの関係

本番のa.rsは当時のStateメソッドを使っています。新しいサンプルはConstructiveState traitを実装した新規生成コード向けなので、そのまま混ぜず、APIを合わせる必要があります。

次回の生成と操作は[当日手順](../../docs/contest-day.md)、探索の考え方は[探索ガイド](../../docs/strategy-guide.md)を参照してください。
