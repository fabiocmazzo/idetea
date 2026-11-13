## 設定 → 外観

settings-theme = テーマ
settings-theme-detail = IDE、エディター、ターミナルの色です。
settings-font-size = フォント サイズ
settings-font-size-detail = エディターとターミナルです。
settings-code-font = コードのフォント
settings-code-font-detail = エディター、行番号、ターミナルです。JetBrains Mono は IDE に付属しています。一覧は、インストール済みの等幅フォントです。
settings-font-weight = フォントの太さ
settings-font-weight-detail = フォントにない太さは、最も近いものになります。
settings-line-height = 行の高さ
settings-line-height-detail = フォント サイズの倍数です。ターミナルも同じ比率に従います。
settings-file-icons = ファイル アイコン
settings-file-icons-detail = 言語やファイルの種類ごとのアイコンです (VS Code の Seti セット)。オフにすると、すべてのファイルが同じアイコンになります。
settings-activity-bar = アクティビティ バー
settings-activity-bar-detail = 左端の帯です。1 クリックでパネルの表示と非表示を切り替えます。
settings-splash = 起動画面
settings-splash-detail = 起動画面です (少なくとも 2 秒)。--no-splash は、その起動だけオフにします。
settings-language = 言語
settings-language-detail = インターフェースの文言です。自動はシステムの言語に従います。IDE を再起動すると反映されます。
settings-language-auto = 自動 ({ $language })
settings-language-restart = IDE を再起動すると反映されます (現在: { $language })。

## コアのコマンド (パレットとヘルプのタイトル)
cmd-palette-open = コマンド パレット
cmd-help-open = ヘルプ: ショートカットとガイド
cmd-settings-open = 設定
cmd-file-new = ファイル: 新規
cmd-file-save = ファイル: 保存
cmd-file-open = ファイル: 開く…
cmd-file-quick-open = ファイル: 名前で開く…
cmd-debug-continue = デバッグ: 開始 / 継続
cmd-debug-stop = デバッグ: 停止
cmd-debug-pause = デバッグ: 一時停止
cmd-debug-next = デバッグ: 次の行
cmd-debug-step-in = デバッグ: ステップ イン
cmd-debug-step-out = デバッグ: ステップ アウト
cmd-debug-toggle-breakpoint = デバッグ: 行のブレークポイントを切り替え
cmd-debug-choose-program = デバッグ: プログラムを選択…
cmd-file-open-folder = ファイル: 新しいウィンドウでフォルダーを開く…
cmd-file-save-as = ファイル: 名前を付けて保存…
cmd-file-save-as-path = ファイル: 名前を付けて保存 (パスを入力)…
cmd-file-close = ファイル: タブを閉じる
cmd-tab-next = タブ: 次へ
cmd-tab-prev = タブ: 前へ
cmd-tab-reopen = タブ: 最後に閉じたタブを開き直す
cmd-editor-find = エディター: 検索
cmd-editor-replace = エディター: 検索と置換
cmd-editor-goto-line = エディター: 行へ移動
cmd-editor-undo = エディター: 元に戻す
cmd-editor-redo = エディター: やり直す
cmd-editor-select-all = エディター: すべて選択
cmd-editor-fold-all = エディター: すべてのブロックを折りたたむ
cmd-editor-unfold-all = エディター: すべてのブロックを展開
cmd-editor-toggle-fold = エディター: カーソル位置のブロックを折りたたむ/展開
cmd-editor-toggle-vim = エディター: vim モードの切り替え
cmd-editor-zoom-in = エディター: 拡大
cmd-editor-zoom-out = エディター: 縮小
cmd-editor-zoom-reset = エディター: ズームをリセット
cmd-lsp-definition = LSP: 定義へ移動
cmd-lsp-references = LSP: 参照を検索
cmd-lsp-implementation = LSP: 実装へ移動
cmd-lsp-type-definition = LSP: 型の定義へ移動
cmd-lsp-incoming-calls = LSP: この関数の呼び出し元
cmd-lsp-outgoing-calls = LSP: この関数の呼び出し先
cmd-nav-file-symbol = ファイル内のシンボルへ移動…
cmd-nav-project-symbol = プロジェクト内のシンボルへ移動…
cmd-nav-back = ナビゲーション: 戻る
cmd-nav-forward = ナビゲーション: 進む
cmd-lsp-hover = LSP: カーソル位置のドキュメントを表示
cmd-lsp-format = LSP: ファイルをフォーマット
cmd-lsp-restart = LSP: サーバーを再起動
cmd-theme-toggle = テーマ: ライト/ダークの切り替え
cmd-config-open = 設定: グローバル設定を開く
cmd-config-open-project = 設定: プロジェクト設定を開く
cmd-config-reload = 設定: 再読み込み
cmd-view-toggle-left = レイアウト: 左を表示/非表示
cmd-view-toggle-right = レイアウト: 右を表示/非表示
cmd-view-toggle-bottom = レイアウト: 下部パネルを表示/非表示
cmd-app-restart = IDE を再起動
cmd-app-quit = 終了
cmd-mascot-toggle = マスコット: 隅に表示/折りたたむ

