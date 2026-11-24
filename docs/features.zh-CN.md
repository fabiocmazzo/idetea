# IDE TEA：功能与快捷键

[English](features.md) · [Français](features.fr.md) · [Deutsch](features.de.md) · [Italiano](features.it.md) · [日本語](features.ja.md) · **中文（简体）**

完整清单，写给想看细节的人。短版见[项目说明](../README.zh-CN.md)。

> 尚未到1.0。已在Linux、macOS（Apple Silicon）和Windows（x86_64）上构建并测试。

## 编辑器

- 按名称打开文件（`Ctrl+P`），模糊搜索；`main.rs:42`直接跳到那一行。
- 转到文件里的符号（函数、类型、方法……）（`Ctrl+Shift+O`），或转到项目里的符号（`Ctrl+T`）。
- 选项卡带已修改标记；打开（`Ctrl+O`）、保存和另存为，使用系统文件对话框。
- 大约200种语言和文件类型的语法高亮（TOML、`.env`、INI、Dockerfile、TypeScript、Zig、[Ziex](https://ziex.dev)的`.zx`……）；浅色和深色主题（另有iced的主题：Nord、Dracula……）。更多语言：把`.sublime-syntax`文件放进`~/.config/idetea/syntaxes/`（IDE打开时读取）。
- 查找和替换、转到行、撤消/重做、多光标、代码折叠，以及可选的vim模式。
- 编辑器临时缩放（`Ctrl+=` / `Ctrl++`、`Ctrl+-`、`Ctrl+0`或`Ctrl+滚轮`），已打开的选项卡和分屏共用；终端和配置的基础字号不变。
- Markdown和SVG预览开在编辑器旁边（`Ctrl+Shift+V`，或选项卡栏上的按钮），随输入更新。
- 图像（png、jpg、gif、webp、bmp、ico……）在选项卡里打开，可以用滚轮缩放，可以拖动。
- PDF在选项卡里打开（由纯Rust的`hayro`渲染），可缩放，可适应宽度。
- CSV、TSV和JSONL以表格打开（选项卡栏的按钮或`table.toggle`回到文本）：排序和筛选不改动文件；编辑一个单元格，文本里只改这个字段（保留引号，`Ctrl+Z`可撤销）。JSONL的单元格接受JSON，无效的值会被拒绝。
  拖动列边界可调整宽度；点击行号（Shift扩展选择），`Ctrl+C`复制为TSV，`Ctrl+Shift+C`复制为CSV。`[table] max_mb`（默认50）设定大小上限。
- 保留CRLF。其他编码的文件按原编码打开和保存（BOM、UTF-16、`.editorconfig`的`charset`，否则用`[editor] fallback_encoding`，默认`windows-1252`）；点击状态栏上的编码可以用另一种编码重新打开或保存。放不下的字符在保存时会被拒绝，绝不变成`?`。二进制文件不会打开。
- 大文件：50MB的文件在0.1秒内打开。只有一行超长行的文件（压缩过的JSON或JS）以轻量模式打开，没有颜色，并且自动换行，这样窗口不会卡死。
- 保存时可选：去掉行尾空白、补上文件末尾换行、用语言服务器格式化。项目里的`.editorconfig`优先。

## 项目

- **文件树：**创建、重命名和删除（送进回收站），在树里定位当前文件，git颜色（已修改、新文件、冲突、已忽略）。
- **项目搜索**随输入进行，遵守`.gitignore`，支持正则、全词匹配和区分大小写。在文件夹或文件上右键，只搜索那一条路径（面板里的标签可以去掉）。要包含和要排除的文件是逗号分隔的 glob（`src/**`、`*.test.rs`），收在“过滤”后面。每一处匹配都会高亮；替换预览里，旧的一段带删除线，新的一段高亮。上和下移动结果，Enter 打开那一行，左和右折叠文件。`F4` 和 `Shift+F4` 跳到下一处和上一处匹配，编辑器有焦点时也一样。上下文行来自 `[search] context_lines`（默认 0）和面板上的按钮。最近 20 条查询留在历史里。
- **右键菜单**出现在文件树、选项卡和git面板里的文件上。
- **任务**（`Ctrl+Shift+B`）：配置里你自己的命令，加上IDE在项目里找到的（cargo、`package.json`的脚本、Makefile目标、`build.zig`的步骤），输出在终端里。MoonBit模块还会得到`moon check`、`build`、`test`和`fmt --check`，在离当前文件最近的`moon.mod`所在目录运行。
- **新建项目**（“项目: 新建…”或`idetea new <folder>`）：文件夹、git、模板的智能体技能（空项目、Rust、网站、Python、FastAPI），以及写进`AGENTS.md`的初始请求。见[skills.zh-CN.md](skills.zh-CN.md)。

## 语言（LSP）

- 错误和警告（编辑器里的波浪下划线和诊断悬停、问题面板，以及状态栏上的计数）、补全、转到定义、格式化。
- 代码操作（`Ctrl+.`）：服务器针对错误或选区提供的修复和重构（缺少的import、生成函数、提取变量），推荐项排在最前。
  在服务器发来新的诊断之前，波浪线会跟着编辑的文字移动。
- 内联提示：变量推断出的类型和调用中的参数名，以灰色显示在代码里。光标会跳过它们，也不会被保存。默认开启；
  `Ctrl+Alt+I`开关，`[lsp] inlay_hints = false`则一直关闭。
- 语义着色：语言服务器告诉每个名字是什么，参数、字段和函数调用在语法颜色之上使用主题为它们定义的颜色（局部变量保持语法的颜色）。
  默认开启，`[lsp] semantic_tokens = false`关闭。
- 转到实现（`Ctrl+F12`）和转到类型；有多条结果时，列表进到面板里。
- 查找引用（`Shift+F12`，或在声明上`Ctrl+单击`），按文件分组；每次跳转之后可以后退/前进（`Alt+←` / `Alt+→`，或鼠标侧键）。
- 调用层次结构（`Alt+Shift+H`）：谁调用了这个函数，它又调用了什么，是一棵随展开而加载的树。
- 大纲面板（`Alt+7`）：文件里的函数、结构体、方法和字段，标出公开、私有或受限（`pub(crate)`、`protected`……），光标下的符号会高亮。
- 编辑器上方的痕迹：`project › folder › file › impl App › fn update`，也就是光标所在的路径和符号。点文件夹会在文件树里显示该文件；点符号会跳过去。`[editor] breadcrumbs = false`可以关掉。
- 代码字体：内置JetBrains Mono；任何已安装的字体都可以（`[editor] font = "Fira Code"`），连字也可用。字重和行高也可以（`font_weight`、`line_height`），设置 → 外观里同样有。编辑器、行号和终端用同一套字体。
- 按语言和类型显示文件图标（内置VS Code的Seti图标集，另有IDE自己的Hare和MoonBit图标），出现在文件树、选项卡、Ctrl+P、痕迹、搜索结果和git面板里。`[ui] file_icons = "none"`（或设置 → 外观）会回到所有文件共用一个图标。
- Shift+单击从上次单击的位置扩展选区（用键盘移动光标之后，则从光标扩展）。
- 按词移动：`Ctrl+Left/Right`移动（按住`Shift`则选择），`Ctrl+Backspace/Delete`删除。`Home`先到行内第一个字符，再到第0列。`Shift+Tab`减少缩进。
- 和以前不一样的地方对齐VS Code的按键：用`Ctrl+Shift+[` / `]`折叠/展开光标处的块，用`Ctrl+Shift+T`重新打开上次关闭的选项卡，底部面板是`Ctrl+J`，终端是`` Ctrl+` ``或`Ctrl+'`。
- 已经配好：rust-analyzer、clangd、pyright、typescript-language-server、gopls、lua-language-server、zls（Zig）、`zx lsp`（[Ziex](https://ziex.dev)；没有`zx`命令行时，Ziex项目会用`zig build`构建自己的服务器）、`hare-lsp -S`和`moon lsp`。装上服务器即可；你可以在配置里改命令、禁用，或再加别的。Hare的服务器目前只限于解析诊断、定义、悬停和文档符号；MoonBit支持`.mbt`、`.mbti`和`.mbt.md`（带MoonBit代码块的Markdown），能做到哪一步取决于已安装的工具链。每种语言的细节：[languages.zh-CN.md](languages.zh-CN.md)。

## 终端

- 底部面板里的集成终端（alacritty），使用主题的颜色和字体。
- 多个shell以选项卡排列（“终端1”、“终端2”……），任务也有自己的选项卡；`+`打开一个，`×`关闭，`exit`关闭该选项卡。
- shell的快捷键（`Ctrl+C`、`Ctrl+W`……）交给shell；IDE的`Ctrl+Shift`快捷键仍然有效。
- 每个选项卡都有一个表示状态的点：运行中、有你还没看过的新输出、需要你的注意（提示音或智能体的hooks），结束时显示退出码（绿色或红色）；鼠标悬停会说明。shell的选项卡使用shell设置的标题。
- 智能体能通知自己的选项卡：每个选项卡都有`IDETEA_TERMINAL`，`idetea mascot setup-hooks`安装的Claude Code hooks会在智能体请求权限或结束本轮时标记它所在的选项卡。
- `Ctrl+Shift+PgDn` / `Ctrl+Shift+PgUp`在终端选项卡之间切换（在终端里也可以）。*终端：转到…*列出各选项卡及其状态；*终端：下一个在等你的标签页*跳到下一个需要你的选项卡。双击选项卡（或*终端：重命名标签页*）可以给它命名。
- 关闭仍在运行的智能体或任务前会先询问。
- 两个终端并排（`Ctrl+Shift+5`或拆分按钮）；在其中一个里点击或输入就会把焦点给它。
- 在终端中查找（放大镜，或*终端：查找*）：纯文本，从最新的匹配开始。
- `[terminal] scrollback`设置每个选项卡保留多少行（默认10000）。
- 每个终端右侧有一条细滚动条，占用自己的宽度，从不遮住文字：拖动滑块，或点击轨道翻一页。在备用屏幕（vim、less、htop）中会隐藏。
- 重新打开 IDE 时终端选项卡会回来：shell 自动恢复并保留名称；启动器打开的智能体要等你确认（*全部恢复*、*选择…*、*不*，
  或稍后用 *终端：恢复上次会话的智能体*）。Claude Code 和 Codex 回到同一个对话，其他智能体开始新的对话。
- `[terminal] keep_scrollback = true` 在关闭时保存每个选项卡的最后几行，并在新提示符上方显示一次。默认关闭：终端里有密码
  和令牌。文本保存在项目之外，只有你能读。

## Git

- 状态栏显示分支、领先/落后的提交数，以及更改数量。
- 面板里可以按文件或全部暂存/取消暂存，有带颜色的差异、提交，以及放弃更改（会确认）。
- 推送、拉取和获取。IDE从不等待密码：它用你的ssh agent或git credential helper。
- 编辑器装订线上标出自上次提交以来添加、修改或删除的行；单击标记会显示旧的行，并可以撤销。
- 状态栏显示光标所在行的作者和提交（blame），可看详情，也有复制哈希的按钮。
- 在命令面板里切换或创建分支（*Git: 切换分支*）。
- 使用已安装的`git`，所以你的配置、钩子和凭据都生效。

## 调试

- 在行号上单击设置断点（`F9`）；编辑文本时，断点跟着文本走。
- `F5`构建并运行：Rust项目用`cargo build`并找到可执行文件；Zig项目用`zig build`，可执行文件在`zig-out/bin`；MoonBit模块用`moon build --target native -g`（实验性，见[languages.zh-CN.md](languages.zh-CN.md#moonbit)）；其他情况由配置里的`[debug]`段给出程序、参数和构建命令。
- 逐过程、逐语句、逐出（`F10`、`F11`、`Shift+F11`），暂停和停止，当前行在编辑器里标出。
- 面板里有调用堆栈（单击跳到该帧）、树形的变量，以及程序输出。
- 使用`gdb` 14或更新版本（Rust、C、C++、Zig，以及实验性的MoonBit），macOS上也可用`lldb-dap`。

## 和智能体一起工作

全部都是项目`.idetea/`文件夹里的普通文件，所以任何能读写文件的智能体都能用。“界面: 教会本项目的智能体”会把说明写进`AGENTS.md`和`CLAUDE.md`。

- **界面**（`.idetea/ui/<name>.json`）：智能体写一份文档，IDE把它显示成表单（文本、Markdown、选择、复选框、输入、按钮、手工测试的步骤、要编辑的文件、图像、图库、动画精灵、带波形和峰值的声音、可选择行的表格），并把你的回答写到`.idetea/ui/responses/`。面板会显示智能体是否还在等。格式见[interfaces/formato.md](interfaces/formato.md)（葡萄牙语）。
- **智能体终端**（`.idetea/agents/<name>.json`）：智能体带一条命令来要终端；IDE在“终端请求”选项卡里显示完整请求，只有你允许之后才运行。进度写到`<name>.status.json`（`pending`、`running`、带退出码的`exited`、`refused`、`closed`、`invalid`、`lost`）；删掉请求就会停掉终端（如果命令已经结束，选项卡会留到你关掉它）。
- **允许列表：**用户配置里的`[agents] allow = ["npm run dev", "claude *"]`会直接运行这些命令，不再询问：要么是整条命令，要么是一个前缀，后面只能跟普通参数（不能有`;`、`&`、`|`、`$`、反引号、重定向……）。写在项目配置里会被忽略，并给出警告。
- **启动智能体**（`Ctrl+Shift+A`，欢迎页上也有）：PATH里找到的编程智能体（Claude Code、Codex、Gemini CLI、OpenCode、Aider、Cursor Agent），加上你自己的`[agents.launchers]`，在终端标签页里启动，位置可以是项目文件夹，也可以是新的工作树（`.idetea/worktrees/launch-<branch>`，关闭标签页时如果没有改动就会移除；分支保留）。`[agents] default`排在最前。启动器只在用户配置里生效。如果`AGENTS.md`/`CLAUDE.md`里没有这个IDE的说明，会先问：写入、只打开，或在这个项目里不再询问。
- **把上下文发给智能体**（编辑器里按`Ctrl+L`）：光标的位置（或选中内容，文本放在代码块里）以及这些行上的问题，会粘贴到屏幕上的智能体标签页，或最近用过的那个，不会直接发送。没有智能体标签页时，先打开启动器。在问题面板里右键可以发送单个问题。`idetea agent [名称] [--worktree <branch>]`从终端请求一个智能体标签页（`--list`列出可用的）。
- **项目记忆：**这些说明还会教一种放在git里的纯文件共享记忆：`docs/memory/`里的事实（每个文件一条，带一个智能体先读的索引），以及`docs/decisions/`里作为ADR的架构决策。任何读`AGENTS.md`/`CLAUDE.md`的智能体都能用。见[project-memory.md](project-memory.md)（英文）。
  **项目记录**面板列出事实和决定，可以搜索、打开和删除，并提醒过时的事实、已经不存在的路径和看起来像凭据的内容。在界面面板的历史里，**记录为决定**会把回答过的问题变成ADR。
- **助手用自己的工作树：**`"worktree": "<branch>"`会在该分支上的`.idetea/worktrees/<name>`里运行命令。每条命令都会得到`IDETEA_ANSWER`，这是主项目里用来放回答的文件。删掉请求时，如果工作树里没有未提交的内容，工作树会被移除；分支始终保留。
- **给智能体的命令：**`idetea ask`（在面板里提一个问题，并等待回答）、`idetea run`（一个终端，等待结束和助手的回答）、`idetea status`（你打开了什么、未保存的更改、光标）、`idetea open file:line`（给你看一个文件）、`idetea history`（你在面板里回答过什么）和`idetea mascot`（吉祥物的实时状态和对话气泡）。它们用一次调用完成上面的文件操作，批准方式相同。
- **会反应的吉祥物（Tea）：**右下角一张像素茶杯卡片（可以收进状态栏，也可以用命令面板开关）。它实时监视`.idetea/`（`idle`、`working`、`thinking`、`attention`、`celebrating`、`error`、`sleeping`，悬停、双击或戳一下时还有会互动的`curious`和`annoyed`）。每种情况有好几句短台词；同一句不会连续出现两次。它也会注意到文件保存、调试器构建完成或失败、长时间空闲（它会打盹，你回来就醒），以及一天里的时辰，包括夜里轻轻提醒你休息。点对话气泡会打开界面、终端或问题面板，并且从不抢走焦点。未保存的修改和磁盘变化顶上、git 冲突、终端任务结束、调试会话结束、编码写不下的字符，以及一次全部替换，它各说一句。警告和刚刚清零的错误只在 `chatty` 时出现。设置会写入同样的开关。`idetea mascot setup-hooks` 在项目里安装 Claude Code 的钩子，已有的钩子留着。`[mascot] tips = true`（并且`chatty`不是`quiet`）时，它偶尔会提起一条真实的快捷键。`[mascot] chatty`可以是`quiet`、`normal`或`chatty`。智能体或钩子也可以直接用`idetea mascot <state> [--text "…"] [--source <name>] [--ttl <secs>]`驱动它。自定义精灵包通过`.idetea/mascot/mascot.toml`或`~/.config/idetea/mascots/<pack>/mascot.toml`生效（`[mascot] enabled = true`）。
- **选择并复制文本：**界面顶部的“选择文本”（当前打开的，或历史里的）会在只读编辑器里显示它的文本：用鼠标选择，用Ctrl+C复制；“显示格式化结果”回到原来的样子。
- **问题的历史：**IDE保留每一个问题、你的回答、你的评论，以及智能体何时读到它，放在`.idetea/history/`（不进git，直到你清空）。左侧的**界面历史**面板（默认隐藏）先列出未回答的问题，再按天列出已回答的，可以搜索；一条记录以只读方式打开，就是你当时回答的样子。
- **修好这个项目**（“界面: 修复此项目”）：在崩溃或一次糟糕的编辑之后，把`.idetea/`恢复到能用的状态。**技能：**`idetea skills`把一个git仓库里的智能体技能安装到项目里（[skills.zh-CN.md](skills.zh-CN.md)）。

详细说明（暂时为葡萄牙语）：[terminais-de-agente.md](terminais-de-agente.md)。技能见[skills.zh-CN.md](skills.zh-CN.md)。

## 界面

- 命令面板列出全部命令及其快捷键（`Ctrl+Shift+P`）。
- 帮助（`F1`）列出当前全部快捷键，并有一份简短指南。
- 设置（`Ctrl+,`）：外观（包括吉祥物）、扩展、快捷键（录下你按下的键）和语言服务器。
- 活动栏用来显示和隐藏面板；面板可以用鼠标调整大小。
- 没有文件打开时的欢迎屏幕：新建文件、打开文件或文件夹、最近的文件和项目，以及主要快捷键。打字或`Ctrl+N`会回到一个未命名的编辑器。
- 面板是更深背景上的圆角小岛，细的胶囊滚动条在鼠标离开后淡出；`[ui] style = "classic"`让一切回到同一块背景上，用线条分开。面板、菜单和对话框轻轻滑入；`[ui] animations = false`关掉这个效果。
- 界面语言有葡萄牙语、英语、西班牙语、法语、德语、意大利语、日语和简体中文（`[ui] language`，或跟随系统）。重启IDE后生效。

## 不弄丢工作

- 关闭选项卡或退出时如果有未保存的更改，一定会先问。
- 删除文件会送到系统回收站。
- IDE外面改过的文件（终端、`cargo fmt`、另一个编辑器、`git checkout`）会被注意到：文件树和git会刷新，没有本地更改的选项卡会重新加载，有更改的选项卡会保留，并给出警告。

## 主要快捷键

| 快捷键 | 作用 |
|---|---|
| `Ctrl+P` | 按名称打开文件 |
| `Ctrl+Shift+P` | 命令面板 |
| `F1` | 帮助：全部快捷键和指南 |
| `Ctrl+,` | 设置 |
| `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S` | 打开 / 保存 / 另存为 |
| `Ctrl+B` | 显示/隐藏文件树 |
| `Ctrl+Shift+F` | 在项目里搜索 |
| `F4` / `Shift+F4` | 下一个 / 上一个搜索结果 |
| `` Ctrl+` `` / `Ctrl+'` | 终端 |
| `Ctrl+Shift+PgDn` / `Ctrl+Shift+PgUp` | 下一个 / 上一个终端选项卡 |
| `Ctrl+Shift+5` | 拆分终端 |
| `Ctrl+Shift+T` | 重新打开上次关闭的选项卡 |
| `Ctrl+Shift+G` | Git |
| `Ctrl+Shift+M` / `F8` | 问题 / 下一个问题 |
| `F12` | 转到定义 |

全部都可以在设置里改，或写在`~/.config/idetea/config.toml`里。

## 从源码构建

```sh
git clone https://github.com/fabiocmazzo/idetea && cd idetea
cargo build --release      # 可执行文件在target/release/idetea
```

装了[`just`](https://github.com/casey/just)的话，`just run`还会为桌面面板安装图标。在终端里，`idetea .`打开IDE并把提示符还给你；`idetea -f .`保持附着（并带上日志）。

## 路线图

- [x] 编辑器、选项卡、主题、命令面板、TOML配置、文件树、状态栏。
- [x] 项目搜索、LSP、问题面板、集成终端。
- [x] Git（包括推送/拉取）、右键菜单、帮助、设置、图标、可调整大小的面板。
- [x] 用gdb调试：断点、单步、调用堆栈和变量。
- [x] 智能体：界面、智能体终端、允许列表、工作树里的助手、带技能的新建项目。
- [ ] 第三方WASM插件，不用重新构建IDE（`plugins`特性后面已有一个原型）。
- [ ] 更多调试适配器（codelldb、debugpy、delve……）。
- [x] UTF-8以外的编码。
- [ ] 三方合并视图；在界面上stash。

分阶段的详细计划在[SPEC.md](../SPEC.md)（葡萄牙语）。
