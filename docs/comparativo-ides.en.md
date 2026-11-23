# An Honest Comparison: IDE TEA vs. Other IDEs on the Market

<p><a href="comparativo-ides.md">Português</a> · <b>English</b> · <a href="comparativo-ides.fr.md">Français</a> · <a href="comparativo-ides.de.md">Deutsch</a> ·
<a href="comparativo-ides.it.md">Italiano</a> · <a href="comparativo-ides.ja.md">日本語</a> · <a href="comparativo-ides.zh-CN.md">中文（简体）</a></p>

This document gives a technical, direct and unvarnished assessment of **IDE TEA** (current state, `v0.15.0`) against the main IDEs and editors on the market:

- **VS Code / VSCodium** (the Electron-based universal default)
- **Cursor / Windsurf** (AI-focused forks of VS Code)
- **Zed** (native Rust IDE focused on speed, collaboration and built-in AI)
- **JetBrains (RustRover, IntelliJ, PyCharm, CLion)** (heavyweight semantic IDEs)
- **Neovim / Helix / Lapce** (lean modal/native editors)

> **Scope of this analysis:** the focus is strictly technical — **stability, performance, editing features, LSP, debugging, Git, ecosystem and agent integration**. It does not take into account team size or who maintains the project.

---

## 1. Executive Overview (At-a-Glance Matrix)

| Dimension | IDE TEA | VS Code | Cursor / Windsurf | Zed | JetBrains | Helix / Neovim |
|---|---|---|---|---|---|---|
| **Cold start time** | **Excellent (< 200 ms)** | Slow (1–3 s) | Slow (1.5–4 s) | **Excellent (< 150 ms)** | Very slow (5–20 s) | **Instant (< 50 ms)** |
| **Idle RAM usage** | **Low (~60–150 MB)** | High (400 MB–1.5 GB) | Very high (600 MB–2 GB) | Low (~120–300 MB) | Very high (1.5–4 GB+) | **Minimal (15–60 MB)** |
| **Huge files (50 MB+ / minified)** | **Excellent (automatic light mode)** | Average (freezes on long lines) | Average | Good | Struggles above 20 MB | Depends on plugins (Neovim) |
| **Privacy / Offline / No account** | **Complete (no network except git push/pull)** | Telemetry by default (except VSCodium) | Requires account and cloud | Asks for account for AI and collaboration | Requires account / license | **Complete** |
| **Structured protocol for agents (declarative UI, command approval, mascot)** | **Unique in its category (native, file-based)** | None (relies on chat extensions) | Focused on proprietary chat/diff | Focused on chat/inline assist | Focused on chat/inline | None (terminal only) |
| **Inline AI autocomplete ("Ghost text" / Tab)** | **None (by design)** | Via Copilot | **Excellent (leader)** | Very good (Zeta) | Good | Via plugins |
| **Text editor maturity** | Good on the basics, lacks advanced features | **Reference** | **Reference** | Very high | **Reference** | **Reference (modal)** |
| **LSP and refactoring depth** | High | **Complete** | **Complete** | High | **Unbeatable (own AST)** | High |
| **Debugging** | Basic (GDB 14+ for Rust/C/C++/Zig, `lldb-dap` on macOS; native MoonBit is experimental) | **Complete (any language)** | **Complete** | Basic/Average (recent DAP) | **Excellent** | Basic (nvim-dap) |
| **Version control (Git)** | Average (gutter marks and blame; no 3-way merge or stash) | **Complete** | **Complete** | Average | **Excellent (best merge)** | Via LazyGit / Fugitive |
| **Cross-platform support** | **Linux, macOS (Apple Silicon), Windows (x86_64)** | **Linux, macOS, Windows, Web** | **Linux, macOS, Windows** | **macOS, Linux, Windows** | **Linux, macOS, Windows** | **Any OS / SSH** |

---

## 2. Stability and Reliability