## 通知、ダイアログ、メニュー
file-new-created-on-save = 新しいファイル: { $path } (保存時に作成)
file-open-error = { $path } を開けませんでした: { $error }
file-changed-on-disk-kept = "{ $name }" はディスク上で変わりましたが、タブに変更があるため、そのままにしました
file-reload-error = { $path } を読み直せませんでした: { $error }
file-changed-on-disk-overwrite = "{ $name }" はディスク上で変わりましたが、タブに変更があります。保存すると上書きします
debug-still-building = デバッグ: まだビルド中です…
debug-already-running = デバッグ: プログラムは既に実行中です
debug-building = デバッグ: ビルド中 ({ $command })…
unknown-error = 不明なエラー
debug-build-failed = デバッグ: ビルドに失敗しました: { $error }
debug-choose-title = どのプログラムをデバッグしますか?
debug-choose-message = プロジェクトは、実行ファイルを複数ビルドしました。
cancel = キャンセル
debug-running = デバッグ: { $program } を実行中
debug-error = デバッグ: { $error }
debug-cancelled = デバッグ: キャンセルしました (ビルドはそのまま終わります)
debug-stopped = デバッグ: 停止しました
debug-no-source = デバッグ: { $name } にソース コードがありません
debug-paused-no-source = デバッグ: { $name } で一時停止 ({ $reason })。ソース コードがありません
debug-paused-at = デバッグ: { $path }:{ $line } で一時停止 ({ $reason })
debug-exited = デバッグ: プログラムは終了しました
debug-exited-code = デバッグ: プログラムは終了しました (コード { $code })
debug-command-failed = デバッグ: { $command } が失敗しました: { $message }
debug-gdb-exited = デバッグ: gdb が終了しました ({ $reason })
debug-no-session = デバッグ: 実行中のセッションがありません (F5 で開始)
debug-already-paused = デバッグ: プログラムは既に一時停止しています
debug-pause-first = デバッグ: プログラムは実行中です (先に一時停止してください)
nav-nothing-forward = 進む先がありません
nav-nothing-back = 戻る先がありません
lsp-no-references = 参照は見つかりませんでした
lsp-no-definition = 定義は見つかりませんでした
lsp-no-implementations = 実装は見つかりませんでした
lsp-no-type = 型は見つかりませんでした
lsp-not-used-elsewhere = "{ $symbol }" は他では使われていません
outline-save-first = 構造を見るには、ファイルを保存してください
outline-reading = 構造を読み取っています…
symbols-type-a-name = 関数、型、メソッドの名前を入力…
searching = 検索中…
symbols-none-found = シンボルは見つかりませんでした (プロジェクトの読み込み中なら、もう一度試してください)
symbols-none-in-file = このファイルにシンボルはありません
lsp-save-first = 言語サーバーを使うには、ファイルを保存してください
lsp-finding-references = 参照を検索しています…
lsp-finding-implementations = 実装を検索しています…
calls-not-in-function = カーソルは関数の中にありません (または、サーバーが呼び出しを知りません)
lsp-formatted = ファイルをフォーマットしました
lsp-level-error = エラー:{" "}
lsp-level-warning = 警告:{" "}
lsp-server-stopped = { $server } が停止しました — { $reason }。「LSP: サーバーを再起動」を使ってください。
lsp-server-crashed = { $server } がクラッシュしました — { $reason }。入力を止めると、自分で戻ります。
lsp-server-crashed-too-often = { $server } が { $minutes } 分のあいだに { $count } 回クラッシュしました — { $reason }。「LSP: サーバーを再起動」を使ってください。
lsp-no-formatting = このファイルではフォーマットを利用できません
lsp-restarted = 言語サーバーを再起動しました
save-changes-title = 変更を保存しますか?
save-changes-message = "{ $name }" に未保存の変更があります。
save = 保存
discard = 破棄
picker-already-open = ファイル ダイアログは既に開いています
save-tab-closed = 保存する前にタブが閉じました
folder-opening = { $path } を新しいウィンドウで開いています
folder-open-error = フォルダーを開けませんでした: { $error }
saved = 保存しました: { $path }
save-error = 保存エラー: { $error }
file-deleted-kept = ファイルはディスク上で削除されました。変更のあるタブはそのままにしました
and-more = … ほか { $count } 件
quit-unsaved-title = 未保存の変更があります。終了しますか?
quit-save-all = すべて保存して終了
quit-without-saving = 保存せずに終了
save-needs-name = "{ $name }" には名前がありません。「ファイル: 名前を付けて保存…」を使ってください
save-file-error = { $path } の保存エラー: { $error }
close-tabs-title = タブ { $count } 個を閉じますか?
close-tabs-unsaved = 未保存の変更:
save-and-close = 保存して閉じる
discard-changes = 変更を破棄
tab-close = 閉じる
tab-close-others = 他を閉じる
tab-close-right = 右を閉じる
tab-close-saved = 保存済みを閉じる
tab-close-all = すべて閉じる
copy-path = パスをコピー
copy-relative-path = 相対パスをコピー
reveal-in-tree = ファイル ツリーで表示
open-in-file-manager = ファイル マネージャーで開く
copied = コピーしました: { $text }
file-manager-error = ファイル マネージャーを開けませんでした: { $error }
save-then-close = 保存先を選んでください。そのあと、タブをもう一度閉じます
config-reloaded-problems = 設定を読み直しましたが、問題があります: { $problems }
config-reloaded = 設定を読み直しました (拡張機能のオン/オフは、再起動後に反映されます)
config-no-home = 設定フォルダーが見つかりませんでした (HOME が設定されていません)
create-error = { $path } の作成エラー: { $error }
config-error = 設定: { $error }
write-error = { $path } の書き込みエラー: { $error }
unknown-command = 不明なコマンド: { $id }
keys-record-hint = Ctrl、Alt、または特殊キー (F1–F12、PageUp…) を使ってください。Esc でキャンセルします。
keys-already-bound = { $key } は既に「{ $command }」でした
keys-rebound = { $key } は「{ $old }」でした。今は「{ $command }」です
keys-bound = { $key } → 「{ $command }」
theme-unknown = 不明なテーマ "{ $name }" のため、"dark" を使います
default = 既定

## 設定
settings-title = 設定
settings-tab-appearance = 外観
settings-tab-extensions = 拡張機能
settings-tab-shortcuts = ショートカット
close-esc = 閉じる (Esc)
settings-no-home = (HOME が設定されていません)
settings-write-to = 保存先
settings-target-global = グローバル
settings-target-project = プロジェクト
settings-open-file = ファイルを開く
keys-filter = 名前、コマンド、キーで絞り込み…
keys-press = キーを押してください… (Esc でキャンセル)
keys-add = ショートカットを追加
keys-help = キーをクリックすると削除します。+ は新しいキーを記録します (使用中のキーでも、割り当て先が変わります)。
settings-lsp = 言語サーバー
settings-lsp-detail = LSP 全体のオンとオフです。
settings-servers = サーバー
settings-server-found = PATH に見つかりました
settings-server-not-found = PATH に見つかりません
settings-lsp-note = 変更は、これから起動するサーバーに適用されます。
settings-restart-servers = サーバーを再起動
theme-dark = ダーク (既定)
theme-light = ライト
slot-left = 左
slot-right = 右
slot-bottom = 下
slot-status = ステータス バー
settings-on-restart = IDE を再起動すると反映されます
default-capitalized = 既定
settings-extensions-note = 拡張機能のオンとオフは、再起動後に反映されます。パネルを別の側へ移すと、すぐ適用されます。
ext-filetree = ファイル ツリー
ext-statusbar = ステータス バー
ext-search = プロジェクト検索
ext-problems = エラーと警告
ext-terminal = 統合ターミナル
ext-git = 変更、ステージ、コミット、差分
ext-debug = デバッグのボタン、スタック、変数、出力
ext-references = 名前の使用箇所 (Shift+F12)
ext-outline = ファイルの関数とメソッド (可視性付き、Alt+7)
ext-preview = エディターの横の Markdown と SVG のプレビュー (Ctrl+Shift+V)
ext-calls = 関数の呼び出し元と呼び出し先 (Alt+Shift+H)
ext-interfaces = エージェントが .idetea/ui/ に書くインターフェース (実験的)

