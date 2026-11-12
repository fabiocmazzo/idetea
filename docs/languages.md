# Languages

[Français](languages.fr.md) · [Deutsch](languages.de.md) · [Italiano](languages.it.md) · [日本語](languages.ja.md) · [中文](languages.zh-CN.md)

What IDE TEA does for each language, and what it does not. The IDE integrates tools you install yourself
(compiler, language server, debugger); it never downloads them. Without them, files still open with colors
and editing works; only the parts that need the tool are missing, and the IDE says what to install.

For the full feature list, see [features.md](features.md).

## Overview

| Language | Files | Language server (default command) | Tasks (`Ctrl+Shift+B`) | Debug (`F5`) |
|---|---|---|---|---|
| Rust | `.rs` | `rust-analyzer` | cargo | gdb, builds with cargo |
| C / C++ | `.c` `.h` `.cc` `.cpp` `.cxx` `.hpp` `.hh` | `clangd` | Makefile targets | gdb, with a `[debug]` section |
| Python | `.py` `.pyi` | `pyright-langserver --stdio` | — | — |
| TypeScript / JavaScript | `.ts` `.tsx` `.js` `.jsx` | `typescript-language-server --stdio` | `package.json` scripts | — |
| Go | `.go` | `gopls` | — | — |
| Lua | `.lua` | `lua-language-server` | — | — |
| Zig | `.zig` `.zon` | `zls` | `build.zig` steps | gdb, builds with `zig build` |
| [Ziex](https://ziex.dev) | `.zx` `.mdzx` | `zx lsp` | `build.zig` steps | — |
| [Hare](#hare) | `.ha` | `hare-lsp -S` | Makefile targets | — |
| [MoonBit](#moonbit) | `.mbt` `.mbti` `.mbt.md` | `moon lsp` | `moon check`, `build`, `test`, `fmt --check` | gdb, builds with `moon build --target native -g` (experimental) |

What a language server offers (completion, rename, formatting...) depends on the server: the IDE uses what it
announces. Every language gets colors; Hare, MoonBit and Ziex bring their own grammar with the IDE.

## Hare

[Hare](https://harelang.org) is a small systems language. Its language server is young, and the IDE says so instead
of hiding it.

**Install**

- `hare` from your distribution (on Arch: `pacman -S hare`).
- `hare-lsp` does not ship with Hare. Build it from [its repository](https://git.sr.ht/~whynothugo/hare-lsp)
  (it needs the [`hare-json`](https://git.sr.ht/~sircmpwn/hare-json) library; on Arch it is in the AUR), then put
  the executable in your `PATH`, for example `~/.local/bin`:

  ```sh
  make build PREFIX=/usr          # PREFIX: where Hare's standard library is installed
  install -Dm755 hare-lsp -t ~/.local/bin/
  ```

**Checked on 2026-11-09** with Hare 0.26.0.1 and `hare-lsp` `e3db7b8`:

| | |
|---|---|
| Colors | yes |
| Hover | yes |
| Go to definition (`F12`), across modules | yes |
| Outline and symbols in the file (`Ctrl+Shift+O`) | yes |
| Syntax errors | yes |
| Type errors and other semantic checks | no (not in the server yet) |
| Completion, references, rename, formatting | no (not in the server yet) |
| Inlay hints | the server has them; the IDE does not show inlay hints yet |
| Tasks | your Makefile targets; a Makefile alone does not mean Hare, so the IDE adds no `hare` tasks. Add your own in `[tasks]` |

**Known problem:** Hare's own parser (`hare::parse`, used by `hare-lsp`) crashes on any statement missing its `;`,
such as `let x = 1` followed by another line. The IDE tells you the first time and brings the server back by
itself when you stop typing.

**Try it:** open [`examples/languages/hare`](../examples/languages/hare) and run `check` from the tasks.

## MoonBit

[MoonBit](https://www.moonbitlang.com) ships its language server with the toolchain, so there is nothing else to
install.

**Install:** the [official installer](https://www.moonbitlang.com/download). It puts `moon` in `~/.moon/bin` (or
`$MOON_HOME/bin`); the IDE looks there too, so it works even when opened from the app menu.

**Checked on 2026-11-09** with `moon 0.1.20261104`:

| | |
|---|---|
| Colors, including string interpolation | yes |
| `.mbti` interface files | yes |
| Hover | yes |
| Type errors | yes |
| Completion, go to definition, formatting through the server | whatever `moon lsp` offers; not checked one by one yet |
| Tasks | `moon check`, `moon build`, `moon test`, `moon fmt --check` |
| Debug (`F5`) | experimental: native backend with gdb, see below |
| `.mbt.md` (Markdown with MoonBit code) | colors in ` ```mbt ` and ` ```moonbit ` blocks, hover inside them; `moon test` runs the ` ```mbt check ` blocks |

Tasks run in the folder of the `moon.mod` closest to the open file, so each module of a workspace runs its own;
without an open file, the project root's `moon.mod` is used. The old `moon.mod.json` does not turn the tasks on:
MoonBit asks you to migrate it with `moon fmt`.

**Debug** (checked on 2026-11-10, Linux, gdb 18): with a `moon.mod` at the root of the open folder and no `[debug]`
build in the config, `F5` runs `moon build --target native -g` and debugs the program it built. Breakpoints in
`.mbt` files, stepping, the call stack with the right files and lines, and the values of numbers and other simple
variables work. What is still rough:

- Function names in the call stack are the compiled ones, such as `_M0FP26idetea7example7sum__to` for `sum_to`.
- Strings show as UTF-16 bytes (`"M\000o\000o\000n..."`).
- Right after a step, a variable may show as `<optimized out>`.
- Only the native backend: Wasm and JavaScript programs cannot be debugged from the IDE.

In `.mbt.md` files, Markdown around and MoonBit inside the code blocks; the Markdown preview works as for any
`.md`. The IDE uses a small Markdown grammar of its own there, so the rest of the text gets fewer colors than in a
plain `.md`.

**Try it:** open [`examples/languages/moonbit`](../examples/languages/moonbit), hover `sum_to` and run
`moon test` from the tasks; `sum.mbt.md` has a test inside its Markdown.

## A language that is not here

Three things in the config cover most languages, without changing the IDE:

```toml
# ~/.config/idetea/config.toml, or .idetea/config.toml in the project
[lsp.servers.mylang]
command = "mylang-lsp"
args = ["--stdio"]
languages = { ml = "mylang" }    # file extension = languageId sent to the server

[tasks]
"mylang test" = "mylang test"
```

For colors, put a `.sublime-syntax` grammar in `~/.config/idetea/syntaxes/`. A grammar there also replaces the
built-in one for the same extension, and `[lsp.servers.<name>]` can change or turn off (`enabled = false`) any
server in the table above.

If you use a language that other IDEs leave out and would like it built in, open an issue saying which project you
would try it on.
