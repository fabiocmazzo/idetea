# Developing IDE TEA

**English** · [Português](DESENVOLVIMENTO.md)

A guide for anyone who is going to build, test or change the code. The full architecture, the decisions and the
detailed roadmap are in [SPEC.md](SPEC.md) (in Portuguese).

## Build and run

You need stable Rust (edition 2024). CI tests on Linux, macOS and Windows; day-to-day development stays on Linux. The file dialog (open, save as)
uses the desktop portal (xdg-desktop-portal) and, without it, `zenity`.

```sh
cargo run --release -- [--no-splash] [folder|file...]
cargo build --release          # binary at target/release/idetea
```

Command line:

| Command | Project root | Open tabs |
|---|---|---|
| `idetea` | current folder | none (empty tab) |
| `idetea src/main.rs` | current folder | `src/main.rs` |
| `idetea ~/other/project` | `~/other/project` | none |
| `idetea a.rs b.rs` | current folder | `a.rs`, `b.rs` (active) |
| `idetea --menu [file]` (the `.desktop` file) | the file's repository, otherwise the last project | the file |

`idetea --install-desktop` / `--uninstall-desktop` adds/removes the IDE to/from the application menu (`~/.local/share`), pointing
at the executable that ran the command (or at the AppImage).

A file that does not exist opens in a new tab and is created on save.

### `just` recipes

| Recipe | What it does |
|---|---|
| `just run [args]` | installs the icon (if needed) and runs in debug |
| `just run-release [args]` | the same, in release |
| `just desktop` / `just undesktop` | installs/removes `idetea.desktop` and the icon in `~/.local/share` |
| `just check` | clippy + tests |
| `just dist [--no-appimage]` | `.tar.gz` and AppImage in `dist/` (the `dist` profile: release without symbols) |
| `just demo` | narrated video in `dist/idetea-demo.mp4` (Xvfb; see `tools/demo-video/README.md`) |

**Why `just desktop`:** on Wayland, the desktop panel (COSMIC, GNOME, KDE) finds a window's icon by its
app id (`idetea`), looking for an `idetea.desktop` with the same name. Once it is installed, even a plain
`cargo run` shows the icon. If an `idetea.desktop` already exists (from an installed version), it is kept.

## Releasing a version

1. Bump the version in `[workspace.package]` in `Cargo.toml` (and run `cargo build` so `Cargo.lock` follows).
2. Commit, `git tag v<version>` and `git push origin main v<version>`.
3. The "Versão" workflow (`.github/workflows/release.yml`) tests, builds the packages for Linux (x86_64 and aarch64),
   macOS (Apple Silicon) and Windows (x86_64) and publishes the release. A tag that differs from the `Cargo.toml` version fails.
4. Update `VERSION=` and the package names in the Install section of the 8 READMEs and in the install pages of the site
   (`site/index.html`, `site/start.html` and the translations in `site/<language>/`).

CI (`ci.yml`) runs clippy (warnings become errors) and the tests on every push to main and on every pull request.

