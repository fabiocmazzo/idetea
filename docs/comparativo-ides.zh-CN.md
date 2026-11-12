# 诚实对比：IDE TEA与市面上的其他IDE

[Português](comparativo-ides.md) · [Français](comparativo-ides.fr.md) · [Deutsch](comparativo-ides.de.md) · [Italiano](comparativo-ides.it.md) · [日本語](comparativo-ides.ja.md) · **中文（简体）**

> 此后的版本也提供macOS（Apple Silicon）和Windows x86_64。下面表格里“目前只在Linux（Wayland）上测试过”仍是`v0.4.0`当时的记录。

本文档对**IDE TEA**（当前状态，`v0.4.0`）相对市面上主要IDE和编辑器做一次技术上直接、不加修饰的评估：

- **VS Code / VSCodium**（基于Electron的通用标准）
- **Cursor / Windsurf**（专注于AI的VS Code分支）
- **Zed**（用Rust写的原生IDE，专注速度、协作和集成AI）
- **JetBrains（RustRover、IntelliJ、PyCharm、CLion）**（偏重的语义IDE）
- **Neovim / Helix / Lapce**（精简的模态/原生编辑器）

> **本分析的范围：**焦点严格是技术——**稳定性、性能、编辑功能、LSP、调试、Git、生态，以及和智能体的集成**。不考虑团队规模，也不考虑谁在维护项目。

---

## 1. 执行摘要（直接对照）

| 维度 | IDE TEA | VS Code | Cursor / Windsurf | Zed | JetBrains | Helix / Neovim |
|---|---|---|---|---|---|---|
| **打开时间（冷启动）** | **优秀（<200ms）** | 慢（1–3秒） | 慢（1.5–4秒） | **优秀（<150ms）** | 很慢（5–20秒） | **即时（<50ms）** |
| **空闲内存** | **低（约60–150MB）** | 高（400MB–1.5GB） | 很高（600MB–2GB） | 低（约120–300MB） | 很高（1.5–4GB以上） | **极低（15–60MB）** |
| **巨大文件（50MB以上 / 压缩过的）** | **优秀（自动轻量模式）** | 中等（长行会卡） | 中等 | 好 | 超过20MB就吃力 | 取决于插件（Neovim） |
| **隐私 / 离线 / 零账号** | **完全（除git push/pull外零网络）** | 默认有遥测（VSCodium除外） | 需要账号和云 | AI和协作要账号 | 需要账号 / 许可证 | **完全** |
| **给智能体的结构化协议（声明式界面、命令批准、吉祥物）** | **同类里独一份（基于文件，原生）** | 没有（依赖聊天扩展） | 专注专有的聊天/差异 | 专注聊天/行内辅助 | 专注聊天/行内 | 没有（只有终端） |
| **行内AI补全（“Ghost text” / Tab）** | **没有（这是取舍）** | 通过Copilot | **优秀（领先）** | 很好（Zeta） | 好 | 通过插件 |
| **文本编辑器的成熟度** | 基础不错，缺高级功能 | **标杆** | **标杆** | 很高 | **标杆** | **标杆（模态）** |
| **LSP与重构的深度** | 中等（没有Code Actions和Inlay Hints） | **完整** | **完整** | 高 | **无可比拟（自有AST）** | 高 |
| **调试（Debug）** | 基础（Rust/C/C++/Zig用GDB 14+；MoonBit原生后端为实验性） | **完整（任何语言）** | **完整** | 基础/中等（DAP较新） | **优秀** | 基础（nvim-dap） |
| **版本控制（Git）** | 基础/中等（没有装订线差异，也没有三方合并） | **完整** | **完整** | 中等 | **优秀（合并最好）** | 通过LazyGit / Fugitive |
| **多平台支持** | **目前只在Linux（Wayland）上测试过** | **Linux、macOS、Windows、Web** | **Linux、macOS、Windows** | **macOS、Linux、Windows** | **Linux、macOS、Windows** | **任何操作系统 / SSH** |

---

## 2. 稳定性与可靠性

### IDE TEA真正强的地方
1. **没有WebView / Chromium的架构：**和VS Code、Cursor、Windsurf不同，这里没有Chromium渲染进程，也没有几十个Node.js *extension host*在一天里慢慢漏内存。二进制是单一的，用安全的Rust编译。
2. **扩展和外部进程的故障隔离：**
   - 语言服务器（`tea-lsp`）、调试器（`tea-dap` / `gdb`）、`git`调用和智能体跑在各自的进程里。`rust-analyzer`或`gdb`卡住或退出，IDE窗口不会冻住。
   - 内部扩展只通过消息和事件（`Request` / `CoreEvent`）和内核通信，不共享应用状态的可变引用。
