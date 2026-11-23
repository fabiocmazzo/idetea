---
status: accepted
date: 2026-11-23
---

# Trazer o iced_term para o workspace (cópia, sem fork)

## Contexto

O terminal da IDE usa o `iced_term` 0.8.0 (MIT, Harzu/iced_term), um widget do iced sobre o `alacritty_terminal`. A
[RFC 0024](../rfc/0024-terminais-de-agente-muitos-e-retomaveis.md) precisa mudar coisas que ele não expõe: o tamanho
do histórico (ele cria o alacritty com `term::Config::default()`), a busca no histórico (o `RegexSearch` do alacritty
só é usado para achar URLs) e, se o M3 vier, um backend que leia de um socket em vez de ser dono do PTY.

## Opções

- **A.** Fork no GitHub, `[patch.crates-io]` por `rev` e PRs ao upstream, como o `iced-code-editor` (ADR 0001).
- **B.** Copiar o crate para `crates/iced_term` e mudar ali, sem fork e sem PR obrigatório.
- **C.** Deixar o histórico e a busca para depois.

## Decisão

Escolhida a **B** (painel, 2026-11-23). O `iced_term` 0.8.0 entra em `crates/iced_term` com o `src/` idêntico ao
publicado (commit `e0e90be` do upstream), a licença MIT e um `UPSTREAM.md` que lista cada mudança feita aqui. O
`tea-ext-terminal` passa a usar `iced_term.workspace = true`.

## Consequências

- Mudanças no terminal saem num commit só da IDE, sem repositório nem rebase de fork.
- Correções do upstream não chegam sozinhas: atualizar é comparar com o `UPSTREAM.md` e trazer à mão.
- O crate entra no `cargo clippy --workspace`; o estilo do upstream que o clippy do workspace reclama fica liberado no
  `Cargo.toml` dele, para o `src/` continuar comparável com o original.
- Uma mudança que sirva a outros usuários do `iced_term` ainda pode ir como PR ao upstream, mas não é condição para
  entrar aqui.
