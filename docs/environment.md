# エディタ・実行環境

## 開くフォルダ

VS Codeで **ahc-rustフォルダ単体** を開く。`ahc-rust.code-workspace` も同じ一つのフォルダを開く互換入口になっている。コンテストの追加は `./ahc new ahcNNN`、手動配置後は `./ahc vscode sync`。

Explorerの配置:

- `contests/`: 今の実装と、過去AHC・introの実装
- `references/atcoder-lib/`: ABC側の `atcoder-rust/src/lib/src` へのリンク（AHC用の別cloneは持たない）
- `references/abc/`: ABCなど旧環境の実装のシンボリックリンク
- `templates/`: 次のコンテストで生成するテンプレート

`⌘⇧F` で `dedup` などを検索する。範囲を絞る場合は「含めるファイル」に `references/abc/**/*.rs` を指定する。リンクはGit対象外だが、プロジェクトの検索設定で対象に含めている。生成物・公式toolsは検索から除く。

この環境では設定済み。**元ファイルの変更がそのまま見えるので、同期・更新コマンドは不要。** 別端末でcloneしたときだけ、初回にリンクを作る。

```sh
python3 scripts/setup-references.py
```

別の配置では `--abc /path/to/atcoder-rust/src/contest --lib /path/to/atcoder-rust/src/lib/src` を指定する。リンクはローカルのファイルを指し、GitHubと自動同期する機能ではない。普段このMacで更新したABCコードはそのまま見える。別端末で更新した内容のpullが必要な場合も、元リポジトリ側だけでよい。

VS Codeの `references/` 経由では読み取り専用。これはエディタの誤操作防止であり、ファイルシステムの権限制限ではない。ターミナルや別のエディタからリンク先を編集すると元も変わる。元リポジトリの権限は変更しない。提出へ必要な実装はコンテスト側にコピーする。

以前のコピーは、独自編集がないことをハッシュで確認してから `.tools/reference-copy-backup-*/references/` に退避する。旧 `refresh-references.py` もリンク設定を呼ぶだけになっており、元ファイルを書き換えない。

スニペットも従来通り `⌃Space` で利用できる。共通lib更新後は `./scripts/sync-vscode-snippets.sh`。自動候補表示を切っているので、候補が勝手に出ないこととrust-analyzerが動かないことは別。

補完が動かなければ `./ahc vscode sync` → VS Codeの `Rust Analyzer: Restart server`。`出力 → Rust Analyzer Language Server` にエラーがないか確認する。保存時はcargo checkを使い、clippyの追加指摘は必要なときに手動実行する。

## 普段使うコマンド

`cd contests/ahcNNN` して実行する。引数の順序は **bin名、seedまたはケース数**。省略すると `a`、seed 0 / 10ケース。

```sh
./scripts/build.sh a
./scripts/run-one.sh a 0
./scripts/debug.sh a 0
./scripts/run-all.sh a 10
./ahc export --solver a --clipboard
```

各スクリプトはCargoの差分ビルドを使う。変更がなければ再コンパイルしない。`run-one` はreleaseで実行・採点・可視化、`debug` は検算とバックトレースを有効化する。エラーはターミナルと `results/logs/`、解は `out/` に残る（詳細パスは実行表示）。`run-all` はseed 0から順に計測する。`./ahc all --solver a --cases 10 --builtin` も同じ入口。

高速な並列比較が必要になったら `./ahc bench --solver a --cases 10 --threads 4`。こちらはpahcerを使う。本番の時間確認は並列計測だけで判断せず、単独のrelease実行でも確認する。

## テンプレートの入力とState

`Input` はmainで一つだけ保持し、`solve_greedy(&input, state)` などへ参照で渡す。Stateの各メソッドにも `&Input` が渡る。Stateには途中の解とキャッシュだけを置く。入力全体をコピーする必要はなく、`Input: Clone` の制約もない。

貪欲・乱択・ビームは同じ構築用Stateを使う。局所探索は完成解を変更する別のStateを使うが、初期解は同じ貪欲関数から取得する。direct / rebuildどちらも最初は山登り。Moveとundoを確認してから焼きなましへ切り替える。

旧コンテストのコードは当時のAPIのまま保存する。新APIのテンプレート追加は旧APIを検出したらコピー前に止める。旧コンテストを復習する場合は当時の実装を使うか、別途API全体を移行する。一部分だけ混ぜない。

## 依存ライブラリ

生成するCargo.tomlは[AtCoderのRust 1.89.0の一覧](https://img.atcoder.jp/file/language-update/2025-10/language-list.html)にあるライブラリを固定バージョンで宣言する。superslice、ac-library-rs、itertoolsなどを毎回足す必要はない。全依存の初回ビルドには時間がかかるため、事前にテンプレート検証を実行してキャッシュを作る。

ローカルrustcと提出先rustcのバージョンが違う場合、新しい標準APIが提出先にない可能性は残る。既存のグローバルtoolchainを自動変更・インストールはしない。

## 環境側の回帰テスト

問題を解く際は不要。テンプレート・ツールを変更した後の開発者向け検証。

```sh
cargo test --offline -p ahc-cli
python3 -m unittest discover -s tests -p 'test_*.py'
python3 scripts/check-templates.py
python3 scripts/check-workflow.py
```

後半二つは `.tools/` に隔離したコピーを生成し、実際のコンテストのソースや記録を書き換えない。検証用ディレクトリは調査のため残す。依存がキャッシュにない端末ではoffline実行に失敗するので、通信可能な事前準備時に依存取得が必要。

debugでは候補の採否に関係なく `debug_validate` を呼ぶ。問題固有の検算を書かなければ何も検出できない。重い検算ではdebugが遅くなるため、時間評価はrelease、正しさの確認は小さい入力・固定反復で分ける。

本番利用前には、更新したテンプレートが実際に事前公開され、提出コードのURLがその内容を参照することを別途確認する。ローカルでの修正・検証成功だけでは公開済みにならない。
