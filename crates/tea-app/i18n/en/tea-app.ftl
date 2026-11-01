## Settings → Appearance

settings-theme = Theme
settings-theme-detail = Colors of the IDE, the editor and the terminal.
settings-font-size = Font size
settings-font-size-detail = Editor and terminal.
settings-activity-bar = Activity bar
settings-activity-bar-detail = Strip on the left edge to show/hide panels with one click.
settings-splash = Splash on start
settings-splash-detail = Start screen (at least 2 s). --no-splash turns it off for one run.
settings-language = Language
settings-language-detail = Interface text. Automatic follows the system language.
settings-language-auto = Automatic ({ $language })
settings-language-restart = Takes effect after restarting the IDE (now: { $language }).

## Comandos do núcleo (título na paleta e na ajuda)
cmd-palette-open = Command palette
cmd-help-open = Help: shortcuts and guide
cmd-settings-open = Settings
cmd-file-new = File: new
cmd-file-save = File: save
cmd-file-open = File: open…
cmd-file-quick-open = File: open by name…
cmd-debug-continue = Debug: start / continue
cmd-debug-stop = Debug: stop
cmd-debug-pause = Debug: pause
cmd-debug-next = Debug: next line
cmd-debug-step-in = Debug: step into function
cmd-debug-step-out = Debug: step out of function
cmd-debug-toggle-breakpoint = Debug: toggle breakpoint on line
cmd-debug-choose-program = Debug: choose the program…
cmd-file-open-folder = File: open folder in new window…
cmd-file-save-as = File: save as…
cmd-file-save-as-path = File: save as (type the path)…
cmd-file-close = File: close tab
cmd-tab-next = Tabs: next
cmd-tab-prev = Tabs: previous
cmd-editor-find = Editor: find
cmd-editor-replace = Editor: find and replace
cmd-editor-goto-line = Editor: go to line
cmd-editor-undo = Editor: undo
cmd-editor-redo = Editor: redo
cmd-editor-select-all = Editor: select all
cmd-editor-fold-all = Editor: fold all blocks
cmd-editor-unfold-all = Editor: unfold all blocks
cmd-editor-toggle-vim = Editor: toggle vim mode
cmd-lsp-definition = LSP: go to definition
cmd-lsp-references = LSP: find references
cmd-lsp-implementation = LSP: go to implementation
cmd-lsp-type-definition = LSP: go to type definition
cmd-lsp-incoming-calls = LSP: who calls this function
cmd-lsp-outgoing-calls = LSP: what this function calls
cmd-nav-file-symbol = Go to symbol in file…
cmd-nav-project-symbol = Go to symbol in project…
cmd-nav-back = Navigation: back
cmd-nav-forward = Navigation: forward
cmd-lsp-hover = LSP: show documentation at cursor
cmd-lsp-format = LSP: format file
cmd-lsp-restart = LSP: restart servers
cmd-theme-toggle = Theme: toggle light/dark
cmd-config-open = Config: open global config
cmd-config-open-project = Config: open project config
cmd-config-reload = Config: reload
cmd-view-toggle-left = Layout: show/hide left side
cmd-view-toggle-right = Layout: show/hide right side
cmd-view-toggle-bottom = Layout: show/hide bottom panel
cmd-app-quit = Quit

