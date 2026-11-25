# 05 — Etapas

A ordem do pedido original (`ideia.md`) foi mudada pelo que o uso mostrou ([01](01-current-state.md#o-que-o-uso-mostra)):
o painel é usado para decidir e para entregar teste manual, e a falta mais repetida é **mandar o que se viu** (12
comentários "tirei um print"). Mídia de jogo (sprite, áudio) já existe e não teve uso; ela cresce quando um
projeto com assets pedir.

| Etapa | O que entrega | Complexidade | Depende de |
| --- | --- | --- | --- |
| 0 | Esta auditoria e a RFC 0026 | — | — |
| 1 | Anexos na resposta: captura da janela da IDE e arquivo escolhido | baixa | — |
| 2 | Decisões melhores: `number`, `slider`, `show_if`, `columns`, `recommended` | média | — |
| 3 | Comparar imagens: `compare` (slider, troca, lado a lado) com zoom e pan | média | — |
| 4 | Som: Windows, pausa, posição, volume e comparação A/B | média | — |
| 5 | Valores calculados e gráficos simples | média a alta | 2 |
| 6 | Módulo WASM do agente | alta | 5 |
| 7 | Vídeo | não recomendado agora | — |
| 8 | Ferramentas externas | já existe (`idetea ask`, terminais pedidos) | — |

1, 2, 3 e 4 são independentes e podem ir em paralelo. 5 precisa do `number` da 2 para ser útil. 6 precisa do
grafo de valores da 5.

**[Recomendação] para a 0.20.0:** etapas 1 e 2. Juntas cobrem os cenários A e D do pedido e a falta que o uso
aponta.

## Etapa 1 — Anexos na resposta

- **Valor:** o usuário manda a captura junto com "Falhou", sem sair do painel e sem dizer onde salvou.
- **Escolhas técnicas:** **[Fato]** o iced 0.14 tem `window::screenshot(id) -> Task<Screenshot>` (RGBA da janela).
  A janela inteira, codificada em PNG pela crate `image`, que já é dependência. "Anexar arquivo…" usa o `rfd`, que o
  `tea-app` já usa. **[Hipótese]** Colar uma imagem da área de transferência pede `arboard`; o `clipboard::read` do
  iced só lê texto. Fica fora desta etapa.
- **Arquivos:** `tea-declarative-ui` (`response.rs`: `attachments`; `view.rs`: miniaturas e botões),
  `tea-ext-ui` (`lib.rs`: pedir a captura e gravar; `store.rs`: pasta `anexos/`; `fix.rs`: órfãos),
  `docs/interfaces/formato.md`, `teach.rs` (instruções), schema JSON.
- **Riscos:** a captura pega o que estiver na janela, inclusive um segredo num terminal. Mitigação: só por clique,
  miniatura antes de enviar, botão de remover.
- **Testes:** gravar a resposta com anexo (caminho, tamanho, `kind`); `fix` acha anexo órfão; PNG com o limite de
  pixels.
- **Ponta a ponta:** o agente entrega um teste manual; o usuário clica "Anexar captura da IDE" e "Falhou"; o agente
  lê o PNG e corrige.
- **Concluída quando:** a resposta tem `attachments` com um PNG que o agente abre, e o `idetea ask --json` imprime
  os caminhos.

## Etapa 2 — Decisões melhores

- **Valor:** perguntas com número (tamanho, réplicas, prazo), campos que só aparecem quando fazem sentido, opções
  comparadas lado a lado com a recomendada marcada.
- **Arquivos:** `document.rs` (`Kind::Number`, `Kind::Slider`, `Kind::Columns`, `show_if` em `Component`,
  `recommended`/`note` em `Choice`), `state.rs` (`requires` ignora escondidos), `view.rs`, schema, `formato.md`,
  `teach.rs`, `check.rs`.
- **Riscos:** `columns` em painel estreito. Mitigação: abaixo de uma largura, vira coluna.
- **Testes:** parse de cada campo, número fora do intervalo, `show_if` com cada operador, `requires` com campo
  escondido, `columns` com id inexistente (erro) ou repetido (erro).
- **Ponta a ponta:** cenário A (exemplo 1 de [07](07-examples.md)).
- **Concluída quando:** os exemplos 1 e 4 de [07](07-examples.md) passam no `idetea ui check` e funcionam no painel.

## Etapa 3 — Comparar imagens

- **Valor:** antes/depois de uma tela, de um sprite, de um gráfico gerado; zoom em pixel art.
- **Escolhas:** `canvas` do iced para o slider e para zoom/pan (a aba de imagem já faz zoom com `pixelated`).
- **Testes:** limites de pixel valem para as duas imagens; `media.compared` na resposta.
- **Concluída quando:** o exemplo 2 de [07](07-examples.md) funciona, com zoom sincronizado.

## Etapa 4 — Som

- **Valor:** comparar efeitos e vozes (cenário C) no Windows também, com pausa e posição.
- **Escolhas:** **[Fato]** a [RFC 0004](../rfc/0004-midia-no-painel.md) decidiu "tocador do sistema; `rodio` só se o
  uso pedir seek ou loop sem intervalo". Esta etapa é esse pedido, e só deve entrar quando um projeto com áudio
  existir. **[Fato]** O `rodio` 0.22 tem `try_seek` e decodifica pelo `symphonia`; no Linux liga com a `libasound`
  (headers para compilar). **[Recomendação]** `rodio` atrás de uma feature `audio`, ligada nos pacotes da release; o
  tocador do sistema continua quando a feature está desligada.
- **Concluída quando:** o exemplo 3 de [07](07-examples.md) toca, pausa, pula e compara A/B nos três sistemas.

## Etapa 5 — Valores calculados e gráficos

- **Valor:** cenário E sem código: o usuário mexe em usuários e réplicas e vê a memória e se cabe.
- **Decisão que muda:** a [RFC 0003](../rfc/0003-interfaces-pelo-uso.md) decidiu "simulações como plugins, sem
  expressões no JSON". A proposta é rever essa decisão **só** para expressões sem laço; o que precisa de lógica de
  verdade continua em código (etapa 6).
- **Escolhas:** **[Fato]** CEL é uma linguagem não Turing-completa, feita para avaliar expressões com segurança; a
  crate `cel` é o interpretador em Rust. **[Recomendação]** Um avaliador próprio, pequeno (aritmética, comparação,
  `if`, `min`, `max`, `round`, `sum`), porque o conjunto é pequeno, os erros saem em português com o caminho, e não
  entra uma dependência para isso. A CEL fica como alternativa se a lista de funções crescer.
- **Gráfico:** `canvas` do iced, barras e linhas. **[Fato]** O `plotters-iced` original não tem versão para o iced
  0.14; o fork `plotters-iced2` 0.14 tem. Não vale uma dependência para dois tipos de gráfico.
- **Testes:** ciclo, referência inválida, divisão por zero, ordem do grafo, `computed` na resposta, 200 valores no
  limite.

## Etapa 6 — Módulo do agente

- **Valor:** uma estimativa que não cabe numa expressão (tabela de preços, simulação de fila).
- **Escolhas:** o runtime de `tea-plugins` (wasmtime 50, prazo por época), com um WIT próprio e mínimo
  (`calcular: func(entrada: string) -> string`), combustível, `StoreLimits` e confirmação por hash.
- **Riscos:** o agente precisa compilar um componente WASM (`cargo component`, `jco`), o que pode ser mais difícil
  que escrever a expressão. Medir com um projeto real antes de seguir.
- **Concluída quando:** um módulo de exemplo responde em menos de 100 ms, um módulo com laço infinito é cortado sem
  travar o painel, e um módulo novo pede confirmação.

## Etapa 7 — Vídeo (não recomendado agora)

**[Fato]** O `iced_video_player` publicado (0.5/0.6) não acompanha o iced 0.14 (há um fork), e todos usam GStreamer,
que é pesado e ligado ao sistema. **[Recomendação]** O agente extrai quadros com `ffmpeg` em PNG e mostra numa
`gallery` ou num `sprite`; `idetea:open` abre o vídeo no player do sistema. Rever se um projeto pedir.

## Etapa 8 — Ferramentas externas

Já existe: `idetea ask`, `idetea run`, `idetea agent`, terminais pedidos com worktree e `IDETEA_ANSWER`. Nada novo
até haver um caso que não caiba neles.

## Fora

3D (glTF): nenhum projeto pediu; o custo (renderizador, materiais) não se paga. Anotar regiões de imagem e trechos
de áudio: o anexo da etapa 1 e o comentário cobrem por enquanto.
