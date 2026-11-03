---
name: novo-projeto-rust
description: Use when creating a new Rust project from the TEA IDE "Rust" template (AGENTS.md initial request mentions Rust). Says what the smallest runnable Rust project looks like - binary or library, workspace or single crate, tests, clippy and CI - and what to ask the user first.
---

# New Rust project

Use together with the `novo-projeto` skill (the common workflow). This one only covers Rust.

## Ask first (in the same panel interface as the common questions)

- **Kind:** command-line program (binary), library, or both (a library with a thin binary). Default: binary.
- **Layout:** one crate, or a workspace with `crates/` (only when the user already sees several parts). Default: one
  crate.
- **Edition:** the newest stable edition `cargo new` uses on this machine. Do not ask unless the user cares.

## Create

- `cargo new --vcs none <name>` style in the current folder (`cargo init --vcs none`): git already exists.
- `src/main.rs` (or `src/lib.rs`) that does one real, small thing from the initial request, not "Hello, world" only.
- At least one test: a unit test next to the code, and for a binary an integration test in `tests/` that runs the
  program (`std::process::Command` on `env!("CARGO_BIN_EXE_<name>")`).
- `.gitignore` with `/target`.
- Optional, only if the user said the project goes to GitHub: `.github/workflows/ci.yml` running
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`.

## Prove it works

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or whatever the program does
```

All of them must pass before the manual test. Do not run `cargo fmt` over files the user wrote by hand without
asking; on files you created it is fine.

## Manual test to hand over

Steps like: run `cargo run -- <example>` and see the expected output; run `cargo test` and see it pass; open
`README.md` and check it explains how to run.
