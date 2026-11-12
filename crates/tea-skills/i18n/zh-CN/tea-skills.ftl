# idetea skills的文本。
skills-bad-arg = idetea skills：“{ $arg }”无效。
skills-usage = 用法：
    idetea skills                          本项目里已安装的技能
    idetea skills list [source]            某个来源的技能（默认：anthropics/skills）
    idetea skills add <source> <name>…     安装到.agents/skills/和.claude/skills/
    idetea skills update [name…] [--apply] 显示来源里改了什么；--apply会装上
    idetea skills remove <name>…           从项目里移除
  来源是owner/repo（GitHub）、git URL或文件夹。--force会换掉手工改过的副本。
skills-error = 错误：{ $error }
skills-no-cache = idetea skills：没有缓存文件夹（请设置HOME或XDG_CACHE_HOME）
skills-bad-source = 来源无效：“{ $source }”（请用owner/repo、git URL或文件夹）
skills-bad-name = { $path }：名称“{ $name }”无效（小写字母、数字、- 和 _）；已跳过
skills-duplicate = { $path }：已经有别的技能叫“{ $name }”；已跳过
skills-none-installed = 本项目没有已安装的技能（运行idetea skills list，看来源里有哪些）。
skills-installed-line = { $name }  ({ $source } @ { $commit })
skills-list-header = { $source } @ { $commit }：{ $count ->
   *[other] { $count } 个技能
}
skills-files = { $count ->
   *[other] { $count } 个文件
}
skills-tag-scripts = 含脚本
skills-tag-skipped = { $count ->
   *[other] 跳过了 { $count } 个链接
}
skills-tag-installed = 已安装
skills-tag-installed-at = 安装于 { $commit }
skills-add-usage = 请说明要安装哪些技能：idetea skills add <source> <name>…
skills-remove-usage = 请说明要移除哪些技能：idetea skills remove <name>…
skills-not-found = { $source } 里没有技能“{ $name }”（idetea skills list { $source }）
skills-not-installed = 技能“{ $name }”没有安装在本项目里
skills-other-source = 技能“{ $name }”已经来自 { $source }；请先移除，或使用--force
skills-refused = 已拒绝（没有任何更改）：{ $reason }
skills-modified = { $path } 自安装后被改过，你的更改会丢掉；仍要替换或移除的话，请用--force
skills-exists = { $path } 已经存在，而且不是IDE安装的；要替换它，请用--force
skills-added = 已安装：{ $name }（@ { $commit }），位于.agents/skills/和.claude/skills/
skills-has-scripts = { $name } 带有智能体可能会运行的脚本：用之前请先读一遍
skills-gone = 技能“{ $name }”在 { $source } 里已经没有了；项目里的副本保持原样
skills-up-to-date = { $name }：已是最新
skills-update = { $name }：{ $old } → { $new }
skills-updated = 已更新：{ $name }（@ { $commit }）
skills-update-hint = 要装上这些更改：idetea skills update --apply
skills-removed = 已移除：{ $name }
