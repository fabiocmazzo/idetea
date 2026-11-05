# RFC 0007 — Histórico das interfaces

- **Status:** aprovada em 2026-11-04 (pelo painel), com as decisões do fim. Base: `main` em `e865359`.
- **Depende de:** [RFC 0001](0001-interfaces-declarativas.md) (painel Interfaces) e
  [RFC 0003](0003-interfaces-pelo-uso.md) (quem respondeu, `ide` na resposta).
- **Origem:** pedido do usuário no chat (2026-11-04): "manter um histórico de todas as perguntas geradas pelos
  agentes (pode ser simplesmente movendo para outro diretório o que foi respondido), e continua aparecendo a última
  como sempre. Mas ter uma barra lateral esquerda, que pode ocupar toda a parte esquerda, mas que vem oculta por
  padrão, lá mostra todas as interfaces respondidas, com suas respostas, quando foi respondida, e as não
  respondidas […] não tem problema responder fora de ordem pq os agentes são responsáveis por manter o estado."

Marcadores: **[Fato]** conferido no código ou no uso; **[Hipótese]** provável, não testado;
**[Recomendação]** proposta, ainda não decidida.

## Resumo

**Cabe no escopo e não é salto de versão grande.** O histórico não muda o protocolo: o formato `idetea-ui`, as
respostas e as instruções dos agentes ficam iguais. Quem guarda é a IDE, no momento do clique, e não o agente. Assim
funciona com qualquer agente, inclusive os que já estão rodando com as instruções `1e2f7175`, e nenhum
`AGENTS.md`/`CLAUDE.md` precisa ser reensinado. Em versão 0.x, recurso novo sem quebra é versão menor: **0.2.0**
(ou junto com outras coisas numa 0.2.0).

Vale pelo que o usuário pediu (rever o que respondeu e quando) e por três coisas que apareceram nesta mesma
sessão: (1) a resposta da pergunta "Site do IDE TEA" (revisão 2) que nunca chegou ao agente e ninguém conseguiu
explicar; com histórico, daria para ver se o clique foi gravado. (2) Uma resposta da interface `skills` de
2026-11-02 está até hoje em `.idetea/ui/responses/skills/`: o agente nunca a leu. O histórico mostra isso. (3)
Depois de uma compactação de contexto, o agente perde o que o usuário respondeu; o histórico é o registro.

## 1. Como é hoje [Fato]

- O painel lê `.idetea/ui/*.json` (`store::scan`). Com mais de uma interface aberta, mostra uma fileira de botões,
  uma por interface (`lib.rs`, `view`); uma interface nova ou atualizada passa a ser a selecionada.
- Cada clique grava `.idetea/ui/responses/<nome>/<seq>-<evento>.json`. O agente lê e apaga. Quando o agente apaga
  o documento e não sobra resposta, a pasta some (`clean_responses`).
- Depois disso, **nada fica**: nem a pergunta, nem a resposta, nem quando foi. O único rastro é o que o agente
  guardou no próprio contexto.
- Responder fora de ordem já funciona: cada interface é independente, e o estado é do agente. A RFC não muda isso.

## 2. Quem guarda: a IDE, não o agente [Recomendação]

A ideia original era "mover para outro diretório o que foi respondido". Se o agente movesse, seria mais um passo
no protocolo (o tipo de peso que a [RFC 0006](0006-ide-como-cli-para-agentes.md) tirou) e um agente que esquece
deixaria buracos. A IDE já tem tudo na mão no momento do clique: o documento, a revisão que o usuário via, os
valores e o comentário. Então:

- **No clique**, junto com a resposta para o agente, a IDE grava uma entrada no histórico: o documento daquela
  revisão (cópia) e a resposta inteira.
- **Quando o agente apaga a resposta**, a IDE anota na entrada "lida pelo agente" e a hora. Resposta que nunca é
  apagada aparece como "não lida", que é exatamente o caso da `skills`.
- **Quando o agente apaga o documento** sem que ninguém tenha clicado, a IDE grava uma entrada "fechada sem
  resposta" (com a última revisão). Documentos só de leitura (sem botões, só para mostrar algo) entram do mesmo
  jeito: são parte da conversa.
- Uma revisão nova do mesmo documento substitui a anterior no painel, como hoje; no histórico, cada revisão
  respondida fica.

### Onde

`.idetea/history/<AAAA-MM-DD>/<hora>-<nome>-r<revisão>.json`, um arquivo por entrada, no formato
`idetea-ui-history`:

```json
{"format": "idetea-ui-history", "version": 1, "name": "passo", "title": "Próximo passo",
 "source": "claude", "revision": 2, "shown": "2026-11-04T19:30:12Z",
 "outcome": "answered", "answered": "2026-11-04T19:32:40Z", "read_by_agent": "2026-11-04T19:32:41Z",
 "document": { … o documento daquela revisão … },
 "response": { … a resposta, igual à que o agente recebeu … }}
```

`outcome`: `answered`, `closed` (o agente apagou sem resposta) ou `shown` (sem botões). Sem `read_by_agent`, a
resposta ainda está lá ou foi perdida.

- Fica no projeto porque é do projeto (o agente daquele projeto pode querer ler) e porque some junto quando o
  projeto é apagado. Entra no `.idetea/.gitignore` que a IDE já escreve (`/history/`), como o `state.json`.
- Um arquivo por entrada: nada de reescrever um arquivo grande, e apagar o antigo é apagar arquivos.
- Mídia (imagens, sons, arquivos de `file`) **não** é copiada: a entrada guarda o caminho, o `size` e o `modified`
  que a resposta já traz. Se o arquivo mudou ou sumiu, o histórico diz isso em vez de mostrar uma imagem errada.

