# MoonBit example

A small [MoonBit](https://www.moonbitlang.com) module to try IDE TEA's MoonBit support: a library with `sum_to` and
`greet`, tests, a program in `cmd/main` and the generated interface `pkg.generated.mbti`.

```sh
idetea examples/languages/moonbit
```

Things to try:

- Hover `sum_to` in `sum.mbt`; change `sum_to(10)` to `sum_to("x")` to see a type error.
- Open `pkg.generated.mbti`, the package's public interface.
- `Ctrl+Shift+B`: `moon check`, `moon build`, `moon test` and `moon fmt --check`, run in this folder (the closest
  `moon.mod`). `moon run cmd/main` prints a greeting.

Needs the MoonBit toolchain; see [languages.md](../../../docs/languages.md#moonbit).
