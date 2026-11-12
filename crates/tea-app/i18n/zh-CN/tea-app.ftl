## 设置 → 外观

settings-theme = 主题
settings-theme-detail = IDE、编辑器和终端的颜色。
settings-font-size = 字体大小
settings-font-size-detail = 编辑器和终端。
settings-code-font = 代码字体
settings-code-font-detail = 编辑器、行号和终端。IDE自带JetBrains Mono；列表里是已安装的等宽字体。
settings-font-weight = 字重
settings-font-weight-detail = 字体如果没有这个字重，就用最接近的一个。
settings-line-height = 行高
settings-line-height-detail = 以字体大小的倍数计。终端按同样的比例跟着变。
settings-file-icons = 文件图标
settings-file-icons-detail = 每种语言或文件类型一个图标（VS Code的Seti图标集）。关闭：所有文件用同一个图标。
settings-activity-bar = 活动栏
settings-activity-bar-detail = 左侧边缘的一条，单击就能显示或隐藏面板。
settings-splash = 启动时显示启动画面
settings-splash-detail = 启动画面（至少2秒）。--no-splash可让这一次不显示。
settings-language = 语言
settings-language-detail = 界面文本。自动跟随系统语言。重启IDE后生效。
settings-language-auto = 自动（{ $language }）
settings-language-restart = 重启IDE后生效（当前：{ $language }）。

## 核心命令（命令面板和帮助里的标题）
cmd-palette-open = 命令面板
cmd-help-open = 帮助: 快捷键和指南
cmd-settings-open = 设置
cmd-file-new = 文件: 新建
cmd-file-save = 文件: 保存
cmd-file-open = 文件: 打开…
cmd-file-quick-open = 文件: 按名称打开…
cmd-debug-continue = 调试: 启动 / 继续
cmd-debug-stop = 调试: 停止
cmd-debug-pause = 调试: 暂停
cmd-debug-next = 调试: 下一行
cmd-debug-step-in = 调试: 逐语句进入函数
cmd-debug-step-out = 调试: 逐出函数
cmd-debug-toggle-breakpoint = 调试: 切换当前行断点
cmd-debug-choose-program = 调试: 选择程序…
cmd-file-open-folder = 文件: 在新窗口中打开文件夹…
cmd-file-save-as = 文件: 另存为…
cmd-file-save-as-path = 文件: 另存为（输入路径）…
cmd-file-close = 文件: 关闭选项卡
cmd-tab-next = 选项卡: 下一个
cmd-tab-prev = 选项卡: 上一个
cmd-tab-reopen = 选项卡: 重新打开上次关闭的
cmd-editor-find = 编辑器: 查找
cmd-editor-replace = 编辑器: 查找和替换
cmd-editor-goto-line = 编辑器: 转到行
cmd-editor-undo = 编辑器: 撤消
cmd-editor-redo = 编辑器: 重做
cmd-editor-select-all = 编辑器: 全选
cmd-editor-fold-all = 编辑器: 折叠所有代码块
cmd-editor-unfold-all = 编辑器: 展开所有代码块
cmd-editor-toggle-fold = 编辑器: 折叠或展开光标处的代码块
cmd-editor-toggle-vim = 编辑器: 切换 vim 模式
cmd-editor-zoom-in = 编辑器: 放大
cmd-editor-zoom-out = 编辑器: 缩小
cmd-editor-zoom-reset = 编辑器: 重置缩放
cmd-lsp-definition = LSP: 转到定义
cmd-lsp-references = LSP: 查找引用
cmd-lsp-implementation = LSP: 转到实现
cmd-lsp-type-definition = LSP: 转到类型定义
cmd-lsp-incoming-calls = LSP: 谁调用此函数
cmd-lsp-outgoing-calls = LSP: 此函数调用了什么
cmd-nav-file-symbol = 转到文件中的符号…
cmd-nav-project-symbol = 转到项目中的符号…
cmd-nav-back = 导航: 后退
cmd-nav-forward = 导航: 前进
cmd-lsp-hover = LSP: 显示光标处的文档
cmd-lsp-format = LSP: 格式化文件
cmd-lsp-restart = LSP: 重启服务器
cmd-theme-toggle = 主题: 切换浅色/深色
cmd-config-open = 配置: 打开全局配置
cmd-config-open-project = 配置: 打开项目配置
cmd-config-reload = 配置: 重新加载
cmd-view-toggle-left = 布局: 显示/隐藏左侧
cmd-view-toggle-right = 布局: 显示/隐藏右侧
cmd-view-toggle-bottom = 布局: 显示/隐藏底部面板
cmd-app-restart = 重启IDE
cmd-app-quit = 退出
cmd-mascot-toggle = 吉祥物: 在角落显示/收起

