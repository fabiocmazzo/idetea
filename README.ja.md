<p align="center">
  <img src="assets/idetea.svg" width="112" alt="IDE TEA のアイコン: アイスティーのグラス">
</p>

<h1 align="center">IDE TEA</h1>

<p align="center"><b>コードだけでいい。氷はあとで。</b></p>

<p align="center">
  純粋な Rust で作った、高速で静かな IDE です。コーディング エージェントは、動く前に尋ねます。<br>
  ずっと無料: <a href="LICENSE">Apache 2.0</a>。アカウントなし、テレメトリなし、マーケットプレースなし。
</p>

<p align="center">
  <a href="https://github.com/fabiocmazzo/idetea/actions/workflows/ci.yml"><img src="https://github.com/fabiocmazzo/idetea/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://github.com/fabiocmazzo/idetea/releases"><img src="https://img.shields.io/github/v/release/fabiocmazzo/idetea?include_prereleases&amp;label=release" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue" alt="License: Apache 2.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024-orange?logo=rust" alt="Rust 2024"></a>
  <a href="https://github.com/iced-rs/iced"><img src="https://img.shields.io/badge/GUI-iced%200.14-3645ff" alt="GUI: iced 0.14"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey" alt="Platforms: Linux, macOS, Windows">
  <a href="https://codeberg.org/fabiocmazzo/idetea"><img src="https://img.shields.io/badge/Codeberg-mirror-2185D0?logo=codeberg&amp;logoColor=white" alt="Codeberg mirror"></a>
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.pt-BR.md">Português</a> · <a href="README.es.md">Español</a> ·
  <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.it.md">Italiano</a> ·
  <b>日本語</b> · <a href="README.zh-CN.md">中文（简体）</a><br>
  <a href="https://idetea.belabs.cloud/ja/"><b>Web サイトとガイド</b></a>
  · <a href="https://github.com/fabiocmazzo/idetea">GitHub</a>
  · <a href="https://codeberg.org/fabiocmazzo/idetea">Codeberg</a> (ミラー)
</p>

<p align="center">
  <a href="https://idetea.belabs.cloud/ja/#demo">
    <img src="assets/demo-preview.gif" width="880" alt="ナレーション付きデモ: クイック オープン、構文の強調表示がある Rust、分割エディター、統合ターミナル、そのあとインターフェースのエージェントのフォーム。マスコット Tea は注目に切り替わります。">
  </a>
</p>

<p align="center">
  <a href="https://idetea.belabs.cloud/ja/#demo"><b>52 秒のデモを見る</b></a>
  · 英語のナレーション。字幕は映像に入っています<br>
  GitHub は、リポジトリ内の動画ファイルを README の中では再生しません。
  上の画像は Web サイトでデモを開き、音声付きで再生します。
  ファイルは <a href="assets/idetea-demo.mp4"><code>assets/idetea-demo.mp4</code></a> です。
</p>

---

> **状態:** 1.0 より前の、開発中です。Linux、macOS (Apple Silicon)、Windows (x86_64) でビルドとテストをしています。
> 粗い部分があります。見つけたら報告してください。

## 対象

エディターのそばのターミナルで、エージェントとコードを書く人向けです。Claude Code、Codex、または
`AGENTS.md` と `CLAUDE.md` を読むエージェントです。IDE TEA は、ウィンドウの中にチャット モデルを置きません。
エージェントには、ファイルを使った小さな質問の手段を渡し、すべてのコマンドは承認制です。

## 使い続ける理由

**高速。** 起動は 200 ms を大きく下回ります。50 MB のファイルは 0.1 秒で開きます。1 行が巨大なミニファイ済みファイルは、ウィンドウを固める代わりに、軽量モードで開きます。

**静か。** アカウントも、テレメトリも、マーケットプレースも、頼んでいないポップアップもありません。ネットワークを使うのは、要求したときだけです (プッシュ、プル、フェッチ、またはインストールするスキル)。外観、キー、パネルは、1 つのプレーンな TOML ファイルにあります。

**実行の承認。** エージェントが小さな JSON を書くと、IDE はフォームを表示します。選択肢、チェックボックス、テキスト、画像、音声、手順のある手動テストです。ターミナルの要求は、先にコマンド全文を表示します。ヘルパーは、専用の git worktree で実行できます。プロジェクトが自分に許可を与えることはできません。許可リストは、利用者の設定にあります。

**必要なところは一通り。** マルチ カーソル、一時的なズーム、任意の vim モード、ファイル ツリー、プロジェクト検索、統合ターミナル、git (ステージ、差分、コミット、プッシュ、プル)、LSP、gdb によるデバッグ、Markdown、SVG、画像、PDF のプレビューです。

**中身は小さい。** コアが編集するのはファイルだけです。ツリー、ターミナル、git、デバッグは、オフにできる拡張機能です。ピクセル アートのティーカップ Tea は、待っている質問を見張り (1 クリックで該当パネルを開きます)、隅を戻したいときはステータス バーへ折りたたみます。

1 つのコマンドが、プロジェクトのエージェントに教えます。「インターフェース: このプロジェクトのエージェントに教える」は、手順を
`AGENTS.md` と `CLAUDE.md` に書きます。

## インストール