### Where IDE TEA is genuinely strong
1. **No WebView / Chromium architecture:** unlike VS Code, Cursor and Windsurf, there are no Chromium renderer processes and no dozens of Node.js *extension hosts* leaking memory throughout the day. It is a single binary, compiled in safe Rust.
2. **Isolation from failures of extensions and external processes:**
   - Language servers (`tea-lsp`), the debugger (`tea-dap` / `gdb`), `git` calls and agents run in separate processes. If `rust-analyzer` or `gdb` hangs or dies, the IDE window does not freeze.
   - Internal extensions talk to the core only through messages and events (`Request` / `CoreEvent`), without sharing mutable references to the application state.
3. **Protection against lost work:**
   - Closing a tab or quitting the IDE with unsaved changes always asks for confirmation.
   - **Automatic crash recovery (`backup.rs`):** tabs with unsaved changes are written to `$XDG_STATE_HOME/idetea/backups/<project>/` per PID and offered for restoration if the window closes unexpectedly.
   - Deleting files in the tree sends them to the **system trash** (no irreversible `unlink`).
   - External changes on disk (`git checkout`, `cargo fmt`, agents editing files) are picked up by the watcher: tabs without local edits reload on their own; tabs with unsaved local edits are **never** silently overwritten.
   - Binary files are refused with a clear warning instead of corrupting the buffer on save. Files in other encodings (UTF-16, Windows-1252, etc.) open and save in their own encoding; a character that does not fit the encoding is refused on save, never replaced with `?`. Without a BOM or a `charset` in `.editorconfig`, a Shift_JIS file still falls back to Windows-1252 (the `[editor] fallback_encoding` default) and has to be reopened with the right encoding from the status bar.
   - The `.idetea/` protocol uses atomic writes (`.tmp` + `rename`) and has a self-repair command (`idetea fix` / *"Interfaces: repair this project"*).

### Where IDE TEA's stability still falls short of the big ones
1. **Linux is where it has the most hours of use:** since 0.5.0 there are official builds for macOS (Apple Silicon) and Windows (x86_64), with the terminal (`conpty` on Windows), tasks, debugging and shortcuts (`Cmd` vs `Ctrl`) adapted, but these two systems have seen far less real-world use. There is no build for Intel macOS or Windows ARM, and the Windows installer is not code-signed yet (SmartScreen warns on first run). VS Code, JetBrains and Zed run maturely on all three systems.
2. **Limitations of the `iced` 0.14 ecosystem for complex applications:**
   - The IDE runs in a single window (`iced::application`). You cannot detach a terminal, a preview or an editor to a second monitor in another native OS window.
   - There is no continuous per-file local history (*Local History* / *Timeline* over days without Git, as in JetBrains and VS Code).
3. **Regex-based syntax highlighting (`syntect`) vs. `tree-sitter`:**
   - Although IDE TEA has an excellent light mode for 50 MB files or huge minified lines, in normal code files with thousands of lines `syntect` (based on regex `.sublime-syntax` grammars) is structurally less resilient to incomplete code than the `tree-sitter` used by Zed, Helix and Neovim.

---

## 3. Working with AI Agents (Where IDE TEA is unique — and where it does not compete)

This is the deepest difference in product philosophy on the market today.

### How Cursor, Windsurf and Zed handle AI
- They are built around **the model inside the editor**:
  - *Predictive autocomplete (Tab / Next Edit Prediction)* that guesses the next lines while you type.
  - *Inline Edit (`Ctrl+K`)* and *Composer/Agent Chat*, which show colored diffs (green/red) right between the editor lines so you can accept/reject hunk by hunk.
  - In return, they tie you to cloud accounts, monthly subscriptions or provider HTTP clients inside the IDE.

### How IDE TEA handles AI (and where it wins)
IDE TEA assumes that **the best agents already run in the terminal** (Claude Code, Codex, Gemini CLI, Aider, OpenCode) and solves the problem that no other IDE has solved in a native and open way: **how these agents talk to the human without turning into a wall of text in the terminal**.