## 提示、对话框和菜单
file-new-created-on-save = 新文件：{ $path }（保存时创建）
file-open-error = 打开 { $path } 时出错：{ $error }
file-changed-on-disk-kept = “{ $name }”在磁盘上已更改，但选项卡也有更改：已保留选项卡
file-reload-error = 重新加载 { $path } 时出错：{ $error }
file-changed-on-disk-overwrite = “{ $name }”在磁盘上已更改，但选项卡也有更改：保存会覆盖磁盘上的文件
debug-still-building = 调试：仍在构建…
debug-already-running = 调试：程序已在运行
debug-building = 调试：正在构建（{ $command }）…
unknown-error = 未知错误
debug-build-failed = 调试：构建失败：{ $error }
debug-choose-title = 要调试哪个程序？
debug-choose-message = 项目构建出了不止一个可执行文件。
cancel = 取消
debug-running = 调试：正在运行 { $program }
debug-error = 调试：{ $error }
debug-cancelled = 调试：已取消（构建会自己结束）
debug-stopped = 调试：已停止
debug-no-source = 调试：{ $name } 没有源代码
debug-paused-no-source = 调试：已在 { $name } 暂停（{ $reason }），没有源代码
debug-paused-at = 调试：已在 { $path }:{ $line } 暂停（{ $reason }）
debug-exited = 调试：程序已退出
debug-exited-code = 调试：程序已退出（代码 { $code }）
debug-command-failed = 调试：{ $command } 失败：{ $message }
debug-gdb-exited = 调试：gdb已退出（{ $reason }）
debug-no-session = 调试：没有正在运行的会话（按F5启动）
debug-already-paused = 调试：程序已经暂停
debug-pause-first = 调试：程序正在运行（请先暂停）
nav-nothing-forward = 没有可前进的位置
nav-nothing-back = 没有可后退的位置
lsp-no-references = 未找到引用
lsp-no-definition = 未找到定义
lsp-no-implementations = 未找到实现
lsp-no-type = 未找到类型
lsp-not-used-elsewhere = “{ $symbol }”没有在别的地方用过
outline-save-first = 请先保存文件，再看它的结构
outline-reading = 正在读取结构…
symbols-type-a-name = 输入函数、类型、方法的名称…
searching = 正在搜索…
symbols-none-found = 未找到符号（如果项目还在加载，请再试一次）
symbols-none-in-file = 此文件里没有符号
lsp-save-first = 请先保存文件，再用语言服务器
lsp-finding-references = 正在查找引用…
lsp-finding-implementations = 正在查找实现…
calls-not-in-function = 光标不在函数里（或服务器不知道它的调用）
lsp-formatted = 文件已格式化
lsp-level-error = 错误：{" "}
lsp-level-warning = 警告：{" "}
lsp-server-stopped = { $server } 已停止 — { $reason }。请使用“LSP: 重启服务器”。
lsp-server-crashed = { $server } 已崩溃 — { $reason }。你一停手，它自己会回来。
lsp-server-crashed-too-often = { $server } 在 { $minutes } 分钟内崩溃了 { $count } 次 — { $reason }。请使用“LSP: 重启服务器”。
lsp-no-formatting = 此文件无法格式化
lsp-restarted = 语言服务器已重启
save-changes-title = 保存更改？
save-changes-message = “{ $name }”有未保存的更改。
save = 保存
discard = 放弃
picker-already-open = 已经有一个文件对话框开着
save-tab-closed = 还没保存，选项卡就关上了
folder-opening = 正在新窗口中打开 { $path }
folder-open-error = 无法打开文件夹：{ $error }
saved = 已保存：{ $path }
save-error = 保存出错：{ $error }
file-deleted-kept = 文件已在磁盘上删除；带着更改的选项卡留了下来
and-more = …以及另外 { $count } 个
quit-unsaved-title = 还有未保存的更改，仍要退出？
quit-save-all = 全部保存并退出
quit-without-saving = 不保存并退出
save-needs-name = “{ $name }”还没有名字：请使用“文件: 另存为…”
save-file-error = 保存 { $path } 出错：{ $error }
close-tabs-title = 
    { $count ->
       *[other] 关闭 { $count } 个选项卡？
    }
close-tabs-unsaved = 有未保存的更改：
save-and-close = 保存并关闭
discard-changes = 放弃更改
tab-close = 关闭
tab-close-others = 关闭其他
tab-close-right = 关闭右侧
tab-close-saved = 关闭已保存的
tab-close-all = 全部关闭
copy-path = 复制路径
copy-relative-path = 复制相对路径
reveal-in-tree = 在文件树中显示
open-in-file-manager = 在文件管理器中打开
copied = 已复制：{ $text }
file-manager-error = 无法打开文件管理器：{ $error }
save-then-close = 请先选好保存位置，然后再关一次这个选项卡
config-reloaded-problems = 配置已重新加载，但有问题：{ $problems }
config-reloaded = 配置已重新加载（打开或关闭扩展要重启后才生效）
config-no-home = 找不到配置文件夹（没有设置HOME）
create-error = 创建 { $path } 出错：{ $error }
config-error = 配置：{ $error }
write-error = 写入 { $path } 出错：{ $error }
unknown-command = 未知命令：{ $id }
keys-record-hint = 请用Ctrl、Alt或特殊键（F1–F12、PageUp…）。按Esc取消。
keys-already-bound = { $key } 本来就是“{ $command }”
keys-rebound = { $key } 原来是“{ $old }”；现在是“{ $command }”
keys-bound = { $key } → “{ $command }”
theme-unknown = 未知主题“{ $name }”，改用 "dark"
default = 默认

## 设置
settings-title = 设置
settings-tab-appearance = 外观
settings-tab-extensions = 扩展
settings-tab-shortcuts = 快捷键
close-esc = 关闭（Esc）
settings-no-home = （没有设置HOME）
settings-write-to = 保存到
settings-target-global = 全局
settings-target-project = 项目
settings-open-file = 打开文件
keys-filter = 按名称、命令或按键筛选…
keys-press = 请按下按键…（Esc取消）
keys-add = 添加快捷键
keys-help = 单击某个按键就能去掉。+ 用来录一个新按键（已经占用的也可以：它们会换主人）。
settings-lsp = 语言服务器
settings-lsp-detail = 把LSP整个打开或关掉。
settings-servers = 服务器
settings-server-found = 已在PATH中找到
settings-server-not-found = 未在PATH中找到
settings-lsp-note = 更改只对之后启动的服务器生效。
settings-restart-servers = 重启服务器
theme-dark = 深色（默认）
theme-light = 浅色
slot-left = 左侧
slot-right = 右侧
slot-bottom = 底部
slot-status = 状态栏
settings-on-restart = 重启IDE后生效
default-capitalized = 默认
settings-extensions-note = 打开或关闭扩展要重启后才生效。把面板挪到另一侧会马上生效。
ext-filetree = 文件树
ext-statusbar = 状态栏
ext-search = 项目搜索
ext-problems = 错误和警告
ext-terminal = 集成终端
ext-git = 更改、暂存、提交和差异
ext-debug = 调试按钮、调用堆栈、变量和输出
ext-references = 一个名称用在哪里（Shift+F12）
ext-outline = 文件里的函数和方法，以及可见性（Alt+7）
ext-preview = 编辑器旁边的Markdown和SVG预览（Ctrl+Shift+V）
ext-calls = 谁调用了函数，以及函数调用了什么（Alt+Shift+H）
ext-interfaces = 智能体写到.idetea/ui/ 的界面（实验性）

