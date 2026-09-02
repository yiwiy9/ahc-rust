# 0002: 探索モデルを明示的に分ける

## 決定

構築型探索と局所探索を別のStateとして扱う。

- `ConstructiveState`: 貪欲、乱択構築、ビームで共用する。
- `DirectLocalSearchState`: 完成したOutput自体をMoveで変更する。
- `RebuildLocalSearchState`: Parametersだけを変更し、Outputを必ず再構築する。

## 防ぐバグ

AHC070では移動ベクトルを変更したあと、古い操作列を保持したまま経路と得点だけを再計算した。再構築型では `Parameters -> construct_output -> Output -> exact_score` を一つの更新として扱い、派生データだけを直接変更できない構成にする。

## 検証

デバッグ時には、得点の全体再計算だけでなく、Parametersから再構築したOutputとの一致と、Moveのapply/undo後のState一致を確認する。
