## ターミナル
terminal-start-error = ターミナル: シェルを起動できませんでした: { $error }
cmd-terminal-focus = ターミナル: 開いてフォーカス
cmd-terminal-new = ターミナル: 新しいターミナル
cmd-terminal-close = ターミナル: ターミナルを閉じる
cmd-terminal-restart = ターミナル: シェルを再起動
terminal-ended-n = ターミナル { $number } を閉じました
terminal-not-running = シェルは動いていません。
terminal-start = ターミナルを開始

## タスク
terminal-task-error = タスク: 開始できませんでした: { $error }
terminal-tab-n = ターミナル { $number }
terminal-close = ターミナルを閉じる
terminal-tab-task = タスク: { $name }
terminal-close-task = タスクを閉じる

## エージェントが要求したターミナル
terminal-tab-agent = エージェント: { $title }
terminal-tab-request = ターミナル要求 { $count } 件
agent-request-notify = エージェントがターミナルを要求しています: { $title }。ターミナル パネルを見てください。
agent-request-auto = ターミナル要求 "{ $title }" は、確認なしで実行中です。設定の [agents] allow ("{ $entry }") に入っています。
agent-request-someone = エージェント
agent-request-title = { $who } がターミナルを要求しています: { $title }
agent-request-command = コマンド (このとおりに実行します):
agent-request-cwd = フォルダー: { $cwd }
agent-request-worktree = 専用の作業ツリー (worktree) で実行します。ブランチ { $branch }、場所は { $path } (ブランチがなければ HEAD から作成)
agent-worktree-kept = 作業ツリー { $path } を残しました: { $error }
agent-request-warning = 実行ユーザーの権限で、手で入力したときと同じように動きます。内容が分かるものだけ許可してください。
agent-request-allow = 許可して実行
agent-request-refuse = 拒否
agent-request-more = このあとに、要求が { $count } 件あります。
agent-done = コード { $code } で終了しました。Enter でタブを閉じます。
run-bad-arg = idetea run: "{ $arg }" は無効です。
run-usage = 使い方: idetea run [--title T] [--name name] [--cwd folder] [--worktree branch] [--keep] [--no-wait] [--timeout seconds] -- <command>; idetea run --wait <name>; idetea run --stop <name>
run-busy = idetea run: 要求 "{ $name }" (またはその状態) が既にあります。別の --name を選ぶか、idetea run --wait { $name } で待ってください
run-asked = idetea run: 要求 "{ $name }" はターミナル パネルにあります。許可と、コマンドの終了を待っています
run-exited = idetea run: "{ $name }" はコード { $code } で終了しました
run-refused = idetea run: "{ $name }" は拒否されました
run-invalid = idetea run: 要求 "{ $name }" は無効です: { $error }
run-ended = idetea run: "{ $name }" はコードなしで終了しました ({ $state })
run-branch = idetea run: 作業はブランチ { $branch } にあります
run-timeout = idetea run: 時間切れです。"{ $name }" はまだターミナルにあります (idetea run --wait { $name } でもう一度待ちます。--stop で終了します)

## タブの状態 (RFC 0024)
terminal-state-running = 実行中
terminal-state-unseen = 前回見たあとに新しい出力
terminal-state-attention = 対応を求めています
terminal-state-exited = コード { $code } で終了
terminal-state-bell = ベルを鳴らしました
terminal-state-turn-done = ターンを終えて待っています
cmd-terminal-next = ターミナル: 次のタブ
cmd-terminal-prev = ターミナル: 前のタブ
cmd-terminal-goto = ターミナル: 移動…
cmd-terminal-next-attention = ターミナル: 待っている次のタブ
cmd-terminal-rename = ターミナル: タブ名を変更
terminal-no-attention = 対応が必要なターミナルタブはありません。
terminal-goto-placeholder = ターミナルタブへ移動…
terminal-rename-placeholder = タブ名
terminal-rename-reset = 既定の名前に戻す
terminal-rename-to = 名前を変更:
terminal-close-running-title = タブを閉じますか？
terminal-close-running = { $name } はまだ実行中です。タブを閉じるとプロセスが終了します。
terminal-close-running-yes = 閉じて終了
terminal-close-running-no = キャンセル
cmd-terminal-find = ターミナル: 検索
terminal-find-placeholder = ターミナル内を検索（Enter: 古い方へ）
terminal-find-older = 古い方（上）
terminal-find-newer = 新しい方（下）
terminal-find-none = 見つかりません
terminal-find-close = 検索を閉じる