1. **Native Declarative Interfaces (`.idetea/ui/` + `idetea ask`):**
   - Instead of the agent asking a question in the middle of 200 lines of terminal log, it publishes a validated JSON file and the IDE renders native forms in `iced`: selectors, checkboxes, lists of manual test steps (where the *Approve* button is enabled only after the steps are done), image/gallery previews, animated spritesheets and WAV audio with a waveform.
   - **No other IDE on the market (VS Code, Cursor, Zed, JetBrains) has a file-based declarative protocol for terminal agents to draw native UI with an auditable history (`.idetea/history/`).**
2. **Execution Governance and Safety (`idetea run`):**
   - When an agent wants to open an interactive terminal or delegate a task, the IDE shows the exact command in an approval tab. The list of allowed commands (`[agents] allow`) is read only from the user's global config (`~/.config/idetea/config.toml`) and **never** from the repository — which keeps a malicious repository from granting itself permission to execute.
3. **Delegation in Isolated Git Worktrees (`--worktree` + `$IDETEA_ANSWER`):**
   - The IDE natively manages creating and cleaning up worktrees so subagents can work in parallel without dirtying the main tree.
4. **Native Reactive Mascot (Tea) + `idetea mascot`:**
   - While tools like *Masko.ai* require macOS + a cloud MCP server to get a mascot that reacts to Claude Code, IDE TEA has **Tea** built in natively, watching `.idetea/ui/`, `.idetea/agents/`, tasks and the LSP in real time, also reacting to the mouse (`curious` / `annoyed`) and accepting `mascot.toml` packs.
5. **Agent launcher (`Ctrl+Shift+A`) and editor context (`Ctrl+L`):**
   - The launcher opens the chosen agent (Claude Code, Codex, Gemini CLI, OpenCode, Aider, Cursor Agent or a launcher of your own) in a terminal tab, in the project folder or in a new worktree.
   - `Ctrl+L` in the editor pastes into that tab the cursor position (or the selection) and the LSP problems on those lines, without sending; a right-click in the **Problems** panel sends a single problem.
6. **Project memory in files (`docs/memory/`, `docs/decisions/`):** the instructions the IDE writes for agents teach a shared memory in Markdown under git (facts and ADRs), with its own panel in the IDE.

### Where IDE TEA loses in the AI workflow
- **No predictive autocomplete ("Ghost Text" / Tab):** if you rely on Cursor's/Copilot's `Tab` to complete boilerplate while typing by hand, IDE TEA has no equivalent.
- **No inline AI diff view in the editor:** when a terminal agent edits a file, the file reloads in the IDE TEA tab. To see what changed, you have to go to the **Git** panel; there are no *"Accept / Reject hunk"* buttons floating inside the code editor as in Cursor or Windsurf.

---

## 4. Code Editor: What it has vs. What it lacks

### What is already solid in IDE TEA
- Quick file opening (`Ctrl+P` with fuzzy search and `file:line` jumps), symbols in the file (`Ctrl+Shift+O`) and in the project (`Ctrl+T`).
- Multi-cursor, word-by-word selection (`Ctrl+Arrows`), `Shift+Click`, code folding, undo/redo, find and replace in the file and **project-wide replace (`Ctrl+Shift+H`)** with a line-by-line preview and regex support.
- Project search (`Ctrl+Shift+F`) scoped to a folder or file (right-click, with a chip in the panel to clear it), include and exclude globs (`src/**`, `*.test.rs`), `F4` / `Shift+F4` for the next and previous result even from the editor, and context lines (`[search] context_lines`, default 0).
- CSV, TSV and JSONL open as a table (sort and filter without touching the file; editing a cell changes only that field in the text, and `Ctrl+Z` undoes it). The button in the tab bar switches back to text.
- Temporary editor zoom (`Ctrl+=`, `Ctrl+-`, `Ctrl+0`, `Ctrl+Scroll`) synced across tabs and splits without affecting the rest of the UI.
- Side-by-side editor split, clickable breadcrumbs (`project › folder › file › symbol`), Seti icons per language and customizable fonts with ligatures (`JetBrains Mono` bundled).
- Optional Vim mode.
- Native tabs for images (with zoom and drag), PDF (rendered in pure Rust via `hayro`) and live side-by-side preview of Markdown and SVG.

