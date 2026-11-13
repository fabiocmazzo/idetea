ui-panel = 界面
cmd-ui-show = 界面: 显示面板
ui-empty = 没有打开的界面。智能体可以往.idetea/ui/写一个文件来创建（见docs/interfaces/formato.md）。要开或关：在配置里写[extensions] interfaces = {"{"} enabled = true {"}"}。
ui-tracked = { $path } 被git跟踪，出于安全已忽略：界面只来自这台机器上正在跑的智能体，绝不会来自克隆下来的仓库。
ui-invalid = 文档无效：{ $error }
ui-origin = { $path }
ui-origin-source = { $path } · { $source }
ui-waiting = { $count ->
   *[other] 已送出 { $count } 条回答；正在等智能体来读。
}
ui-received = 智能体已收到回答。
ui-agent-waiting = 智能体正在等回答。
ui-agent-stopped = { $minutes ->
   *[other] 智能体已停止等待 { $minutes } 分钟：回答会留着，直到它再来看。
}
ui-new = 来自智能体的新界面：{ $title }
ui-updated = 界面已更新：{ $title }
ui-open-later = 智能体要求打开 { $path }：它在界面面板里（没有直接打开，免得打断你打字）。
ui-write-failed = 无法写入回答：{ $error }
ui-prepare-failed = 无法准备.idetea/ui文件夹：{ $error }
ui-not-a-file = 不是普通文件
ui-surface-mismatch = surface字段是 "{ $found }"，但文件名是 { $expected }.json
cmd-ui-teach = 界面: 教会本项目的智能体
ui-teach-hint = 要让编程智能体知道怎么用这个面板，把说明写进项目的AGENTS.md。
ui-teach-button = 教会本项目的智能体
ui-teach-title = 教会本项目的智能体？
ui-teach-detail = IDE会把界面面板的说明写到：{ $files }。文字留在标记之间；再跑一次只更新那一段，其余不动。这些是项目文件：提交前请看一遍。
ui-teach-file-new = { $name }（新建）
ui-teach-write = 写入
ui-cancel = 取消
ui-teach-up-to-date = 智能体说明已是最新。
ui-teach-done = 说明已写入 { $files }。
ui-teach-failed = 无法写入说明：{ $error }
cmd-ui-new-project = 项目: 新建…
new-title = 新建项目
new-intro = IDE会创建文件夹和git仓库，放上模板的技能和面板说明，并在新窗口里打开项目。真正的项目由你的智能体来建，它会跟着 **AGENTS.md** 里的初始请求走。如果下面选了智能体，新窗口会请求在IDE终端里打开它（命令由你确认）。
new-field-parent = 创建在
new-field-name = 文件夹名称
new-field-name-hint = 例如my-app
new-field-model = 模板
new-field-description = 项目是什么（可选）
new-field-description-hint = 一句话；会写进给智能体的初始请求
new-field-git = 创建git仓库
new-field-agent = 要在IDE终端里打开的智能体
new-agent-none = 无（我自己打开）
new-agent-title = 智能体：{ $agent }
new-bad-agent = 智能体名称无效："{ $agent }"（只能是字母、数字、- 和 _）
new-create = 创建并打开
new-model = { $model ->
    [vazio] 空（由智能体询问它是什么）
    [rust] Rust
    [site] 网站
    [python] Python
    [fastapi] FastAPI（Python Web API）
   *[other] { $model }
}
new-creating = 正在创建 { $dir }（技能从网络取过来）…
new-created = 项目已创建于 { $dir }；正在新窗口里打开。
new-open-failed = 项目已创建于 { $dir }，但窗口打不开：{ $error }
new-failed = 无法创建项目：{ $error }
new-bad-model = 未知模板：{ $model }
new-bad-dir = 文件夹无效：{ $dir }（请用完整路径）
new-bad-name = 文件夹名称无效："{ $name }"
new-not-empty = { $dir } 已经存在，而且里面有文件：请换个名字
new-done-dir = 文件夹：{ $dir }
new-done-git = git：已创建仓库
new-done-agents = 带初始请求的AGENTS.md和CLAUDE.md，以及面板说明
new-done-agent = { $agent } 的终端请求：请在新窗口里确认
new-open-hint = 完成。这样打开：idetea { $dir }
new-usage = idetea new："{ $arg }" 无效。用法：idetea new <folder> [--modelo id] [--descricao text] [--sem-git] [--agente name]；模板：{ $models }
cmd-ui-fix = 界面: 修复此项目
ui-fix-title = 修复此项目？
ui-fix-will = IDE将：
ui-fix-wont = 只是告诉你（保持原样）：
ui-fix-button = 修复
ui-fix-nothing = 此项目的界面没什么要修的。
ui-fix-done = { $count ->
   *[other] 已修复：{ $count } 项。
}
ui-fix-failed = 无法修复：{ $error }
ui-fix-hint = 要修标着 "-" 的项：idetea ui fix --apply
ui-fix-usage = idetea ui fix：未知参数 "{ $arg }"（用法：idetea ui fix [--apply]）
ui-fix-abandoned = 删除 { $path }：已经 { $since } 没有人在等这个回答
ui-fix-unread = { $path }：{ $count ->
   *[other] { $count } 条智能体还没读的回答
}（保留；请让智能体去读，或手动删除）
ui-fix-leftover = 删除 { $path }：某次没写完或没等完留下的
ui-fix-check = 重写.idetea/ui/check：它调用的 { $target } 已经不存在
ui-fix-instructions-older = 重写 { $file } 里的说明：版本是 { $found }，比这个IDE旧
ui-fix-instructions-unversioned = 重写 { $file } 里的说明：来自旧版IDE，没有版本行
ui-fix-instructions-newer = { $file } 里的说明来自更新的IDE（{ $found }）：请用那个IDE打开项目（我不会往回改）
ui-outdated = 此IDE已过时（打开之后又重新构建过）：回答里会告诉智能体。命令面板：“重启IDE”。
ui-audio-failed = 无法播放声音：{ $error }
ui-check-ok = { $file }：正常（{ $count ->
       *[other] { $count } 个组件
    }）
