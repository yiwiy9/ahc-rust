# 外部・グローバル影響の記録

## 方針

AHC環境の作成・利用で、既存の `atcoder-rust`、シェル設定、PATH、グローバルCargo/npmパッケージを変更しない。

## 初期構築時に行ったこと

- `/private/tmp/ahc-workspace` で新規リポジトリを作成した。
- `ahc-studio` の依存関係はそのリポジトリ内の `node_modules` にのみインストールした。
- pahcer 0.4.0は `ahc-rust/.tools` にのみインストールする。PATHやグローバルCargo binaryは変更しない。
- pahcerの初回ビルドはCargoの通常のregistry cacheを利用する。構築確認時、lock済みのyankされた `chacha20 0.10.0` と将来Rustで拒否予定の依存に関する警告が出たが、pahcer 0.4.0自体のビルドとAHC070実行は成功した。
- npm、Rust、Docker、VS Codeのグローバル設定は変更していない。
- Rust toolchainを追加インストールしていない。Cargoには `rust-version` を記録し、特定toolchainを自動導入する `rust-toolchain.toml` は置かない。
- 既存の `/Users/yiwiy/Codes/atcoder/atcoder-rust` は読み取りだけに使用した。
- `practice-algorithm-rust-snippets` を兄弟ディレクトリ `atcoder-lib` へcloneした。構築時revisionは `c308629`。
- AHC070公式tool ZIPをAtCoderから新リポジトリ内へ取得し、入れ子のCargo workspaceとしてビルドできるよう公式 `tools/Cargo.toml` 末尾へ空の `[workspace]` を追加した。solver/tool本体は変更していない。

## 2026-09-13: introの完成例を追加

- 旧環境から解答5本と公式toolsのソース・lock・seed一覧を `contests/intro-heuristics` へコピー。旧ファイルは変更・削除していない。
- 新環境の `ahc.toml` とコンテスト用 `ahc` ランチャーを追加し、既存の実行・採点・保存コマンドで扱う。VS CodeのlinkedProjectsへCargo.tomlを追加した。
- ビルド時に不足していた公式toolsの依存をCargo registry cacheへ取得した。既存キャッシュの削除、グローバルコマンド・toolchainのインストール、PATH・シェル・VS Codeユーザー設定の変更はない。
- ケース・ビルド成果物・計測結果はintro配下のGit無視対象。登録を戻す場合は追加したコンテストとlinkedProjectsの該当項目だけが対象で、旧環境への復旧作業は不要。Cargoの共有キャッシュは他プロジェクトでも使うため一括削除しない。

最終配置先は `/Users/yiwiy/Codes/atcoder/ahc-workspace`。このディレクトリを削除すれば、今回追加する環境全体を元に戻せる。

pahcerだけ撤去する場合は `ahc-rust/.tools` を削除する。計測履歴は各contestの無視対象 `results/pahcer` にあり、solverコードとは分離されている。

任意の `scripts/check-linux.sh` を実行した場合だけ、Dockerが `rust:1.89-bookworm` imageとビルドcacheを保持することがある。不要ならDocker Desktopからimage/cacheを削除できる。
