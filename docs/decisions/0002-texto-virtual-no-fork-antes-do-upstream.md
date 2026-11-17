---
status: accepted
date: 2026-11-17
---

# Fazer o texto virtual (inlay hints) no fork, sem esperar a API de decorações do upstream

Complementa a [ADR 0001](0001-iced-code-editor-como-dependencia.md): troca só a consequência "inlay hints [...]
esperam a API de decorações no upstream". O resto da 0001 continua valendo.

## Contexto

Inlay hints são o item de LSP que falta à IDE. Um hint no meio da linha é texto que não está no buffer e empurra o
que vem depois dele, e isso só o widget `iced-code-editor` pode desenhar. A ADR 0001 deixou esse tipo de coisa
para a API de decorações proposta no upstream (issue #41), que ainda não teve resposta. A análise está na
[RFC 0018](../rfc/0018-inlay-hints.md).

## Opções

- **A.** Esperar a resposta da issue #41.
- **B.** Só hints no fim da linha, agora.
- **C.** Texto virtual completo no fork, na forma que a issue #41 propõe, em PRs separados para o upstream.

## Decisão

Escolhida a **C** (aprovada pelo painel em 2026-11-17):

1. no fork, uma refatoração sem mudança de comportamento que põe a conversão coluna ↔ x numa função só, depois o
   texto virtual (`VirtualText`, inline e no fim da linha); cada passo é um commit no fork e um PR no upstream que
   cita a issue #41;
2. na IDE, o cliente LSP pede `textDocument/inlayHint` do trecho visível; tipos e nomes de parâmetro; **ligados por
   padrão**, com `ctrl+alt+i` e `[lsp] inlay_hints` para desligar.

Se o mantenedor responder à issue #41 com outra forma para a API antes do texto virtual entrar, seguimos a dele.

## Consequências

- O fork cresce (de 5 para 7 ou 8 commits) até os PRs entrarem, e cada rebase sobre o upstream fica mais caro; uma
  mudança de renderização no upstream pode conflitar com a refatoração.
- O texto virtual também serve aos valores de variáveis no texto durante o debug.
- Se o upstream recusar, o código fica só no fork, e a RFC 0017 conta isso como condição para reabrir a decisão de
  trazer o widget para o projeto.
