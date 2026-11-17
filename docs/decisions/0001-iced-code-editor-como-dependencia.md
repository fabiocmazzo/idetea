---
status: accepted
date: 2026-11-16
---

# Manter o `iced-code-editor` como dependência, não trazer o código para o projeto

## Contexto

O editor de texto da IDE é o widget `iced-code-editor` (MIT), usado só pela fachada `tea-editor`, com um fork
(branch `idetea`) via `[patch.crates-io]`. O widget não tem API para desenhar dentro do texto, e isso bloqueia
inlay hints, semantic tokens, valores do debug no texto, faixas atrás do texto e diff inline. Trazer o código para
o workspace daria acesso total, mas o crate tem ~43 mil linhas, quase o tamanho da IDE inteira, e o upstream está
ativo e aceita PRs em poucos dias. A análise completa está na
[RFC 0017](../rfc/0017-iced-code-editor-dependencia-ou-codigo-proprio.md).

## Opções

- **A.** Dependência, com o fork só para o que ainda não entrou no upstream, e PRs.
- **B.** Trazer o código para `crates/tea-code-editor` (fork rígido).
- **C.** Trazer o código e sincronizar com `git subtree`.
- **D.** Escrever um editor próprio.

## Decisão

Escolhida a **A** (aprovada pelo painel em 2026-11-16):

1. alinhar o branch `idetea` do fork com o upstream 0.7;
2. mandar como PR os patches que só estão no fork (sintaxes extras, sublinhado de diagnósticos, barra em pílula);
3. propor no upstream uma API de decorações: marcas na margem, fundos atrás do texto, estilos por intervalo e
   texto virtual.

Esta decisão é revista (com uma ADR nova que a substitui) se o upstream recusar a API de decorações ou deixar a
proposta sem resposta por mais de 4 semanas, ficar mais de 3 meses sem commits, mudar de rumo de um jeito que
quebre a fachada, ou se precisarmos de tree-sitter ou rope e o upstream não quiser.

## Consequências

- As correções do upstream (IME, CJK, foco, desempenho) continuam chegando sem custo, e o código que mantemos
  continua em ~3 mil linhas na fachada.
- Inlay hints, semantic tokens e valores do debug no texto esperam a API de decorações no upstream. (Inlay hints e valores
  do debug: complementado pela [ADR 0002](0002-texto-virtual-no-fork-antes-do-upstream.md), que os faz no fork.)
- Code actions, encodings, snippets e seleção por AST são feitos na fachada (com PRs pequenos quando faltar API).
- Enquanto o `[patch]` existir, toda mudança no widget passa por branch no fork, `rev` no `Cargo.toml` e PR.