ui-check-error = { $file }：错误：{ $error }
ui-check-warning = 警告：{ $warning }
ui-check-media = 媒体：{ $path }：{ $problem }
ui-check-name = 文件名无效（"{ $name }"）：小写字母、数字、_ 和 -，以.json结尾
ui-check-usage = 用法：idetea ui check <file.json>…
ui-check-instructions-older = 说明：{ $file } 的版本是 { $found }，这个IDE是 { $ours }：请让用户运行“界面: 教会本项目的智能体”，发布前再读一遍该文件
ui-check-instructions-newer = 说明：{ $file } 的版本是 { $found }，来自比这个IDE（{ $ours }）更新的IDE：打开的面板可能读不懂该文件教的内容
ui-check-instructions-unversioned = 说明：{ $file } 有一段旧内容，没有版本行；这个IDE是 { $ours }：请让用户运行“界面: 教会本项目的智能体”，发布前再读一遍该文件
ui-teach-newer-file = { $name }（版本 { $version }）
ui-teach-newer = 警告：{ $files } 由比这个更新的IDE写入（这个的版本是 { $ours }，不认识那一个）。现在写入会把说明退回到这个IDE的版本；通常应该用更新的IDE打开项目。
ui-teach-write-anyway = 仍然写入
ask-title = 提问
ask-choose = 选择
ask-ok = 确认
ask-yes = 是
ask-no = 否
ask-not-object = 文档必须是JSON对象
ask-bad-arg = idetea ask：“{ $arg }”无效。
ask-usage = 用法：idetea ask <document.json | -> | --text T [--options "a|b|c" | --confirm | --input Label] [--title T] [--name name] [--value /pointer] [--json] [--keep] [--timeout seconds]；idetea ask --resume <name>；idetea ask --close <name>
ask-invalid = idetea ask：面板会拒绝这个提问：{ $error }
ask-no-such = idetea ask：面板里没有提问“{ $name }”
ask-timeout = idetea ask：没有及时得到回答；提问“{ $name }”留在面板里（idetea ask --resume { $name } 会再等一次）
ask-outdated = idetea ask：回答来自一个打开后又重新构建过的IDE（ide.outdated）；如果正在测试改动，请要求重启IDE
ask-comment = 评论：{ $comment }

# 界面历史（RFC 0007）
ui-history-panel = 界面历史
cmd-ui-history = 界面: 历史
cmd-ui-clear-history = 界面: 清除历史
ui-history-failed = 无法写入历史：{ $error }
ui-history-cleared = { $count ->
   *[other] 已从历史里删掉 { $count } 条。
}
ui-history-link = 历史（{ $count }）
ui-history-empty = 历史是空的。
ui-history-clear-title = 清除历史？
ui-history-clear-detail = { $count ->
   *[other] 将删除本项目的 { $count } 条历史（.idetea/history/）。
} 面板里打开的界面会留下。此操作无法撤消。
ui-history-clear-button = 清除历史
ui-history-search = 搜索标题、回答和评论
ui-history-open = 打开（{ $count }）
ui-history-waiting-you = 正在等你
ui-history-answered-open = 已回答，仍开着
ui-history-today = 今天
ui-history-yesterday = 昨天
ui-history-none = 还没有回答过的提问。你在界面面板里给出的每个回答都会留在这里（.idetea/history/，不进git）。
ui-history-no-match = 什么都没找到。
ui-history-back = ← 返回
ui-history-record-decision = 记录为决定
ui-history-decision-recorded = 决定已记录到{ $path }：请检查并写上后果。
ui-history-decision-failed = 无法记录决定：{ $error }
adr-context = 问题和前提
adr-options = 想过的选项
adr-decision = 决定
adr-chosen = 选择
adr-consequences = 后果
adr-consequences-todo = _待写：哪些变好，哪些变差。_
adr-origin = 记录自IDE TEA界面面板里的问题“{ $name }”，回答于{ $date }。
ui-history-shown = 显示于：{ $time }
ui-history-answered = 回答于：{ $time }
ui-history-closed = 没回答就关上了：{ $time }
ui-history-read = 智能体已读：{ $time }
ui-history-button = 按钮：{ $action }
ui-history-not-read = 智能体没有读过这个回答（文件还在.idetea/ui/responses/ 里，或是在IDE关闭时被删掉了）。
ui-history-not-read-short = 未读
ui-history-unanswered = 无回答
ui-history-media-changed = { $path } 在回答之后改过，或已经不在了：这里看到的是现在的版本。
ui-history-no-audio = 历史记录不播放声音
history-cli-comment = 评论
history-cli-not-read = 智能体没有读过这个回答
history-cli-unanswered = 没有回答（智能体删掉了提问）
history-cli-bad = idetea history：“{ $arg }”无效。
history-cli-usage = 用法：idetea history [--name name] [--since YYYY-MM-DD] [--last N | --all] [--json]
history-cli-empty = idetea history：这个项目的历史里什么都没有。
