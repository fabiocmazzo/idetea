ui-panel = インターフェース
cmd-ui-show = インターフェース: パネルを表示
ui-empty = 開いているインターフェースはありません。エージェントは .idetea/ui/ にファイルを書くと作れます (docs/interfaces/formato.md を参照)。オンとオフは、設定の [extensions] interfaces = {"{"} enabled = true {"}"} です。
ui-tracked = { $path } は git で追跡されているため、安全のために無視しました。インターフェースは、このマシンで動いているエージェントからだけ受け付け、クローンしたリポジトリからは受け付けません。
ui-invalid = 無効なドキュメント: { $error }
ui-origin = { $path }
ui-origin-source = { $path } · { $source }
ui-waiting = 応答を { $count } 件送りました。エージェントが読むのを待っています。
ui-received = エージェントが応答を受け取りました。
ui-agent-waiting = エージェントが回答を待っています。
ui-agent-stopped = { $minutes ->
    [0] エージェントはたった今、待つのをやめました。回答は、次に見るまで残します。
   *[other] エージェントは { $minutes } 分前に待つのをやめました。回答は、次に見るまで残します。
}
ui-new = エージェントからの新しいインターフェース: { $title }
ui-updated = インターフェースを更新しました: { $title }
ui-open-later = エージェントが { $path } を開くよう求めました。インターフェース パネルにあります (入力の邪魔にならないよう、開いてはいません)。
ui-write-failed = 応答を書き込めませんでした: { $error }
ui-prepare-failed = .idetea/ui フォルダーを用意できませんでした: { $error }
ui-not-a-file = 通常のファイルではありません
ui-surface-mismatch = surface フィールドは "{ $found }" ですが、ファイル名は { $expected }.json です
cmd-ui-teach = インターフェース: このプロジェクトのエージェントに教える
ui-teach-hint = コーディング エージェントがこのパネルの使い方を知るには、手順をプロジェクトの AGENTS.md に書いてください。
ui-teach-button = このプロジェクトのエージェントに教える
ui-teach-title = このプロジェクトのエージェントに教えますか?
ui-teach-detail = IDE は、インターフェース パネルの手順を次に書きます: { $files }。テキストはマーカーの間に残ります。もう一度実行するとその部分だけを更新し、残りはそのままです。これらはプロジェクトのファイルです。コミットする前に確認してください。
ui-teach-file-new = { $name } (新規)
ui-teach-write = 書き込む
ui-cancel = キャンセル
ui-teach-up-to-date = エージェント向けの手順は、既に最新です。
ui-teach-done = 手順を { $files } に書きました。
ui-teach-failed = 手順を書き込めませんでした: { $error }
cmd-ui-new-project = プロジェクト: 新規…
new-title = 新しいプロジェクト
new-intro = IDE はフォルダー、git、テンプレートのスキル、パネルの手順を作り、新しいウィンドウでプロジェクトを開きます。実際のプロジェクトは、**AGENTS.md** の最初の依頼に従ってエージェントが作ります。下でエージェントを選ぶと、新しいウィンドウが IDE のターミナルで開く確認を出します (コマンドは確認してから実行します)。
new-field-parent = 作成先
new-field-name = フォルダー名
new-field-name-hint = 例: my-app
new-field-model = テンプレート
new-field-description = プロジェクトの内容 (任意)
new-field-description-hint = 1 文。エージェントへの最初の依頼に入ります
new-field-git = git リポジトリを作成
new-field-agent = IDE のターミナルで開くエージェント
new-agent-none = なし (手動で開く)
new-agent-title = エージェント: { $agent }
new-bad-agent = 無効なエージェント名: "{ $agent }" (文字、数字、-、_ のみ)
new-create = 作成して開く
new-model = { $model ->
    [vazio] 空 (エージェントが内容を確認します)
    [rust] Rust
    [site] Web サイト
    [python] Python
    [fastapi] FastAPI (Python の Web API)
   *[other] { $model }
}
new-creating = { $dir } を作成しています (スキルはネットワーク経由です)…
new-created = プロジェクトを { $dir } に作成しました。新しいウィンドウで開きます。
new-open-failed = プロジェクトは { $dir } に作成しましたが、ウィンドウを開けませんでした: { $error }
new-failed = プロジェクトを作成できませんでした: { $error }
new-bad-model = 不明なテンプレート: { $model }
new-bad-dir = 無効なフォルダー: { $dir } (フルパスを使ってください)
new-bad-name = 無効なフォルダー名: "{ $name }"
new-not-empty = { $dir } は既にあり、ファイルが入っています。別の名前にしてください
new-done-dir = フォルダー: { $dir }
new-done-git = git: リポジトリを作成しました
new-done-agents = AGENTS.md (最初の依頼付き) と CLAUDE.md (パネルの手順付き)
new-done-agent = { $agent } のターミナル要求: 新しいウィンドウで確認してください
new-open-hint = 完了です。次で開きます: idetea { $dir }
new-usage = idetea new: "{ $arg }" は無効です。使い方: idetea new <folder> [--modelo id] [--descricao text] [--sem-git] [--agente name]。テンプレート: { $models }
cmd-ui-fix = インターフェース: このプロジェクトを直す
ui-fix-title = このプロジェクトを直しますか?
ui-fix-will = IDE が行うこと:
ui-fix-wont = 参考 (このまま残します):
ui-fix-button = 修正
ui-fix-nothing = このプロジェクトのインターフェースに、直すものはありません。
ui-fix-done = 修正しました: { $count } 件。
ui-fix-failed = 修正できませんでした: { $error }
ui-fix-hint = "-" の付いた項目を直すには: idetea ui fix --apply
ui-fix-usage = idetea ui fix: 不明な引数 "{ $arg }" (使い方: idetea ui fix [--apply])
ui-fix-abandoned = { $path } を削除: { $since } のあいだ、回答を待っている人はいません
ui-fix-unread = { $path }: { $count ->
   *[other] エージェントが読んでいない回答 { $count } 件
} (残します。エージェントに読むよう頼むか、手で削除してください)
ui-fix-leftover = { $path } を削除: 書き込みか待ちが終わらずに残りました
ui-fix-check = .idetea/ui/check を書き換え: もうない { $target } を呼んでいます
ui-fix-instructions-older = { $file } の手順を書き換え: バージョン { $found } で、この IDE より古いです
ui-fix-instructions-unversioned = { $file } の手順を書き換え: バージョン行のない、古い IDE のものです
ui-fix-instructions-newer = { $file } には、より新しい IDE の手順があります ({ $found })。その IDE でプロジェクトを開いてください (こちらでは戻せません)
ui-outdated = この IDE は古くなっています (開いたあとで再ビルドされました)。応答はそのことをエージェントに伝えます。パレット: 「IDE を再起動」。
ui-audio-failed = 音声を再生できませんでした: { $error }
ui-check-ok = { $file }: 正常 ({ $count ->
       *[other] コンポーネント { $count } 個
    })