## 命令面板、对话框、LSP、调试，以及应用的其余部分
palette-commands = 输入命令…
palette-save-as = 文件路径（相对于项目根）
palette-files = 文件名（file:line转到该行，> 用来切到命令）
palette-file-symbols = 此文件中的函数、类型、方法…
palette-project-symbols = 项目中的函数、类型、方法…
palette-no-commands = 没有找到命令
palette-save-hint = Enter保存 · Esc取消
palette-reading-files = 正在读取项目里的文件…
palette-no-files = 没有找到文件
palette-no-symbols = 没有符号
palette-no-matching-symbols = 没有和搜索匹配的符号
dialog-hint = Enter确认 · Esc取消
lsp-command-not-found = { $server }：在PATH里找不到命令“{ $command }”（请安装，或在配置里改[lsp.servers.{ $server }]）
lsp-start-error = { $server }：无法启动：{ $error }
lsp-disabled = 配置里关掉了LSP（[lsp] enabled = false）
lsp-no-server-for = 没有为 .{ $ext } 文件配置语言服务器
lsp-not-running = { $server } 没有在运行：{ $reason }
lsp-not-ready = { $server } 还没准备好
lsp-hint-rustup-install = { $reason }。请这样安装：rustup component add rust-analyzer
lsp-hint-rustup = { $reason }。如果用了rustup：rustup component add rust-analyzer
lsp-hint-ziex = { $reason }。请安装Ziex CLI（https://ziex.dev/learn），或打开项目文件夹（它的build.zig.zon依赖ziex）：然后IDE会用zig构建服务器
lsp-hint-moonbit = { $reason }。请安装MoonBit工具链（https://www.moonbitlang.com/download），里面有`moon lsp`；IDE也会到 ~/.moon/bin（或 $MOON_HOME/bin）里找
lsp-hint-hare = { $reason }。hare-lsp不随Hare附带：请从https://git.sr.ht/~whynothugo/hare-lsp构建，并放进PATH（例如 ~/.local/bin）
config-syntax-error = 语法 { $file }：{ $error }
visibility-file-only = 仅此文件
picker-save-as = 另存为
picker-open-files = 打开文件
picker-open-folder = 打开文件夹
picker-debug-program = 要调试的程序
splash-ready = 就绪
splash-loading = 正在加载…
watch-too-many = 文件夹太大：只监视前 { $count } 个文件夹
watch-partial = 没法监视所有文件夹（{ $error }）
watch-problem = 磁盘上的更改：{ $message }。需要的时候，在文件树里用“刷新”。
watch-failed = 磁盘上的更改：没法监视（{ $error }）
debug-no-gdb = 在PATH里找不到gdb（调试要用gdb 14或更新的版本）
debug-program-not-found = 找不到程序：{ $path }
debug-gdb-start-error = 无法启动gdb：{ $error }
debug-reason-breakpoint = 断点
debug-reason-step = 单步
debug-reason-pause = 已暂停
debug-reason-exception = 异常
debug-reason-entry = 入口
debug-reason-goto = 跳转
build-run-error = 无法运行“{ $command }”：{ $error }
build-no-message = 失败了，而且没有消息

## 帮助和命令行
help-key-find = 在文件中查找
help-key-replace = 查找和替换
help-key-goto-line = 转到行
help-key-undo = 撤消 / 重做
help-key-next-match = 选中下一处（多光标）
help-key-quick-fix = LSP快速操作（有的话）
help-key-ctrl-click = Ctrl+单击
help-key-definition = 转到定义（LSP）
help-key-tab = 插入制表符
help-title = 帮助
help-tab-shortcuts = 快捷键
help-tab-guide = 指南
help-tab-about = 关于
help-close = 关闭（Esc）
help-filter = 按名称、命令或按键筛选…
help-group-general = 常规
help-editor-keys = 编辑器（固定）
help-in-palette = （命令面板）
about-no-home = （没有设置HOME）
about-version = 版本
about-license = 许可证
about-license-value = { $license }（自由开源，一直如此）
about-project = 项目
about-global-config = 全局配置
about-project-config = 项目配置
about-built-with = 用Rust写成，基于iced、iced-code-editor、alacritty_terminal（经由iced_term）和syntect。
about-icons = 图标：Codicons © Microsoft，许可证CC BY 4.0。
cli-usage = 
    用法：idetea [选项] [路径...]
    
      路径           要打开的文件（可以还不存在：保存时才创建）
                     或当作项目根的文件夹
      --no-splash    不显示启动画面（也可以：IDETEA_NO_SPLASH=1）
      --menu         从菜单打开：没有文件夹时，打开该文件所在的项目，或上次的项目
      -f, --foreground  保持附着在终端上（否则，从终端启动时IDE会脱离，
                     日志写到~/.local/state/idetea/idetea.log）
      --install-desktop    把IDE TEA加进应用程序菜单（带图标）
      --uninstall-desktop  从应用程序菜单里去掉
      ui check <file.json>…  检查界面文档（按面板读取它们的方式）
      new <folder> [--modelo id]  新建项目：文件夹、git、模板的技能和说明
      skills [list|add|update|remove]  在项目里安装智能体技能（idetea skills --help）
      ui fix [--apply]  列出（加上--apply则修好）.idetea/ui/里留下的东西
      ask …          在界面面板里提问并等待回答（单独的idetea ask：用法）
      run … -- <command>  向IDE要一个终端并等到结束（单独的idetea run：用法）
      status [--json]  本项目里打开的IDE正在显示什么（选项卡、更改、光标）
      open <file[:line[:column]]>  已打开的IDE显示该文件，并把光标放到位
      history [--json]  你在界面面板里回答过的内容（--name、--since、--last N、--all）
      mascot <状态> [--text "…"]  更新IDE吉祥物（idle、working、thinking、attention、celebrating、error、sleeping、clear、status）
      -h, --help     显示此帮助
      -V, --version  显示版本