**Linux、macOS、Windows。すぐに実行できます。** プラットフォーム用のパッケージを
[Releases](https://github.com/fabiocmazzo/idetea/releases) からダウンロードします。Linux x86_64/aarch64 は `.tar.gz` または `.AppImage`、
macOS aarch64 は `IDE TEA.app` 入りの `.tar.gz`、Windows x86_64 は `-setup.exe` インストーラーまたはポータブルな `.zip` です。
`SHA256SUMS` にチェックサムがあります。

Linux では、`./install.sh` が `idetea` を `~/.local/bin` に置き、アプリ メニューに追加します。AppImage はそのまま実行します
(先に `chmod +x`)。`--install-desktop` でメニューに追加します。macOS では、`--install-desktop` が
`~/Applications/IDE TEA.app` を作ります。

**ソースから。** [Rust](https://rustup.rs) (stable) が必要です。そのあと:

```sh
cargo install --locked --git https://github.com/fabiocmazzo/idetea tea-app
idetea --install-desktop   # 任意: アプリ メニューの項目とアイコン
```

任意: git 統合用の `git`、デバッグ用の `gdb` 14 以降 (macOS では `lldb-dap`)、使う言語サーバーです (rust-analyzer、clangd、pyright、typescript-language-server、gopls、lua-language-server、zls、`zx lsp`、Hare 用の `hare-lsp`、MoonBit 用の `moon lsp` は設定済みです)。

## コード署名とプライバシー

Windows の無料コード署名は [SignPath.io](https://signpath.io) が提供し、証明書は
[SignPath Foundation](https://signpath.org) のものです。このプログラムは、利用者が明示的に要求しない限り、情報を他のネットワーク上のシステムへ転送しません。
[`CODE_SIGNING_POLICY.md`](CODE_SIGNING_POLICY.md) を参照してください。

## クイック スタート

```sh
idetea                # 現在のフォルダーをプロジェクトとして開く
idetea src/main.rs    # 現在のフォルダーと、このファイル
idetea ~/project      # 別のフォルダー
idetea new my-app --model rust --agent claude     # 新しいプロジェクト。開始できるエージェント付き
```

IDE の中では、`Ctrl+Shift+P` がコマンド パレット、`Ctrl+P` が名前でファイルを開く、`F1` がすべてのショートカットです。
ガイドは [Web サイト](https://idetea.belabs.cloud/ja/) にあります。

## IDE TEA を選ぶとき、選ばないとき

[`docs/comparativo-ides.ja.md`](docs/comparativo-ides.ja.md) は、VS Code、Cursor、Zed、JetBrains、
Helix との率直な比較で、0.4.0 前後に書かれています。今も製品の核である部分:

- ターミナルのエージェントは、ファイルベースのネイティブな手順を得ます (フォーム、承認したコマンド、worktree、マスコット Tea)。あの文書の他のエディターには、これがありません。

### Tea

隅のティーカップには、気分ごとにいくつかの台詞があります。保存、長い休止、時刻、デバッグのビルドに気づき、
実在するショートカットを時々示します。`[mascot] chatty` は `quiet`、`normal`、`chatty` です。
`[mascot] tips` はヒントのオンとオフです。インターフェース全体 (Tea を含む) は、ポルトガル語、英語、
スペイン語、フランス語、ドイツ語、イタリア語、日本語、簡体字中国語です (`[ui] language`、またはシステムの言語)。
フォーカスは取りません。
- ゴースト テキストの補完は、意図してありません。Tab 補完やインライン差分が必要なら、あの文書は Cursor、Windsurf、Zed を示します。
- コード アクション、インレイ ヒント、デバッグの対応範囲は、VS Code と JetBrains より後ろです。

同じ文書は、まだ「Linux のみ」と書いています。それ以降のリリースでは、macOS (Apple Silicon) と Windows (x86_64) も配布しています。

## リポジトリにスターを付ける

探していたエディターなら、[IDE TEA にスターを付けてください](https://github.com/fabiocmazzo/idetea)。1 人のプロジェクトで、
スターが次の人の見つけ方です。

## 名前について

冗談から始まりました。IDE は **iced** で作られているので、**Iced Tea** になるほかありませんでした。

それから、もう一つの意味が現れました。ポルトガル語では、**TEA** は *Transtorno do Espectro Autista*（自閉スペクトラム症）です。
作者は自閉スペクトラム症で、この二重の意味がしっくりきたので、名前は残りました。IDE のいくつかの性質も、これで説明がつくかもしれません。
予測できること、読めるプレーンなテキスト、できるだけ少ない雑音を好みます。

詳しくは [マニフェスト](MANIFESTO.ja.md) です。

## ドキュメント

- [マニフェスト](MANIFESTO.ja.md): IDE TEA が信じることと、しないこと。
- [Web サイト](https://idetea.belabs.cloud/ja/): デモ、インストール、ガイド。
- [機能とショートカット](docs/features.ja.md): 一覧。
- [言語](docs/languages.ja.md): 各言語で IDE がすること。Hare と MoonBit を含み、
  試す [例](examples/languages) があります。
- [エージェント向けインターフェース](docs/interfaces/formato.md) と [エージェント ターミナル](docs/terminais-de-agente.md)
  （現時点ではポルトガル語）。
- [SPEC.md](SPEC.md) と [DESENVOLVIMENTO.md](DESENVOLVIMENTO.md): 仕様、アーキテクチャ、ビルド、テスト、
  拡張機能の書き方（現時点ではポルトガル語）。

## ライセンス

IDE TEA は [Apache License 2.0](LICENSE) のフリー ソフトウェアです。使用、研究、変更、共有ができます。
商用も同様です。無償であり、これからも無償です。

インターフェースのアイコンは [Codicons](https://github.com/microsoft/vscode-codicons) © Microsoft で、
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) です
([`crates/tea-core/assets/NOTICE.md`](crates/tea-core/assets/NOTICE.md) を参照)。
