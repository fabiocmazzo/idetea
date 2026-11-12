## ドキュメントのエラー (パネルに表示。ユーザーとエージェントが直す)
ui-too-big = ドキュメントは { $size } バイトです。上限は { $max } です。
ui-not-json = ファイルは有効な JSON ではありません: { $error }
ui-not-object = ドキュメントは JSON オブジェクトである必要があります。
ui-wrong-format = "format" は "idetea-ui" である必要があります (実際の値: { $found })。
ui-unsupported-version = フォーマットのバージョン { $version } はサポートされていません (この IDE が理解するのは { $supported } です)。
ui-missing = フィールド "{ $field }" がありません。
ui-wrong-type = "{ $field }" の型は { $expected } である必要があります。
ui-too-many = "{ $field }" の項目が { $max } を超えています。
ui-too-long = "{ $field }" が { $max } バイトを超えています。
ui-bad-id = "{ $field }": 無効な id ("{ $id }") です。文字、数字、"_"、"-" を使ってください (64 まで)。
ui-duplicate-id = id が重複しています: "{ $id }"。
ui-bad-path = "{ $field }": "{ $path }" は有効なパスではありません (例: "/name")。
ui-bad-file-path = "{ $field }": "{ $path }" は、プロジェクトのルートからの相対パスである必要があります (".." は使えません)。
ui-data-too-big = "data" が { $max } バイトを超えています。
ui-data-too-deep = "data" の入れ子が { $max } 階層を超えています。

## 警告 (ドキュメントはそのまま開きます)
ui-unknown-field = 不明なフィールドを無視しました: "{ $field }"。
ui-unknown-value = "{ $field }": 不明な値 "{ $value }" のため、既定値を使います。
ui-unknown-component = コンポーネント "{ $component }" は、このバージョンではサポートされていません。
ui-link-error = { $url } を開けませんでした: { $error }
ui-link-ignored = リンクを無視しました (Web、プロジェクトのファイル、idetea:config のみ): { $url }
ui-comment-label = エージェントへのコメント (任意)
ui-comment-placeholder = クリックしたボタンと一緒に送られます
ui-copy = コピー
ui-copied = コピーしました。
ui-steps-missing = まだ確認していないステップが { $count } 件あります。
ui-step-optional = (任意)
ui-requires-unbound = "{ $field }": "{ $path }" はどのコンポーネントの "value" でもないため、無視しました。
ui-fields-missing = 未入力: { $fields }
ui-too-many-images = ドキュメントの画像が { $max } 枚を超えています。
ui-image-loading = { $path }: 読み込み中…
ui-image-error = 読み取れませんでした: { $error }
ui-image-not-file = ファイルではありません (シンボリック リンクはたどりません)
ui-image-too-big = { $max } MB を超えています
ui-image-too-large = { $width }×{ $height } px。1 辺の上限は { $max } です
ui-image-total = ドキュメントの画像が合計 { $max } メガピクセルを超えています
ui-too-many-sounds = ドキュメントの音声が { $max } 個を超えています。
ui-audio-play = 再生
ui-audio-stop = 停止
ui-audio-info = { $seconds } 秒 · { $rate } Hz · { $channels ->
        [2] ステレオ
       *[other] { $channels } チャンネル
    } · { $bits } ビット · { $peak }
ui-audio-peak = ピーク { $db } dBFS
ui-audio-silent = 無音
ui-audio-loop = ループ中
ui-audio-clips = 0 dBFS に達しています。クリッピングしている可能性があります。
ui-audio-extension = 音声ではありません (受け付ける形式: { $list })。
ui-audio-too-big = { $max } MB を超えています。再生はしますが、長さと波形は出しません。
ui-audio-unreadable = 長さも波形もありません ({ $error })。再生はします。
ui-audio-no-player = 音声プレーヤーが見つかりません (pw-play、paplay、aplay、afplay)。
ui-audio-plugin = プラグインのパネルでは音声を再生しません。
ui-sprite-pause = 一時停止
ui-sprite-play = アニメーション
ui-sprite-step = 次のフレーム
ui-sprite-info = フレーム { $frame } / { $count } · { $fps } fps
ui-sprite-frame = { $field }: frame_width と frame_height はセットで指定します。
ui-sprite-too-small = シート ({ $width }×{ $height }) が 1 フレームより小さいです。
ui-open-blocked = { $path }: idetea:open のリンクが開けるのは { $list } だけです。
ui-open-not-file = { $path } は通常のファイルではありません (ない、フォルダー、またはシンボリック リンク)。
ui-select-text = テキストを選択
ui-select-done = 整形して表示