3. **防止丢掉工作：**
   - 关闭选项卡或退出IDE时，只要有未保存的更改，一定会要求确认。
   - **故障自动恢复（`backup.rs`）：**有未保存更改的选项卡按PID写到`$XDG_STATE_HOME/idetea/backups/<项目>/`，窗口意外关闭时可以恢复。
   - 在文件树里删除文件会送到**系统回收站**（不是不可逆的`unlink`）。
   - 磁盘上的外部更改（`git checkout`、`cargo fmt`、智能体在改文件）由监视器发现：没有本地编辑的选项卡自己重新加载；有未保存本地编辑的选项卡**绝不会**被悄悄覆盖。
   - 二进制文件或非UTF-8文件会被拒绝，并给出明确警告，而不是在保存时弄坏缓冲区。
   - `.idetea/`协议使用原子写入（`.tmp` + `rename`），并有自恢复命令（`idetea fix` / “界面: 修复此项目”）。

### IDE TEA的稳定性还输给大产品的地方
1. **只在Linux（Wayland）上测试过：**今天IDE TEA是在Linux上开发和验证的。macOS和Windows在打包、字体、快捷键（`Cmd`对`Ctrl`）、终端PTY（Windows上的`conpty`）和路径上有真实差异，这些还没有经过实战。VS Code、JetBrains和Zed在三个系统上都跑得很成熟。
2. **`iced` 0.14对复杂应用的限制：**
   - IDE跑在单窗口里（`iced::application`）。不能把终端、预览或编辑器拖到第二台显示器上的另一个操作系统原生窗口。
   - 没有按文件连续保存的本地历史（没有Git时跨越多天的*Local History* / *Timeline*，像JetBrains和VS Code那样）。
3. **基于正则的语法高亮（`syntect`）对比`tree-sitter`：**
   - IDE TEA对50MB文件或压缩后的超长行有很好的轻量模式，但在几千行的普通代码文件上，`syntect`（基于带正则的`.sublime-syntax`语法）在结构上不如Zed、Helix和Neovim用的`tree-sitter`那样扛得住不完整的代码。

---

## 3. 和AI智能体一起工作（IDE TEA独特的地方，以及它不竞争的地方）

这是今天市场上产品哲学差别最深的一点。

### Cursor、Windsurf和Zed怎么对待AI
- 它们为**模型待在编辑器里面**而设计：
  - *预测补全（Tab / Next Edit Prediction）*在你打字时猜测接下来几行。
  - *行内编辑（`Ctrl+K`）*和*Composer/Agent Chat*直接在编辑器的行与行之间显示彩色差异（绿/红），让你逐个hunk接受或拒绝。
  - 作为交换，它们把你绑到云账号、按月订阅，或IDE里面的提供商HTTP客户端上。

### IDE TEA怎么对待AI（以及它赢在哪里）
IDE TEA认定**最好的智能体已经在终端里跑**（Claude Code、Codex、Gemini CLI、Aider、OpenCode），并解决一个还没有别的IDE用原生、开放的方式解决的问题：**这些智能体怎样和人说话，而不变成终端里的一堵文字墙。**

1. **原生的声明式界面（`.idetea/ui/` + `idetea ask`）：**
   - 智能体不必在终端200行日志中间发问。它发布一份经过校验的JSON，IDE用`iced`渲染原生表单：选择器、复选框、手工测试的步骤列表（*批准*按钮只有在步骤做完之后才启用）、图像/图库预览、动画spritesheet，以及带波形的WAV音频。
   - **市面上没有别的IDE（VS Code、Cursor、Zed、JetBrains）拥有一套基于文件的声明式协议，让终端里的智能体画出带可审计历史（`.idetea/history/`）的原生UI。**
2. **执行的治理与安全（`idetea run`）：**
   - 当智能体想打开一个交互式终端或委派任务时，IDE在一个批准选项卡里显示确切的命令。允许的命令列表（`[agents] allow`）只从用户的全局配置（`~/.config/idetea/config.toml`）读取，**绝不**从仓库读取——这样恶意仓库就不能自己给自己执行权限。
3. **在隔离的Git工作树里委派（`--worktree` + `$IDETEA_ANSWER`）：**
   - IDE原生管理工作树的创建和清理，让子智能体并行工作，又不弄脏主工作树。
4. **原生的、会反应的吉祥物（Tea）+ `idetea mascot`：**
   - *Masko.ai*这类工具要有一个会回应Claude Code的吉祥物，需要macOS加上云端的MCP服务器。IDE TEA把**Tea**原生嵌进来，实时观察`.idetea/ui/`、`.idetea/agents/`、任务和LSP，也会对鼠标做出反应（`curious` / `annoyed`），并接受`mascot.toml`包。