## パレット、ダイアログ、LSP、デバッグ、アプリの残り
palette-commands = コマンドを入力…
palette-save-as = ファイルのパス (プロジェクトのルートからの相対)
palette-files = ファイル名 (file:line でその行へ、> でコマンド)
palette-file-symbols = このファイルの関数、型、メソッド…
palette-project-symbols = プロジェクトの関数、型、メソッド…
palette-no-commands = コマンドは見つかりませんでした
palette-save-hint = Enter で保存 · Esc でキャンセル
palette-reading-files = プロジェクトのファイルを読み取っています…
palette-no-files = ファイルは見つかりませんでした
palette-no-symbols = シンボルはありません
palette-no-matching-symbols = 検索に一致するシンボルはありません
dialog-hint = Enter で確認 · Esc でキャンセル
lsp-command-not-found = { $server }: PATH にコマンド "{ $command }" がありません (インストールするか、設定の [lsp.servers.{ $server }] を調整してください)
lsp-start-error = { $server }: 起動できませんでした: { $error }
lsp-disabled = 設定で LSP はオフです ([lsp] enabled = false)
lsp-no-server-for = .{ $ext } ファイル用の言語サーバーが設定されていません
lsp-not-running = { $server } は動いていません: { $reason }
lsp-not-ready = { $server } はまだ準備できていません
lsp-hint-rustup-install = { $reason }。次のコマンドでインストールしてください: rustup component add rust-analyzer
lsp-hint-rustup = { $reason }。rustup の場合: rustup component add rust-analyzer
lsp-hint-ziex = { $reason }。Ziex CLI (https://ziex.dev/learn) をインストールするか、プロジェクト フォルダーを開いてください。build.zig.zon が ziex に依存していれば、IDE が zig でサーバーをビルドします
lsp-hint-moonbit = { $reason }。MoonBit のツールチェーン (https://www.moonbitlang.com/download) をインストールしてください。`moon lsp` が含まれます。IDE は ~/.moon/bin (または $MOON_HOME/bin) も見ます
lsp-hint-hare = { $reason }。hare-lsp は Hare に同梱されていません。https://git.sr.ht/~whynothugo/hare-lsp からビルドし、PATH に置いてください (例: ~/.local/bin)
config-syntax-error = 構文 { $file }: { $error }
visibility-file-only = このファイルのみ
picker-save-as = 名前を付けて保存
picker-open-files = ファイルを開く
picker-open-folder = フォルダーを開く
picker-debug-program = デバッグするプログラム
splash-ready = 準備完了
splash-loading = 読み込み中…
watch-too-many = フォルダーが大きすぎます。監視するのは、最初の { $count } フォルダーだけです
watch-partial = すべてのフォルダーを監視できませんでした ({ $error })
watch-problem = ディスク上の変更: { $message }。必要なときは、ファイル ツリーの「更新」を使ってください。
watch-failed = ディスク上の変更: 監視できませんでした ({ $error })
debug-no-gdb = PATH に gdb がありません (デバッグは gdb 14 以降を使います)
debug-program-not-found = プログラムが見つかりません: { $path }
debug-gdb-start-error = gdb を起動できませんでした: { $error }
debug-reason-breakpoint = ブレークポイント
debug-reason-step = ステップ
debug-reason-pause = 一時停止
debug-reason-exception = 例外
debug-reason-entry = エントリ
debug-reason-goto = ジャンプ
build-run-error = "{ $command }" を実行できませんでした: { $error }
build-no-message = メッセージなしで失敗しました

## ヘルプとコマンド ライン
help-key-find = ファイル内を検索
help-key-replace = 検索と置換
help-key-goto-line = 行へ移動
help-key-undo = 元に戻す / やり直す
help-key-next-match = 次の出現を選択 (マルチ カーソル)
help-key-quick-fix = LSP のクイック アクション (利用できる場合)
help-key-ctrl-click = Ctrl+クリック
help-key-definition = 定義へ移動 (LSP)
help-key-tab = タブを挿入
help-title = ヘルプ
help-tab-shortcuts = ショートカット
help-tab-guide = ガイド
help-tab-about = 情報
help-close = 閉じる (Esc)
help-filter = 名前、コマンド、キーで絞り込み…
help-group-general = 全般
help-editor-keys = エディター (固定)
help-in-palette = (パレット)
about-no-home = (HOME 未設定)
about-version = バージョン
about-license = ライセンス
about-license-value = { $license } (常に無料でオープン)
about-project = プロジェクト
about-global-config = グローバル設定
about-project-config = プロジェクト設定
about-built-with = Rust と、iced、iced-code-editor、alacritty_terminal (iced_term 経由)、syntect で作られています。
about-icons = アイコン: Codicons © Microsoft。ライセンスは CC BY 4.0 です。
cli-usage = 
    使い方: idetea [options] [path...]
    
      path           開くファイル (まだなくてもよい。保存時に作成)
                     またはプロジェクトのルートにするフォルダー
      --no-splash    起動画面を出さない (IDETEA_NO_SPLASH=1 でも可)
      --menu         メニューから起動。フォルダーがなければ、ファイルのプロジェクトか前回のプロジェクトを開く
      -f, --foreground  ターミナルに付けたままにする (付けないと、ターミナルから起動した IDE は
                     切り離され、ログは ~/.local/state/idetea/idetea.log へ)
      --install-desktop    アプリケーション メニューに IDE を追加 (アイコン付き)
      --uninstall-desktop  アプリケーション メニューから削除
      ui check <file.json>…  インターフェースのドキュメントを検査 (パネルと同じ読み方)
      new <folder> [--modelo id]  新しいプロジェクトを作成。フォルダー、git、テンプレートのスキルと手順
      skills [list|add|update|remove]  エージェントのスキルをプロジェクトにインストール (idetea skills --help)
      ui fix [--apply]  .idetea/ui/ に残ったものを表示 (--apply で修正)
      ask …          インターフェース パネルで質問し、回答を待つ (idetea ask だけだと使い方)
      run … -- <command>  IDE にターミナルを要求し、終了を待つ (idetea run だけだと使い方)
      agent [名前] [--worktree <branch>]  IDE のターミナル タブでエージェントを開く (--list: 使えるもの)
      status [--json]  このプロジェクトで開いている IDE の表示 (タブ、変更、カーソル)
      open <file[:line[:column]]>  開いている IDE がファイルを表示し、カーソルを置く
      history [--json]  インターフェース パネルでの回答 (--name、--since、--last N、--all)
      mascot <state> [--text "…"]  IDE のマスコットを更新 (idle、working、thinking、attention、celebrating、error、sleeping、clear、status)
      -h, --help     このヘルプを表示
      -V, --version  バージョンを表示
cli-unknown-option = 不明なオプションを無視しました: { $option }
cli-extra-root = ルート フォルダーは一度に 1 つです。{ $path } は無視します

## ヘルプ → ガイド (プレースホルダーは、そのコマンドのショートカットになります)
guide-start = はじめに
guide-start-1 = フォルダーで `idetea` を実行して IDE を開きます (そのフォルダーがツリーのルートになります)。`idetea ファイル...` では、ファイルがタブで開き、ルートは現在のフォルダーのままです。
guide-start-2 = コマンドはすべてパレットにあります: { $palette-open }。名前の一部を入力すると、横にショートカットが出ます。
guide-start-3 = このヘルプは { $help-open } で開きます。
guide-start-4 = 左端のアイコン バーは、1 クリックでパネルの表示と非表示を切り替えます (開いているものは強調されます)。パネルとエディターの境目をドラッグすると大きさが変わり、ダブルクリックで既定に戻ります。
guide-files = ファイルとタブ
guide-files-1 = 名前で開く: { $file-quick_open } (名前の一部を入力。`main.rs:42` でその行へ。`>` でコマンドへ切り替え)。
guide-files-2 = シンボル (関数、型、メソッド…) へ移動: ファイル内は { $nav-file_symbol }、プロジェクト全体は { $nav-project_symbol }。言語サーバーが必要です。
guide-files-3 = 新規: { $file-new } · 開く: { $file-open } · 保存: { $file-save } · 名前を付けて保存: { $file-save_as } · タブを閉じる: { $file-close }。
guide-files-4 = 開くと名前を付けて保存は、システムのファイル ダイアログを使います。パスを入力するには { $file-save_as_path }。別のフォルダーを開く ({ $file-open_folder }) と、新しい IDE のウィンドウが開きます。
guide-files-5 = Markdown と SVG のプレビュー: { $preview-toggle } (またはタブ バー端のボタン) で、アクティブなファイルに追従し、入力に合わせて更新されるサイド パネルが開きます。Web のリンクはブラウザーで、ファイルへのリンクはタブで開きます。
guide-files-6 = 画像 (png、jpg、gif、webp…) は専用のタブで開きます。ホイールで拡大縮小、ドラッグで移動です。
guide-files-7 = PDF もタブで開きます。スクロールでページを送り、− と + でズームを変え、「幅に合わせる」は幅に追従します。
guide-files-8 = タブの切り替え: { $tab-next } と { $tab-prev } (Ctrl+Tab は、エディターにタブ文字を挿入します)。
guide-files-9 = 未保存の変更があるタブを閉じるときや終了するときは、先に確認します。保存、破棄、キャンセルです。
guide-files-10 = 外でファイルが変わったとき (ターミナル、別のエディター): 未保存の変更がなければ、タブは自分で読み直します。変更があればそのままにして、通知を出します。
guide-files-11 = タブを右クリック: 他を閉じる、右を閉じる、保存済みを閉じる、すべて閉じる、パスをコピー、ツリーで表示、ファイル マネージャーで開く。中クリックでタブを閉じます。
guide-tree = ファイル ツリー
guide-tree-1 = 表示/非表示: { $filetree-toggle } · 現在のファイルを表示: { $filetree-reveal }。
guide-tree-2 = 項目を右クリック: 新しいファイル、新しいフォルダー、名前の変更、削除、パスのコピー、ファイル マネージャーで開く。削除はシステムのゴミ箱へ送ります。
guide-tree-3 = Git の色: 黄 = 変更、緑 = 新規、赤 = 競合、薄い = 無視。中に変更があるフォルダーには点が付きます。
guide-bottom = 下部パネル
guide-bottom-1 = プロジェクト検索 ({ $search-focus })、問題 ({ $problems-focus })、ターミナル ({ $terminal-focus })、Git ({ $git-focus }) は、下部パネルのタブです。{ $view-toggle_bottom } でパネルの表示と非表示を切り替えます。
guide-bottom-2 = 次の問題: { $problems-next }。
guide-terminal = ターミナル
guide-terminal-1 = ターミナルにフォーカスがあるとき、Ctrl と文字、ファンクション キー、矢印はシェルへ渡ります (Ctrl+C、Ctrl+W…)。Ctrl+Shift の IDE ショートカットは動いたままなので、パレットはいつでも開きます。
guide-terminal-2 = ターミナルでのコピーと貼り付け: Ctrl+Shift+C と Ctrl+Shift+V。`exit` と入力するとパネルが閉じます。
guide-debug = デバッグ
guide-debug-1 = ブレークポイント: 行番号をクリックするか、{ $debug-toggle_breakpoint }。開始/継続: { $debug-continue }。Rust のプロジェクトでは cargo build でビルドし、実行ファイルを自分で見つけます。それ以外は設定の [debug] (program、args、cwd、build) を使うか、実行ファイルを選びます。プログラムの変更: { $debug-choose_program }。
guide-debug-2 = 停止中: { $debug-next } で次の行、{ $debug-step_in } で関数へステップ イン、{ $debug-step_out } でステップ アウト、{ $debug-pause } で一時停止、{ $debug-stop } で停止。黄色い矢印がプログラムの位置です。gdb 14 以降を使います。
guide-debug-3 = デバッグ パネル ({ $debug-focus }): ボタン、呼び出しスタック (クリックでフレームへ移動し、その変数を表示)、変数 (クリックで構造体とベクタを展開)、プログラムとビルドの出力。
guide-git = Git
guide-git-1 = ブランチはステータス バーに出ます。Git パネル ({ $git-focus }) は変更を表示します。+ と − でステージとステージの解除、ファイルのクリックで差分、メッセージで Enter するとコミットします。
guide-git-2 = Git のファイルを右クリック: 開く、差分を表示、ステージ、変更を破棄 (確認あり)、パスをコピー。
guide-git-3 = フェッチ ({ $git-fetch })、プル ({ $git-pull })、プッシュ ({ $git-push }) は、パネル上部のボタンです。パスワードはここでは聞きません。ssh エージェント (ssh-add) か、資格情報ヘルパーを使ってください。新しいブランチは、リモートへ公開する前に確認します。
guide-git-4 = パネルは、保存したときと、リポジトリが変わったとき (ターミナルでのコミットや checkout) に更新されます。
guide-lsp = 言語 (LSP)
guide-lsp-1 = 言語サーバーは、ファイルを開くと自分で起動します。rust-analyzer、clangd、pyright、typescript-language-server、gopls、lua-language-server です (インストールが必要です)。
guide-lsp-2 = 定義へ移動: { $lsp-definition } または Ctrl+クリック · ドキュメント: { $lsp-hover }、またはコードの上でマウスを止める · フォーマット: { $lsp-format }。
guide-lsp-3 = 使用箇所: { $lsp-references }、または宣言そのものを Ctrl+クリック。一覧は参照パネルに出ます。クリックで使用箇所へ移動します。
guide-lsp-4 = 実装: { $lsp-implementation } (トレイトやインターフェース、またはそのメソッドの上で)。カーソル下の型は、パレットの「LSP: 型の定義へ移動」。結果が複数のとき (定義も含む) は一覧が参照パネルへ行き、1 件だけならそのまま移動します。
guide-lsp-5 = 関数の呼び出し元: カーソルを合わせて { $lsp-incoming_calls }。呼び出しパネルにツリーが出ます (段を開くと、その呼び出し元を呼んでいるものが見えます)。「呼び出し先」に切り替えると、その関数が行う呼び出しになります。クリックで呼び出しへ移動します。
guide-lsp-6 = ジャンプ (定義、参照、名前で開く、パネル内のクリック) のあとの戻ると進む: { $nav-back } と { $nav-forward }、またはマウスのサイド ボタン。
guide-lsp-7 = ファイルのアウトライン ({ $outline-toggle }): 関数、構造体、メソッド、フィールドのツリーです。開いた錠はパブリック、閉じた錠はプライベート、鍵はそのコードの一部だけで見えるもの (pub(crate)、protected、package) です。カーソル下のシンボルは強調されます。クリックでそこへ移動します。
guide-lsp-8 = サーバーが失敗すると、ステータス バーに理由が出ます。「LSP: サーバーを再起動」でもう一度試します。
guide-config = 設定
guide-config-1 = 設定画面 ({ $settings-open }) では、テーマ、フォント、拡張機能、ショートカット、LSP を変え、グローバル設定かプロジェクト設定に書きます。変更はすぐ反映されます。
guide-config-2 = グローバル設定と、プロジェクトごとの設定 (`.idetea/config.toml`) があります。プロジェクト側が優先します。ファイルを直接編集するには「設定: グローバル設定を開く」か「設定: プロジェクト設定を開く」。保存すると、すべて読み直します。
guide-config-3 = ショートカットは [keys] にあります。"ctrl+alt+t" = "terminal.focus"。空のテキストは、既定のショートカットをオフにします。コマンド id はショートカット タブに出ます。
guide-config-4 = 言語: 設定 → 外観 → 言語 (自動はシステムに従います)。IDE を再起動すると反映されます。

## セッション
settings-restore-session = 前回の状態で開き直す
settings-restore-session-detail = プロジェクトを開くと、タブ、カーソル、パネルが戻ります。idetea ファイル... は、そのファイルだけを開きます。

## 最近使った項目
cmd-file-open-recent = ファイル: 最近使った項目を開く…
palette-recent = 以前に開いたプロジェクトまたはファイル
palette-no-recent = まだ、以前に開いたものはありません
guide-files-12 = 最近使った項目: { $file-open_recent } は、プロジェクト (新しいウィンドウで開く) と、すべてのウィンドウで最後に開いたファイルを一覧します。

## 未保存の復元
untitled-tab = 無題
recover-title = IDE は、ファイル { $count } 個を保存せずに閉じました
recover-detail = 復元すると、変更がタブで開きます。まだ未保存です (元に戻すと、ディスク上の内容に戻ります)。「あとで」を選ぶと、次回にもう一度聞きます。
recover = 復元
recover-later = あとで
recovered = ファイル { $count } 個を復元しました (まだ保存していません)

## シンボルの名前変更と、すべて保存
cmd-lsp-rename = LSP: シンボルの名前を変更…
cmd-file-save-all = ファイル: すべて保存
save-all-untitled = 無題のタブは保存しませんでした (「ファイル: 名前を付けて保存…」を使ってください)
palette-rename = 新しい名前
palette-rename-hint = プロジェクト全体で "{ $name }" の名前を変更 · Enter で変更内容を表示 · Esc でキャンセル
rename-no-word = 名前を変更するには、名前の上にカーソルを置いてください
rename-asking = "{ $name }" の名前を変更: サーバーに確認しています…
rename-nothing = "{ $name }" について、名前を変更するものはありません
rename-failed = 名前を変更できませんでした: { $message }
rename-title = "{ $old }" を "{ $new }" に名前を変更しますか?
rename-summary = 
    { $changes ->
       *[other] 変更 { $changes } 件
    }、{ $files ->
       *[other] ファイル { $files } 個:
    }
rename-detail = ファイルはタブ上で変更されたまま、まだ保存されていません。それぞれ Ctrl+Z で元に戻せます。「ファイル: すべて保存」ですべて書き込みます。
rename = 名前の変更
renamed = ファイル { $count } 個で名前を変更しました (まだ保存していません)

guide-lsp-9 = 名前の変更: 名前の上にカーソルを置いて { $lsp-rename }。プレビューで、変更の件数とファイルが出ます。確認すると、タブ上で変更されたままになります (それぞれ Ctrl+Z で元に戻せます)。{ $file-save_all } ですべて保存します。

## プロジェクト内の置換
replaced = ファイル { $files } 個で { $count } 件置換しました
replaced-failed = ファイル { $files } 個で { $count } 件置換しました。{ $failed } 件は変更できませんでした (権限がないか、テキストではありません)

guide-bottom-3 = プロジェクト内の置換: { $search-replace } (または検索の横の ▸) で「置換後の文字列」が開きます。各結果は、置換後の見た目を示します。「置換」はその行またはファイルを変え、「すべて置換」は先に確認します。タブで開いているファイルは、変更されたまま未保存です (Ctrl+Z で元に戻します)。それ以外はディスクへ直接書き込まれます。

## エディターと保存
format-save-timeout = サーバーが時間内にフォーマットしなかったため、フォーマットせずに保存しました

settings-tab-editor = エディター
settings-wrap = 長い行を折り返す
settings-wrap-detail = ウィンドウより広い行は、横へスクロールせず、下へ続きます。
settings-whitespace = 空白文字を表示
settings-whitespace-detail = スペースは ·、タブは → で表示します。
settings-indent-guides = インデント ガイド
settings-indent-guides-detail = インデントの各段に、細い縦線を引きます。
settings-tab-size = インデントのサイズ
settings-tab-size-detail = Tab が挿入するスペースの数です。
settings-insert-spaces = スペースでインデント
settings-insert-spaces-detail = オフにすると、Tab はタブ文字を挿入します。既にタブでインデントされたファイルは、タブのままです。
settings-trim = 保存時に行末の空白を削除
settings-trim-detail = Markdown は除きます。行末のスペース 2 つは、そこで改行になるためです。
settings-final-newline = 保存時に末尾の改行
settings-final-newline-detail = 最終行が改行で終わるようにします。
settings-format-on-save = 保存時にフォーマット
settings-format-on-save-detail = Ctrl+S は、書き込む前に言語サーバーでフォーマットします (最大 2 秒待ちます)。すべて保存ではフォーマットしません。
settings-breadcrumbs = パンくずリスト
settings-breadcrumbs-detail = エディターの上に、ファイルのパスと、カーソル位置のシンボルを出します。フォルダーをクリックするとツリーで表示し、シンボルをクリックするとそこへ移動します。
settings-editorconfig-note = プロジェクトの .editorconfig は、これらのオプション (インデント、タブかスペース、保存時の後始末) を上書きします。

guide-config-5 = エディター: 設定のエディター タブ ({ $settings-open }) には、折り返し、空白文字の表示、インデント ガイド、インデントのサイズ、保存時の動作 (行末スペースの削除、末尾の改行、フォーマット) があります。プロジェクトの .editorconfig がこれらを上書きします。なければ、既にタブでインデントされたファイルはタブのままです。

guide-lsp-10 = パラメーター ヒント: 呼び出しで ( を入力すると、カーソルの上に関数のシグネチャが出て、今のパラメーターが強調されます。コンマごとに進み、括弧を閉じるか Esc で消えます。

guide-git-5 = ガターの印: 行番号の左で、緑 = 新しい行、青 = 変更、赤い刻み = 削除された行です。前回のコミットと比べます (入力に追従し、保存は不要です)。印をクリックすると以前の内容が出て、「変更を元に戻す」があります (Ctrl+Z で戻せます)。

palette-no-items = 検索に一致するものがありません

guide-git-6 = ブランチ: { $git-checkout } (または、ステータス バーや Git パネルのブランチ名をクリック) でブランチを一覧します。最近使ったものが先です。ここにないリモートのブランチはあとに続き、選ぶとローカルになります。新しい名前を入力すると、今の位置からブランチを作ります。未コミットの変更は一緒に移ります。もう一方のブランチと衝突すると、git は拒否して理由を示します。

guide-git-7 = 行の作者: ステータス バーのブランチの横に、カーソル行を最後に変えた人、時期、コミット メッセージが出ます (クリックでハッシュを見て、コピーできます)。タブに未保存の変更があるあいだは、保存するまで隠れます (ディスク上の行と一致しないことがあるためです)。

## タスク
cmd-tasks-run = タスク: 実行…
cmd-tasks-rerun = タスク: 直前のタスクをもう一度実行
cmd-tasks-test-here = Rust: カーソル位置のテストを実行
tasks-placeholder = タスクを実行…
tasks-none = タスクがありません。設定の [tasks] に追加するか、Cargo.toml、package.json、Makefile のあるプロジェクトを開いてください。
tasks-no-terminal = タスクにはターミナル拡張機能が必要ですが、オフになっています。
tasks-no-test = カーソルは Rust のテストの中にありません (#[test] fn)。
tasks-no-package = このファイルのパッケージの Cargo.toml が見つかりませんでした。
tasks-done = [{ $name } 終了: コード { $code } · Enter で閉じる]

guide-terminal-3 = タスク: { $tasks-run } は、プロジェクトのコマンド (設定の [tasks]、cargo build/run/test/clippy、package.json のスクリプト、Makefile のターゲット) を一覧し、選んだものをターミナルの「タスク」タブで実行します。シェルには触れません。最後に終了コードが出て、Enter で閉じます。Ctrl+C はそのコマンドだけを止めます。{ $tasks-rerun } は、直前のタスクをもう一度実行します。
guide-terminal-4 = カーソル位置のテスト (Rust): カーソルが #[test] fn の中にあるとき、{ $tasks-test_here } はそのテストだけを実行します (cargo test -p パッケージ名)。既定のキーはありません。よく使うなら、[keys] に追加してください。

## 分割エディター
cmd-editor-split = エディター: タブを反対側へ移動 (分割)
cmd-editor-focus-left = エディター: 左側へ移動
cmd-editor-focus-right = エディター: 右側へ移動
split-one-tab = エディターを分割するには、別のファイルを開いてください。左右で、別のタブを表示します。
tab-move-right = 右へ移動 (分割)
tab-move-left = 左へ移動
guide-files-13 = エディターの分割: { $editor-split } (またはタブ メニューの「右へ移動 (分割)」) で、タブを横に置き、そちらにも独自のタブが付きます。ファイルを開くと、フォーカスのある側に入ります。側をクリックする (または { $editor-focus_left } / { $editor-focus_right }) と、そこで作業できます。区切りをドラッグすると幅が変わります (ダブルクリックで中央に戻します)。ある側の最後のタブを戻すと、再び 1 つになります。

## アプリケーション メニュー (--install-desktop)
desktop-no-home = ホーム フォルダーが見つかりませんでした (HOME)。
desktop-error = ショートカットをインストールできませんでした: { $error }
desktop-installed = 
    アプリケーション メニューの IDE TEA: { $desktop }
    実行: { $exe }
desktop-removed = アプリケーション メニューから IDE TEA を削除しました。

## 大きなファイル
file-light-mode = "{ $name }" には非常に長い行があるため、軽量モードで開きました。色分けなし、言語サーバーなし、行の折り返しありです
lsp-light-mode = 軽量モードのファイルです (非常に長い行)。言語サーバーはありません

## 再コンパイルされた実行ファイル
binary-changed = IDE が再ビルドされました。新しいバージョンを使うには、パレットの「IDE を再起動」を実行してください。
binary-changed-title = [新しいバージョン: 再起動]
binary-restart-error = 再起動できませんでした: { $error }

## idetea status / idetea open (RFC 0006)
status-usage =
    使い方: idetea status [--json]
    
    このプロジェクトで開いている IDE の表示です。カーソルのあるアクティブなタブと、開いているファイル
    (変更があるものには "unsaved")。--json は同じ内容を JSON で出し、"open" が付きます。
    終了コード: 0 開いている、2 引数が不正、3 このプロジェクトで IDE が開いていない。
status-bad-arg = 不明な引数: { $arg }
status-not-open = このプロジェクトで IDE は開いていません ({ $root })
status-active = アクティブ: { $path }
status-no-active = アクティブなファイルはありません
status-modified = {" "}(未保存)
open-usage =
    使い方: idetea open <file[:line[:column]]>
    
    このプロジェクトで開いている IDE がファイルを表示し、カーソルを行と列に置きます。
    終了コード: 0 表示した、2 パスが不正かプロジェクトの外、3 IDE が開いていない (または要求を受け取らなかった)。
open-outside = { $path } はプロジェクトの中にありません ({ $root })
open-write-error = { $path } に書き込めませんでした
open-not-taken = IDE は 3 秒以内に要求を受け取りませんでした

## 反応するマスコット (RFC 0012)
mascot-state-idle = 落ち着き
mascot-state-working = 作業中
mascot-state-thinking = 考え中
mascot-state-attention = 注目
mascot-state-celebrating = 完了
mascot-state-error = エラー
mascot-state-sleeping = 睡眠
mascot-state-curious = 好奇心
mascot-state-annoyed = ちょっと不満
mascot-idle-1 = コードだけでいい。氷はあとで。
mascot-idle-2 = 湯気は立っています。ここにいます。
mascot-idle-3 = 静かなカップ、目は開けたまま。
mascot-idle-4 = 茶葉は落ち着いていい。
mascot-idle-5 = 急がなくていい。お茶は温かい。
mascot-idle-6 = 飲みたくなったら、どうぞ。
mascot-idle-7 = カップは味方です。
mascot-idle-8 = ソーサーの上は穏やかです。
mascot-hover-curious-1 = ん？何か用ですか。
mascot-hover-curious-2 = こんにちは。調子はどうですか。
mascot-hover-curious-3 = 感じました。どうしました。
mascot-hover-curious-4 = お茶の上を通り過ぎていますか。
mascot-hover-curious-5 = 挨拶ですか、手を貸しましょうか。
mascot-hover-curious-6 = カップが気づきました。
mascot-hover-curious-7 = 気になります。どうぞ。
mascot-hover-curious-8 = ようこそ。次は何をしますか。
mascot-hover-annoyed-1 = ゆっくり。お茶が渦を巻いています。
mascot-hover-annoyed-2 = マウスが多いです。カップがくらくらします。
mascot-hover-annoyed-3 = 手は静かに。お茶は熱いです。
mascot-hover-annoyed-4 = 一度なら挨拶です。今のは嵐でした。
mascot-hover-annoyed-5 = ソーサーが鳴っています。
mascot-hover-annoyed-6 = 見えています。渦は少なめで。
mascot-poke-curious-1 = ん？何でしょう。
mascot-poke-curious-2 = カップがくすぐったいです。
mascot-poke-curious-3 = 呼びましたか。
mascot-poke-curious-4 = 一回。聞いています。
mascot-poke-curious-5 = 注意、満杯です。
mascot-poke-curious-6 = ソーサーから、こんにちは。
mascot-poke-double-1 = 二回。湯気が倍です。
mascot-poke-double-2 = ダブルクリック、了解。中身はお茶のままです。
mascot-poke-double-3 = 二度ですか。わかりました。
mascot-poke-double-4 = カップを二回、こんこん。
mascot-poke-double-5 = 二回なら、もう会話です。
mascot-poke-double-6 = 二度たたいても、こぼれていません。
mascot-poke-annoyed-1 = こぼしますよ、気をつけて。
mascot-poke-annoyed-2 = お茶はもう熱いです。
mascot-poke-annoyed-3 = 一杯には、もう十分な回数です。
mascot-poke-annoyed-4 = あなたは好きです。ソーサーは少し迷い中。
mascot-poke-annoyed-5 = そっと。このカップは小さいです。
mascot-poke-annoyed-6 = 伝わりました。少し冷まします。
mascot-task-1 = 取りかかります: { $name }
mascot-task-2 = { $name } を淹れています…
mascot-task-3 = タスク { $name } を実行中です。
mascot-task-4 = { $name }。やかんを見ておきます。
mascot-task-5 = { $name } を始めます。
mascot-task-6 = { $name } — 蒸らしておきましょう。
mascot-term-pending-1 = ターミナルが承認を待っています: { $title }
mascot-term-pending-2 = コマンドが待っています: { $title }
mascot-term-pending-3 = 判断はあなたに: { $title }
mascot-term-pending-4 = 許可をお願いします: { $title }
mascot-term-pending-5 = やかんを持ったままです: { $title }
mascot-term-pending-6 = { $title } — 実行してよいですか。
mascot-term-running-1 = 実行中: { $title }
mascot-term-running-2 = { $title } は火にかかっています。
mascot-term-running-3 = 進行中: { $title }
mascot-term-running-4 = { $title } — まだ蒸らしています。
mascot-term-running-5 = { $title } に取り組んでいます。
mascot-term-running-6 = { $title } は進んでいます。
mascot-term-ok-1 = 完了: { $title }
mascot-term-ok-2 = { $title } はきれいに終わりました。
mascot-term-ok-3 = 問題なし — { $title }。
mascot-term-ok-4 = { $title } はうまく出ました。
mascot-term-ok-5 = 正常終了: { $title }。
mascot-term-ok-6 = { $title }。カップは喜んでいます。
mascot-term-err-1 = { $title } が失敗しました ({ $code })。
mascot-term-err-2 = 終了 { $code }: { $title }
mascot-term-err-3 = { $title } がこぼれました (コード { $code })。
mascot-term-err-4 = これは崩れました: { $title } ({ $code })。
mascot-term-err-5 = { $title } は終わりませんでした ({ $code })。
mascot-term-err-6 = { $title } のコード { $code }。
mascot-answered-1 = 了解: { $title }
mascot-answered-2 = 回答済み: { $title }
mascot-answered-3 = { $title } — ありがとう。
mascot-answered-4 = メモしました: { $title }
mascot-answered-5 = { $title } は片付きました。
mascot-answered-6 = { $title } に答えましたね。
mascot-wait-1 = エージェントが待つのをやめました: { $title }
mascot-wait-2 = { $title } は静かになりました。
mascot-wait-3 = もう { $title } を待っている人はいません。
mascot-wait-4 = { $title } は冷めました。
mascot-wait-5 = 開いたままですが、待ちは終わりです: { $title }
mascot-errors-1 = プロジェクトにエラーが { $count } 件。
mascot-errors-2 = 茶葉がエラーを { $count } 件見つけました。
mascot-errors-3 = { $count } 件、見たくなったら。
mascot-errors-4 = 問題: { $count }。急がなくていいです。
mascot-errors-5 = エラー { $count } 件。場所は示せます。
mascot-errors-6 = ソーサー報告: エラー { $count } 件。
mascot-save-1 = { $name } を保存しました。カップは賛成です。
mascot-save-2 = { $name } はディスクにあります。
mascot-save-3 = 冷める前に { $name } を受け取りました。
mascot-save-4 = { $name } — しまいました。
mascot-save-5 = 書き込み完了: { $name }。
mascot-save-6 = { $name } は無事です。お茶に戻ります。
mascot-wake-1 = おかえり。お茶はまだ温かいです。
mascot-wake-2 = うとうとしていました。どこまででしたか。
mascot-wake-3 = おかえりなさい。また見ています。
mascot-wake-4 = 目が覚めました。カップも。
mascot-wake-5 = いい一休みでした。
mascot-wake-6 = 再開ですか。見張っています。
mascot-sleep-1 = 静かに蒸らしています…
mascot-sleep-2 = ここにいます。
mascot-sleep-3 = 弱火です。必要なら呼んでください。
mascot-sleep-4 = 眠いカップ。コードは休んでいい。
mascot-sleep-5 = ソーサーで一眠り。
mascot-sleep-6 = 明かりを落とします。あなたが戻ったら起きます。
mascot-greet-morning-1 = おはよう。最初の一杯はこちらから。
mascot-greet-morning-2 = おはようございます。コードのあとで氷を。
mascot-greet-morning-3 = 早い光、新しい茶葉。
mascot-greet-morning-4 = 朝です。差分の前に、短い一口。
mascot-greet-morning-5 = おはよう。やかんはもう火にかかっています。
mascot-greet-afternoon-1 = こんにちは。ポットはまだ大丈夫です。
mascot-greet-afternoon-2 = こんにちは。ペースはそのままで。
mascot-greet-afternoon-3 = 昼です。一気に飲まなくていい。
mascot-greet-afternoon-4 = 午後の様子見、カップより。
mascot-greet-afternoon-5 = 日は高い。お茶も。
mascot-greet-evening-1 = こんばんは。もう一ファイル、それともソーサー。
mascot-greet-evening-2 = こんばんは。カップはまだ温かいです。
mascot-greet-evening-3 = 夕方の光。切り上げるのはいつでも。
mascot-greet-evening-4 = 一日が火を弱めています。
mascot-greet-evening-5 = 夜です。ビルドが残らなくても、私は残ります。
mascot-greet-late-1 = 遅い時間です。お茶は大丈夫。休めるなら休んで。
mascot-greet-late-2 = 静かな時間。眠るのも機能です。
mascot-greet-late-3 = まだ起きていますか。考えを閉じて、休んでください。
mascot-greet-late-4 = カップは残れる。あなたは休んでいい。
mascot-greet-late-5 = 深夜です。見張ります。短めに。
mascot-tip-1 = コマンドはすべてパレットにあります: { $palette-open }。
mascot-tip-2 = ファイル名で開くには { $file-quick-open }。
mascot-tip-3 = { $file-save } でファイルをディスクへ。
mascot-tip-4 = ショートカットと短い案内: { $help-open }。
mascot-tip-5 = プロジェクトのタスクは { $tasks-run }。
mascot-tip-6 = ターミナルは { $terminal-focus }。
mascot-tip-7 = 問題: { $problems-focus }。次は { $problems-next }。
mascot-tip-8 = 定義へ跳ぶには { $lsp-definition }。
mascot-build-ok-1 = ビルドが終わりました。お茶はまだ熱いです。
mascot-build-ok-2 = コンパイルできました。いい茶葉です。
mascot-build-ok-3 = ビルドはきれいです。
mascot-build-ok-4 = 緑のビルド。一杯注ぎます。
mascot-build-ok-5 = コンパイル完了。エディタに戻ります。
mascot-build-err-1 = ビルドは通りませんでした。
mascot-build-err-2 = ビルド失敗。カップも感じました。
mascot-build-err-3 = コンパイルエラー。見たくなったら。
mascot-build-err-4 = そのビルドはこぼれました。
mascot-build-err-5 = ビルド失敗。問題パネルが詳しいかもしれません。
mascot-working-1 = 火にかかっています。
mascot-working-2 = 作業中。湯気が忙しいです。
mascot-working-3 = 蒸らしておきましょう。
mascot-working-4 = まだ取り組んでいます。
mascot-working-5 = やかんは使用中です。
mascot-thinking-1 = 茶葉をかき混ぜています…
mascot-thinking-2 = 少しだけ。考えています。
mascot-thinking-3 = 蒸らしています。
mascot-thinking-4 = 静かな考え、温かいカップ。
mascot-thinking-5 = 落ち着いて見ています。
mascot-celebrate-1 = 完了。うまく出ました。
mascot-celebrate-2 = 準備できています。
mascot-celebrate-3 = きれいな終わり。乾杯。
mascot-celebrate-4 = カップは賛成です。
mascot-celebrate-5 = いいですね。湯気は保ちます。
mascot-error-1 = 何かがこぼれました。
mascot-error-2 = それは通りませんでした。
mascot-error-3 = 引っかかりました。まだここにいます。
mascot-error-4 = 今回はだめです。
mascot-error-5 = 見る価値はあります。気が向いたら。
mascot-open-interfaces = Interfaces パネルを開く →
mascot-open-terminal = ターミナルパネルを開く →
mascot-open-problems = 問題パネルを開く →
mascot-idle-hint = エージェントとタスクを見守っています
mascot-bad-state = マスコットの状態が不明です: { $state }
mascot-usage =
    使い方: idetea mascot <状態> [--text "…"] [--source <名前>] [--ttl <秒>]
            idetea mascot clear
            idetea mascot status [--json]

    状態: idle, working, thinking, attention, celebrating, error, sleeping, curious, annoyed。
    ファイル .idetea/mascot.json を書き、IDE のマスコットがすぐ反応します (--ttl で期限、既定は 30 秒)。
## ようこそ画面
welcome-start = 開始
welcome-new-file = 新しいファイル
welcome-quick-open = プロジェクトのファイルを開く…
welcome-open-file = ファイルを開く…
welcome-open-folder = 新しいウィンドウでフォルダーを開く…
welcome-recent = 最近使った項目を開く…
welcome-recent-files = 最近使ったファイル
welcome-recent-projects = 最近使ったプロジェクト
welcome-hint-palette = すべてのコマンド
welcome-hint-search = プロジェクトを検索
welcome-hint-terminal = ターミナル

## Lançador de agentes (RFC 0016)
cmd-agents-launch = エージェント: 起動…
agent-usage =
    使い方: idetea agent [名前] [--worktree <branch>]
    
      名前           エージェント: claude、codex... または [agents.launchers] の名前 (なし: [agents] default、
                     なければ最初に見つかったもの)
      --worktree <branch>  そのブランチの新しい worktree で実行する
      --list         使えるエージェントを表示する (キー、名前、コマンド)
    
    ターミナル要求として送られます。[agents] allow にない限り、IDE はコマンドを見せて確認します。
    要求の名前を表示します (idetea run --wait <名前> で終了を待てます)。
agent-not-found = その名前のエージェントはありません: { $name } (idetea agent --list で使えるものを表示)
agent-bad-arg = 無効な引数: { $arg }
cmd-agents-send-context = エージェント: 場所と選択部分を送る
agents-send-nothing = 送るものがありません。ファイルを開くか、テキストを選択してください。
agents-send-too-big = 選択部分が大きすぎます。テキストなしで場所だけを送りました。
launcher-placeholder = エージェントを起動…
launcher-worktree = { $name } · 新しい worktree で…
launcher-branch-placeholder = { $name } の worktree のブランチ (例: fix/login)
launcher-branch-create = このブランチで worktree を作成
launcher-none = PATH にエージェントがありません ({ $known })。インストールするか、ユーザー設定の [agents.launchers] に追加してください。
launcher-teach-title = 先にエージェントに教えますか？
launcher-teach-detail = このプロジェクトの AGENTS.md/CLAUDE.md には、この IDE の手順がまだありません (または古い版です)。手順がないと、{ $name } はインターフェースパネルを使えず、ターミナルも頼めません。
launcher-teach-write = 書いて起動
launcher-teach-open = 起動だけ
launcher-teach-skip = このプロジェクトでは聞かない
launcher-teach-done = 手順を { $files } に書きました。
launcher-teach-failed = 手順を書けませんでした: { $error }
welcome-launch-agent = エージェントを起動
