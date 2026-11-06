# IDE TEA: features and shortcuts

The full list, for the technical reader. For the short version, see the [README](../README.md).

> Before 1.0. Built and tested on Linux (Wayland). macOS and Windows have not been tested yet.

## Editor

- Open a file by name (`Ctrl+P`) with fuzzy search; `main.rs:42` jumps straight to the line.
- Go to a symbol (function, type, method…) in the file (`Ctrl+Shift+O`) or in the project (`Ctrl+T`).
- Tabs with a modified marker; open (`Ctrl+O`), save and save as, with the system file dialog.
- Syntax highlighting for about 200 languages and file types (TOML, `.env`, INI, Dockerfile, TypeScript…); light
  and dark themes (plus iced's themes: Nord, Dracula…).
- Find and replace, go to line, undo/redo, multi-cursor, code folding and an optional vim mode.
- Markdown and SVG preview next to the editor (`Ctrl+Shift+V` or the button on the tab bar), updated as you type.
- Images (png, jpg, gif, webp, bmp, ico…) open in a tab, with mouse-wheel zoom and drag.
- PDF opens in a tab (rendered by `hayro`, pure Rust), with zoom and fit to width.
- Keeps CRLF. A file that is not UTF-8 is not opened (with a warning) instead of being corrupted.
- Big files: a 50 MB file opens in under 0.1 s. A file with one giant line (minified JSON or JS) opens in a light
  mode, without colors and with wrapping, so the window does not freeze.
- Optional on save: trim trailing whitespace, final newline, format with the language server. An `.editorconfig`
  in the project wins.

## Project

- **File tree:** create, rename and delete (to the trash), reveal the current file, git colors (modified, new,
  conflict, ignored).
- **Project search** as you type, respecting `.gitignore`, with regex, whole word and match case.
- **Right-click menus** on the tree, the tabs and the files in the git panel.
- **Tasks** (`Ctrl+Shift+B`): your own commands from the config, plus what the IDE finds in the project (cargo,
  `package.json` scripts, Makefile targets, `build.zig` steps), with the output in the terminal.
- **New project** ("Project: new…" or `idetea new <folder>`): folder, git, the agent skills of a template (empty,
  Rust, website, Python, FastAPI) and an initial request in `AGENTS.md`. See [skills.md](skills.md).

## Languages (LSP)

- Errors and warnings (problems panel and a count on the status bar), hover, completion, go to definition, format.
- Go to implementation (`Ctrl+F12`) and to type; with several results, the list goes to the panel.
- Find references (`Shift+F12`, or `Ctrl+click` on the declaration) grouped by file, and back/forward after each
  jump (`Alt+←` / `Alt+→`, or the mouse side buttons).
- Call hierarchy (`Alt+Shift+H`): who calls a function, and what it calls, as a tree that loads as you open it.
- Outline panel (`Alt+7`): functions, structs, methods and fields of the file, showing what is public, private or
  restricted (`pub(crate)`, `protected`…), with the symbol under the cursor highlighted.
- Breadcrumbs above the editor: `project › folder › file › impl App › fn update`, the path and the symbols where the
  cursor is. A folder shows the file in the tree; a symbol jumps to it. `[editor] breadcrumbs = false` turns them off.
- Code font: JetBrains Mono comes built in; any installed font works (`[editor] font = "Fira Code"`), with its
  ligatures. Weight and line height too (`font_weight`, `line_height`), also in Settings → Appearance. The editor,
  its line numbers and the terminal use the same font.
- File icons by language and type (the Seti set from VS Code, built in) in the file tree, tabs, Ctrl+P, breadcrumbs,
  search results and the git panel. `[ui] file_icons = "none"` (or Settings → Appearance) goes back to one icon for
  every file.
- Shift+click extends the selection from the last click (or from the cursor, after moving it with the keyboard).
- Word by word: `Ctrl+Left/Right` moves (with `Shift`, selects), `Ctrl+Backspace/Delete` deletes. `Home` goes to the
  first character of the line, then to column 0. `Shift+Tab` removes indentation.
- VS Code keys where they used to differ: fold/unfold the block at the cursor with `Ctrl+Shift+[` / `]`, reopen the
  last closed tab with `Ctrl+Shift+T`, bottom panel with `Ctrl+J`, terminal with `` Ctrl+` `` or `Ctrl+'`.
- Already set up: rust-analyzer, clangd, pyright, typescript-language-server, gopls, lua-language-server, zls (Zig)
  and `zx lsp` ([Ziex](https://ziex.dev); without the `zx` CLI, a Ziex project builds its own server with
  `zig build`). Just install the server; you can change the command or add others in the config.

## Terminal

- Integrated terminal (alacritty) in the bottom panel, with the theme's colors and font.
- Several shells in tabs ("Terminal 1", "Terminal 2"…), plus the task's own tab; `+` opens one, `×` closes it,
  `exit` closes the tab.
- Shell shortcuts (`Ctrl+C`, `Ctrl+W`…) go to the shell; the IDE's `Ctrl+Shift` shortcuts keep working.

## Git

- Branch, commits ahead/behind and number of changes on the status bar.
- Panel with stage/unstage per file or all, colored diff, commit, discard changes (with confirmation).
- Push, pull and fetch. The IDE never waits for a password: it uses your ssh agent or your git credential helper.
- Uses the installed `git`, so your config, hooks and credentials apply.

## Debug

- Breakpoints with a click on the line number (`F9`); they move with the text as you edit.
- `F5` builds and runs: in Rust projects it uses `cargo build` and finds the executable; in Zig projects,
  `zig build` and the executables in `zig-out/bin`; elsewhere, a `[debug]`
  section in the config says the program, the arguments and the build command.
- Step over, into and out (`F10`, `F11`, `Shift+F11`), pause and stop, with the current line marked in the editor.
- Panel with the call stack (click to go to the frame), variables as a tree, and the program's output.
- Uses `gdb` 14 or newer (Rust, C and C++).

## Working with agents

All of it is plain files in the project's `.idetea/` folder, so any agent that reads and writes files can use it.
"Interfaces: teach this project's agents" writes the instructions into `AGENTS.md` and `CLAUDE.md`.

- **Interfaces panel** (`.idetea/ui/<name>.json`): the agent writes a document, the IDE shows it as a form (text,
  Markdown, select, checkbox, input, buttons, steps for manual tests, files to edit, images, galleries, animated
  sprites, sounds with waveform and peak level) and writes your answer to `.idetea/ui/responses/`. The panel shows
  whether the agent is still waiting. Format: [docs/interfaces/formato.md](interfaces/formato.md).
- **Agent terminals** (`.idetea/agents/<name>.json`): the agent asks for a terminal with a command; the IDE shows
  the whole request in a "terminal request" tab and runs it only after you allow it. Progress goes to
  `<name>.status.json` (`pending`, `running`, `exited` with the code, `refused`, `closed`, `invalid`, `lost`);
  deleting the request stops the terminal (if the command already ended, the tab stays until you close it).
- **Allow list:** `[agents] allow = ["npm run dev", "claude *"]` in your user config runs those commands without
  asking: the exact command, or a prefix followed only by plain arguments (no `;`, `&`, `|`, `$`, backticks,
  redirections…). In a project config it is ignored, with a warning.
- **Helpers in their own worktree:** `"worktree": "<branch>"` runs the command in `.idetea/worktrees/<name>` on that
  branch. Every command gets `IDETEA_ANSWER`, a file in the main project for the answer. Deleting the request
  removes the worktree if it has nothing uncommitted; the branch always stays.
- **Commands for agents:** `idetea ask` (a question in the panel, waiting for the answer), `idetea run` (a terminal,
  waiting for the end and the helper's answer), `idetea status` (what you have open, unsaved changes, the cursor) ,
  `idetea open file:line` (shows you a file) and `idetea history` (what you answered in the panel). They do the file work above in one call, with the same approvals.
- **Select and copy text:** "Select text" at the top of an interface (open or in the history) shows its texts in a
  read-only editor: select with the mouse, copy with Ctrl+C; "Show formatted" goes back.
- **History of the questions:** the IDE keeps every question, your answer, your comment and when the agent read
  it, in `.idetea/history/` (out of git, until you clear it). The **Interfaces history** panel on the left (hidden
  by default) lists the open questions first, then the answered ones by day, with search; an entry opens read-only,
  as you answered it.
- **Fix this project** ("Interfaces: fix this project"): puts `.idetea/` back in a usable state after a crash or a
  bad edit. **Skills:** `idetea skills` installs agent skills from a git repository into the project
  ([skills.md](skills.md)).

Details (in Portuguese for now): [terminais-de-agente.md](terminais-de-agente.md), [skills.md](skills.md).

## Interface

- Command palette with every command and its shortcut (`Ctrl+Shift+P`).
- Help (`F1`) with all current shortcuts and a short guide.
- Settings (`Ctrl+,`): appearance, extensions, shortcuts (records the key you press) and language servers.
- Activity bar to show and hide panels; panels resize with the mouse.
- Interface in English, Portuguese and Spanish (`[ui] language`, or the system's).

## Not losing work

- Closing a tab or quitting with unsaved changes always asks first.
- Deleting a file sends it to the system trash.
- Files changed outside the IDE (terminal, `cargo fmt`, another editor, `git checkout`) are noticed: the tree and
  git refresh, tabs without changes reload, and tabs with changes are kept, with a warning.

## Main shortcuts

| Shortcut | What |
|---|---|
| `Ctrl+P` | Open a file by name |
| `Ctrl+Shift+P` | Command palette |
| `F1` | Help: every shortcut and the guide |
| `Ctrl+,` | Settings |
| `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S` | Open / save / save as |
| `Ctrl+B` | Show/hide the file tree |
| `Ctrl+Shift+F` | Search in the project |
| `` Ctrl+` `` / `Ctrl+'` | Terminal |
| `Ctrl+Shift+T` | Reopen the last closed tab |
| `Ctrl+Shift+G` | Git |
| `Ctrl+Shift+M` / `F8` | Problems / next problem |
| `F12` | Go to definition |

All of them can be changed in the settings or in `~/.config/idetea/config.toml`.

## Building from source

```sh
git clone https://github.com/fabiocmazzo/idetea && cd idetea
cargo build --release      # executable in target/release/idetea
```

With [`just`](https://github.com/casey/just), `just run` also installs the icon for your desktop panel. In a
terminal, `idetea .` opens the IDE and gives the prompt back; `idetea -f .` stays attached (with the logs).

## Roadmap

- [x] Editor, tabs, themes, palette, TOML config, file tree, status bar.
- [x] Project search, LSP, problems panel, integrated terminal.
- [x] Git (including push/pull), context menus, help, settings, icons, resizable panels.
- [x] Debug with gdb: breakpoints, stepping, call stack and variables.
- [x] Agents: Interfaces panel, agent terminals, allow list, helpers in worktrees, new project with skills.
- [ ] Third-party plugins in WASM, without rebuilding the IDE (a prototype exists behind the `plugins` feature).
- [ ] More debug adapters (codelldb, debugpy, delve…).
- [ ] Error underlines and git marks inside the editor; encodings other than UTF-8.

The detailed plan, phase by phase, is in [SPEC.md](../SPEC.md) (in Portuguese).
