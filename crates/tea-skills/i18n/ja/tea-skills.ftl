# idetea skills の文言。
skills-bad-arg = idetea skills: "{ $arg }" は無効です。
skills-usage = 使い方:
    idetea skills                          このプロジェクトにインストール済みのスキル
    idetea skills list [source]            ソースのスキル (既定: anthropics/skills)
    idetea skills add <source> <name>…     .agents/skills/ と .claude/skills/ にインストール
    idetea skills update [name…] [--apply] ソース側の変更を表示。--apply でインストール
    idetea skills remove <name>…           プロジェクトから削除
  ソースは owner/repo (GitHub)、git の URL、またはフォルダーです。--force は、手で編集したコピーを置き換えます。
skills-error = エラー: { $error }
skills-no-cache = idetea skills: キャッシュ フォルダーがありません (HOME または XDG_CACHE_HOME を設定してください)
skills-bad-source = 無効なソース: "{ $source }" (owner/repo、git の URL、またはフォルダーを使ってください)
skills-bad-name = { $path }: 名前 "{ $name }" は無効です (小文字、数字、-、_)。スキップしました
skills-duplicate = { $path }: 別のスキルが既に "{ $name }" です。スキップしました
skills-none-installed = このプロジェクトにインストールされたスキルはありません (ソースを見るには idetea skills list)。
skills-installed-line = { $name }  ({ $source } @ { $commit })
skills-list-header = { $source } @ { $commit }: スキル { $count } 個
skills-files = ファイル { $count } 個
skills-tag-scripts = スクリプトあり
skills-tag-skipped = スキップしたリンク { $count } 件
skills-tag-installed = インストール済み
skills-tag-installed-at = { $commit } にインストール済み
skills-add-usage = インストールするスキルを指定してください: idetea skills add <source> <name>…
skills-remove-usage = 削除するスキルを指定してください: idetea skills remove <name>…
skills-not-found = { $source } にスキル "{ $name }" はありません (idetea skills list { $source })
skills-not-installed = スキル "{ $name }" はこのプロジェクトにインストールされていません
skills-other-source = スキル "{ $name }" は既に { $source } から入っています。先に削除するか、--force を使ってください
skills-refused = 拒否しました (変更なし): { $reason }
skills-modified = { $path } はインストール後に変更されています。このままではその変更が失われます。それでも置き換えるか削除するには、--force を使ってください
skills-exists = { $path } は既にあり、IDE がインストールしたものではありません。置き換えるには --force を使ってください
skills-added = インストールしました: { $name } (@ { $commit }) を .agents/skills/ と .claude/skills/ へ
skills-has-scripts = { $name } には、エージェントが実行しうるスクリプトがあります。使う前に読んでください
skills-gone = スキル "{ $name }" は { $source } にもうありません。プロジェクトのコピーはそのまま残します
skills-up-to-date = { $name }: 最新です
skills-update = { $name }: { $old } → { $new }
skills-updated = 更新しました: { $name } (@ { $commit })
skills-update-hint = 変更をインストールするには: idetea skills update --apply
skills-removed = 削除しました: { $name }