## Tests and checks

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
```

The git tests create temporary repositories (including a local "remote") and skip themselves if `git` is missing.

Large files, without opening a window:

```sh
# Widget operations (open, type, search, undo) with N lines.
cargo run --release -p tea-editor --example large_file -- 50000 [--write grande.rs]
# Real frames (layout, drawing and text on the GPU, no window): a 50 MB Rust file, a 10 MB JSON on a single line
# and minified JS. Scroll to the middle and the end, scroll slowly and type; a frame over 1 s is flagged.
cargo run --release -p tea-editor --example large_render -- [rust|json|js|todos] [bytes]
QUADROS=/tmp/quadros cargo run --release -p tea-editor --example large_render -- json   # writes the last frame as PNG
```

## Environment variables

| Variable | Effect |
|---|---|
| `IDETEA_NO_SPLASH=1` | does not show the splash screen (the same as `--no-splash` or `[ui] splash = false`) |
| `IDETEA_LSP_LOG=1` | prints the LSP servers' log to the terminal |
| `IDETEA_DAP_LOG=1` | prints the debugger (DAP) events to the terminal |

## Where the files live

| File | Contents |
|---|---|
| `~/.config/idetea/config.toml` | global config (`$XDG_CONFIG_HOME`) |
| `<project>/.idetea/config.toml` | project config (on top of the global one) |
| `~/.local/state/idetea/state.toml` | interface state: panel sizes (`$XDG_STATE_HOME`) |

The defaults live in `tea_core::config::DEFAULT_CONFIG`; the command "Config: abrir config global" ("Config: open global config")
creates the file with them commented out.

## Structure

```
crates/
├── tea-core/      extension API (Extension trait, Context, events), config, shortcuts, icons, menu
├── tea-declarative-ui/  the idetea-ui format: validates, stores what the user changed, draws, builds the response
├── tea-plugins/      WASM plugin host (wasmtime + WIT in wit/; `plugins` feature, a Phase 4 prototype)
├── tea-editor/    facade over iced-code-editor (no other crate uses the widget directly)
├── tea-lsp/       LSP client (process + JSON-RPC), no UI
├── tea-dap/       DAP (debug) client, no UI
├── tea-app/       the binary: window, layout, palette, dialogs, help, settings, LSP
└── ext/
    ├── tea-ext-filetree/   file tree
    ├── tea-ext-statusbar/  status bar
    ├── tea-ext-search/     project search
    ├── tea-ext-problems/   errors and warnings
    ├── tea-ext-terminal/   terminal (alacritty via iced_term)
    ├── tea-ext-git/        git (calls the git binary)
    ├── tea-ext-debug/      Debug panel (the core talks to gdb; the panel receives CoreEvent::Debug)
    ├── tea-ext-references/ References panel (where a name is used, from the LSP)
    ├── tea-ext-outline/    Outline panel (the file's symbols; visibility comes from tea-app/src/outline.rs)
    ├── tea-ext-calls/      Calls panel (who calls / what it calls, from the LSP)
    ├── tea-ext-preview/    Preview panel (Markdown and SVG of the active file; receives CoreEvent::ActiveText)
    └── tea-ext-ui/         Interfaces panel (reads .idetea/ui/, writes the responses; off by default)
```

Golden rule: **the built-in extensions use only the public API of `tea-core`**, the same one a plugin would use.
If an extension needs something the API does not have, the API is incomplete: extend the API, do not open a shortcut.

## Writing an extension

1. Create the crate in `crates/ext/tea-ext-<name>/`, depending on `iced` and `tea-core`.
2. Implement the `tea_core::Extension` trait:

   ```rust
   impl Extension for MinhaExtensao {
       type Message = Message;
       fn id(&self) -> &'static str { "minha" }               // command prefix: "minha.algo"
       fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
           ctx.register_panel(PANEL, "Minha", Slot::Left);    // panel and preferred slot
           ctx.panel_icon(PANEL, Icon::Files);                // icon in the icon bar
           ctx.register_command("minha.algo", "Minha: fazer algo");
           Task::none()
       }
       fn on_event(&mut self, ev: &CoreEvent, ctx: &mut Context) -> Task<Message> { /* file saved, theme... */ }
       fn on_command(&mut self, cmd: &str, ctx: &mut Context) -> Task<Message> { /* "minha.algo" */ }
       fn update(&mut self, msg: Message, ctx: &mut Context) -> Task<Message> { /* messages from its own UI */ }
       fn view(&self, panel: PanelId) -> Element<'_, Message> { /* iced widgets */ }
   }
   ```

3. Wire it into the app: an `ext-<name>` feature in `crates/tea-app/Cargo.toml` (and in `default`, if it is on by default)
   and one line in `builtin_extensions()` in `crates/tea-app/src/main.rs`.

What `Context` offers (requests that the app carries out after the call returns): open a file (and at a line),
register panels and commands, run commands, notify, show/hide a panel, confirmation dialog
(`show_dialog`), context menu (`show_menu` + `context_area`), signal that files changed on disk, and publish
file status. The events (`CoreEvent`) report what happened: file opened/saved/active, diagnostics,
theme and font, git status, panel opened from the icon bar...

Extensions do not call each other directly: they talk only through **commands** and **events**. Any extension can be turned off
in `[extensions] <id> = { enabled = false }` (takes effect on restart) or moved to the other side with `slot = "right"`.

## Built-in language packs

A language pack bundles, for one language, the file types, the highlighting grammar, the default LSP server and,
when there is a reliable project marker, tasks. Decisions and limits of the format:
[RFC 0014](docs/rfc/0014-language-packs-hare-e-moonbit.md) (in Portuguese). What each language offers the user:
[docs/languages.md](docs/languages.md).

**Before you start**, check the RFC's criteria (section 5.4): a compatible license, a real project or user for
dogfooding, official or maintained LSP/CLI, nothing downloaded or run automatically, and a reason to bundle it instead of just
documenting the configuration (`[lsp.servers]`, `[tasks]` and `~/.config/idetea/syntaxes/` already solve a lot).

**Steps**

1. Create `crates/tea-core/assets/languages/<id>/` with:
   - `pack.toml` (format 1). Fields: `format`, `id`, `name`; `[files]` with `extensions` (extension -> `languageId`
     sent to the LSP) and `syntaxes`; optional `[project] markers`, `[lsp]` (`name`, `command`, `args`) and
     `[[tasks]]` (`name`, `command`) and `[icon]` (`glyph`, `color`, `light`). An unknown field is an error. Tasks only with a marker that proves the language
     (`moon.mod` yes; `Makefile` no).
   - the `.sublime-syntax` grammar, copied from an identified revision, and its `LICENSE.txt`;
   - `README.md` in Portuguese: support, provenance (URL, revision, license, local changes) and validation
     (versions tested, date, what was checked and the limitations).
2. Register the pack in `builtins()` (`crates/tea-core/src/languages.rs`) and add the grammar to
   `crates/tea-core/assets/NOTICE.md`.
3. Tests, in `languages.rs`: the pack in `builtin_packs_describe_hare_and_moonbit` (or in a test of its own) and a small
   example in `builtin_syntaxes_produce_language_scopes`, with the scopes for keyword, type, number,
   string and comment. The uniqueness tests (ids, extensions, servers, tasks) and the parsing-by-the-editor tests
   already cover the new pack on their own.
4. Icon: if Seti does not have the language, draw a glyph in `tools/language-icons.py` (a single color, in Seti's
   frame), generate the font (`venv/bin/python tools/language-icons.py --preview <folder>` shows the drawing as SVG) and
   point `[icon]` at the character. The `pack_icons_exist_in_the_icon_font` test checks that it is in the font.
5. If the server needs an install hint, add a case in `with_hint` (`crates/tea-app/src/lsp.rs`) and
   the text in the three `tea-app.ftl` files. If the toolchain installs into its own folder (like `~/.moon/bin`), see
   `extra_tool_dirs` (`crates/tea-core/src/os.rs`).
6. An example in `examples/languages/<id>/` (with a `README.md` in English) and the language's row and section in
   `docs/languages.md`; the `every_pack_is_documented` test fails if one is missing. Also update the list of servers
   in `docs/features.md`.
7. Dogfooding with the real toolchain (colors, LSP, tasks and the IDE without the toolchain) and record the result in the
   pack's README.

Behavior that does not fit in the TOML (executable discovery, a fallback like Ziex's) becomes a Rust hook in the IDE, not
a new field in the manifest.

## Interface texts (languages)

The interface speaks Portuguese, English and Spanish ([Fluent](https://projectfluent.org), via `i18n-embed`).
The language comes from `[ui] language` (`"auto"` follows the system) and is chosen once, at startup.

Each crate with texts has:

- `i18n.toml` with `fallback_language = "pt-BR"` and `assets_dir = "i18n"`;
- `i18n/pt-BR/<crate-name>.ftl`, `i18n/en/...` and `i18n/es/...`;
- at the top of `lib.rs`/`main.rs`, before the `mod`s:

```rust
tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}
```

For a new text: write the message in the three `.ftl` files and use `fl!("id")` or `fl!("id", name = value)`.
Portuguese is the source: a key that does not exist in it does not compile. The `translations_are_complete` test
(in `tea-app`) checks that English and Spanish have the same messages and the same values. Plural:

```fluent
references = { $count ->
    [one] 1 referência
   *[other] { $count } referências
}
```

Cautions:

- No `fl!` before `tea_core::i18n::init` (in `boot`): the first use fixes the crate's language.
  Text created before that (e.g. command-line warnings) keeps the data and becomes text later.
- Do not compare translated texts in code (`msg.starts_with("Commit")`): use a type.
- Titles registered by id (core and tree commands) use `LOADER.get(id)`, which the compiler does not
  check; there are tests for that.
- Help guide: the command goes in as a hyphenated value, `{ $palette-open }`, and becomes the user's current
  shortcut (`get_with_markers`).
- A continuation line in an `.ftl` file cannot start with `[`, `*` or `.` (Fluent reads it as a variant).

## Conventions

- Interface texts in Portuguese in the `.ftl` files (with English and Spanish alongside); log messages, comments and
  documentation in Portuguese.
- Heavy work, or work that waits (git, search, network), runs off the interface thread.
- Each roadmap item is only marked as done in [SPEC.md](SPEC.md) after a manual test on screen.
