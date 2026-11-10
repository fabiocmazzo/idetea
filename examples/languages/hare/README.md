# Hare example

A small [Hare](https://harelang.org) project to try IDE TEA's Hare support: a `sum` module, a program in
`cmd/example` that uses it, a test and a Makefile.

```sh
idetea examples/languages/hare
```

Things to try:

- Hover `upto` in `cmd/example/main.ha`, and `F12` to jump to `sum/sum.ha`.
- The Outline (`Alt+7`) of `sum/sum.ha`.
- Delete a `;`: the syntax error shows up (see the known problem in [languages.md](../../../docs/languages.md#hare)).
- `Ctrl+Shift+B`: `all`, `check` and `run` come from the Makefile.

Needs `hare` and `hare-lsp`; how to install them is in [languages.md](../../../docs/languages.md#hare).
