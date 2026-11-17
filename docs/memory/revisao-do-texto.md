---
name: revisao-do-texto
description: Editor::text_revision só anda quando há um LspClient ligado ao widget; sem servidor, quem acompanha o texto precisa do Counter
type: gotcha
updated: 2026-11-17
---
`Editor::text_revision()` (`crates/tea-editor/src/lib.rs`) é um contador que só sobe no `did_change` de um
`LspClient` ligado ao widget (`lsp::Bridge`, ou `lsp::Counter`, que não fala com servidor nenhum e só conta). Numa aba
sem servidor LSP ele fica parado, e quem compara revisões para saber se o texto mudou nunca vê mudança.

A vista de tabela (RFC 0020) liga o `Counter` (`Editor::count_edits`) quando a aba não tem servidor, e de novo no
`detach_lsp`. Algo novo que acompanhe o texto de abas sem LSP precisa do mesmo.

**Why:** no primeiro teste manual das tabelas, editar uma célula mudava o texto mas a tabela não relia (um CSV não
tem servidor).