cli-unknown-option = 已忽略未知选项：{ $option }
cli-extra-root = 一次只能有一个根文件夹，已忽略 { $path }

## 帮助 → 指南（`{ $comando-com-hifens }`会变成该命令的快捷键）
guide-start = 入门
guide-start-1 = 在文件夹里用`idetea`打开IDE（这个文件夹就是文件树的根），或用`idetea file...`（文件在选项卡里打开；根仍是当前文件夹）。
guide-start-2 = 所有命令都在命令面板里：{ $palette-open }。打名称的一部分就行，快捷键显示在旁边。
guide-start-3 = 用 { $help-open } 打开这份帮助。
guide-start-4 = 左侧边缘的图标栏，单击就能显示或隐藏面板（开着的那个会高亮）。拖面板和编辑器之间的边可以改大小；双击恢复默认。
guide-files = 文件和选项卡
guide-files-1 = 按名称打开：{ $file-quick_open }（打名称里的几段就行；`main.rs:42`转到那一行；`>`切到命令）。
guide-files-2 = 转到符号（函数、类型、方法…）：文件里用 { $nav-file_symbol }，整个项目用 { $nav-project_symbol }。需要语言服务器。
guide-files-3 = 新建：{ $file-new } · 打开：{ $file-open } · 保存：{ $file-save } · 另存为：{ $file-save_as } · 关闭选项卡：{ $file-close }。
guide-files-4 = 打开和另存为走系统的文件对话框。想自己打路径：{ $file-save_as_path }。打开另一个文件夹（{ $file-open_folder }）会新开一个IDE窗口。
guide-files-5 = Markdown和SVG预览：{ $preview-toggle }（或选项卡栏最右边的按钮）打开侧边面板，跟着当前文件走，你一打字它就更新。网页链接在浏览器里打开；指向文件的链接在选项卡里打开。
guide-files-6 = 图像（png、jpg、gif、webp…）在自己的选项卡里打开：滚轮缩放，拖动平移。
guide-files-7 = PDF也在选项卡里打开：滚动翻页；− 和 + 改缩放，“适合”跟着宽度走。
guide-files-8 = 切换选项卡：{ $tab-next } 和 { $tab-prev }（Ctrl+Tab在编辑器里插入的是制表符）。
guide-files-9 = 关掉还有未保存更改的选项卡，或这样退出时，会先问你：保存、放弃或取消。
guide-files-10 = 文件在外面被改了（终端、别的编辑器）：选项卡如果没有未保存的更改，会自己重新加载；如果有，就保持原样，并给你一条提示。
guide-files-11 = 右键单击选项卡：关闭其他、右侧、已保存的或全部，复制路径，在文件树中显示，以及在文件管理器中打开。中键单击关掉这个选项卡。
guide-tree = 文件树
guide-tree-1 = 显示或隐藏：{ $filetree-toggle } · 定位当前文件：{ $filetree-reveal }。
guide-tree-2 = 右键单击某一项：新建文件、新建文件夹、重命名、删除、复制路径，以及在文件管理器中打开。删除会送进系统回收站。
guide-tree-3 = Git颜色：黄色 = 已修改，绿色 = 新建，红色 = 冲突，变淡 = 已忽略。里面有更改的文件夹会显示一个圆点。
guide-bottom = 底部面板
guide-bottom-1 = 项目搜索（{ $search-focus }）、问题（{ $problems-focus }）、终端（{ $terminal-focus }）和Git（{ $git-focus }）是底部面板里的选项卡。{ $view-toggle_bottom } 用来显示或隐藏这个面板。
guide-bottom-2 = 下一个问题：{ $problems-next }。
guide-terminal = 终端
guide-terminal-1 = 焦点在终端时，Ctrl加字母、功能键和方向键都交给shell（Ctrl+C、Ctrl+W…）。带Ctrl+Shift的IDE快捷键仍然管用，所以命令面板总能打开。
guide-terminal-2 = 在终端里复制和粘贴：Ctrl+Shift+C和Ctrl+Shift+V。输入`exit`会关掉面板。
guide-debug = 调试
guide-debug-1 = 断点：单击行号，或 { $debug-toggle_breakpoint }。启动或继续：{ $debug-continue }。Rust项目会用cargo build构建，并自己找到可执行文件；别的项目请用配置里的[debug]（program、args、cwd、build），或自己选可执行文件。换程序：{ $debug-choose_program }。
guide-debug-2 = 停下来之后：{ $debug-next } 下一行，{ $debug-step_in } 逐语句进入函数，{ $debug-step_out } 逐出，{ $debug-pause } 暂停，{ $debug-stop } 停止。黄色箭头标着程序停在哪。要用gdb 14+。
guide-debug-3 = 调试面板（{ $debug-focus }）：按钮、调用堆栈（单击可转到那一帧，并看到它的变量）、变量（单击展开结构体和向量），以及程序和构建的输出。
guide-git = Git
guide-git-1 = 分支显示在状态栏。Git面板（{ $git-focus }）列出改了什么：+ 和 − 用来暂存和取消暂存，单击文件看差异，在消息框里按Enter提交。
guide-git-2 = 在Git里右键单击文件：打开、查看差异、暂存、放弃更改（会先问你）以及复制路径。
guide-git-3 = 获取（{ $git-fetch }）、拉取（{ $git-pull }）和推送（{ $git-push }）是面板顶上的按钮。这里不问密码：请用ssh-agent（ssh-add）或凭据助手。新分支发布到远程之前会先问你。
guide-git-4 = 一保存，面板就更新；仓库有变化时也会更新（在终端里提交或检出）。
guide-lsp = 语言（LSP）
guide-lsp-1 = 打开文件时，语言服务器会自己启动：rust-analyzer、clangd、pyright、typescript-language-server、gopls和lua-language-server（得先装好）。
guide-lsp-2 = 转到定义：{ $lsp-definition }，或Ctrl+单击 · 文档：{ $lsp-hover }，或把鼠标停在代码上 · 格式化：{ $lsp-format }。
guide-lsp-3 = 看它用在哪：{ $lsp-references }，或在声明自己上面Ctrl+单击。列表出现在引用面板里；单击就跳到那一处。
guide-lsp-4 = 实现：{ $lsp-implementation }（在trait或接口上，或在它的某个方法上）。光标底下那个的类型：命令面板里的“LSP: 转到类型定义”。结果不止一个时（定义也一样），列表进引用面板；只有一个就直接跳过去。
guide-lsp-5 = 谁调用了这个函数：光标放在函数上，用 { $lsp-incoming_calls }。调用面板是一棵树（展开一层，能看到谁又调用了这些调用方）；“被调用方”换成它自己发出的调用。单击就跳到那次调用。
guide-lsp-6 = 跳完之后可以后退和前进（定义、引用、按名称打开、在面板里单击）：{ $nav-back } 和 { $nav-forward }，或者鼠标侧键。
guide-lsp-7 = 文件大纲（{ $outline-toggle }）：函数、结构体、方法和字段排成一棵树。开着的锁是公共，关上的锁是私有，钥匙表示只在一部分代码里看得见（pub(crate)、protected、package）。光标下的符号会高亮；单击就跳过去。
guide-lsp-8 = 服务器要是失败了，状态栏会说为什么；“LSP: 重启服务器”会再试一次。
guide-config = 设置
guide-config-1 = 设置界面（{ $settings-open }）可以改主题、字体、扩展、快捷键和LSP，写到全局配置或项目配置里。改完马上生效。
guide-config-2 = 有一份全局配置，每个项目还有一份（`.idetea/config.toml`）；项目的那份优先。要直接改文件：“配置: 打开全局配置”或“配置: 打开项目配置”。一保存就全部重新加载。
guide-config-3 = 快捷键写在[keys]里："ctrl+alt+t" = "terminal.focus"。写成空文本，就会关掉一条默认快捷键。命令id显示在“快捷键”选项卡里。
guide-config-4 = 语言：设置 → 外观 → 语言（“自动”跟着系统走）。重启IDE后生效。

