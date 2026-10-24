# 探索の追加

基本は **1解法1ファイル** です。自分が作った貪欲版を複製し、このフォルダの追加コードを末尾へ貼ります。入出力やStateをサンプルのintro実装に戻す必要はありません。追加コードにあるコメントどおりmainを変更してください。

貼るのは **新規生成されたコンテスト内の `samples/*.rs`** です。リポジトリ直下の `templates/` は保守用の生成元で、includeが残るため直接貼る用途ではありません。

| 追加部分 | 内容 | mainの変更 |
| --- | --- | --- |
| `beam.rs` | 軽量評価して上位だけ状態を作るビーム | solve_greedyの1行を差し替え |
| `beam_fast.rs` | 採用手の履歴を最後に再生するビーム | 同上 |
| `random_search.rs` | 評価上位K候補から乱択し、複数回構築 | 同上 |
| `local_search_direct.rs` | 完成解を1か所変更する焼きなまし | main冒頭で時刻を記録し、貪欲outputをimproveへ渡す |
| `local_search_rebuild.rs` | パラメータ変更後に解を再構築する焼きなまし | 同上 |

```sh
cp -n src/bin/a.rs src/bin/beam.rs
# このフォルダのbeam.rs全体を、src/bin/beam.rsの末尾へ貼る
# main: let state = beam::solve_beam(&input, State::new(&input), 100);
./scripts/run-one.sh beam 0
./scripts/run-all.sh beam 10
```

ビーム・乱択は同じConstructiveStateを使います。introのStateは軽量な `evaluated_value_after` を実装済みです。自分の問題に合わせて変える際も、評価時に履歴をcloneしないようにすると展開数が多い場合に効きます。debugでは通常の更新結果と照合するため、速さはreleaseで測ります。

ビームは「同じ段階の状態は同じ深さで完了する」構築を対象にした例です。可変長の完了解を混ぜる、重複状態を除く、候補を制限するといった変更は、自分のファイル内の探索関数へ直接加えられます。

焼きなましは **近傍の表現・更新・評価を問題に合わせて変更** します。サンプルのdirectは「開催する種類を1日だけ変える」、rebuildは「その日に評価上位何番目を選ぶかを変え、全日程を再構築する」例です。後者は遅くても正しい再構築を示すための実装です。山登りにする場合は `Acceptance::HillClimbing` へ変更します。

## 自分の問題へ適用するときに変える場所

入出力とConstructiveStateを自分の問題に合わせて作った後は、次の箇所を変更します。

| 探索 | そのまま使う部分 | 問題に合わせて変える部分 |
| --- | --- | --- |
| ビーム | 自作の構築State、候補評価と上位選抜のループ | mainの呼び出し・幅。必要なら重複排除等も直接変更 |
| 乱択貪欲 | 自作の構築State、上位K候補からの抽選・最良解保持 | mainの呼び出し・試行回数・K・乱数seed |
| directの山登り／焼きなまし | 採否・冷却・時間管理・最良解保持のengine | stateのOutputや評価関数への参照、Move、propose_move、apply_move、undo_move、debug_validate。最後に時間・温度 |
| rebuildの山登り／焼きなまし | 同じ局所探索engine | stateの型・評価関数へのimport、Parameters、ranked_actions、初期解をParametersへ変換するnew、construct_output、Move、更新・取り消し・検算。最後に時間・温度 |

焼きなましのサンプルは「変更操作まで自分の問題に合っている」ものではありません。introで動く近傍を見ながら、変更する対象と評価を差し替えて使う例です。採否・温度・時間管理のループは書き直す必要がありません。

rebuildの `ranked_actions` はintroのAction.contestを使う補助関数です。自分のActionに合わせて変更し、別の再構築方法にするなら削除します。使わなくなったintro固有の補助関数やimportも残さないでください。

迷ったらdirectから始めます。全体再計算でも正しく動かし、debugでapply→undo後に状態全体が戻ることを確認してから差分評価へ進みます。変更した後に後半を作り直す表現ならrebuildを使います。

## 完成例を先に動かす

`complete/` の各ファイルは、introの入出力・State・mainも含む自己完結した例です。

```sh
cp -n samples/complete/local_search_direct.rs src/bin/sa_intro.rs
./scripts/run-one.sh sa_intro 0
```

これはintro用です。別問題へ取り組んでいるときに完成例を丸ごと使うと入出力もintroへ戻るので、自分の解へ追加する場合は上の追加部分を使います。

乱数seedは42。乱択構築の反復数・上位候補数はmainで指定します。局所探索は通常1,800msまで、再現確認は次のように固定反復数で実行できます。

```sh
AHC_ITERATIONS=100 ./scripts/debug.sh sa_intro 0
AHC_SAMPLE_DELTAS=1 AHC_ITERATIONS=10000 ./scripts/run-one.sh sa_intro 0 --no-vis
```

サンプル内の温度はintroの例です。改悪幅を測り、問題に合わせて変更します。入力や初期解生成にも時間がかかるため、時間制限には余裕を持たせます。
