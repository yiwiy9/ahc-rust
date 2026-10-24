# 構成と境界

`ahc-rust` は競技・ローカル採点、別repoの `ahc-studio` は配信を担当する。各contestは独立したCargo workspaceで、Cargo.lockとtargetを閉じ込める。配信環境は確定したresults/latest.jsonを読み、競技コードを変更・起動しない。

## ソースの正本

生成元は `templates/contest/src` の入出力・State・構築エンジンと、`templates/framework`・`templates/states`・`templates/snippets`・`templates/solvers` に分けて管理する。

newはincludeを再帰展開し、コンテストには次を生成する。

- `src/bin/a.rs`: 型付き入出力、ConstructiveState、貪欲、mainが入る一枚。
- `samples/*.rs`: a.rsの複製へ貼る、探索ごとの追加部分。
- `samples/complete/*.rs`: introの入出力とStateまで含む動く完成例。

生成後の正本はコンテスト内の各bin。build/run/exportで元テンプレートから再生成せず、手元の探索関数も自由に編集する。テンプレート更新は既存の解法へ自動反映しない。

## 探索と状態

ConstructiveStateは貪欲・乱択・ビームで共用する。入力全体をcloneせず、各メソッドへ&Inputを渡す。候補評価と状態生成を分け、ビームでは親・手・評価値で上位を選んだ後に採用Stateだけをcloneする。

局所探索は完成解に対する別StateとMoveを持つ。directは出力を直接変更、rebuildはパラメータから出力と評価をまとめて再構築する。どちらもintroで実際に変化する近傍を実装し、apply/undo・キャッシュ・再構築の整合を検算する。

## 実行の入口

公開する日常コマンドはscriptsに統一する。ahc-cliはスクリプトの実装と外部連携用の互換入口として残す。旧addは新しい一枚構成のコピー手順を表示し、既存ソースを変更しない。古い分割APIへ新しいサンプルの一部分だけを注入しない。

共通libの実体はABC側一つ。referencesはその検索用リンク。solverが必要とするコードはcontestへコピーする。