## 会话
settings-restore-session = 按上次的样子重新打开
settings-restore-session-detail = 打开项目时，把它的选项卡、光标和面板找回来。idetea file...只打开那个文件。

## 最近使用
cmd-file-open-recent = 文件: 打开最近使用的项…
palette-recent = 以前打开过的项目或文件
palette-no-recent = 还没有打开过什么
guide-files-12 = 打开最近使用的项：{ $file-open_recent } 列出各个项目（在新窗口里打开），以及所有窗口最近打开的文件。

## 恢复未保存的内容
untitled-tab = 未命名
recover-title = 
    { $count ->
       *[other] IDE关掉的时候，有 { $count } 个文件没保存
    }
recover-detail = “恢复”会在选项卡里打开这些更改，仍然不保存（撤消会回到磁盘上的内容）。“稍后”下次还会再问。
recover = 恢复
recover-later = 稍后
recovered = 
    { $count ->
       *[other] 已恢复 { $count } 个文件（还没保存）
    }

## 重命名符号，以及全部保存
cmd-lsp-rename = LSP: 重命名符号…
cmd-file-save-all = 文件: 全部保存
save-all-untitled = 未命名的选项卡没有保存（请使用“文件: 另存为…”）
palette-rename = 新名称
palette-rename-hint = 在整个项目里重命名“{ $name }” · Enter看看会改哪里 · Esc取消
rename-no-word = 把光标放在要改名的那个名字上
rename-asking = 重命名“{ $name }”：正在问服务器…
rename-nothing = “{ $name }”没有什么可重命名的
rename-failed = 无法重命名：{ $message }
rename-title = 把“{ $old }”重命名为“{ $new }”？
rename-summary = 
    { $changes ->
       *[other] { $changes } 处更改
    }，共 { $files ->
       *[other] { $files } 个文件：
    }
rename-detail = 文件在各自的选项卡里保持更改，还没保存：每个里用Ctrl+Z撤消，“文件: 全部保存”会全部写下去。
rename = 重命名
renamed = 
    { $count ->
       *[other] 已在 { $count } 个文件中重命名（还没保存）
    }

guide-lsp-9 = 重命名：光标放在名字上，用 { $lsp-rename }。预览会告诉你有多少处更改、在哪些文件里；确认之后，它们在各自选项卡里保持更改（每个里Ctrl+Z可撤消），{ $file-save_all } 会全部保存。

## 在项目中替换
replaced = 已在 { $files } 个文件中替换 { $count } 处
replaced-failed = 已在 { $files } 个文件中替换 { $count } 处；{ $failed } 处改不了（没有权限，或不是文本）

guide-bottom-3 = 在项目里替换：{ $search-replace }（或搜索旁边的 ▸）会打开“替换为”。每条结果都会显示换成什么样；“替换”改这一行或这一个文件，“全部替换”会先问你。开在选项卡里的文件保持更改、不保存（Ctrl+Z撤消）；其余的直接写进磁盘。

## 编辑器和保存
format-save-timeout = 服务器没能及时格式化：文件保存了，但没有格式化

