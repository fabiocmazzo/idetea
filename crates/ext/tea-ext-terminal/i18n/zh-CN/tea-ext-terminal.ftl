## 终端
terminal-start-error = 终端：无法启动shell：{ $error }
cmd-terminal-focus = 终端: 打开并聚焦
cmd-terminal-new = 终端: 新建终端
cmd-terminal-close = 终端: 关闭终端
cmd-terminal-restart = 终端: 重启shell
terminal-ended-n = 终端 { $number } 已关闭
terminal-not-running = shell没有在运行。
terminal-start = 启动终端

## 任务
terminal-task-error = 任务：无法启动：{ $error }
terminal-tab-n = 终端 { $number }
terminal-close = 关闭终端
terminal-tab-task = 任务：{ $name }
terminal-close-task = 关闭任务

## 智能体请求的终端
terminal-tab-agent = 智能体：{ $title }
terminal-tab-request = { $count ->
   *[other] { $count } 个终端请求
}
agent-request-notify = 有智能体请求终端：{ $title }。请看终端面板。
agent-request-auto = 终端请求“{ $title }”未确认就运行了：它在你配置的[agents] allow里（"{ $entry }"）。
agent-request-someone = 某个智能体
agent-request-title = { $who } 请求终端：{ $title }
agent-request-command = 命令（将原样运行）：
agent-request-cwd = 所在文件夹：{ $cwd }
agent-request-worktree = 在自己的工作树里运行：分支 { $branch }，位置 { $path }（分支不存在时从HEAD创建）
agent-worktree-kept = 工作树 { $path } 保留了下来：{ $error }
agent-request-warning = 它会用你的权限运行，就跟你自己敲进去一样。只允许你认得的命令。
agent-request-allow = 允许并运行
agent-request-refuse = 拒绝
agent-request-more = { $count ->
   *[other] 此后还有 { $count } 个请求。
}
agent-done = 已结束，代码 { $code }。按Enter关闭选项卡。
run-bad-arg = idetea run：“{ $arg }”无效。
run-usage = 用法：idetea run [--title T] [--name name] [--cwd folder] [--worktree branch] [--keep] [--no-wait] [--timeout seconds] -- <command>；idetea run --wait <name>；idetea run --stop <name>
run-busy = idetea run：已有请求“{ $name }”（或其状态）；请另选--name，或用idetea run --wait { $name } 等待
run-asked = idetea run：请求“{ $name }”已在终端面板里；正在等用户允许，并等命令结束
run-exited = idetea run：“{ $name }”已结束，代码 { $code }
run-refused = idetea run：用户拒绝了“{ $name }”
run-invalid = idetea run：请求“{ $name }”无效：{ $error }
run-ended = idetea run：“{ $name }”结束时没有代码（{ $state }）
run-branch = idetea run：工作在分支 { $branch } 上
run-timeout = idetea run：时间到了，“{ $name }”仍在终端里（idetea run --wait { $name } 会再等一次；--stop会结束它）

## 标签页状态 (RFC 0024)
terminal-state-running = 运行中
terminal-state-unseen = 上次查看后有新输出
terminal-state-attention = 需要你的注意
terminal-state-exited = 已结束，代码 { $code }
terminal-state-bell = 发出了提示音
terminal-state-turn-done = 本轮已完成，正在等你
cmd-terminal-next = 终端：下一个标签页
cmd-terminal-prev = 终端：上一个标签页
cmd-terminal-goto = 终端：转到…
cmd-terminal-next-attention = 终端：下一个在等你的标签页
cmd-terminal-rename = 终端：重命名标签页
terminal-no-attention = 没有需要注意的终端标签页。
terminal-goto-placeholder = 转到终端标签页…
terminal-rename-placeholder = 标签页名称
terminal-rename-reset = 恢复默认名称
terminal-rename-to = 重命名为
terminal-close-running-title = 关闭标签页？
terminal-close-running = { $name } 仍在运行。关闭标签页会结束该进程。
terminal-close-running-yes = 关闭并结束
terminal-close-running-no = 取消
