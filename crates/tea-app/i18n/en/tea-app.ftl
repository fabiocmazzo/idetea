## Settings → Appearance

settings-theme = Theme
settings-theme-detail = Colors of the IDE, the editor and the terminal.
settings-font-size = Font size
settings-font-size-detail = Editor and terminal.
settings-code-font = Code font
settings-code-font-detail = Editor, line numbers and terminal. JetBrains Mono comes with the IDE; the list has the monospaced fonts installed.
settings-font-weight = Font weight
settings-font-weight-detail = A weight the font doesn't have uses the closest one.
settings-line-height = Line height
settings-line-height-detail = In multiples of the font size. The terminal follows in the same proportion.
settings-file-icons = File icons
settings-file-icons-detail = An icon for each language or file type (the Seti set from VS Code). Off: the same icon for every file.
settings-activity-bar = Activity bar
settings-activity-bar-detail = Strip on the left edge to show/hide panels with one click.
settings-splash = Splash on start
settings-splash-detail = Start screen (at least 2 s). --no-splash turns it off for one run.
settings-language = Language
settings-language-detail = Interface text. Automatic follows the system language. French, German, Italian, Japanese and Simplified Chinese include the mascot; other text falls back to English.
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
cmd-tab-reopen = Tabs: reopen the last closed
cmd-editor-find = Editor: find
cmd-editor-replace = Editor: find and replace
cmd-editor-goto-line = Editor: go to line
cmd-editor-undo = Editor: undo
cmd-editor-redo = Editor: redo
cmd-editor-select-all = Editor: select all
cmd-editor-fold-all = Editor: fold all blocks
cmd-editor-unfold-all = Editor: unfold all blocks
cmd-editor-toggle-fold = Editor: fold or unfold the block at the cursor
cmd-editor-toggle-vim = Editor: toggle vim mode
cmd-editor-zoom-in = Editor: zoom in
cmd-editor-zoom-out = Editor: zoom out
cmd-editor-zoom-reset = Editor: reset zoom
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
cmd-app-restart = Restart the IDE
cmd-app-quit = Quit
cmd-mascot-toggle = Mascot: show/collapse in corner

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
lsp-server-crashed = { $server } crashed — { $reason }. It comes back by itself when you stop typing.
lsp-server-crashed-too-often = { $server } crashed { $count } times in { $minutes } minutes — { $reason }. Use "LSP: restart servers".
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
ext-interfaces = Interfaces an agent writes to .idetea/ui/ (experimental)

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
lsp-hint-ziex = { $reason }. Install the Ziex CLI (https://ziex.dev/learn) or open the project folder, whose build.zig.zon depends on ziex: then the IDE builds the server with zig
lsp-hint-moonbit = { $reason }. Install the MoonBit toolchain (https://www.moonbitlang.com/download), which includes `moon lsp`; the IDE also looks in ~/.moon/bin (or $MOON_HOME/bin)
lsp-hint-hare = { $reason }. hare-lsp does not ship with Hare: build it from https://git.sr.ht/~whynothugo/hare-lsp and put it in the PATH (for example, ~/.local/bin)
config-syntax-error = syntax { $file }: { $error }
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
      -f, --foreground  stay attached to the terminal (otherwise, started from a terminal, the IDE
                     detaches and logs go to ~/.local/state/idetea/idetea.log)
      --install-desktop    adds the IDE to the applications menu (with the icon)
      --uninstall-desktop  removes it from the applications menu
      ui check <file.json>…  checks Interfaces panel documents (as the panel reads them)
      new <folder> [--modelo id]  creates a new project: folder, git, the template's skills and instructions
      skills [list|add|update|remove]  installs agent skills in the project (idetea skills --help)
      ui fix [--apply]  shows (and, with --apply, fixes) what was left behind in .idetea/ui/
      ask …          asks in the Interfaces panel and waits for the answer (idetea ask alone: usage)
      run … -- <command>  asks the IDE for a terminal and waits for the end (idetea run alone: usage)
      status [--json]  what the IDE open in this project shows (tabs, changes, cursor)
      open <file[:line[:column]]>  the open IDE shows the file, with the cursor in place
      history [--json]  what you answered in the Interfaces panel (--name, --since, --last N, --all)
      mascot <state> [--text "…"]  updates the IDE mascot (idle, working, thinking, attention, celebrating, error, sleeping, clear, status)
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
settings-breadcrumbs = Breadcrumbs
settings-breadcrumbs-detail = Above the editor: the file's path and the symbol where the cursor is. Click a folder to show it in the tree, a symbol to jump to it.
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

## Arquivos grandes
file-light-mode = "{ $name }" has a very long line and opened in light mode: no colors, no language server and with line wrap
lsp-light-mode = file in light mode (very long line): no language server

## Executável recompilado
binary-changed = The IDE was rebuilt: run "Restart the IDE" (palette) to use the new version.
binary-changed-title = [new version: restart]
binary-restart-error = Could not restart: { $error }

## idetea status / idetea open (RFC 0006)
status-usage =
    Usage: idetea status [--json]
    
    What the IDE open in this project shows: the active tab with the cursor and the open files
    (with "unsaved" on those with changes). --json: the same as JSON, with "open".
    Exit codes: 0 open, 2 bad argument, 3 the IDE is not open in this project.
status-bad-arg = unknown argument: { $arg }
status-not-open = the IDE is not open in this project ({ $root })
status-active = active: { $path }
status-no-active = no active file
status-modified = {" "}(unsaved)
open-usage =
    Usage: idetea open <file[:line[:column]]>
    
    The IDE open in this project shows the file, with the cursor at the line and column.
    Exit codes: 0 shown, 2 bad path or outside the project, 3 the IDE is not open (or did not take the request).
open-outside = { $path } is not inside the project ({ $root })
open-write-error = could not write { $path }
open-not-taken = the IDE did not take the request within 3 seconds

## Mascote reativo (RFC 0012)
mascot-state-idle = Calm
mascot-state-working = Working
mascot-state-thinking = Thinking
mascot-state-attention = Attention
mascot-state-celebrating = Done!
mascot-state-error = Error
mascot-state-sleeping = Sleeping
mascot-state-curious = Curious
mascot-state-annoyed = Annoyed
mascot-idle-1 = Just code. Add ice.
mascot-idle-2 = Steam's up. I'm here.
mascot-idle-3 = Quiet cup, steady watch.
mascot-idle-4 = The leaves can settle.
mascot-idle-5 = No rush. The tea is warm.
mascot-idle-6 = Sip when you're ready.
mascot-idle-7 = The cup is on your side.
mascot-idle-8 = All calm on the saucer.
mascot-hover-curious-1 = Hmm? Need something?
mascot-hover-curious-2 = Hey there. Everything alright?
mascot-hover-curious-3 = I felt that. What's up?
mascot-hover-curious-4 = Hovering over the tea?
mascot-hover-curious-5 = A hello, or a hand?
mascot-hover-curious-6 = The cup noticed you.
mascot-hover-curious-7 = Curious. Go on.
mascot-hover-curious-8 = Warm welcome. What's the plan?
mascot-hover-annoyed-1 = Easy — you're swirling the tea.
mascot-hover-annoyed-2 = That's a lot of mouse. The cup is dizzy.
mascot-hover-annoyed-3 = Steady hands. Hot tea.
mascot-hover-annoyed-4 = One pass is a hello. That was a storm.
mascot-hover-annoyed-5 = The saucer is rattling.
mascot-hover-annoyed-6 = Alright, I see you. Less swirl.
mascot-poke-curious-1 = Hmm? What is it?
mascot-poke-curious-2 = That tickles the cup.
mascot-poke-curious-3 = You rang?
mascot-poke-curious-4 = A tap. I'm listening.
mascot-poke-curious-5 = Careful, it's full.
mascot-poke-curious-6 = Hello from the saucer.
mascot-poke-double-1 = Two taps. Double the steam.
mascot-poke-double-2 = Double click noted. Still just tea.
mascot-poke-double-3 = Twice? Alright, you have me.
mascot-poke-double-4 = A double knock on the cup.
mascot-poke-double-5 = Two is a conversation.
mascot-poke-double-6 = Double tap. Nothing spilled.
mascot-poke-annoyed-1 = Careful, you'll spill it.
mascot-poke-annoyed-2 = The tea is already hot.
mascot-poke-annoyed-3 = That's enough tapping for one cup.
mascot-poke-annoyed-4 = I like you. The saucer is less sure.
mascot-poke-annoyed-5 = Easy. This is a small cup.
mascot-poke-annoyed-6 = Point made. I'll cool off.
mascot-task-1 = On it: { $name }
mascot-task-2 = Brewing { $name }…
mascot-task-3 = Task { $name } is running.
mascot-task-4 = { $name }. I'll watch the pot.
mascot-task-5 = Starting { $name }.
mascot-task-6 = { $name } — leave it to steep.
mascot-term-pending-1 = Terminal wants a yes: { $title }
mascot-term-pending-2 = A command is waiting: { $title }
mascot-term-pending-3 = Your call: { $title }
mascot-term-pending-4 = Permission, please: { $title }
mascot-term-pending-5 = Holding the kettle: { $title }
mascot-term-pending-6 = { $title } — shall I run it?
mascot-term-running-1 = Running: { $title }
mascot-term-running-2 = { $title } is on the boil.
mascot-term-running-3 = In progress: { $title }
mascot-term-running-4 = { $title } — still steeping.
mascot-term-running-5 = Working through { $title }.
mascot-term-running-6 = { $title } is underway.
mascot-term-ok-1 = Done: { $title }
mascot-term-ok-2 = { $title } finished clean.
mascot-term-ok-3 = All set — { $title }.
mascot-term-ok-4 = { $title } came out fine.
mascot-term-ok-5 = Clean exit: { $title }.
mascot-term-ok-6 = { $title }. The cup celebrates.
mascot-term-err-1 = { $title } failed ({ $code }).
mascot-term-err-2 = Exit { $code }: { $title }
mascot-term-err-3 = { $title } spilled (code { $code }).
mascot-term-err-4 = That one broke: { $title } ({ $code }).
mascot-term-err-5 = { $title } didn't finish ({ $code }).
mascot-term-err-6 = Code { $code } from { $title }.
mascot-answered-1 = Got it: { $title }
mascot-answered-2 = Answered: { $title }
mascot-answered-3 = { $title } — thanks.
mascot-answered-4 = Noted: { $title }
mascot-answered-5 = { $title } is settled.
mascot-answered-6 = You answered { $title }.
mascot-wait-1 = The agent stopped waiting: { $title }
mascot-wait-2 = { $title } went quiet.
mascot-wait-3 = Nobody is waiting on { $title } now.
mascot-wait-4 = { $title } cooled off.
mascot-wait-5 = Still open, but the wait ended: { $title }
mascot-errors-1 = { $count } error(s) in the project.
mascot-errors-2 = The leaves found { $count } error(s).
mascot-errors-3 = { $count } to look at, when you want.
mascot-errors-4 = Problems: { $count }. No rush.
mascot-errors-5 = { $count } error(s). I can point.
mascot-errors-6 = Saucer report: { $count } error(s).
mascot-save-1 = Saved { $name }. The cup approves.
mascot-save-2 = { $name } is on disk.
mascot-save-3 = Caught { $name } before it cooled.
mascot-save-4 = { $name } — tucked in.
mascot-save-5 = Write complete: { $name }.
mascot-save-6 = { $name } is safe. Back to the tea.
mascot-wake-1 = Oh — you're back. The tea is still warm.
mascot-wake-2 = I dozed off. Where were we?
mascot-wake-3 = Welcome back. I'll watch again.
mascot-wake-4 = Eyes open. The cup too.
mascot-wake-5 = That was a good pause.
mascot-wake-6 = Back at it? I'll keep watch.
mascot-sleep-1 = Steeping quietly…
mascot-sleep-2 = I'll be here.
mascot-sleep-3 = Low heat. Call if you need me.
mascot-sleep-4 = Drowsy cup. The code can rest.
mascot-sleep-5 = Napping on the saucer.
mascot-sleep-6 = Lights low. I'll wake when you do.
mascot-greet-morning-1 = Morning. The first cup is on me.
mascot-greet-morning-2 = Good morning. Code, then ice.
mascot-greet-morning-3 = Early light, fresh leaves.
mascot-greet-morning-4 = Morning. A short sip before the diff.
mascot-greet-morning-5 = Hello. The kettle is already on.
mascot-greet-afternoon-1 = Afternoon. The pot is still good.
mascot-greet-afternoon-2 = Good afternoon. Steady pace.
mascot-greet-afternoon-3 = Midday. No need to gulp it.
mascot-greet-afternoon-4 = Afternoon check-in from the cup.
mascot-greet-afternoon-5 = The sun is high. So is the tea.
mascot-greet-evening-1 = Evening. One more file, or the saucer.
mascot-greet-evening-2 = Good evening. The cup is still warm.
mascot-greet-evening-3 = Evening light. Wrap up when you like.
mascot-greet-evening-4 = The day is steeping down.
mascot-greet-evening-5 = Evening. I'm here if the build isn't.
mascot-greet-late-1 = Late. The tea is fine. Rest soon, if you can.
mascot-greet-late-2 = Quiet hours. Sleep is a feature.
mascot-greet-late-3 = Still up? Finish the thought, then rest.
mascot-greet-late-4 = The cup can stay. You might pause.
mascot-greet-late-5 = Late night. I'll keep watch. Keep it short.
mascot-tip-1 = Every command is in the palette: { $palette-open }.
mascot-tip-2 = Open a file by name with { $file-quick-open }.
mascot-tip-3 = { $file-save } puts the file on disk.
mascot-tip-4 = Shortcuts and a short guide: { $help-open }.
mascot-tip-5 = Run a project task with { $tasks-run }.
mascot-tip-6 = The terminal is { $terminal-focus }.
mascot-tip-7 = Problems: { $problems-focus }. The next one is { $problems-next }.
mascot-tip-8 = Jump to the definition with { $lsp-definition }.
mascot-build-ok-1 = Build finished. The tea is still hot.
mascot-build-ok-2 = It compiled. Nice leaves.
mascot-build-ok-3 = Build's clean.
mascot-build-ok-4 = Green build. I'll pour one.
mascot-build-ok-5 = Compiled. Back to the editor.
mascot-build-err-1 = The build didn't take.
mascot-build-err-2 = Build failed. The cup felt that.
mascot-build-err-3 = Compile error. Have a look when you want.
mascot-build-err-4 = That build spilled.
mascot-build-err-5 = Build failed. Problems might know more.
mascot-working-1 = On the boil.
mascot-working-2 = Working. The steam is busy.
mascot-working-3 = Leave it to steep.
mascot-working-4 = Still at it.
mascot-working-5 = The kettle is occupied.
mascot-thinking-1 = Stirring the leaves…
mascot-thinking-2 = One moment. Thinking.
mascot-thinking-3 = Letting it steep.
mascot-thinking-4 = Quiet thought, warm cup.
mascot-thinking-5 = Looking that over.
mascot-celebrate-1 = Done. That came out well.
mascot-celebrate-2 = All set.
mascot-celebrate-3 = Clean finish. Cheers.
mascot-celebrate-4 = The cup approves.
mascot-celebrate-5 = Nice. I'll keep the steam up.
mascot-error-1 = Something spilled.
mascot-error-2 = That one didn't take.
mascot-error-3 = A snag. I'm still here.
mascot-error-4 = Not this time.
mascot-error-5 = Worth a look, when you want.
mascot-open-interfaces = Open Interfaces panel →
mascot-open-terminal = Open Terminal panel →
mascot-open-problems = Open Problems panel →
mascot-idle-hint = Watching agents and tasks
mascot-bad-state = unknown mascot state: { $state }
mascot-usage =
    Usage: idetea mascot <state> [--text "…"] [--source <name>] [--ttl <seconds>]
           idetea mascot clear
           idetea mascot status [--json]
    
    States: idle, working, thinking, attention, celebrating, error, sleeping, curious, annoyed.
    Writes .idetea/mascot.json so the IDE mascot reacts in real time (expires after --ttl, default 30s).

## Welcome screen
welcome-start = Start
welcome-new-file = New file
welcome-quick-open = Open a project file…
welcome-open-file = Open file…
welcome-open-folder = Open folder in a new window…
welcome-recent = Open recent…
welcome-recent-files = Recent files
welcome-recent-projects = Recent projects
welcome-hint-palette = all commands
welcome-hint-search = search the project
welcome-hint-terminal = terminal