ui-check-error = { $file }: エラー: { $error }
ui-check-warning = 警告: { $warning }
ui-check-media = メディア: { $path }: { $problem }
ui-check-name = 無効なファイル名 ("{ $name }"): 小文字、数字、_、- で、.json で終わる必要があります
ui-check-usage = 使い方: idetea ui check <file.json>…
ui-check-instructions-older = 手順: { $file } のバージョンは { $found }、この IDE は { $ours } です。「インターフェース: このプロジェクトのエージェントに教える」を実行し、公開前にファイルを読み直すよう頼んでください
ui-check-instructions-newer = 手順: { $file } のバージョンは { $found } で、この IDE ({ $ours }) より新しいです。開いているパネルは、ファイルが教えている内容を理解できないことがあります
ui-check-instructions-unversioned = 手順: { $file } はバージョン行のない古いブロックです。この IDE は { $ours } です。「インターフェース: このプロジェクトのエージェントに教える」を実行し、公開前にファイルを読み直すよう頼んでください
ui-teach-newer-file = { $name } (バージョン { $version })
ui-teach-newer = 警告: { $files } は、この IDE (バージョン { $ours }。そちらは未知) より新しい IDE が書きました。書き込むと、手順はこの IDE のバージョンに戻ります。通常は、より新しい IDE でプロジェクトを開きます。
ui-teach-write-anyway = それでも書き込む
ask-title = 質問
ask-choose = 選択
ask-ok = 確認
ask-yes = はい
ask-no = いいえ
ask-not-object = ドキュメントは JSON オブジェクトである必要があります
ask-bad-arg = idetea ask: "{ $arg }" は無効です。
ask-usage = 使い方: idetea ask <document.json | -> | --text T [--options "a|b|c" | --confirm | --input Label] [--title T] [--name name] [--value /pointer] [--json] [--keep] [--timeout seconds]; idetea ask --resume <name>; idetea ask --close <name>
ask-invalid = idetea ask: パネルがこの質問を拒否します: { $error }
ask-no-such = idetea ask: パネルに質問 "{ $name }" はありません
ask-timeout = idetea ask: 時間内に回答がありません。質問 "{ $name }" はパネルに残ります (idetea ask --resume { $name } でもう一度待ちます)
ask-outdated = idetea ask: 回答は、開いたあとで再ビルドされた IDE からのものです (ide.outdated)。変更を試しているなら、IDE の再起動を頼んでください
ask-comment = コメント: { $comment }

