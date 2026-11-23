---
name: teste-com-duas-ides
description: no teste manual com a build nova aberta ao lado da IDE de uso, pedidos de terminal podem cair na IDE antiga
type: gotcha
updated: 2026-11-23
---
No dogfooding, o teste manual costuma abrir a build nova (`./target/release/idetea -f .`) ao lado da IDE em uso,
no mesmo projeto. As duas olham `.idetea/agents/`, e um pedido de terminal (`idetea run`, `idetea agent`) fica com
a que o vê primeiro: pode rodar na IDE antiga, sem a mudança em teste. No roteiro do teste, peça para conferir em
qual janela a aba abriu (ou para rodar o comando pelo terminal da build nova e fechar a aba se abrir na outra).

O painel Interfaces não tem esse problema: a pergunta aparece nas duas, e o `ide.pid` da resposta diz qual respondeu.

**Why:** em 2026-11-23 um teste do estado das abas (RFC 0024) "falhou" porque o `idetea run` foi atendido pela IDE
antiga.
