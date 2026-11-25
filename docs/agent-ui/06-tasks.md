# 06 — Tarefas

Cada tarefa é pequena o bastante para uma revisão. **Paralela** quer dizer que mexe em arquivos que outra tarefa
paralela não mexe; as outras entram em sequência. A maioria passa por `document.rs`, `view.rs`, o schema e o
`formato.md`: duas tarefas que mexem neles ao mesmo tempo conflitam, então a paralelização é entre **etapas**, não
dentro delas.

Prioridade: **P1** para a 0.20.0, **P2** depois, **P3** só se um projeto pedir.

## Etapa 1 — Anexos (P1)

| Id | Tarefa | Depende | Aceite |
| --- | --- | --- | --- |
| T1.1 | `Response.attachments` (`path`, `kind`, `size`, `width`, `height`, `from`) e a gravação em `responses/<nome>/anexos/<seq>-<n>.png` | — | teste: resposta com dois anexos, caminhos relativos à raiz, arquivos existem |
| T1.2 | "Anexar captura da IDE": `window::screenshot`, PNG pela `image`, miniatura com "remover" antes do clique | T1.1 | manual: a miniatura aparece; a resposta leva o PNG |
| T1.3 | "Anexar arquivo…": `rfd`, cópia para `anexos/`, limite de 8 MB, `from` com `~` | T1.1 | teste da cópia e do limite; manual do diálogo |
| T1.4 | `attach` em `button` (espera um anexo, como `requires`) | T1.1 | teste em `state.rs`: `waiting()` conta o anexo |
| T1.5 | `idetea ui fix` acha `anexos/` sem resposta; o histórico guarda os caminhos | T1.1 | teste do `fix` |
| T1.6 | `formato.md`, schema, `teach.rs` (instrução: "leia os anexos e apague a pasta junto com a resposta") e o `idetea ask` imprime os anexos | T1.1–T1.4 | `idetea ask --json` mostra `attachments` |

## Etapa 2 — Decisões (P1)

| Id | Tarefa | Depende | Aceite |
| --- | --- | --- | --- |
| T2.1 | `number`: parse (`min`, `max`, `step`, `unit`), valor número JSON, aviso fora do intervalo | — | testes de parse e de valor |
| T2.2 | `slider` em cima do `slider` do iced, mesmo modelo do `number` | T2.1 | teste de parse; manual |
| T2.3 | `show_if` em qualquer componente (`equals`, `not_equals`, `in`, `truthy`); `requires` ignora escondido | — | testes em `state.rs` |
| T2.4 | `columns` (até 4 ids, sem aninhar, erro em id inexistente ou repetido; volta a coluna no estreito) | — | testes de parse; manual em painel estreito |
| T2.5 | `recommended` e `note` em `Choice` (select e, por coerência, itens de `gallery`) | — | teste de parse; manual |
| T2.6 | `formato.md`, schema, `teach.rs`, `check.rs` | T2.1–T2.5 | os exemplos 1 e 4 de [07](07-examples.md) passam no `idetea ui check` |

## Etapa 3 — Comparar imagens (P2)

| Id | Tarefa | Depende | Aceite |
| --- | --- | --- | --- |
| T3.1 | `compare`: parse e leitura das duas imagens com os limites de `MediaLimits` | — | testes de parse e limite |
| T3.2 | Desenho em `canvas`: `slider`, `toggle`, `side`; estado do slider em `Rendered` | T3.1 | manual |
| T3.3 | Zoom e pan sincronizados, `pixelated`, fundo quadriculado | T3.2 | manual com pixel art |
| T3.4 | `media.compared` na resposta; documentação | T3.2 | teste da resposta |

## Etapa 4 — Som (P3, quando houver projeto com áudio)

| Id | Tarefa | Depende | Aceite |
| --- | --- | --- | --- |
| T4.1 | Feature `audio` com `rodio`: tocar, pausar, posição, volume, numa thread | — | manual nos três sistemas |
| T4.2 | Linha de posição na forma de onda (o WAV já tem a forma) e clique para pular | T4.1 | manual |
| T4.3 | `"compare": ["a", "b"]` em `audio`: alterna os dois na mesma posição | T4.1 | manual |
| T4.4 | Pacotes da release com a feature; CI com `libasound2-dev` | T4.1 | release com som no Windows |

## Etapa 5 — Valores calculados (P2)

| Id | Tarefa | Depende | Aceite |
| --- | --- | --- | --- |
| T5.1 | Avaliador: análise, tipos, funções, limite de 256 nós | — | testes de cada operador, divisão por zero, `NaN` |
| T5.2 | Grafo: dependências, ordem, ciclo e referência inválida recusam o documento com o caminho | T5.1 | testes |
| T5.3 | `State.computed`, refeito a cada `Set`; `computed` na resposta | T5.2 | testes em `state.rs` |
| T5.4 | `{/caminho}` em `text` e `markdown` | T5.3 | teste de desenho |
| T5.5 | `chart` (`bar`, `line`) em `canvas` | T5.3 | manual com o exemplo 5 de [07](07-examples.md) |
| T5.6 | Documentação; nova ADR revendo a decisão 4 da RFC 0003 | T5.1–T5.5 | ADR aceita |

## Etapa 6 — Módulo do agente (P3)

| Id | Tarefa | Depende | Aceite |
| --- | --- | --- | --- |
| T6.1 | WIT mínimo `calcular` e runtime com combustível, prazo e `StoreLimits`, reusando `tea-plugins` | T5.3 | teste: laço infinito cortado em 100 ms; 64 MB respeitado |
| T6.2 | Confirmação por hash, `[ui] modules = "ask" | "never"` | T6.1 | teste: hash novo pede; repetido não |
| T6.3 | Exemplo compilado e guia para o agente compilar | T6.1 | o exemplo roda em menos de 100 ms |

## O que pode ir em paralelo

- **Agora:** etapa 1 (`response.rs`, `store.rs`, botões do comentário) e etapa 2 (`document.rs`, componentes novos)
  mexem em partes diferentes de `view.rs`; dá para fazer em duas worktrees e integrar a 1 primeiro (é menor).
- **Depois:** 3 e 5 em paralelo; 4 independente de todas; 6 só depois da 5.
- **Sempre em sequência:** `formato.md`, schema e `teach.rs` de cada etapa, porque todas mexem neles.