settings-tab-editor = 编辑器
settings-wrap = 长行自动换行
settings-wrap-detail = 比窗口宽的行接到下面继续，不再横向滚动。
settings-whitespace = 显示空白字符
settings-whitespace-detail = 空格显示成 ·，制表符显示成 →。
settings-indent-guides = 缩进参考线
settings-indent-guides-detail = 每一级缩进画一条细竖线。
settings-tab-size = 缩进大小
settings-tab-size-detail = Tab一次插入几个空格。
settings-insert-spaces = 用空格缩进
settings-insert-spaces-detail = 关闭：Tab插入的是制表符。已经用制表符缩进的文件，会继续用制表符。
settings-trim = 保存时去掉行尾空白
settings-trim-detail = Markdown除外，因为行尾两个空格表示换行。
settings-final-newline = 保存时补上末尾换行
settings-final-newline-detail = 保证最后一行以换行结束。
settings-format-on-save = 保存时格式化
settings-format-on-save-detail = Ctrl+S会先让语言服务器格式化再写入（最多等2秒）。全部保存不会格式化。
settings-breadcrumbs = 痕迹
settings-breadcrumbs-detail = 编辑器上方：文件的路径，以及光标所在的符号。单击文件夹，在文件树里显示它；单击符号，跳过去。
settings-editorconfig-note = 项目里的.editorconfig会盖过这些选项（缩进、制表符还是空格、保存时清理）。

guide-config-5 = 编辑器：设置（{ $settings-open }）里的编辑器选项卡有自动换行、看得见的空白字符、缩进参考线、缩进大小，以及保存时做什么（去掉行尾空格、补上末尾换行、格式化）。项目里的.editorconfig会盖过它们；没有的话，已经用制表符缩进的文件会继续用制表符。

