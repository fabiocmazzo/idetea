# Interface languages

The IDE ships eight locales. Portuguese (Brazil) is the source language checked by the `fl!` macro.
Every other locale must contain the same Fluent message ids and the same `{ $variables }` as English.
`translations_are_complete` fails otherwise.

| Tag | Folder | Glossary |
|---|---|---|
| pt-BR | `i18n/pt-BR/` | source |
| en | `i18n/en/` | reference for the parity test |
| es | `i18n/es/` | |
| fr | `i18n/fr/` | [glossary-fr.md](glossary-fr.md) |
| de | `i18n/de/` | [glossary-de.md](glossary-de.md) |
| it | `i18n/it/` | [glossary-it.md](glossary-it.md) |
| ja | `i18n/ja/` | [glossary-ja.md](glossary-ja.md) |
| zh-CN | `i18n/zh-CN/` | [glossary-zh-CN.md](glossary-zh-CN.md) |

Do not translate product names (IDE TEA, Tea, MoonBit, Hare, iced), command ids, config keys, or CLI flags.

Japanese and Simplified Chinese glyphs: the system CJK font is used when it is installed. The binary also embeds a subset of Droid Sans Fallback (`crates/tea-core/assets/fonts/DroidSansFallback-Subset.ttf`), regenerated with `tools/fonts/subset_cjk.py`, so the shipped UI still draws without a system CJK font. Regenerate that subset when ja or zh-CN strings gain new characters.

Left in their original language: `docs/rfc/`, `SPEC.md`, `DESENVOLVIMENTO.md`, `docs/interfaces/` (the agent protocol), `docs/terminais-de-agente.md`, and `docs/licoes-iced.md`. Those are design notes or the canonical protocol, not the product UI. User-facing docs that are translated use a `.<locale>.md` suffix next to the original.