### IDE TEA在AI流程上输的地方
- **零预测补全（“Ghost Text” / Tab）：**如果你靠Cursor/Copilot的`Tab`在手工打字时补上样板代码，IDE TEA没有对应物。
- **编辑器里没有AI的行内差异：**终端里的智能体改了一个文件之后，文件会在IDE TEA的选项卡里重新加载。要看改了什么，你得去**Git**面板；编辑器里面没有飘着的*“Accept / Reject hunk”*按钮，不像Cursor或Windsurf。
- **把编辑器上下文交给智能体仍然是手工的：**智能体可以运行`idetea status`来得知当前文件和光标所在行，但IDE里还没有一个快捷键（例如`Ctrl+L`）把选中的片段或一条LSP错误直接送进终端里已打开的智能体提示词。

---

## 4. 代码编辑器：有什么，缺什么

### IDE TEA里已经扎实的部分
- 快速打开文件（`Ctrl+P`，模糊搜索，以及`文件:行`跳转）、文件内符号（`Ctrl+Shift+O`）和项目内符号（`Ctrl+T`）。
- 多光标、按词选择（`Ctrl+方向键`）、`Shift+单击`、代码折叠、撤消/重做、文件内查找和替换，以及**在整个项目里替换（`Ctrl+Shift+H`）**，带逐行预览和正则。
- 编辑器临时缩放（`Ctrl+=`、`Ctrl+-`、`Ctrl+0`、`Ctrl+滚轮`），在选项卡和分屏之间同步，不影响界面的其余部分。
- 编辑器左右分屏，可单击的痕迹（`项目 › 文件夹 › 文件 › 符号`），按语言的Seti图标，以及可自定义、带连字的字体（内置`JetBrains Mono`）。
- 可选的vim模式。
- 图像的原生选项卡（可缩放、可拖动）、PDF（由纯Rust的`hayro`渲染），以及Markdown和SVG的实时并排预览。

### 相对VS Code、Zed和JetBrains，编辑器还缺的
1. **代码片段：**没有片段引擎（Tab停靠点`$1`、`$2`）来展开一小块常用代码。
2. **按AST的结构化选择（Expand/Shrink Selection）：**在Zed、Helix和JetBrains里，一个快捷键可以把选区从词 → 表达式 → `if`块 → 整个函数，靠`tree-sitter`/AST扩大。
3. **旧编码：**只打开UTF-8（保留LF/CRLF）。用`ISO-8859-1` / `Windows-1252`文件的人没法在IDE TEA里编辑它们。
4. **同一实例里的多窗口 / 多项目：**每个项目是一个单独的`idetea`进程；同一个窗口里没有“Multi-root Workspaces”。

---

## 5. 语言智能（LSP）与重构

### IDE TEA已经做得很好的
- 实时诊断：**编辑器文本上的波浪下划线**、悬停时的错误提示、状态栏计数，以及**问题**面板（`Ctrl+Shift+M` / `F8`）。
- 自动补全、参数签名、转到定义（`F12`）、转到类型和转到实现（`Ctrl+F12`）。
- 查找引用（`Shift+F12`，或在声明上`Ctrl+单击`），按文件分组，并有后退/前进导航历史（`Alt+←` / `Alt+→`，以及鼠标侧键）。
- **调用层次结构（`Alt+Shift+H`）：**谁调用这个函数，它又调用了什么。
- **结构 / 大纲面板（`Alt+7`）：**文件的符号树，标出可见性（`pub`、`pub(crate)`、私有），并高亮光标下的符号。
- **重命名符号（`F2`）**作用于整个项目，应用之前有预览对话框，说明会影响到多少文件和多少处更改。
- 保存时格式化，并遵守`.editorconfig`。

### 相对VS Code、Zed和JetBrains，LSP上的重要缺口
1. **没有Code Actions / 快速修复（`Ctrl+.` / LSP的灯泡）：**
   - 这是今天用Rust/TypeScript/Go时**日常最缺的一项**：当`rust-analyzer`建议*“Import `std::collections::HashMap`”*、*“Add missing match arms”*或*“Fill struct fields”*时，在IDE TEA里你看得到诊断信息，但不能用快捷键应用*Quick Fix*（`textDocument/codeAction`）。
2. **没有Inlay Hints：**
   - 不显示行内推断类型（`let x: Vec<String> = ...`），也不显示函数调用里的参数名。这是VS Code、Zed和RustRover里写Rust和TypeScript时很常用的功能。
3. **没有LSP的Semantic Tokens：**
   - 语法颜色只来自`.sublime-syntax`语法（`syntect`），没有LSP的语义增强（例如按语言服务器的颜色区分可变变量和不可变变量，或区分宏和函数）。

---

## 6. 调试（Debug）