guide-lsp-10 = 参数提示：在调用里打一个 ( ，光标上方会露出函数签名，当前参数高亮；每打一个逗号就往前走，括号一合上，或按Esc，它就消失。

guide-git-5 = 装订线标记：行号左边，绿色 = 新行，蓝色 = 改过，红色缺口 = 删掉的行，对照的是上次提交（跟着你打字走，不必先保存）。点一下标记能看到原来什么样，还有“还原更改”（Ctrl+Z可以改回来）。

palette-no-items = 没有和搜索匹配的

guide-git-6 = 分支：{ $git-checkout }（或单击状态栏、Git面板里的分支名）列出分支，最近用的在前；这边还没有的远程分支排在后面，选中就变成本地分支。打一个新名字，就从你现在的位置建分支。没提交的更改会跟着走；要是和另一个分支撞上，git会拒绝，并说明原因。

guide-git-7 = 行作者：状态栏里、分支旁边，能看到谁最后改过光标这一行、什么时候改的，以及提交消息（单击可看哈希并复制）。选项卡还有未保存的更改时，它会先藏起来，等你保存（磁盘上的行可能对不上）。

## 任务
cmd-tasks-run = 任务: 运行…
cmd-tasks-rerun = 任务: 再次运行上一个
cmd-tasks-test-here = Rust: 运行光标处的测试
tasks-placeholder = 运行任务…
tasks-none = 没有任务：在配置的[tasks]下加一条，或打开带Cargo.toml、package.json或Makefile的项目。
tasks-no-terminal = 任务要用终端扩展，而它关着。
tasks-no-test = 光标不在Rust测试里（#[test] fn）。
tasks-no-package = 找不到这个文件所属包的Cargo.toml。
tasks-done = [{ $name } 已结束：代码 { $code } · Enter关闭]

guide-terminal-3 = 任务：{ $tasks-run } 列出项目命令（配置里的[tasks]、cargo build/run/test/clippy、package.json的脚本、Makefile的目标），在终端的“任务”选项卡里跑你选的那条，不动你的shell。结束时给出退出码；按Enter关掉。Ctrl+C只停这条命令。{ $tasks-rerun } 再跑上一次的那条。
guide-terminal-4 = 光标处的测试（Rust）：光标在`#[test] fn`里面时，{ $tasks-test_here } 只跑这一个测试（cargo test -p 包名 测试名）。没有默认按键：常用的话，在[keys]下自己加。

## 拆分编辑器
cmd-editor-split = 编辑器: 把选项卡移到另一侧（拆分）
cmd-editor-focus-left = 编辑器: 转到左侧
cmd-editor-focus-right = 编辑器: 转到右侧
split-one-tab = 再打开一个文件才能拆分编辑器：每一侧各显示一个不同的选项卡。
tab-move-right = 移到右侧（拆分）
tab-move-left = 移到左侧
guide-files-13 = 拆分编辑器：{ $editor-split }（或选项卡菜单里的“移到右侧（拆分）”）把选项卡放到旁边，那边有自己的选项卡；之后打开文件，进的是当前聚焦的一侧。单击某一侧（或 { $editor-focus_left } / { $editor-focus_right }）就在那边干活；拖分隔条改宽度（双击回到正中）。把某一侧最后一个选项卡移回去，两边就又合成一个。

## 应用程序菜单（--install-desktop）
desktop-no-home = 找不到主文件夹（HOME）。
desktop-error = 无法安装快捷方式：{ $error }
desktop-installed = 
    IDE TEA已出现在应用程序菜单里：{ $desktop }
    运行：{ $exe }
desktop-removed = 已从应用程序菜单里去掉IDE TEA。

## 大文件
file-light-mode = “{ $name }”有一行特别长，已用轻量模式打开：没有颜色，没有语言服务器，并且自动换行
lsp-light-mode = 文件处于轻量模式（有一行特别长）：没有语言服务器

## 可执行文件已重新编译
binary-changed = IDE重新构建过了：请运行“重启IDE”（命令面板）来用新版本。
binary-changed-title = [新版本：请重启]
binary-restart-error = 无法重启：{ $error }

## idetea status / idetea open（RFC 0006）
status-usage =
    用法：idetea status [--json]
    
    本项目里打开的IDE正在显示什么：带着光标的活动选项卡，以及打开的文件
    （有更改的标成“未保存”）。--json：同样的内容，JSON，带字段 "open"。
    退出码：0表示开着，2表示参数不对，3表示IDE没在这个项目里打开。
status-bad-arg = 未知参数：{ $arg }
status-not-open = IDE没在这个项目里打开（{ $root }）
status-active = 活动：{ $path }
status-no-active = 没有活动文件
status-modified = {" "}（未保存）
open-usage =
    用法：idetea open <file[:line[:column]]>
    
    本项目里打开的IDE会显示这个文件，并把光标放到对应的行和列。
    退出码：0表示已显示，2表示路径不对或在项目外面，3表示IDE没开着（或没接住这次请求）。
open-outside = { $path } 不在项目里（{ $root }）
open-write-error = 无法写入 { $path }
open-not-taken = IDE没有在3秒内接住这次请求

## 会回应的吉祥物（RFC 0012）
mascot-state-idle = 平静
mascot-state-working = 忙碌
mascot-state-thinking = 思考
mascot-state-attention = 留意
mascot-state-celebrating = 完成
mascot-state-error = 错误
mascot-state-sleeping = 小憩
mascot-state-curious = 好奇
mascot-state-annoyed = 有点烦
mascot-idle-1 = 只管写代码。冰，你自己加。
mascot-idle-2 = 有蒸汽。我也在。
mascot-idle-3 = 杯子安静，眼睛睁着。
mascot-idle-4 = 茶叶可以沉一沉。
mascot-idle-5 = 不急。茶还是热的。
mascot-idle-6 = 想喝的时候再喝。
mascot-idle-7 = 杯子站在你这边。
mascot-idle-8 = 茶碟上很平静。
mascot-hover-curious-1 = 嗯？需要什么吗。
mascot-hover-curious-2 = 你好。还顺利吗。
mascot-hover-curious-3 = 感觉到了。怎么了。
mascot-hover-curious-4 = 在茶上面路过？
mascot-hover-curious-5 = 打个招呼，还是搭把手。
mascot-hover-curious-6 = 杯子看见你了。
mascot-hover-curious-7 = 有点好奇。请说。
mascot-hover-curious-8 = 欢迎。下一步是什么。
mascot-hover-annoyed-1 = 慢一点，茶在打旋。
mascot-hover-annoyed-2 = 鼠标太多了。杯子有点晕。
mascot-hover-annoyed-3 = 手稳一点。茶是热的。
mascot-hover-annoyed-4 = 经过一次是问好。刚才那阵是风。
mascot-hover-annoyed-5 = 茶碟在响。
mascot-hover-annoyed-6 = 好，我看见了。少转一点。
mascot-poke-curious-1 = 嗯？什么事。
mascot-poke-curious-2 = 杯子有点痒。
mascot-poke-curious-3 = 你叫我？
mascot-poke-curious-4 = 点了一下。我在听。
mascot-poke-curious-5 = 小心，满着。
mascot-poke-curious-6 = 茶碟上的你好。
mascot-poke-double-1 = 两下。蒸汽也加倍。
mascot-poke-double-2 = 双击记下了。里面还是茶。
mascot-poke-double-3 = 两次？好，我在。
mascot-poke-double-4 = 在杯子上敲了两下。
mascot-poke-double-5 = 两下就算聊天了。
mascot-poke-double-6 = 点了两次。没有洒出来。
mascot-poke-annoyed-1 = 小心，要洒了。
mascot-poke-annoyed-2 = 茶已经是热的。
mascot-poke-annoyed-3 = 一只杯子，点得够多了。
mascot-poke-annoyed-4 = 我喜欢你。茶碟没那么确定。
mascot-poke-annoyed-5 = 轻一点。杯子不大。
mascot-poke-annoyed-6 = 收到了。我凉一凉。
mascot-task-1 = 开始了: { $name }
mascot-task-2 = 正在沏 { $name }…
mascot-task-3 = 任务 { $name } 在跑。
mascot-task-4 = { $name }。我看着壶。
mascot-task-5 = 启动 { $name }。
mascot-task-6 = { $name } — 让它泡一会儿。
mascot-term-pending-1 = 终端在等一声同意: { $title }
mascot-term-pending-2 = 有条命令在等: { $title }
mascot-term-pending-3 = 你来定: { $title }
mascot-term-pending-4 = 请允许一下: { $title }
mascot-term-pending-5 = 壶先拿着: { $title }
mascot-term-pending-6 = { $title } — 要跑吗。
mascot-term-running-1 = 正在运行: { $title }
mascot-term-running-2 = { $title } 在火上。
mascot-term-running-3 = 进行中: { $title }
mascot-term-running-4 = { $title } — 还在泡。
mascot-term-running-5 = 正在处理 { $title }。
mascot-term-running-6 = { $title } 已经在路上。
mascot-term-ok-1 = 完成: { $title }
mascot-term-ok-2 = { $title } 干净地结束了。
mascot-term-ok-3 = 没问题 — { $title }。
mascot-term-ok-4 = { $title } 出来得不错。
mascot-term-ok-5 = 正常退出: { $title }。
mascot-term-ok-6 = { $title }。杯子想庆祝一下。
mascot-term-err-1 = { $title } 失败了 ({ $code })。
mascot-term-err-2 = 退出 { $code }: { $title }
mascot-term-err-3 = { $title } 洒了（代码 { $code }）。
mascot-term-err-4 = 这个没成: { $title } ({ $code })。
mascot-term-err-5 = { $title } 没有做完 ({ $code })。
mascot-term-err-6 = { $title } 的代码 { $code }。
mascot-answered-1 = 知道了: { $title }
mascot-answered-2 = 已回答: { $title }
mascot-answered-3 = { $title } — 谢谢。
mascot-answered-4 = 记下了: { $title }
mascot-answered-5 = { $title } 安顿好了。
mascot-answered-6 = 你回答了 { $title }。
mascot-wait-1 = 代理不再等了: { $title }
mascot-wait-2 = { $title } 安静下来了。
mascot-wait-3 = 没有人还在等 { $title }。
mascot-wait-4 = { $title } 凉了。
mascot-wait-5 = 还开着，但等待结束了: { $title }
mascot-errors-1 = 项目里有 { $count } 个错误。
mascot-errors-2 = 茶叶找到了 { $count } 个错误。
mascot-errors-3 = { $count } 个，想看的时候再看。
mascot-errors-4 = 问题: { $count }。不急。
mascot-errors-5 = { $count } 个错误。我可以指给你。
mascot-errors-6 = 茶碟报告: { $count } 个错误。
mascot-save-1 = 已保存 { $name }。杯子赞成。
mascot-save-2 = { $name } 已经在磁盘上。
mascot-save-3 = 在变凉之前接住了 { $name }。
mascot-save-4 = { $name } — 收好了。
mascot-save-5 = 写入完成: { $name }。
mascot-save-6 = { $name } 安全了。回到茶上。
mascot-wake-1 = 你回来了。茶还是温的。
mascot-wake-2 = 我打了个盹。我们到哪儿了。
mascot-wake-3 = 欢迎回来。我再看着。
mascot-wake-4 = 眼睛睁开了。杯子也是。
mascot-wake-5 = 这一觉歇得不错。
mascot-wake-6 = 继续？我守着。
mascot-sleep-1 = 安静地泡着…
mascot-sleep-2 = 我在这儿。
mascot-sleep-3 = 小火。需要就叫我。
mascot-sleep-4 = 杯子犯困。代码可以歇一歇。
mascot-sleep-5 = 在茶碟上打盹。
mascot-sleep-6 = 灯调暗了。你回来我就醒。
mascot-greet-morning-1 = 早。第一杯我来。
mascot-greet-morning-2 = 早上好。先写代码，再加冰。
mascot-greet-morning-3 = 早光，新茶叶。
mascot-greet-morning-4 = 早晨。看 diff 之前，先小喝一口。
mascot-greet-morning-5 = 你好。壶已经坐在火上了。
mascot-greet-afternoon-1 = 下午好。壶里的还行。
mascot-greet-afternoon-2 = 下午好。节奏稳住就好。
mascot-greet-afternoon-3 = 中午。不用一口气喝完。
mascot-greet-afternoon-4 = 下午的问候，来自杯子。
mascot-greet-afternoon-5 = 太阳高。茶也是。
mascot-greet-evening-1 = 晚上好。再一个文件，或者茶碟。
mascot-greet-evening-2 = 晚上好。杯子还是热的。
mascot-greet-evening-3 = 傍晚的光。想收就收。
mascot-greet-evening-4 = 这一天在把火调小。
mascot-greet-evening-5 = 晚上。构建不留下的话，我留下。
mascot-greet-late-1 = 很晚了。茶没事。能歇就歇一会儿。
mascot-greet-late-2 = 安静的时辰。睡觉也是一项功能。
mascot-greet-late-3 = 还醒着？把这个念头收好，然后休息。
mascot-greet-late-4 = 杯子可以留着。你可以暂停。
mascot-greet-late-5 = 深夜。我守着。尽量短一点。
mascot-tip-1 = 所有命令都在命令面板里: { $palette-open }。
mascot-tip-2 = 按名字打开文件: { $file-quick-open }。
mascot-tip-3 = { $file-save } 把文件写到磁盘。
mascot-tip-4 = 快捷键和一份短说明: { $help-open }。
mascot-tip-5 = 运行项目任务: { $tasks-run }。
mascot-tip-6 = 终端是 { $terminal-focus }。
mascot-tip-7 = 问题: { $problems-focus }。下一个是 { $problems-next }。
mascot-tip-8 = 跳到定义: { $lsp-definition }。
mascot-build-ok-1 = 构建结束了。茶还是热的。
mascot-build-ok-2 = 编译过了。茶叶不错。
mascot-build-ok-3 = 构建是干净的。
mascot-build-ok-4 = 绿色的构建。我倒一杯。
mascot-build-ok-5 = 编译完了。回到编辑器。
mascot-build-err-1 = 构建没有成。
mascot-build-err-2 = 构建失败了。杯子感觉到了。
mascot-build-err-3 = 编译错误。想看的时候再看。
mascot-build-err-4 = 这次构建洒了。
mascot-build-err-5 = 构建失败。问题面板也许知道更多。
mascot-working-1 = 在火上。
mascot-working-2 = 忙着。蒸汽也忙。
mascot-working-3 = 让它泡一会儿。
mascot-working-4 = 还在做。
mascot-working-5 = 壶占着。
mascot-thinking-1 = 在搅茶叶…
mascot-thinking-2 = 等一下。在想。
mascot-thinking-3 = 让它再泡一泡。
mascot-thinking-4 = 安静的念头，温热的杯子。
mascot-thinking-5 = 慢慢看。
mascot-celebrate-1 = 好了。出来得不错。
mascot-celebrate-2 = 都齐了。
mascot-celebrate-3 = 收得干净。干杯。
mascot-celebrate-4 = 杯子赞成。
mascot-celebrate-5 = 不错。蒸汽我留着。
mascot-error-1 = 洒了一点。
mascot-error-2 = 这个没成。
mascot-error-3 = 绊了一下。我还在。
mascot-error-4 = 这次不行。
mascot-error-5 = 值得看一眼，你想看的时候。
mascot-open-interfaces = 打开 Interfaces 面板 →
mascot-open-terminal = 打开终端面板 →
mascot-open-problems = 打开问题面板 →
mascot-idle-hint = 看着代理和任务
mascot-bad-state = 不认识的吉祥物状态: { $state }
mascot-usage =
    用法: idetea mascot <状态> [--text "…"] [--source <名字>] [--ttl <秒>]
          idetea mascot clear
          idetea mascot status [--json]

    状态: idle, working, thinking, attention, celebrating, error, sleeping, curious, annoyed。
    写入 .idetea/mascot.json，让 IDE 里的吉祥物马上有反应（--ttl 之后过期，默认 30 秒）。


## 欢迎屏幕
welcome-start = 开始
welcome-new-file = 新建文件
welcome-quick-open = 打开项目文件…
welcome-open-file = 打开文件…
welcome-open-folder = 在新窗口中打开文件夹…
welcome-recent = 打开最近使用的项…
welcome-recent-files = 最近的文件
welcome-recent-projects = 最近的项目
welcome-hint-palette = 全部命令
welcome-hint-search = 搜索项目
welcome-hint-terminal = 终端