## Avisos, diálogos e menus
file-new-created-on-save = New file: { $path } (created when saved)
file-open-error = Error opening { $path }: { $error }
file-changed-on-disk-kept = "{ $name }" changed on disk, but the tab has changes: it was kept
file-reload-error = Error reloading { $path }: { $error }
file-changed-on-disk-overwrite = "{ $name }" changed on disk, but the tab has changes: saving overwrites it
debug-still-building = Debug: still building…
debug-already-running = Debug: the program is already running
debug-building = Debug: building ({ $command })…
unknown-error = unknown error
debug-build-failed = Debug: the build failed: { $error }
debug-choose-title = Which program to debug?
debug-choose-message = The project built more than one executable.
cancel = Cancel
debug-running = Debug: running { $program }
debug-error = Debug: { $error }
debug-cancelled = Debug: cancelled (the build finishes on its own)
debug-stopped = Debug: stopped
debug-no-source = Debug: { $name } has no source code
debug-paused-no-source = Debug: paused in { $name } ({ $reason }), no source code
debug-paused-at = Debug: paused at { $path }:{ $line } ({ $reason })
debug-exited = Debug: the program exited
debug-exited-code = Debug: the program exited (code { $code })
debug-command-failed = Debug: { $command } failed: { $message }
debug-gdb-exited = Debug: gdb exited ({ $reason })
debug-no-session = Debug: no session running (F5 starts one)
debug-already-paused = Debug: the program is already paused
debug-pause-first = Debug: the program is running (pause it first)
nav-nothing-forward = Nothing to go forward to
nav-nothing-back = Nothing to go back to
lsp-no-references = No references found
lsp-no-definition = Definition not found
lsp-no-implementations = No implementations found
lsp-no-type = Type not found
lsp-not-used-elsewhere = "{ $symbol }" is not used anywhere else
outline-save-first = Save the file to see its structure
outline-reading = Reading the structure…
symbols-type-a-name = Type the name of a function, type, method…
searching = Searching…
symbols-none-found = No symbols found (if the project is still loading, try again)
symbols-none-in-file = No symbols in this file
lsp-save-first = save the file to use the language server
lsp-finding-references = Finding references…
lsp-finding-implementations = Finding implementations…
calls-not-in-function = The cursor is not in a function (or the server does not know its calls)
lsp-formatted = File formatted
lsp-level-error = error:{" "}
lsp-level-warning = warning:{" "}
lsp-server-stopped = { $server } stopped — { $reason }. Use "LSP: restart servers".
lsp-no-formatting = Formatting is not available for this file
lsp-restarted = Language servers restarted
save-changes-title = Save changes?
save-changes-message = "{ $name }" has unsaved changes.
save = Save
discard = Discard
picker-already-open = A file dialog is already open
save-tab-closed = The tab was closed before saving
folder-opening = Opening { $path } in a new window
folder-open-error = Could not open the folder: { $error }
saved = Saved: { $path }
save-error = Error saving: { $error }
file-deleted-kept = File deleted on disk; the tab with changes was kept
and-more = … and { $count } more
quit-unsaved-title = Quit with unsaved changes?
quit-save-all = Save all and quit
quit-without-saving = Quit without saving
save-needs-name = "{ $name }" has no name: use "Save as"
save-file-error = Error saving { $path }: { $error }
close-tabs-title = 
    { $count ->
        [one] Close 1 tab?
       *[other] Close { $count } tabs?
    }
close-tabs-unsaved = With unsaved changes:
save-and-close = Save and close
discard-changes = Discard changes
tab-close = Close
tab-close-others = Close others
tab-close-right = Close to the right
tab-close-saved = Close saved
tab-close-all = Close all
copy-path = Copy path
copy-relative-path = Copy relative path
reveal-in-tree = Reveal in file tree
open-in-file-manager = Open in file manager
copied = Copied: { $text }
file-manager-error = Could not open the file manager: { $error }
save-then-close = Choose where to save; then close the tab again
config-reloaded-problems = Config reloaded with problems: { $problems }
config-reloaded = Config reloaded (turning extensions on/off only takes effect after restarting)
config-no-home = Could not find the config folder (HOME is not set)
create-error = Error creating { $path }: { $error }
config-error = Config: { $error }
write-error = Error writing { $path }: { $error }
unknown-command = Unknown command: { $id }
keys-record-hint = Use Ctrl, Alt or a special key (F1–F12, PageUp…). Esc cancels.
keys-already-bound = { $key } was already "{ $command }"
keys-rebound = { $key } was "{ $old }"; now it is "{ $command }"
keys-bound = { $key } → "{ $command }"
theme-unknown = unknown theme "{ $name }", using "dark"
default = default