### What the editor lacks compared to VS Code, Zed and JetBrains
1. **Snippets:** there is no snippet engine (tab stops `$1`, `$2`) to expand quick blocks of code.
2. **AST-based structural selection (Expand/Shrink Selection):** in Zed, Helix and JetBrains, a shortcut expands the selection from word → expression → `if` block → entire function via `tree-sitter`/AST.
3. ~~Legacy encodings~~ — **done:** files in Windows-1252, ISO-8859-*, UTF-16, Shift_JIS, GBK and others open and save in their own encoding; the status bar shows the encoding and reopens or saves with another one.
4. **Multiple windows / multiple projects in the same instance:** each project is a separate `idetea` process; there are no "Multi-root Workspaces" in the same window.

---

## 5. Language Intelligence (LSP) and Refactoring

### What IDE TEA already delivers very well
- Real-time diagnostics with a **squiggly underline in the editor text**, an error tooltip on hover, a count in the status bar and a **Problems** panel (`Ctrl+Shift+M` / `F8`).
- Autocomplete, signature help, go to definition (`F12`), go to type and go to implementation (`Ctrl+F12`).
- Find references (`Shift+F12` or `Ctrl+Click` on the declaration) grouped by file, with Back/Forward navigation history (`Alt+←` / `Alt+→` and the mouse side buttons).
- **Call hierarchy (`Alt+Shift+H`):** who calls the function and what it calls.
- **Structure / Outline panel (`Alt+7`):** the file's symbol tree showing visibility (`pub`, `pub(crate)`, private) and highlighting the symbol under the cursor.
- **Rename symbol (`F2`)** across the whole project, with a preview dialog showing how many files and changes will be affected before applying.
- Format on save and respect for `.editorconfig`.

### Important LSP gaps compared to VS Code, Zed and JetBrains
1. ~~No Code Actions / Quick Fixes (`Ctrl+.` / LSP light bulb)~~ — **done:** `Ctrl+.` lists the server's fixes and refactorings for the error or selection (missing import, generate function, extract variable).
2. ~~No Inlay Hints~~ — **done:** the inferred type of variables and parameter names appear in gray inside the code; `Ctrl+Alt+I` toggles them.
3. ~~No LSP Semantic Tokens~~ — **done:** parameters, fields and function calls get the theme's color on top of the grammar, on by default (`[lsp] semantic_tokens`). With the default theme the difference in calls is subtle, and mutable variables only change color in a theme that has its own color for them.

---

## 6. Debugging