# インターフェースの履歴 (RFC 0007)
ui-history-panel = インターフェースの履歴
cmd-ui-history = インターフェース: 履歴
cmd-ui-clear-history = インターフェース: 履歴を消去
ui-history-failed = 履歴に書き込めませんでした: { $error }
ui-history-cleared = 履歴から { $count } 件削除しました。
ui-history-link = 履歴 ({ $count })
ui-history-empty = 履歴は空です。
ui-history-clear-title = 履歴を消去しますか?
ui-history-clear-detail = { $count ->
   *[other] このプロジェクトの履歴 { $count } 件 (.idetea/history/) を削除します。
} パネルで開いているインターフェースはそのままです。元に戻せません。
ui-history-clear-button = 履歴を消去
ui-history-search = タイトル、回答、コメントを検索
ui-history-open = 開いているもの ({ $count })
ui-history-waiting-you = 回答待ち
ui-history-answered-open = 回答済み、まだ開いている
ui-history-today = 今日
ui-history-yesterday = 昨日
ui-history-none = まだ回答した質問はありません。インターフェース パネルでの回答は、ここに残ります (.idetea/history/、git の外)。
ui-history-no-match = 見つかりませんでした。
ui-history-back = ← 戻る
ui-history-shown = 表示: { $time }
ui-history-answered = 回答: { $time }
ui-history-closed = 回答なしで閉じた時刻: { $time }
ui-history-read = エージェントが読んだ時刻: { $time }
ui-history-button = ボタン: { $action }
ui-history-not-read = エージェントはこの回答を読んでいません (ファイルがまだ .idetea/ui/responses/ にあるか、IDE が閉じているあいだに削除されました)。
ui-history-not-read-short = 未読
ui-history-unanswered = 回答なし
ui-history-media-changed = { $path } は回答のあとで変わったか、なくなっています。ここに見えているのは、今の版です。
ui-history-no-audio = 履歴では音声を再生しません
history-cli-comment = コメント
history-cli-not-read = エージェントはこの回答を読んでいません
history-cli-unanswered = 回答なし (エージェントが質問を削除しました)
history-cli-bad = idetea history: "{ $arg }" は無効です。
history-cli-usage = 使い方: idetea history [--name name] [--since YYYY-MM-DD] [--last N | --all] [--json]
history-cli-empty = idetea history: このプロジェクトの履歴には何もありません。