## Configurações
settings-title = Settings
settings-tab-appearance = Appearance
settings-tab-extensions = Extensions
settings-tab-shortcuts = Shortcuts
close-esc = Close (Esc)
settings-no-home = (HOME is not set)
settings-write-to = Save to
settings-target-global = Global
settings-target-project = Project
settings-open-file = Open file
keys-filter = Filter by name, command or key…
keys-press = Press the keys… (Esc cancels)
keys-add = Add shortcut
keys-help = Click a key to remove it. + records a new key (also works for keys already in use: they change owner).
settings-lsp = Language servers
settings-lsp-detail = Turns LSP on or off as a whole.
settings-servers = Servers
settings-server-found = found in PATH
settings-server-not-found = not found in PATH
settings-lsp-note = Changes apply to servers started from now on.
settings-restart-servers = Restart servers
theme-dark = Dark (default)
theme-light = Light
slot-left = Left
slot-right = Right
slot-bottom = Bottom
slot-status = Status bar
settings-on-restart = takes effect after restarting the IDE
default-capitalized = Default
settings-extensions-note = Turning an extension on or off takes effect after restarting. Moving a panel to another side applies right away.
ext-filetree = File tree
ext-statusbar = Status bar
ext-search = Project search
ext-problems = Errors and warnings
ext-terminal = Integrated terminal
ext-git = Changes, stage, commit and diff
ext-debug = Debug buttons, stack, variables and output
ext-references = Where a name is used (Shift+F12)
ext-outline = Functions and methods of the file, with visibility (Alt+7)
ext-preview = Markdown and SVG preview next to the editor (Ctrl+Shift+V)
ext-calls = Who calls a function, and what it calls (Alt+Shift+H)

## Paleta, diálogos, LSP, debug e o resto do app
palette-commands = Type a command…
palette-save-as = File path (relative to the project root)
palette-files = File name (file:line goes to the line, > for commands)
palette-file-symbols = Function, type, method… in this file
palette-project-symbols = Function, type, method… in the project
palette-no-commands = No commands found
palette-save-hint = Enter saves · Esc cancels
palette-reading-files = Reading the project files…
palette-no-files = No files found
palette-no-symbols = No symbols
palette-no-matching-symbols = No symbols match the search
dialog-hint = Enter confirms · Esc cancels
lsp-command-not-found = { $server }: command "{ $command }" not found in PATH (install it or adjust [lsp.servers.{ $server }] in the config)
lsp-start-error = { $server }: could not start: { $error }
lsp-disabled = LSP is off in the config ([lsp] enabled = false)
lsp-no-server-for = no language server configured for .{ $ext } files
lsp-not-running = { $server } is not running: { $reason }
lsp-not-ready = { $server } is not ready yet
lsp-hint-rustup-install = { $reason }. Install it with: rustup component add rust-analyzer
lsp-hint-rustup = { $reason }. With rustup: rustup component add rust-analyzer
visibility-file-only = this file only
picker-save-as = Save as
picker-open-files = Open files
picker-open-folder = Open folder
picker-debug-program = Program to debug
splash-ready = ready
splash-loading = loading…
watch-too-many = folder too large: only the first { $count } folders are watched
watch-partial = could not watch all folders ({ $error })
watch-problem = Changes on disk: { $message }. Use "Refresh" in the file tree when needed.
watch-failed = Changes on disk: could not watch ({ $error })
debug-no-gdb = gdb not found in PATH (debugging uses gdb 14 or newer)
debug-program-not-found = program not found: { $path }
debug-gdb-start-error = could not start gdb: { $error }
debug-reason-breakpoint = breakpoint
debug-reason-step = step
debug-reason-pause = paused
debug-reason-exception = exception
debug-reason-entry = entry
debug-reason-goto = jump
build-run-error = could not run "{ $command }": { $error }
build-no-message = failed without a message

