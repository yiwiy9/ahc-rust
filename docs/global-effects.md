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

## 2026-09-14: AHC071後の使い勝手修正

- 変更範囲はahc-rust内。旧atcoder-rustと兄弟atcoder-libは参照用コピー作成の読み取りのみ。
- VS Codeの変更はリポジトリ内の設定とworkspaceファイルのみ。ユーザー設定・拡張インストール・シェル・PATH・toolchainは変更していない。
- referencesにはRustソースの検索用コピーを生成。元のパスとハッシュをmanifest.jsonへ記録。元コードには編集が波及しない。
- 検証ビルドは既存Cargoキャッシュを使うoffline実行。成果物はリポジトリ内のtarget/.toolsおよびintroの既存target。ネットワーク経由の依存導入はしていない。
- AHC071のソース、manifest、提出ファイル、当日の実行記録は不変。追加はscriptsの4ファイルのみ。
- 設定を戻す場合は今回のGit差分のうちエディタ設定・workspace設定が対象。生成referencesは元から再生成できるコピー。検証用コピーは .tools/template-check-* と .tools/workflow-check-* に区別して残している。.tools全体にはpahcerやスニペット用ツールもあるため一括削除しない。
- コミット、push、Gist更新、外部公開は行っていない。

## 2026-09-16: 参照をシンボリックリンクへ変更

参照コピーの更新作業をなくすため、references/atcoder-libとreferences/abcを元ディレクトリへのリンクに変更。旧コピーはハッシュ検査後に.tools/reference-copy-backup-*へ退避して保持。元のコード・権限は変更しない。VS CodeとCLI検索はリンクを辿り、rust-analyzerのlinkedProjectsは変更しない。VS Codeの参照パスは読み取り専用（OSレベルの保護ではない）。手順はdocs/environment.mdを参照。

実際の退避先は `.tools/reference-copy-backup-63k4c066/references`。VS Codeでdedup検索・リンク先のRead-only表示・AHC071のsolve_greedyへの定義ジャンプを確認した。補助スクリプトの3テスト（同期なし反映、編集保護と退避、既存コマンド互換）も成功。

## 2026-09-16: 共通libの実体をABC側へ統一

- ABC側の `atcoder-rust/src/lib/src` とAHC側cloneは、全branch/refが同じ `c3086295e850bcbb0ca23bab5c480bfb68f6002c` で、両方のworktreeに独自変更なし。ファイル比較ではABC側だけにある `.gitkeep` 以外一致した。
- `references/atcoder-lib` を `../../../atcoder-rust/src/lib/src` へ変更。workspace.toml、CLIの既定値、スニペット生成はこの参照口を使う。setup-common-lib.shもcloneせず、既存ABC側へのリンクを設定する。
- 不要な `/Users/yiwiy/Codes/atcoder/ahc-workspace/atcoder-lib` は `/Users/yiwiy/.Trash/ahc-atcoder-lib-20260916` へ移動。元の配置からは撤去済みで、ゴミ箱から復元できる。ABC側のコード・Git設定・権限は変更していない。
- ABC側ソースからAHCのVS Codeスニペットを再生成し、従来の生成結果と差分なし。別cloneを維持する必要はなくなった。スニペットの生成済みJSON自体はAHCリポジトリに残す。