### IDE TEA今天做的
- 与`gdb` 14+原生集成（经由DAP）：在行号上单击设置断点（`F9`），编辑文件时断点跟着文本走；`F5`（在Rust、Zig项目里自动编译并运行，MoonBit为实验性）；逐过程（`F10`）、逐语句（`F11`）、逐出（`Shift+F11`）、暂停和停止。
- 面板里有调用堆栈（单击跳到该帧的那一行）、可检查的局部变量树，以及程序输出。

### 输给VS Code和JetBrains的地方
- **绑在GDB 14+上（Rust、C、C++、Zig，以及实验性的MoonBit）：**还不支持Python（`debugpy`）、Go（`delve`）、JavaScript/TypeScript（`js-debug`）的DAP适配器，也不支持`codelldb`（macOS）。Hare在IDE里没有调试器。
- **缺少的高级调试功能：**没有条件断点（*“只在`i == 42`时停下”*）、logpoint、任意*Watch Expressions*窗口（断点期间的REPL），也没有画在编辑器代码旁边的行内变量值。

---

## 7. Git与版本控制

### IDE TEA今天做的
- 状态栏显示当前分支、领先/落后的提交（`ahead/behind`），以及已修改文件的总数。
- Git面板可以按文件或全部暂存/取消暂存，查看带颜色的差异，有提交消息框，放弃更改时会确认，以及用系统的`git`和凭据/SSH进行推送、拉取和获取。
- 文件树上的Git状态颜色。

### 输给VS Code、Zed和JetBrains的地方
1. **编辑器装订线上没有Git标记（装订线差异）：**你编辑文件时，行号旁边没有那些绿/蓝/红的小条，用来表示自上次提交以来哪些行被添加或改过。
2. **没有行内Git Blame：**不显示当前行是谁改的、在哪一次提交里。
3. **没有可视化的合并冲突解决（三方合并）：**遇到冲突时，你在文本里编辑`<<<<<<<`标记，或用终端解决。
4. **界面上不能切换/创建分支，也不能Stash：**要切换分支、创建分支或执行`git stash`，用集成终端。

---

## 8. 诚实的小结：今天什么时候选IDE TEA（以及什么时候不选）

### 下面这些时候，IDE TEA已经更好，或是独一份
1. **你在Linux上、旁边跑着终端智能体（Claude Code、Codex、Gemini CLI等）写程序**，并且希望智能体提出结构化的问题、展示计划/手工测试、运行命令前明确请求许可、使用隔离的工作树，并有一个会反应的吉祥物（**Tea**），而不依赖云或订阅。
2. **你把速度、可预测和低感官噪声放在前面：**不到200ms就打开，内存占用低，没有账号，没有遥测，不会自己改UI，并且尊重简单的TOML文件。
3. **你做的是Rust、Zig、C/C++、Python、Go、Hare、MoonBit或Web项目**，主流程是快速编辑、按LSP的符号/引用导航、在集成终端里跑任务/测试，以及审查提交。

### 下面这些时候，另一款IDE仍然更好
1. **你想要每按一个键都有AI预测补全（`Tab` / Ghost Text），或按差异做行内编辑：** **Cursor**、**Windsurf**或**Zed**更适合这种写法。
2. **你很依赖*Code Actions*（`Ctrl+.`）辅助的重构、*Inlay Hints*，或GUI里的Python/JS/Go调试器：**今天的**VS Code**或**JetBrains**完整得多。
3. **你在macOS或Windows上工作，或依赖SSH Remote Development / DevContainers：** **VS Code**和**Zed**对这些场景已经有成熟支持。

---

## 9. 影响最大的接下来5步（靠近领先者，同时不丢掉IDE TEA的灵魂）

如果目标是缩小和VS Code、Zed的功能差距，同时保持二进制轻、宣言里的哲学还在：

1. **LSP的Code Actions / 快速修复（`Ctrl+.`）：**让人能应用语言服务器已经算得出来的*quick fix*和自动*import*。这是写Rust/TS时最缺的第1项。
2. **编辑器里的Git标记（装订线差异）+ 在状态栏切换分支：**在编辑器装订线显示添加/修改的行，并允许单击状态栏上的分支名，在命令面板里切换分支。
3. **“把选区 / 诊断发给智能体”的快捷键（`Ctrl+L`）：**把编辑器/LSP和集成终端里正在跑的智能体接成一个环。
4. **LSP的Inlay Hints（推断类型，以及可选的参数名）：**显示行内类型提示，并有一个快捷键快速开关。
5. **移植到macOS和Windows + 更多DAP适配器（`lldb-dap`、`debugpy`、`delve`）：**让Linux以外的开发者也能用这个IDE，并支持GDB以外的调试。