## Ajuda e linha de comando
help-key-find = Find in file
help-key-replace = Find and replace
help-key-goto-line = Go to line
help-key-undo = Undo / redo
help-key-next-match = Select the next occurrence (multi-cursor)
help-key-fold = Fold / unfold blocks
help-key-quick-fix = LSP quick actions (when available)
help-key-ctrl-click = Ctrl+Click
help-key-definition = Go to definition (LSP)
help-key-tab = Insert a tab
help-title = Help
help-tab-shortcuts = Shortcuts
help-tab-guide = Guide
help-tab-about = About
help-close = Close (Esc)
help-filter = Filter by name, command or key…
help-group-general = General
help-editor-keys = Editor (fixed)
help-in-palette = (palette)
about-no-home = (HOME not set)
about-version = Version
about-license = License
about-license-value = { $license } (free and open, always)
about-project = Project
about-global-config = Global config
about-project-config = Project config
about-built-with = Built in Rust with iced, iced-code-editor, alacritty_terminal (via iced_term) and syntect.
about-icons = Icons: Codicons © Microsoft, CC BY 4.0 license.
cli-usage = 
    Usage: idetea [options] [path...]
    
      path           file to open (may not exist yet: it is created on save)
                     or folder to use as the project root
      --no-splash    don't show the splash screen (also: IDETEA_NO_SPLASH=1)
      --menu         opened from the menu: without a folder, opens the file's project or the last project
      --install-desktop    adds the IDE to the applications menu (with the icon)
      --uninstall-desktop  removes it from the applications menu
      -h, --help     show this help
      -V, --version  show the version
cli-unknown-option = unknown option ignored: { $option }
cli-extra-root = only one root folder at a time, ignoring { $path }

