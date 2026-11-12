# 语言

[English](languages.md) · [Français](languages.fr.md) · [Deutsch](languages.de.md) · [Italiano](languages.it.md) · [日本語](languages.ja.md) · **中文（简体）**

IDE TEA对每种语言做了什么，以及没做什么。IDE集成的是你自己安装的工具（编译器、语言服务器、调试器）；它从不替你下载。没有这些工具时，文件仍然带着颜色打开，编辑也正常；只是依赖工具的那部分会缺，IDE会告诉你该装什么。

完整功能清单见[features.zh-CN.md](features.zh-CN.md)。

## 概览

| 语言 | 文件 | 语言服务器（默认命令） | 任务（`Ctrl+Shift+B`） | 调试（`F5`） |
|---|---|---|---|---|
| Rust | `.rs` | `rust-analyzer` | cargo | gdb，用cargo构建 |
| C / C++ | `.c` `.h` `.cc` `.cpp` `.cxx` `.hpp` `.hh` | `clangd` | Makefile目标 | gdb，需要一段`[debug]` |
| Python | `.py` `.pyi` | `pyright-langserver --stdio` | — | — |
| TypeScript / JavaScript | `.ts` `.tsx` `.js` `.jsx` | `typescript-language-server --stdio` | `package.json`脚本 | — |
| Go | `.go` | `gopls` | — | — |
| Lua | `.lua` | `lua-language-server` | — | — |
| Zig | `.zig` `.zon` | `zls` | `build.zig`步骤 | gdb，用`zig build`构建 |
| [Ziex](https://ziex.dev) | `.zx` `.mdzx` | `zx lsp` | `build.zig`步骤 | — |
| [Hare](#hare) | `.ha` | `hare-lsp -S` | Makefile目标 | — |
| [MoonBit](#moonbit) | `.mbt` `.mbti` `.mbt.md` | `moon lsp` | `moon check`、`build`、`test`、`fmt --check` | gdb，用`moon build --target native -g`构建（实验性） |

语言服务器能提供什么（补全、重命名、格式化……）取决于服务器：IDE用它自己声明的能力。每种语言都有颜色；Hare、MoonBit和Ziex的语法随IDE一起带来。

## Hare

[Hare](https://harelang.org)是一门很小的系统语言。它的语言服务器还年轻，IDE会把这一点说出来，而不是藏起来。

**安装**

- 从你的发行版安装`hare`（Arch上：`pacman -S hare`）。
- `hare-lsp`不随Hare一起发布。从[它的仓库](https://git.sr.ht/~whynothugo/hare-lsp)构建（需要[`hare-json`](https://git.sr.ht/~sircmpwn/hare-json)库；Arch上它在AUR里），然后把可执行文件放进`PATH`，例如`~/.local/bin`：

  ```sh
  make build PREFIX=/usr          # PREFIX：Hare标准库的安装位置
  install -Dm755 hare-lsp -t ~/.local/bin/
  ```

**核对日期：2026-11-09**，Hare 0.26.0.1，`hare-lsp`为`e3db7b8`：

| | |
|---|---|
| 颜色 | 有 |
| 悬停 | 有 |
| 转到定义（`F12`），跨模块 | 有 |
| 大纲和文件内符号（`Ctrl+Shift+O`） | 有 |
| 语法错误 | 有 |
| 类型错误和其他语义检查 | 没有（服务器里还没有） |
| 补全、引用、重命名、格式化 | 没有（服务器里还没有） |
| 内联提示 | 服务器有；IDE还不会显示内联提示 |
| 任务 | 你的Makefile目标；只有一个Makefile并不等于这是Hare，所以IDE不会添加`hare`任务。在`[tasks]`里自己加 |

**已知问题：**Hare自己的解析器（`hare::parse`，`hare-lsp`在用）会在任何缺少`;`的语句上崩溃，例如`let x = 1`后面再跟一行。IDE会在第一次告诉你，并在你停止输入后自己把服务器拉起来。

**试一试：**打开[`examples/languages/hare`](../examples/languages/hare)，从任务里运行`check`。

## MoonBit

[MoonBit](https://www.moonbitlang.com)的语言服务器随工具链一起发布，所以没有别的东西要装。

**安装：**[官方安装程序](https://www.moonbitlang.com/download)。它把`moon`放在`~/.moon/bin`（或`$MOON_HOME/bin`）；IDE也会到那里找，所以从应用菜单打开也能用。

**核对日期：2026-11-09**，`moon 0.1.20261104`：

| | |
|---|---|
| 颜色，包括字符串插值 | 有 |
| `.mbti`接口文件 | 有 |
| 悬停 | 有 |
| 类型错误 | 有 |
| 通过服务器的补全、转到定义、格式化 | `moon lsp`提供什么就是什么；还没有逐项核对 |
| 任务 | `moon check`、`moon build`、`moon test`、`moon fmt --check` |
| 调试（`F5`） | 实验性：原生后端加gdb，见下文 |
| `.mbt.md`（带MoonBit代码的Markdown） | ` ```mbt `和` ```moonbit `代码块里有颜色，块内可以悬停；`moon test`会运行` ```mbt check `代码块 |

任务在离打开文件最近的`moon.mod`所在文件夹里运行，所以工作区里的每个模块跑自己的任务；没有打开文件时，用项目根目录的`moon.mod`。旧的`moon.mod.json`不会打开这些任务：MoonBit要求你用`moon fmt`迁移它。

**调试**（核对日期：2026-11-10，Linux，gdb 18）：打开的文件夹根目录有`moon.mod`，并且配置里没有`[debug]`构建时，`F5`会运行`moon build --target native -g`，并调试它构建出来的程序。`.mbt`文件里的断点、单步、带正确文件和行号的调用堆栈，以及数字和其他简单变量的值，都可以用。还粗糙的地方：

- 调用堆栈里的函数名是编译后的名字，例如`sum_to`会显示成`_M0FP26idetea7example7sum__to`。
- 字符串显示成UTF-16字节（`"M\000o\000o\000n..."`）。
- 刚单步之后，一个变量可能显示为`<optimized out>`。
- 只有原生后端：Wasm和JavaScript程序不能从IDE调试。

在`.mbt.md`文件里，代码块外面是Markdown，里面是MoonBit；Markdown预览和普通`.md`一样。IDE在这里用的是一份自己的小型Markdown语法，所以其余文本的颜色比普通`.md`少。

**试一试：**打开[`examples/languages/moonbit`](../examples/languages/moonbit)，悬停`sum_to`，并从任务里运行`moon test`；`sum.mbt.md`的Markdown里有一个测试。

## 这里还没有的语言

配置里的三处就覆盖了大多数语言，不用改IDE：

```toml
# ~/.config/idetea/config.toml，或项目里的.idetea/config.toml
[lsp.servers.mylang]
command = "mylang-lsp"
args = ["--stdio"]
languages = { ml = "mylang" }    # 文件扩展名 = 发给服务器的languageId

[tasks]
"mylang test" = "mylang test"
```

要颜色的话，把一份`.sublime-syntax`语法放进`~/.config/idetea/syntaxes/`。那里的语法也会替换同一扩展名的内置语法，而`[lsp.servers.<name>]`可以改掉或关掉（`enabled = false`）上表中的任何服务器。

如果你用的语言是其他IDE放过的，并且希望它内置进来，开一个issue，写上你打算在哪个项目上试。
