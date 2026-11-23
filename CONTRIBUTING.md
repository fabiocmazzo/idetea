# Contributing to IDE TEA

Thanks for taking a look. The project is small, and careful changes help more than large ones.

This repository follows the [Contributor Covenant](CODE_OF_CONDUCT.md).

A longer guide (architecture, extensions, release process) is in [`DEVELOPMENT.md`](DEVELOPMENT.md) (the Portuguese original is [`DESENVOLVIMENTO.md`](DESENVOLVIMENTO.md)).

## Build and run

You need [Rust](https://rustup.rs) (stable, edition 2024). From the repository root:

```sh
cargo run --release -p tea-app -- [folder-or-file]
```

Or `just run` / `just run-release` if you have [`just`](https://github.com/casey/just). The package is `tea-app`; the binary is named `idetea`.

On Linux, the file dialogs use the desktop portal (xdg-desktop-portal) and, without it, `zenity`.

## Tests and checks

CI runs on Linux, macOS and Windows, on every pull request:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

`just check` runs both (clippy without `-D warnings` locally; CI treats warnings as errors).

Do not run `cargo fmt --all`. The tree is not in the default rustfmt style, and a full format would rewrite more than a hundred files. Match the style of the file you are editing.

Git tests skip themselves when `git` is missing. Tests that need clangd, gdb or the `wasm32-wasip2` target also skip when those tools are absent.

If you change the website or `docs/features*.md`, build the site and check its internal links:

```sh
python3 site/build.py /tmp/idetea-site
```

## Translations

The interface, the website and the READMEs are in eight languages: English, Portuguese (Brazil), Spanish, French, German, Italian, Japanese and Simplified Chinese. Fluent (`.ftl`) files live next to each crate, under `i18n/<locale>/`. Glossaries and conventions are in [`docs/i18n/`](docs/i18n/README.md). If you change a user-visible string, update every locale in the same pull request when you can.

## Pull requests

- One topic per pull request.
- Say what changed and how you checked it.
- Do not bump the version or publish a release unless that is the subject of the PR.

Open the PR against `main`. The author reviews and merges.

## Issues

Use the Bug or Feature templates. A useful bug report includes the IDE TEA version (`idetea --version`), the OS, and whether you used a release package or built from source.