## Ajuda → Guia (`{ $comando-com-hifens }` vira o atalho do comando)
guide-start = Getting started
guide-start-1 = Open the IDE with `idetea` in a folder (it becomes the root of the tree) or with `idetea file...` (the files open in tabs; the root stays the current folder).
guide-start-2 = Every command is in the palette: { $palette-open }. Type part of the name; the shortcut shows next to it.
guide-start-3 = This help opens with { $help-open }.
guide-start-4 = The icon bar on the left edge shows and hides the panels with one click (the open one is highlighted). Drag the border between a panel and the editor to resize it; double-click restores the default.
guide-files = Files and tabs
guide-files-1 = Open by name: { $file-quick_open } (type parts of the name; `main.rs:42` goes to the line; `>` switches to commands).
guide-files-2 = Go to a symbol (function, type, method…): { $nav-file_symbol } in the file, { $nav-project_symbol } in the whole project. Needs the language server.
guide-files-3 = New: { $file-new } · Open: { $file-open } · Save: { $file-save } · Save as: { $file-save_as } · Close tab: { $file-close }.
guide-files-4 = Open and Save as use the system file dialog. To type the path: { $file-save_as_path }. Opening another folder ({ $file-open_folder }) opens a new IDE window.
guide-files-5 = Markdown and SVG preview: { $preview-toggle } (or the button at the end of the tab bar) opens a side panel that follows the active file and updates as you type. Web links open in the browser; links to files open in a tab.
guide-files-6 = Images (png, jpg, gif, webp...) open in their own tab: the mouse wheel zooms and dragging moves.
guide-files-7 = PDFs also open in a tab: scroll through the pages; − and + change the zoom and "Fit" follows the width.
guide-files-8 = Switch tabs: { $tab-next } and { $tab-prev } (Ctrl+Tab inserts a tab in the editor).
guide-files-9 = Closing a tab or quitting with unsaved changes asks first: Save, Discard or Cancel.
guide-files-10 = File changed outside (terminal, another editor): the tab reloads by itself if it has no unsaved changes; if it does, it stays as it is and a notice appears.
guide-files-11 = Right-click a tab: close others, to the right, saved or all, copy path, reveal in the tree and open in the file manager. Middle-click closes the tab.
guide-tree = File tree
guide-tree-1 = Show/hide: { $filetree-toggle } · Reveal the current file: { $filetree-reveal }.
guide-tree-2 = Right-click an item: new file, new folder, rename, delete, copy path and open in the file manager. Delete sends it to the system trash.
guide-tree-3 = Git colors: yellow = modified, green = new, red = conflict, dimmed = ignored. A folder with changes inside shows a dot.
guide-bottom = Bottom panel
guide-bottom-1 = Project search ({ $search-focus }), Problems ({ $problems-focus }), Terminal ({ $terminal-focus }) and Git ({ $git-focus }) are tabs in the bottom panel. { $view-toggle_bottom } shows/hides the panel.
guide-bottom-2 = Next problem: { $problems-next }.
guide-terminal = Terminal
guide-terminal-1 = With the focus on the terminal, Ctrl+letter, F-keys and arrows go to the shell (Ctrl+C, Ctrl+W...). IDE shortcuts with Ctrl+Shift keep working, so the palette always opens.
guide-terminal-2 = Copy and paste in the terminal: Ctrl+Shift+C and Ctrl+Shift+V. Typing `exit` closes the panel.
guide-debug = Debug
guide-debug-1 = Breakpoint: click the line number or { $debug-toggle_breakpoint }. Start/continue: { $debug-continue }. In Rust projects it builds with cargo build and finds the executable by itself; for others, use [debug] in the config (program, args, cwd, build) or pick the executable. Change the program: { $debug-choose_program }.
guide-debug-2 = Stopped: { $debug-next } next line, { $debug-step_in } steps into the function, { $debug-step_out } steps out, { $debug-pause } pauses, { $debug-stop } stops. The yellow arrow shows where the program is. Uses gdb 14+.
guide-debug-3 = Debug panel ({ $debug-focus }): buttons, call stack (click goes to the frame and shows its variables), variables (click expands structs and vectors) and the program and build output.
guide-git = Git
guide-git-1 = The branch shows in the status bar. The Git panel ({ $git-focus }) shows what changed: + and − stage and unstage, clicking a file shows the diff, Enter in the message commits.
guide-git-2 = Right-click a file in Git: open, view diff, stage, discard changes (asks for confirmation) and copy path.
guide-git-3 = Fetch ({ $git-fetch }), pull ({ $git-pull }) and push ({ $git-push }) are the buttons at the top of the panel. The password is not asked here: use the ssh agent (ssh-add) or a credential helper. A new branch asks before being published on the remote.
guide-git-4 = The panel updates on save and when the repository changes (commit or checkout done in the terminal).
guide-lsp = Languages (LSP)
guide-lsp-1 = Language servers start by themselves when you open a file: rust-analyzer, clangd, pyright, typescript-language-server, gopls and lua-language-server (they must be installed).
guide-lsp-2 = Go to definition: { $lsp-definition } or Ctrl+Click · Documentation: { $lsp-hover } or rest the mouse on the code · Format: { $lsp-format }.
guide-lsp-3 = Where it is used: { $lsp-references }, or Ctrl+Click on the declaration itself. The list shows in the References panel; click goes to the use.
guide-lsp-4 = Implementations: { $lsp-implementation } (on a trait or interface, or on one of its methods). Type of what is under the cursor: "LSP: go to type definition" in the palette. With more than one result (including for definition), the list goes to the References panel; with only one, it jumps straight there.
guide-lsp-5 = Who calls a function: { $lsp-incoming_calls } with the cursor on it. The Calls panel shows the tree (open a level to see who calls the callers); "Callees" switches to the calls it makes. Click goes to the call.
guide-lsp-6 = Back and forward after a jump (definition, reference, open by name, clicks in the panels): { $nav-back } and { $nav-forward }, or the mouse side buttons.
guide-lsp-7 = File outline ({ $outline-toggle }): functions, structs, methods and fields in a tree. The open padlock is public, the closed one is private and the key is visible only in part of the code (pub(crate), protected, package). The symbol under the cursor is highlighted; click goes to it.
guide-lsp-8 = If a server fails, the status bar shows why; "LSP: restart servers" tries again.
guide-config = Settings
guide-config-1 = The settings screen ({ $settings-open }) changes theme, font, extensions, shortcuts and LSP, writing to the global or the project config. Changes apply right away.
guide-config-2 = There is a global config and one per project (`.idetea/config.toml`); the project one wins. To edit the file directly: "Config: open global config" or "Config: open project config". On save, everything reloads.
guide-config-3 = Shortcuts live in [keys]: "ctrl+alt+t" = "terminal.focus". Empty text turns off a default shortcut. Command ids show in the Shortcuts tab.
guide-config-4 = Language: Settings → Appearance → Language (Automatic follows the system). Takes effect after restarting the IDE.

