---
status: accepted
date: 2026-11-22
---

# Agentes de código rodam no terminal da IDE, não numa sessão ACP

Substitui as decisões das RFCs 0022 e 0023 (cliente ACP), revertidas da `main` em `2e08d33` (#20).

## Contexto

As RFCs 0022 e 0023 levaram a IDE a ser cliente do Agent Client Protocol: a IDE iniciava o agente, mostrava a
sessão numa aba própria e respondia aos pedidos de permissão. O marco 1 funcionou, mas o que faltava para ficar
usável (Markdown, copiar, entrada multilinha, permissões e ferramentas legíveis, contexto do editor) era refazer
dentro da IDE a interface que cada agente já tem no terminal, e cada agente cobre uma parte diferente do protocolo.

## Opções

- **A.** Seguir com o ACP (marcos 2a e 2b da RFC 0023).
- **B.** Agentes no terminal da IDE, com o canal estruturado em arquivos (painel Interfaces, pedidos de terminal,
  `idetea ask/run/agent`, hooks do mascote), e investir no terminal.

## Decisão

Escolhida a **B** (decisão do usuário, 2026-11-22). O código do ACP fica na história da `main` (`7ba0695`,
`bb25311`) e na tag `arquivo/acp` (em `bb25311`); as branches de ACP foram apagadas em 2026-11-23. O que o terminal precisa para isso está na [RFC 0024](../rfc/0024-terminais-de-agente-muitos-e-retomaveis.md).

## Consequências

- Cada recurso novo de um agente (comandos, modos, retomada) chega à IDE sem código novo.
- A IDE não lê a conversa: o estado do agente vem de hooks, do sino/OSC e da saída do processo, não do texto da tela.
- O terminal passa a ser a peça central: muitas abas, estado por aba, retomada da conversa (RFC 0024).
- Reabrir o ACP pede uma ADR nova que substitua esta.
