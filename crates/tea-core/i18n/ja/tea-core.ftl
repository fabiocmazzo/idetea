## 件数 (参照パネル)
count-references = 参照 { $count } 件
count-definitions = 定義 { $count } 件
count-implementations = 実装 { $count } 件
count-types = 型 { $count } 件

## シンボル (ファイル構造)
kind-module = モジュール
kind-class = クラス
kind-struct = 構造体
kind-interface = インターフェース/トレイト
kind-enum = 列挙型
kind-variant = バリアント
kind-method = メソッド
kind-function = 関数
kind-constructor = コンストラクター
kind-field = フィールド
kind-property = プロパティ
kind-variable = 変数
kind-constant = 定数
kind-type-parameter = 型パラメーター
kind-block = ブロック
kind-other = シンボル
visibility-public = パブリック
visibility-private = プライベート

## 設定とショートカットのエラー
chord-empty = 空のショートカット
chord-unknown-modifier = 不明な修飾キー: { $modifier }
chord-no-key = キーのないショートカット: { $chord }
config-key-not-text = { $layer }: [keys] の { $chord } はテキストである必要があります。実際の値は { $value } です
config-keys-not-table = { $layer }: [keys] はテーブルである必要があります
config-unknown-section = 不明なセクション: [{ $section }]
config-agents-project = { $layer }: [agents] はユーザー設定 (~/.config/idetea/config.toml) でのみ有効です。ここでは無視します
config-font-missing = [editor] font = "{ $name }": フォントがインストールされていないため、{ $default } を使います
config-font-weight = [editor] font_weight = "{ $name }": { $choices } のいずれかを使ってください
config-file-icons = [ui] file_icons = "{ $value }": "seti" または "none" を使ってください
