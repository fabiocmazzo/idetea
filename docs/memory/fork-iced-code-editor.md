---
name: fork-iced-code-editor
description: onde fica o fork do iced-code-editor e como a IDE aponta para ele
type: reference
updated: 2026-11-16
---
O fork está em `~/NewProjects/Belabs/iced-code-editor` (remotes: `origin` = fabiocmazzo/iced-code-editor,
`upstream` = LuDog71FR/iced-code-editor). A IDE usa o branch indicado no `[patch.crates-io]` do `Cargo.toml`, por
`rev`. A versão em `[workspace.dependencies]` precisa casar com a do fork (ex.: `"0.7"` para 0.7.x). Se não
casar, o Cargo ignora o patch em silêncio e baixa a versão do crates.io, e aí faltam as APIs do fork.
Para testar uma mudança local no widget, troque o patch por `path = "../iced-code-editor/iced-code-editor"` e
volte para `git` + `rev` antes de commitar. Decisão de manter como dependência: [ADR 0001](../decisions/0001-iced-code-editor-como-dependencia.md).
