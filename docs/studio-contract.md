# ahc-studioとのデータ契約

競技側が書くのはcontest配下だけです。

- `results/latest.json`: 最新の複数seed計測
- `results/events.jsonl`: 計測・snapshot保存などの追記イベント

`ahc-studio` は `results/latest.json` と `results/benchmarks/*.json` を読む側であり、solverソース、出力、公式ツールを変更しません。builtinとpahcerの出力はCLIが同じJSONへ正規化し、runner、並列数、相対・順位スコア、AC数、最大実行時間も含めます。直近の計測はStudio上でも同条件の行を比較できます。`events.jsonl` は競技側の履歴として保存し、必要になった時だけ連携対象へ加えます。

順位やAtCoder得点はこのローカル計測とは別の決定的データです。AIへ数値を推測させず、順位取得adapterまたは手入力からstudioの状態へ保存します。
