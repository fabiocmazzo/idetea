<p align="center">
  <img src="assets/idetea.svg" width="112" alt="IDE TEA icon: a glass of iced tea">
</p>

<h1 align="center">IDE TEA</h1>

<p align="center"><b>Just code. Add ice.</b></p>

<p align="center">
  A fast, quiet IDE written in pure Rust with <a href="https://github.com/iced-rs/iced">iced</a>,<br>
  built for working side by side with coding agents. Free, always: <a href="LICENSE">Apache 2.0</a>.
</p>

<p align="center">
  <b>English</b> · <a href="README.pt-BR.md">Português</a> · <a href="README.es.md">Español</a><br>
  <a href="https://idetea.belabs.cloud/"><b>Website and guides</b></a>
</p>

<p align="center"><img src="assets/demo.gif" width="886" alt="An agent asks a question in the Interfaces panel, then asks for a terminal to run the tests; the user answers and allows it."></p>

---

IDE TEA is an IDE that opens in a blink, stays out of your way and tells you what it is doing. No account, no
telemetry, no marketplace, no pop-ups asking for your attention. It touches the network only when you ask it to
(push, pull, fetch).

It is also an IDE designed for the way many of us code now: **with an agent in the terminal next to us**. Instead
of a chat box bolted to the side, IDE TEA gives agents a small, file-based protocol to talk to you through the
IDE, and keeps you in charge of everything they run.

> **Status:** in active development, before 1.0. Built and tested on Linux, macOS (Apple Silicon) and Windows (x86_64).
> Expect rough edges, and please report them.

## Why IDE TEA

**Fast.** Starts in well under 200 ms. A 50 MB file opens in a tenth of a second, and a minified file with a
single giant line opens in a light mode instead of freezing the window.

**Quiet.** One window, no notifications you did not cause, no surprise updates. The look, the keys and the panels
live in one plain TOML file (global or per project), with a settings screen for the same things.

**Complete where it counts.** Editor with multi-cursor, temporary zoom and an optional vim mode, file tree, project search,
integrated terminal with tabs, git (stage, diff, commit, push and pull), LSP for the usual languages, debugging
with gdb, and previews for Markdown, SVG, images and PDF.

**Small inside.** The core only knows how to edit files. Everything else (tree, terminal, git, debug…) is an
extension you can turn off.

**Agent-native, with you in control.**

- **Questions in a real panel, not in the chat.** An agent writes a small JSON file and the IDE shows it as a form:
  options, checkboxes, text fields, images, sounds, step-by-step manual tests with "approve" and "failed"
  buttons. Your answer goes back as a file the agent reads.
- **Terminals the agent asks for, and you approve.** An agent can ask the IDE to open a terminal running a
  command: a dev server, another agent, an interactive tool. You see the whole command before anything runs. A
  short allow list in *your* config (never the project's) can skip the question for commands you trust.
- **Agents that delegate to agents.** A helper agent can run in its own git worktree on its own branch, so two
  agents never edit the same files. The helper's answer comes back in a file; its code comes back through the
  branch and your usual review.
- **New projects that start with the right skills.** "Project: new…" creates the folder, git, the agent skills for
  the stack (Rust, Python, FastAPI, website…) and an initial request, then opens the agent in an IDE terminal for
  you to approve.
- **One command to teach any agent.** "Interfaces: teach this project's agents" writes the protocol into `AGENTS.md` and
  `CLAUDE.md`. It works with Claude Code, Codex and any agent that reads those files.
- **Reactive mascot (Tea) that watches your agents.** A pixel-art teacup in the corner of the IDE (collapsible into
  the status bar) reacts in real time: waves when a question or terminal request is waiting (one click opens the
  right panel), steams while an agent is working, celebrates when it finishes, gets curious or annoyed when you poke
  it with the mouse, and accepts states from the CLI (`idetea mascot`) or custom sprite packs (`mascot.toml`).

## Install

**Linux, macOS and Windows, ready to run:** download the package for your platform (`.tar.gz` or `.AppImage` for Linux x86_64/aarch64, `.tar.gz` with `IDE TEA.app` for macOS aarch64, or the `-setup.exe` installer / portable `.zip` for Windows x86_64) from
[Releases](https://github.com/fabiocmazzo/idetea/releases). On Linux, `./install.sh` puts `idetea` in
`~/.local/bin` and adds it to the app menu; the AppImage runs as is (`chmod +x` first) and `--install-desktop` adds it
to the menu. On macOS, `--install-desktop` creates `~/Applications/IDE TEA.app`. `SHA256SUMS` has the checksums.

**From source:** you need [Rust](https://rustup.rs) (stable). Then:

```sh
cargo install --locked --git https://github.com/fabiocmazzo/idetea tea-app
idetea --install-desktop   # optional: app menu entry and icon
```

Optional: `git` for the git integration, `gdb` 14+ (or `lldb-dap` on macOS) for debugging, and the language servers you use
(rust-analyzer, clangd, pyright, typescript-language-server, gopls and lua-language-server are already set up).

## Code Signing & Privacy Policy

Free code signing on Windows is provided by [SignPath.io](https://signpath.io), certificate by the
[SignPath Foundation](https://signpath.org). This program will not transfer any information to other networked systems
unless specifically requested by the user. See [`CODE_SIGNING_POLICY.md`](CODE_SIGNING_POLICY.md) for the full code
signing and privacy policy.

## Quick start

```sh
idetea                # open the current folder as a project
idetea src/main.rs    # current folder + this file
idetea ~/project      # another folder
idetea new my-app --model rust --agent claude     # a new project, with an agent ready to start it
```

Inside the IDE: `Ctrl+Shift+P` opens the command palette, `Ctrl+P` opens a file by name, `F1` shows every shortcut.

## About the name

It started as a joke. The IDE is built with **iced**, so of course it had to be **Iced Tea**.

Then the other meaning showed up. In Portuguese, **TEA** stands for *Transtorno do Espectro Autista*: autism
spectrum disorder. The author is autistic, and the double meaning felt right, so it stuck. Maybe it also explains
a few things about the IDE: it likes predictability, plain text you can read, and as little noise as possible.

Read more in the [manifesto](MANIFESTO.md).

## Documentation

- [Manifesto](MANIFESTO.md): what IDE TEA believes in, and what it will not do.
- [Features and shortcuts](docs/features.md): the full list, for the technical reader.
- [Interfaces for agents](docs/interfaces/formato.md) and [agent terminals](docs/terminais-de-agente.md) (in
  Portuguese for now).
- [SPEC.md](SPEC.md) and [DESENVOLVIMENTO.md](DESENVOLVIMENTO.md): specification, architecture and how to build,
  test and write an extension (in Portuguese for now).

## License

IDE TEA is free software under the [Apache License 2.0](LICENSE): use it, study it, change it and share it,
commercially too. It is free of charge and it will stay that way.

The interface icons come from [Codicons](https://github.com/microsoft/vscode-codicons) © Microsoft, under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) (see
[`crates/tea-core/assets/NOTICE.md`](crates/tea-core/assets/NOTICE.md)).