### Quanto tempo

Proposta: `[ui] history_days = 90` e um teto de tamanho. **Decidido:** para sempre, até o usuário apagar (ver as
decisões no fim).

## 3. A barra lateral [Recomendação]

Um painel novo, **Histórico das interfaces**, no lado esquerdo (`Slot::Left`, o mesmo lugar da árvore), oculto por
padrão. Abre pela barra de ícones, pela paleta ("Interfaces: histórico") e por um botão discreto no topo do painel
Interfaces. Pode ser alargado até ocupar o lado esquerdo inteiro, arrastando a borda, como os outros.

```
┌ Histórico ─────────────────────── [⤢] ┐
│ [ buscar…                         ]   │
│ ● Abertas (2)                         │
│   Próximo passo        claude  19:30  │
│   Cache backend        codex   19:12  │
│ Hoje                                  │
│   Site do IDE TEA r2   ✓ ok    18:55  │
│   Teste do GIF         ✓ merge 17:40  │
│   skills               ! não lida     │
│ Ontem                                 │
│   …                                   │
└───────────────────────────────────────┘
```

- **Abertas** no topo: as interfaces que estão em `.idetea/ui/` agora. Clicar seleciona ela no painel
  Interfaces, que continua sendo onde se responde. Com várias, responder em qualquer ordem, como já funciona.
- **Respondidas**, por dia, mais novas primeiro: título, quem perguntou (`source`), o botão clicado e a hora.
  Marcas para "fechada sem resposta" e "não lida pelo agente".
- **Abrir uma entrada** mostra a interface como estava, só para leitura: os campos com os valores respondidos, o
  botão clicado destacado, os outros desligados, o comentário e as horas (mostrada, respondida, lida). Reaproveita
  o desenho do painel Interfaces (`surface_view`) com uma bandeira de leitura.
- **Busca** pelo título e pelo texto das respostas e comentários; **filtro** por agente.
- **Fora do escopo:** responder de novo uma interface do histórico. O agente já apagou o documento e seguiu em
  frente; uma resposta nova não teria quem lesse. Se for preciso, copiar o texto e dizer no chat.

## 4. Para o agente (opcional, etapa C) [Recomendação]

`idetea history` (com `--json`, `--name`, `--since`) lista o que o usuário respondeu. Serve para o agente que
perdeu o contexto numa compactação, ou para um agente novo no projeto. Seria uma linha nas instruções, então muda
a versão delas; por isso fica numa etapa à parte, decidida depois de usar A e B.

## 5. Privacidade e segurança [Recomendação]

- As instruções já dizem para não pôr segredos nos documentos. Mesmo assim, o histórico guarda texto que o usuário
  digitou (inputs, comentários). Fica só no disco do usuário, fora do git, com prazo e com um botão "Apagar
  histórico" (com confirmação) no painel e na paleta.
- A pasta `.idetea/history/` segue as regras da `.idetea/ui/`: sem symlink, arquivo novo com `create_new`, nome
  validado, temporário e `rename`.
- Um repositório clonado que traga `.idetea/history/` versionado: a IDE ignora entradas versionadas, como já faz
  com documentos versionados (`tracked`).
- O histórico nunca vira entrada para o painel: só a IDE escreve, só a barra lê.

## 6. Etapas [Recomendação]

| Etapa | O quê | Muda o protocolo? |
|---|---|---|
| A | Gravar o histórico (clique, lida, fechada, mostrada), prazo e limpeza, `.gitignore` | Não |
| B | Barra lateral: abertas, lista por dia, leitura, busca, apagar histórico | Não |
| C | `idetea history` para agentes, com uma linha nas instruções | Sim (versão das instruções) |

A e B saem juntas numa 0.2.0. A sozinha já ajuda a investigar casos como o da revisão 2 do "Site do IDE TEA".

## 7. Custo [Hipótese]

- A: pequeno. Um módulo `history.rs` no `tea-ext-ui`, chamado em `write_response` e no `scan` (quando um documento
  ou uma resposta some). Testes no mesmo estilo dos de `store.rs`.
- B: médio. Um painel novo, mas a leitura reaproveita o desenho que já existe; o trabalho maior é preencher os
  componentes com os valores da resposta em vez do `data`.
- Desempenho: um arquivo pequeno por clique. A barra lê o índice (título, hora, resultado) e só abre o documento
  inteiro ao clicar numa entrada.

## 8. Para decidir

1. Prazo padrão: 90 dias? Ligado por padrão?
2. Onde guardar: no projeto (`.idetea/history/`, recomendado) ou na pasta de dados do usuário
   (`~/.local/share/idetea/`), que não suja o projeto mas fica longe dele?
3. Entram também os documentos só de leitura e os fechados sem resposta, ou só os respondidos?
4. A etapa C (`idetea history` para agentes) entra no plano, ou espera o uso de A e B?
5. Versão: 0.2.0 só com isto, ou junto com outras coisas?

## Decisões (pelo painel, 2026-11-04)

1. **Prazo:** para sempre. O histórico só sai quando o usuário manda ("Interfaces: apagar o histórico", com
   confirmação); sem `history_days` nem teto de tamanho por enquanto.
2. **Onde:** no projeto, `.idetea/history/`, fora do git.
3. **O que entra:** tudo: respondidas, fechadas sem resposta e as só de leitura. Revisões intermediárias sem
   resposta (substituídas por outra) não entram: só a última, quando o agente apaga o documento.
4. **Etapa C já entra:** `idetea history`, com a linha nas instruções (versão `4503cd30`).
5. **Versão:** 0.2.0 com A, B e C.
