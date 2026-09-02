# 外部・グローバル影響の記録

## 方針

AHC環境の作成・利用で、既存の `atcoder-rust`、シェル設定、PATH、グローバルCargo/npmパッケージを変更しない。

## 初期構築時に行ったこと

- `/private/tmp/ahc-workspace` で新規リポジトリを作成した。
- `ahc-studio` の依存関係はそのリポジトリ内の `node_modules` にのみインストールした。
- npm、Rust、Docker、VS Codeのグローバル設定は変更していない。
- Rust toolchainを追加インストールしていない。Cargoには `rust-version` を記録し、特定toolchainを自動導入する `rust-toolchain.toml` は置かない。
- 既存の `/Users/yiwiy/Codes/atcoder/atcoder-rust` は読み取りだけに使用した。
- `practice-algorithm-rust-snippets` を兄弟ディレクトリ `atcoder-lib` へcloneした。構築時revisionは `c308629`。
- AHC070公式tool ZIPをAtCoderから新リポジトリ内へ取得し、入れ子のCargo workspaceとしてビルドできるよう公式 `tools/Cargo.toml` 末尾へ空の `[workspace]` を追加した。solver/tool本体は変更していない。

最終配置先は `/Users/yiwiy/Codes/atcoder/ahc-workspace`。このディレクトリを削除すれば、今回追加する環境全体を元に戻せる。

任意の `scripts/check-linux.sh` を実行した場合だけ、Dockerが `rust:1.89-bookworm` imageとビルドcacheを保持することがある。不要ならDocker Desktopからimage/cacheを削除できる。