### What IDE TEA does today
- Native integration with `gdb` 14+ (or `lldb-dap` on macOS) via DAP: breakpoints by clicking the line number (`F9`) that follow the text if you edit the file, `F5` (builds and runs automatically in Rust, Zig and, experimentally, MoonBit projects), Step Over (`F10`), Step Into (`F11`), Step Out (`Shift+F11`), Pause and Stop.
- A panel with the call stack (a click takes you to the frame's line), an inspectable local variables tree and the program output.

### Where it loses to VS Code and JetBrains
- **Tied to GDB 14+ (Rust, C, C++, Zig and, experimentally, MoonBit) and to `lldb-dap` on macOS:** it does not yet support DAP adapters for Python (`debugpy`), Go (`delve`), JavaScript/TypeScript (`js-debug`) or `codelldb`. Hare has no debugger in the IDE.
- **Missing advanced debugging features:** there are no conditional breakpoints (*"stop only when `i == 42`"*), logpoints, arbitrary *Watch Expressions* window (REPL while stopped at a breakpoint) or variable values drawn inline next to the code in the editor.

---

## 7. Git and Version Control

### What IDE TEA does today
- Status bar with the current branch, commits ahead/behind (`ahead/behind`) and the total number of modified files.
- Git panel with stage/unstage per file or all at once, colored diff view, a commit message box, discarding changes with confirmation, and `push`, `pull` and `fetch` buttons using the system's `git` and credentials/SSH.
- Git status colors in the file tree.
- Marks in the editor gutter on lines added, changed and deleted since the last commit; clicking a mark shows the old snippet and lets you revert it.
- Author and commit of the cursor line (blame) in the status bar, with details and copying of the hash.
- Switching and creating branches from the palette (*Git: switch branch*).

### Where it loses to VS Code, Zed and JetBrains
1. **No visual merge conflict resolver (3-way merge):** when there is a conflict, you resolve it by editing the `<<<<<<<` markers in the text or from the terminal.
2. **No Stash in the UI:** for `git stash`, you use the integrated terminal.
3. **Blame for the cursor line only:** it lives in the status bar; there is no annotation next to every line and no blame view of the whole file.

---

## 8. Honest Summary: When to choose IDE TEA today (and when not to)

### IDE TEA is already better or unique when you:
1. **Code alongside terminal agents (Claude Code, Codex, Gemini CLI, etc.), on Linux, macOS or Windows,** and want the agent to ask structured questions, show plans/manual tests, ask explicit permission to run commands, use isolated worktrees and have a reactive mascot (**Tea**) without depending on the cloud or a subscription.
2. **Prioritize speed, predictability and low sensory noise:** it opens in under 200 ms, uses little memory, has no account, no telemetry, does not change the UI on its own and respects simple TOML files.
3. **Work on projects in Rust, Zig, C/C++, Python, Go, Hare, MoonBit or the Web** where the main workflow is editing quickly, navigating LSP symbols/references, running tasks/tests in the integrated terminal and reviewing commits.

### Another IDE is still better when you:
1. **Want predictive AI autocomplete on every keystroke (`Tab` / Ghost Text) or inline diff editing:** **Cursor**, **Windsurf** or **Zed** serve that style better.
2. **Rely heavily on GUI debuggers for Python/JS/Go:** **VS Code** or **JetBrains** are much more complete today.
3. **Depend on SSH Remote Development / DevContainers, or use Intel macOS / Windows ARM:** **VS Code** and **Zed** already have mature support for these scenarios.

---

## 9. Delivered and Still Open (To catch up with the leaders without losing IDE TEA's soul)

This section used to list five next steps; four have been delivered. What is delivered and what remains open, keeping the binary lean and the philosophy of the manifesto:

**Already delivered**

1. ~~LSP Code Actions / Quick Fixes (`Ctrl+.`)~~ — **done:** the list opens in the palette, the recommended one first, and the changes stay in the tabs unsaved.
2. ~~Git marks in the editor (Gutter Diff) + Branch switching~~ — **already done:** gutter marks and switching/creating branches from the palette.
3. ~~"Send Selection / Diagnostic to Agent" shortcut (`Ctrl+L`)~~ — **done in 0.8.0**, together with the agent launcher (`Ctrl+Shift+A`).
4. ~~LSP Inlay Hints~~ — **done:** inferred types and parameter names inside the code, on by default, with `Ctrl+Alt+I` to toggle them.

**Still open**

1. **More DAP adapters (`debugpy`, `delve`, `js-debug`, `codelldb`):** support debugging beyond GDB and `lldb-dap`. Porting to macOS and Windows, which was also part of this item, shipped in 0.5.0.
2. **3-way merge view and stash in the UI** (today, from the terminal; see section 7).
3. **Third-party plugins in WASM**, without recompiling the IDE (there is a prototype behind the `plugins` feature).