## Sessão
settings-restore-session = Reopen as it was
settings-restore-session-detail = When a project opens, its tabs, cursor and panels come back. idetea file... opens just the file.

## Recentes
cmd-file-open-recent = File: open recent…
palette-recent = Project or file opened before
palette-no-recent = Nothing opened before yet
guide-files-12 = Open recent: { $file-open_recent } lists the projects (they open in a new window) and files opened last, from every window.

## Recuperar não salvos
untitled-tab = untitled
recover-title = 
    { $count ->
        [one] The IDE closed without saving 1 file
       *[other] The IDE closed without saving { $count } files
    }
recover-detail = Recover opens the changes in tabs, still unsaved (undo goes back to what is on disk). Later asks again next time.
recover = Recover
recover-later = Later
recovered = 
    { $count ->
        [one] 1 file recovered (not saved yet)
       *[other] { $count } files recovered (not saved yet)
    }

## Renomear símbolo e salvar todos
cmd-lsp-rename = LSP: rename symbol…
cmd-file-save-all = File: save all
save-all-untitled = Untitled tabs were not saved (use Save as)
palette-rename = New name
palette-rename-hint = Rename "{ $name }" across the project · Enter shows what changes · Esc cancels
rename-no-word = Put the cursor on a name to rename it
rename-asking = Rename "{ $name }": asking the server…
rename-nothing = Nothing to rename for "{ $name }"
rename-failed = Could not rename: { $message }
rename-title = Rename "{ $old }" to "{ $new }"?
rename-summary = 
    { $changes ->
        [one] 1 change
       *[other] { $changes } changes
    } in { $files ->
        [one] 1 file:
       *[other] { $files } files:
    }
rename-detail = The files stay changed in their tabs, not saved yet: Ctrl+Z undoes in each one, and "File: save all" writes everything.
rename = Rename
renamed = 
    { $count ->
        [one] Renamed in 1 file (not saved yet)
       *[other] Renamed in { $count } files (not saved yet)
    }

guide-lsp-9 = Rename: { $lsp-rename } with the cursor on the name. A preview shows how many changes and in which files; once confirmed, they stay changed in their tabs (Ctrl+Z undoes in each) and { $file-save_all } saves them all.

## Substituir no projeto
replaced = { $count } replacement(s) in { $files } file(s)
replaced-failed = { $count } replacement(s) in { $files } file(s); { $failed } could not be changed (no permission or not text)

guide-bottom-3 = Replace in project: { $search-replace } (or ▸ next to the search) opens "Replace with". Each result shows how it will look; "Replace" changes the line or the file, and "Replace all" asks first. Files open in tabs stay changed, not saved (Ctrl+Z undoes); the others are written straight to disk.

## Editor e salvar
format-save-timeout = The server did not format in time: saved without formatting

