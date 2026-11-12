# 言語

<p><a href="languages.md">English</a> · <a href="languages.fr.md">Français</a> · <a href="languages.de.md">Deutsch</a> ·
<a href="languages.it.md">Italiano</a> · <b>日本語</b> · <a href="languages.zh-CN.md">中文（简体）</a></p>

IDE TEA が各言語ですることと、しないことです。IDE は、自分でインストールしたツール
(コンパイラ、言語サーバー、デバッガー) を統合します。ダウンロードはしません。ツールがなくても、ファイルは色付きで開き、
編集はできます。足りないのはツールが必要な部分だけで、IDE は何をインストールするかを示します。

機能の一覧は [features.ja.md](features.ja.md) を参照してください。

## 概要

| 言語 | ファイル | 言語サーバー (既定のコマンド) | タスク (`Ctrl+Shift+B`) | デバッグ (`F5`) |
|---|---|---|---|---|
| Rust | `.rs` | `rust-analyzer` | cargo | gdb。cargo でビルド |
| C / C++ | `.c` `.h` `.cc` `.cpp` `.cxx` `.hpp` `.hh` | `clangd` | Makefile のターゲット | gdb。`[debug]` セクションが必要 |
| Python | `.py` `.pyi` | `pyright-langserver --stdio` | — | — |
| TypeScript / JavaScript | `.ts` `.tsx` `.js` `.jsx` | `typescript-language-server --stdio` | `package.json` のスクリプト | — |
| Go | `.go` | `gopls` | — | — |
| Lua | `.lua` | `lua-language-server` | — | — |
| Zig | `.zig` `.zon` | `zls` | `build.zig` のステップ | gdb。`zig build` でビルド |
| [Ziex](https://ziex.dev) | `.zx` `.mdzx` | `zx lsp` | `build.zig` のステップ | — |
| [Hare](#hare) | `.ha` | `hare-lsp -S` | Makefile のターゲット | — |
| [MoonBit](#moonbit) | `.mbt` `.mbti` `.mbt.md` | `moon lsp` | `moon check`、`build`、`test`、`fmt --check` | gdb。`moon build --target native -g` でビルド (実験的) |

言語サーバーが提供するもの (補完、名前の変更、フォーマットなど) は、サーバー次第です。IDE は、サーバーが宣言したものを使います。
どの言語にも色が付きます。Hare、MoonBit、Ziex は、IDE に専用の文法を同梱しています。

## Hare

[Hare](https://harelang.org) は、小さなシステム言語です。言語サーバーはまだ若く、IDE はそれを隠さず、そのように示します。

**インストール**

- `hare` はディストリビューションから (Arch では `pacman -S hare`)。
- `hare-lsp` は Hare に同梱されていません。[リポジトリ](https://git.sr.ht/~whynothugo/hare-lsp) からビルドします
  ([`hare-json`](https://git.sr.ht/~sircmpwn/hare-json) ライブラリが必要。Arch では AUR にあります)。実行ファイルを `PATH` に置きます。例は `~/.local/bin` です。

  ```sh
  make build PREFIX=/usr          # PREFIX: Hare の標準ライブラリのインストール先
  install -Dm755 hare-lsp -t ~/.local/bin/
  ```

**2026-11-09 に確認。** Hare 0.26.0.1 と `hare-lsp` `e3db7b8` です。

| | |
|---|---|
| 色 | あり |
| ホバー | あり |
| 定義へ移動 (`F12`)。モジュールをまたぐ | あり |
| アウトラインと、ファイル内のシンボル (`Ctrl+Shift+O`) | あり |
| 構文エラー | あり |
| 型エラーと、その他の意味検査 | なし (サーバーにはまだない) |
| 補完、参照、名前の変更、フォーマット | なし (サーバーにはまだない) |
| インレイ ヒント | サーバーにはある。IDE はまだインレイ ヒントを表示しない |
| タスク | Makefile のターゲット。Makefile だけでは Hare とは限らないため、IDE は `hare` のタスクを追加しない。`[tasks]` に追加する |

**既知の問題:** Hare 自身のパーサー (`hare::parse`。`hare-lsp` が使用) は、`;` がない文でクラッシュします。
例は、次の行が続く `let x = 1` です。IDE は最初の 1 回を伝え、入力を止めるとサーバーを自分で戻します。

**試す:** [`examples/languages/hare`](../examples/languages/hare) を開き、タスクから `check` を実行します。

## MoonBit

[MoonBit](https://www.moonbitlang.com) は、言語サーバーをツールチェーンに同梱しています。他にインストールするものはありません。

**インストール:** [公式インストーラー](https://www.moonbitlang.com/download)。`moon` を `~/.moon/bin`
(または `$MOON_HOME/bin`) に置きます。IDE もそこを見るので、アプリ メニューから開いても動きます。

**2026-11-09 に確認。** `moon 0.1.20261104` です。

| | |
|---|---|
| 色。文字列補間を含む | あり |
| `.mbti` のインターフェース ファイル | あり |
| ホバー | あり |
| 型エラー | あり |
| 補完、定義へ移動、サーバー経由のフォーマット | `moon lsp` が提供するもの。1 つずつは未確認 |
| タスク | `moon check`、`moon build`、`moon test`、`moon fmt --check` |
| デバッグ (`F5`) | 実験的。ネイティブ バックエンドと gdb。下記 |
| `.mbt.md` (MoonBit のコードを含む Markdown) | ` ```mbt ` と ` ```moonbit ` のブロックに色。ブロック内のホバー。`moon test` は ` ```mbt check ` のブロックを実行 |

タスクは、開いているファイルに最も近い `moon.mod` のフォルダーで実行します。ワークスペースの各モジュールは、自分のタスクを実行します。
開いているファイルがなければ、プロジェクト ルートの `moon.mod` を使います。古い `moon.mod.json` ではタスクは有効になりません。
MoonBit は、`moon fmt` での移行を求めています。

**デバッグ** (2026-11-10 に確認。Linux、gdb 18): 開いているフォルダーのルートに `moon.mod` があり、設定に `[debug]`
のビルドがないとき、`F5` は `moon build --target native -g` を実行し、ビルドしたプログラムをデバッグします。
`.mbt` ファイルのブレークポイント、ステップ実行、正しいファイルと行の呼び出しスタック、数値とその他の単純な変数の値は動きます。
まだ粗いもの:

- 呼び出しスタックの関数名は、コンパイル後の名前です。`sum_to` は `_M0FP26idetea7example7sum__to` のようになります。
- 文字列は UTF-16 のバイト列として出ます (`"M\000o\000o\000n..."`)。
- ステップの直後、変数が `<optimized out>` と出ることがあります。
- ネイティブ バックエンドだけです。Wasm と JavaScript のプログラムは、IDE からデバッグできません。

`.mbt.md` では、コード ブロックの外は Markdown、中は MoonBit です。Markdown のプレビューは、通常の
`.md` と同じように動きます。IDE はそこでは小さな独自の Markdown 文法を使うので、残りのテキストの色は、
プレーンな `.md` より少なくなります。

**試す:** [`examples/languages/moonbit`](../examples/languages/moonbit) を開き、`sum_to` にホバーし、タスクから
`moon test` を実行します。`sum.mbt.md` には、Markdown の中にテストがあります。

## ここにはない言語

設定の 3 つで、IDE を変えずに、ほとんどの言語をカバーします。

```toml
# ~/.config/idetea/config.toml、またはプロジェクトの .idetea/config.toml
[lsp.servers.mylang]
command = "mylang-lsp"
args = ["--stdio"]
languages = { ml = "mylang" }    # 拡張子 = サーバーへ送る languageId

[tasks]
"mylang test" = "mylang test"
```

色には、`.sublime-syntax` の文法を `~/.config/idetea/syntaxes/` に置きます。そこにある文法は、同じ拡張子の
組み込み文法も置き換えます。`[lsp.servers.<name>]` は、上の表のサーバーを変更するか、オフにできます (`enabled = false`)。

他の IDE が置いていかない言語を使い、組み込みにしてほしいときは、どのプロジェクトで試すかを書いて issue を開いてください。
