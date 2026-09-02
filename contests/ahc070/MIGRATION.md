# 移植検証

旧 `atcoder-rust/src/contest/ahc070` は読み取りだけにし、solver断面を新リポジトリへコピーしました。

2026-09-02のMacネイティブ検証結果:

| solver | seed 0 | 旧snapshotのseed 0 | 備考 |
|---|---:|---:|---|
| `a` | 1,347,429 | 1,433,961 | 1.9秒の時間探索なので実行環境で変動 |
| `beam` | 800,894 | 800,894 | 決定的に一致 |
| `sa_improved` | 2,374,530 | 2,580,632 | 1.6秒の時間探索なので実行環境で変動 |

`a` はseed 0〜2の公式tool採点も実行し、平均1,373,079.33でした。これは移植動作の確認値であり、AtCoderの本番得点や今後の比較基準を置き換えるものではありません。

CLIのrelease/debug実行、公式visualizer HTML生成、複数seed JSON、snapshot、検索用solution、単一ファイルexportまで確認しています。