settings-tab-editor = Editor
settings-wrap = Wrap long lines
settings-wrap-detail = A line wider than the window continues below, with no sideways scrolling.
settings-whitespace = Show whitespace
settings-whitespace-detail = Spaces show as · and tabs as →.
settings-indent-guides = Indent guides
settings-indent-guides-detail = Thin vertical lines at each indent level.
settings-tab-size = Indent size
settings-tab-size-detail = How many spaces Tab inserts.
settings-insert-spaces = Indent with spaces
settings-insert-spaces-detail = Off: Tab inserts a tab character. A file already indented with tabs keeps tabs.
settings-trim = Trim trailing whitespace on save
settings-trim-detail = Except in Markdown, where two trailing spaces break the line.
settings-final-newline = Final newline on save
settings-final-newline-detail = Makes sure the last line ends with a newline.
settings-format-on-save = Format on save
settings-format-on-save-detail = Ctrl+S formats with the language server before writing (waits up to 2 s). Save all does not format.
settings-editorconfig-note = A .editorconfig in the project overrides these options (indent, tabs or spaces, cleanup on save).

guide-config-5 = Editor: the Editor tab in settings ({ $settings-open }) has line wrap, visible whitespace, indent guides, indent size and what to do on save (trim trailing spaces, final newline, format). A .editorconfig in the project overrides them; without one, a file already indented with tabs keeps tabs.

guide-lsp-10 = Parameter hints: typing ( in a call shows the function signature above the cursor, with the current parameter highlighted; it moves on with each comma and goes away when the parenthesis closes or with Esc.

guide-git-5 = Gutter marks: left of the line numbers, green = new line, blue = changed, red notch = deleted lines, compared with the last commit (follows your typing, no need to save). Clicking a mark shows how it was, with "Revert change" (Ctrl+Z brings it back).

palette-no-items = Nothing matches the search

guide-git-6 = Branches: { $git-checkout } (or click the branch name, in the status bar or the Git panel) lists branches, most recent first; remote ones that do not exist here come after and become local when chosen. Type a new name to create a branch from where you are. Uncommitted changes come along; if they clash with the other branch, git refuses and explains.

guide-git-7 = Line author: in the status bar, next to the branch, you see who last changed the cursor line, when, and the commit message (click to see the hash and copy it). While the tab has unsaved changes it hides until you save (disk lines may not match).

## Tarefas
cmd-tasks-run = Tasks: run…
cmd-tasks-rerun = Tasks: run the last one again
cmd-tasks-test-here = Rust: run the test at the cursor
tasks-placeholder = Run task…
tasks-none = No tasks: add one under [tasks] in the config, or open a project with Cargo.toml, package.json or Makefile.
tasks-no-terminal = Tasks need the terminal extension, which is turned off.
tasks-no-test = The cursor is not in a Rust test (#[test] fn).
tasks-no-package = Could not find the Cargo.toml of this file's package.
tasks-done = [{ $name } finished: code { $code } · Enter closes]

guide-terminal-3 = Tasks: { $tasks-run } lists the project commands ([tasks] in the config, cargo build/run/test/clippy, package.json scripts, Makefile targets) and runs the chosen one in a "Task" tab of the terminal, without touching the shell. At the end the exit code shows up; Enter closes it. Ctrl+C stops only the command. { $tasks-rerun } runs the last one again.
guide-terminal-4 = Test at the cursor (Rust): with the cursor inside a #[test] fn, { $tasks-test_here } runs just that test (cargo test -p package name). No default key: add one under [keys] if you use it a lot.

## Editor dividido
cmd-editor-split = Editor: move the tab to the other side (split)
cmd-editor-focus-left = Editor: go to the left side
cmd-editor-focus-right = Editor: go to the right side
split-one-tab = Open another file to split the editor: each side shows a different tab.
tab-move-right = Move to the right (split)
tab-move-left = Move to the left
guide-files-13 = Split editor: { $editor-split } (or "Move to the right" in the tab menu) puts the tab beside, with its own tabs; opening a file goes to the focused side. Click a side (or { $editor-focus_left } / { $editor-focus_right }) to work there; drag the divider to change the width (double click centers it). Moving the last tab of a side back joins everything again.

## Menu de aplicativos (--install-desktop)
desktop-no-home = Could not find the home folder (HOME).
desktop-error = Could not install the shortcut: { $error }
desktop-installed = 
    IDE TEA in the applications menu: { $desktop }
    Runs: { $exe }
desktop-removed = IDE TEA removed from the applications menu.
