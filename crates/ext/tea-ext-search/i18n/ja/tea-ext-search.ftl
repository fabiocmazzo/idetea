## 検索パネル
search-panel = 検索
cmd-search-focus = 検索: プロジェクトを検索
cmd-search-clear = 検索: クリア
search-placeholder = プロジェクトを検索 (Enter で再検索)
search-bad-regex = 無効な正規表現: { $error }
search-running = 検索中…
search-truncated = {" "}({ $count } 件で停止)
search-summary = 結果 { $results } 件、{ $files } ファイル{ $more } · 読み取ったファイル { $scanned } 個
search-min-chars = { $count } 文字以上入力してください

## 置換
cmd-search-replace = 検索: プロジェクト内で置換
search-replace-placeholder = 置換後の文字列 (Enter ですべて置換)
search-replace-all = すべて置換
search-replace-one = 置換
search-cancel = キャンセル
search-replace-all-title = { $files } ファイルで置換しますか?
search-replace-all-message = 
    "{ $query }" の結果 { $results } 件が "{ $replacement }" になります。
    
    タブで開いているファイルは、変更されたまま未保存です (Ctrl+Z で元に戻します)。それ以外はディスクへ直接書き込まれ、元に戻せません。
search-replace-all-truncated = 検索は結果 { $count } 件で停止しました。一覧にあるファイルだけが変わります。残りは、そのあとでもう一度検索してください。
## 範囲、フィルター、移動
cmd-search-next = 検索: 次の結果
cmd-search-prev = 検索: 前の結果
search-filters = フィルター
search-context = 前後の行
search-history = 最近の検索
search-history-empty = 最近の検索はありません
search-include-placeholder = 含めるファイル (例: src/**, *.rs)
search-exclude-placeholder = 除外するファイル (例: *.test.rs)
search-bad-glob = 無効なパターン: { $error }
search-no-results = 結果はありません
search-scope-folder = フォルダー { $path } 内
search-scope-file = ファイル { $path } 内
search-scope-clear = このパスに検索を限定するのをやめる
