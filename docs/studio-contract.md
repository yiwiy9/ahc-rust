# ahc-studioとのデータ契約

競技側が書くのはcontest配下だけです。

- `results/latest.json`: 最新の複数seed計測
- `results/events.jsonl`: 計測・snapshot保存などの追記イベント

`ahc-studio` は現在 `results/latest.json` だけを読む側であり、solverソース、出力、公式ツールを変更しません。`events.jsonl` は競技側の履歴として保存し、必要になった時だけ連携対象へ加えます。JSONには `schema_version` を持たせ、将来フィールドが増えても古い画面が停止しにくい形にします。

順位やAtCoder得点はこのローカル計測とは別の決定的データです。AIへ数値を推測させず、順位取得adapterまたは手入力からstudioの状態へ保存します。
