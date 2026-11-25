# RFC 0026 — Interfaces para revisar, comparar e simular

- **Status:** aprovada no painel em 2026-11-25, com as cinco recomendações (ver Decisões). Na 0.20.0: etapas 1 e 2.
- **Continua:** [RFC 0001](0001-interfaces-declarativas.md) (o formato), [RFC 0003](0003-interfaces-pelo-uso.md)
  (o que o uso pediu), [RFC 0004](0004-midia-no-painel.md) (mídia) e [RFC 0002](0002-plugins-wasm.md) (plugins).
- **Detalhes:** [`docs/agent-ui/`](../agent-ui/), sete documentos: [estado atual](../agent-ui/01-current-state.md),
  [arquitetura](../agent-ui/02-architecture.md), [protocolo](../agent-ui/03-protocol.md),
  [segurança](../agent-ui/04-security.md), [etapas](../agent-ui/05-roadmap.md), [tarefas](../agent-ui/06-tasks.md)
  e [exemplos](../agent-ui/07-examples.md).

## Resumo

O pedido foi evoluir as interfaces para que um agente apresente, simule, compare e peça revisão de artefatos. A
investigação achou um mecanismo mais completo do que o pedido supunha: 13 componentes, inclusive galeria, sprite
animado, som com forma de onda e tabela; histórico; `idetea ask`; validação pela linha de comando; plugins WASM com
prazo. Não há expressões nem valores calculados: a RFC 0003 decidiu fazer simulações como plugins.

O uso real mudou a ordem das etapas. Em 241 documentos de três projetos, o painel serve para **decidir** e para
**entregar teste manual**; a mídia quase não foi usada (nenhum `sprite` ou `audio`, uma `gallery`). A falta que mais
aparece está nas respostas: 49 trazem comentário, e 12 dizem "tirei um print", porque a resposta não leva imagem.

## A proposta, em uma linha por etapa

1. **Anexos na resposta:** "Anexar captura da IDE" e "Anexar arquivo…"; a IDE grava em `responses/<nome>/anexos/`
   e a resposta lista os caminhos. Baixa complexidade.
2. **Decisões melhores:** `number`, `slider`, `show_if`, `columns` e `recommended` nas opções. Média.
3. **Comparar imagens:** `compare` (slider, troca, lado a lado) com zoom e pan. Média.
4. **Som:** Windows, pausa, posição, volume, A/B com `rodio` atrás de uma feature. Média; quando houver projeto
   com áudio.
5. **Valores calculados e gráficos:** expressões sem laço em `data` (`"=usuarios * mb / replicas"`), grafo de
   dependências que recusa ciclos, `chart` simples em `canvas`. Média a alta.
6. **Módulo WASM do agente:** uma função pura com combustível, prazo e memória limitados, confirmada uma vez por
   hash. Alta.
7. **Vídeo e 3D:** não agora; quadros em PNG numa galeria e `idetea:open` cobrem o caso.

O transporte continua sendo arquivo, o formato continua `idetea-ui` 1, e tudo que é novo é ignorado com aviso
por uma IDE mais velha.

## Decisões (painel, 2026-11-25)

1. **Na 0.20.0, as etapas 1 e 2:** anexos na resposta e decisões melhores (`number`, `slider`, `show_if`,
   `columns`, `recommended`).
2. **A decisão 4 da RFC 0003 é revista só para expressões sem laço**, com um avaliador próprio e pequeno (etapa 5).
   O que precisa de lógica de verdade continua em código (módulo WASM, etapa 6). Uma ADR registra isso quando a
   etapa 5 começar.
3. **A captura é só da janela da IDE**, só por clique, com miniatura e "remover" antes de enviar.
4. **Som com `rodio` só quando houver um projeto com áudio**, como a RFC 0004 decidiu.
5. **Vídeo e 3D ficam fora:** quadros em PNG numa galeria e `idetea:open` cobrem o caso.
