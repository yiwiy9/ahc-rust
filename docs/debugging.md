# AHCとRustのデバッグ

## まず見る場所

`./ahc run 0 --debug` はsolverの標準エラーを `results/logs/...` に保存し、失敗時は端末にも表示します。panicでは `RUST_BACKTRACE=1` が有効です。通常出力は解として扱われるため、デバッグ表示は必ず標準エラーへ出します。

```rust
eprintln!("turn={turn} score={score} position={position:?}");
```

## assertの使い分け

- `assert!` / `assert_eq!`: releaseでも絶対守るべき前提。壊れた解を出し続けるより停止したい箇所
- `debug_assert!` / `debug_assert_eq!`: 全再計算など重い検算。debug buildだけで動く
- `eprintln!`: 値や経過を観察する。判定には使わない
- `dbg!`: 式の値と場所を素早く出す。所有権を意識し、提出前に消す

件数が少ないときだけ `debug_assert_eq!` を使う、というより「releaseの速度から外したい検算」に使います。毎100回、小さい入力だけ、など頻度も落とすと調査しやすくなります。

## 得点一致だけでは足りない

AHC070の事故のように、誤った行動列が偶然同じ内部得点を持つことがあります。debug時は別々の性質を確認します。

1. 出力形式・値域・個数が正しい
2. 各行動を先頭から再生できる
3. 再生後の位置や盤面がStateの保持値と一致する
4. Parametersから再構築したOutputが保持中のOutputと一致する
5. 差分得点が素直な全体計算と一致する
6. `apply_move` の後に `undo_move` するとState全体が戻る

`#[derive(Debug, Clone, PartialEq, Eq)]` はこの検証を安く書くために積極的に付けます。浮動小数点を含む場合は `Eq` を付けず、誤差つき比較にします。

## バグを固定する順序

```text
入力seed固定
乱数seed固定
AHC_ITERATIONSで反復数固定
debug build
失敗する最小反復数を探す
Move適用前後を出す
全再構築・全得点と比較する
```

時間制限だけで動かすと、実行速度の違いでMove列が変わり、直ったか判断しにくくなります。

## Rustで頻出する事故

- `usize` の引き算によるunderflow: 差は `i64` 等へ変換してから計算
- 0-index / 1-index混在: 内部は0-index、読み書きの境界だけ変換
- 合計型が小さい: 積を取る前に `i64` / `i128` へ変換
- cloneしたStateの一部だけ更新: 派生値の更新経路を1メソッドへ集約
- 時間計測開始が遅い: `Instant::now()` はmain冒頭
- `stdout` へのログ: 出力破損を避け `eprintln!`
