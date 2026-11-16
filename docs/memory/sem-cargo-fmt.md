---
name: sem-cargo-fmt
description: o código não segue o rustfmt padrão; não rode cargo fmt no workspace
type: gotcha
updated: 2026-11-16
---
O workspace não tem `rustfmt.toml` e o código não está no formato padrão do rustfmt (linhas de até ~120 colunas,
`let … else { continue };` numa linha só). `cargo fmt --all` reformata mais de cem arquivos de uma vez.
Formate à mão, no estilo do arquivo em volta. O fork do `iced-code-editor` é outra história: lá o `cargo fmt` vale
(o CI do upstream confere).

**Why:** em 2026-11-16 um `cargo fmt --all` mexeu em 119 arquivos e teve de ser desfeito.
