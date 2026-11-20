# RFC 0023 — Sessão ACP: acabamento antes da versão e integração com o editor

- **Status:** aprovada no painel em 2026-11-20: marco 2a inteiro e o item 8 do 2b antes da versão. Base: `main` em `7ba0695` (cliente ACP v1, #17).
- **Continua:** [RFC 0022](0022-cliente-acp.md). Não muda as decisões dela: ACP v1 estável, crate
  `agent-client-protocol` =5.4.0, permissão explícita e sem auto-aprovação, hooks do mascote intactos.
- **Depende de:** [RFC 0012](0012-chat-multimodelo-e-mascote.md) (mascote) e
  [RFC 0016](0016-lancador-de-agentes-e-memoria-do-projeto.md) (lançador, `[agents]` só no config do usuário).

Marcadores: **[Fato]** conferido nesta sessão no código ou na fonte da crate (`agent-client-protocol` 5.4.0,
`agent-client-protocol-schema` 1.19.0); **[Referência]** dito por documentação externa, não conferido;
**[Hipótese]** provável, não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

O marco 1 da RFC 0022 fez o caminho funcionar: abrir a sessão, conversar, aprovar cada ação, várias abas, e o
estado no mascote. O que falta é o que se sente no uso: a resposta chega como texto cru (sem Markdown, sem copiar),
a caixa de mensagem é de uma linha, o pedido de permissão mostra JSON, a ferramenta é uma linha que não abre nada,
e a sessão não sabe nada do editor (nem o arquivo aberto, nem o que não foi salvo).

**[Recomendação]** Três marcos:

- **2a — antes da versão:** um bug de turnos simultâneos, legibilidade (Markdown, copiar, entrada multilinha,
  permissão e ferramentas legíveis) e recomeçar uma sessão encerrada. Nada de protocolo novo além do que a 5.4.0
  já entrega e o cliente hoje descarta.
- **2b — a sessão conhece o editor:** mandar o arquivo e a seleção como contexto, `fs` (o agente lê o buffer não
  salvo e escreve pelo editor, com desfazer), comandos `/` do agente, plano, modo e modelo.
- **3 — depois da versão:** terminais ACP, retomar sessão, agentes configuráveis, uso de contexto.

## O que existe hoje

Sessão (`crates/tea-acp/src/session.rs`):

- **[Fato]** O cliente anuncia só `auth.terminal` no `initialize` (linha 135). Não anuncia `fs.readTextFile`,
  `fs.writeTextFile` nem `terminal`, então o agente lê e grava direto no disco, sem passar pelo editor.
- **[Fato]** `apply_update` trata `AgentMessageChunk`, `AgentThoughtChunk`, `ToolCall` e `ToolCallUpdate`; o resto
  cai em `_ => {}` (linha 310). Descartados: `Plan`, `AvailableCommandsUpdate`, `CurrentModeUpdate`,
  `ConfigOptionUpdate`, `SessionInfoUpdate`, `UsageUpdate`, `Notice`, `UserMessageChunk`. Todos existem no v1 do
  schema 1.19.0 **sem** feature instável.
- **[Fato]** Da ferramenta, só passam `id`, `title` e `status`. `kind` (`read`, `edit`, `execute`…), `locations`
  (arquivo e linha) e `content` (`Content`, `Diff`, `Terminal`) se perdem.
- **[Fato]** `block_text` aceita só `ContentBlock::Text` (linha 314): imagem ou link de recurso na resposta somem.
- **[Fato]** O texto da permissão é `raw_input.to_string()` (linha 286): JSON numa linha só.
- **[Fato]** `Ready { agent }` leva o **id da sessão** (linha 160), não o nome do agente; o `agent_info` do
  `initialize` não é lido.
- **[Fato]** O prompt é sempre um bloco de texto (`send_prompt`); `promptCapabilities` do agente não é lido.
- **[Fato]** A mensagem de pânico é fixa em português (linha 115), fora do i18n.

Painel (`crates/ext/tea-ext-acp/src/lib.rs`):

- **[Fato]** O botão Enviar só confere se a sessão está ligada (`s.cmd`, linha 387); `Send` (linha 588) manda
  `Command::Prompt` mesmo com um turno em andamento, e a sessão chama `send_prompt` na hora (session.rs, linha 203).
  Só na espera de uma permissão a mensagem vai para a fila. **[Hipótese]** Dois `session/prompt` ao mesmo tempo
  na mesma sessão: o v1 não prevê; cada agente reage de um jeito (erro, cancelar o primeiro, ou misturar). E o
  primeiro `PromptResponse` já põe `busy = false` com o segundo turno ainda rodando: some o botão Cancelar.
- **[Fato]** A resposta é um `text` simples (linha 445): sem Markdown, sem bloco de código, sem selecionar ou
  copiar. O workspace já usa `iced` com a feature `markdown` (`tea-ext-preview`, `tea-declarative-ui`).
- **[Fato]** A entrada é um `text_input` de uma linha (linha 393): sem quebra de linha, sem histórico.
- **[Fato]** A linha de ferramenta é texto (`título  status`), sem ícone por tipo e sem clique.
- **[Fato]** O estado da sessão (pensando, ferramenta, fim do turno) só vai para o mascote. Com
  `[mascot] enabled = false`, a aba não mostra estado nenhum além do botão Cancelar.
- **[Fato]** Sessão encerrada (processo saiu, login exigido) fica com a conversa, mas não há como recomeçar na
  mesma aba: é abrir outra e fechar esta. Depois do login no terminal, idem.
- **[Fato]** O erro `shutdown` é reconhecido comparando a mensagem como texto (linha 284).

## Marco 2a — antes da versão

### 1. Um turno por vez, com fila visível

**[Recomendação]** Enviar com um turno em andamento põe a mensagem na **fila da sessão** (a mesma que já existe
para a espera de permissão). A aba mostra a mensagem como "na fila" (texto apagado) até o turno acabar e ela ser
enviada. Cancelar o turno não esvazia a fila; você tira uma mensagem dela pelo ✕. `busy` passa a
contar turnos abertos, não um booleano.

Teste: no stub, dois `Send` seguidos produzem dois `session/prompt` **em sequência**, nunca sobrepostos.

### 2. Markdown e copiar

**[Recomendação]** A resposta do agente passa pelo `iced::widget::markdown` (o mesmo do preview), com o tema da
IDE e realce nos blocos de código. Cada bloco de código ganha **Copiar**; cada resposta ganha **Copiar resposta**
(o texto Markdown original). Parse incremental: o Markdown é reconstruído só para a última resposta enquanto ela
cresce, as anteriores ficam em cache.

**[Hipótese]** Re-parsear a resposta inteira a cada pedaço fica caro em respostas longas; o `markdown::Content`
do iced aceita `push_str`, que é o caminho a medir primeiro.

### 3. Entrada de várias linhas e histórico

**[Recomendação]** `text_editor` no lugar do `text_input`: **Enter** envia, **Shift+Enter** quebra a linha
(o mesmo atalho do Claude Code e do OpenCode), altura de 1 a 8 linhas. **↑** na entrada vazia traz a última
mensagem enviada nesta aba. Um comando "Agente (ACP): focar a mensagem" com atalho (sem conflito com
`Ctrl+Shift+A`, que é o lançador de terminal).

### 4. Permissão legível

**[Recomendação]** O pedido mostra o que importa pelo `kind` da ferramenta:

| `kind` | O que aparece |
| --- | --- |
| `execute` | o comando, em bloco de código, e a pasta |
| `edit` | o diff (`ToolCallContent::Diff`: caminho, antes, depois), em linhas `+`/`−` |
| `read`, `delete`, `move` | o caminho (clicável) |
| `fetch` | a URL |
| outro | o `raw_input` em JSON **indentado**, recolhido por padrão |

O diálogo continua com todos os botões secundários (Enter e Esc não aprovam, RFC 0022).

### 5. Ferramentas que abrem alguma coisa

**[Recomendação]** A linha da ferramenta ganha o ícone do `kind` e, quando há `locations`, o caminho vira link que
abre o arquivo na linha. Um clique na linha expande o `content`: a saída (texto), o diff de uma edição. Recolhida
por padrão, para a conversa continuar curta.

### 6. Estado na aba quando o mascote está desligado

**[Recomendação]** Com o mascote ligado, nada muda (a decisão de tirar o estado da aba foi testada e aprovada).
Com `[mascot] enabled = false`, uma linha discreta acima da entrada mostra o mesmo `AgentCue` ("pensando…",
a ferramenta, "terminou"). As abas de trás ganham um ponto animado enquanto trabalham, nos dois casos.

### 7. Recomeçar e erros

- **[Recomendação]** Aba encerrada mostra **Recomeçar** (abre uma sessão nova do mesmo alvo, na mesma aba, com a
  conversa anterior acima de um separador). Depois do login no terminal (`acp-login-done`), o botão aparece em
  destaque.
- **[Recomendação]** O nome do agente e a versão do `agent_info` vão para o `Ready` e para a dica da aba.
- **[Recomendação]** `max_tokens`, `max_turn_requests` e `refusal` deixam uma nota curta na aba (hoje só o mascote
  sabe), porque explicam por que a resposta parou no meio.
- **[Recomendação]** O pânico e os demais textos de `tea-acp` passam a ser códigos (`SessionEvent::Failed` com um
  tipo de erro), traduzidos no painel; `shutdown` deixa de ser comparado como texto.
- **[Fato]** O `AcpAgent` da 5.4.0 já põe o fim do stderr no erro quando o processo sai com código diferente de
  zero. **[Recomendação]** Mostrar esse trecho num bloco recolhido junto do aviso.

## Marco 2b — a sessão conhece o editor

### 8. Contexto: arquivo, seleção, `@`

**[Recomendação]**

- Comando **"Agente (ACP): perguntar sobre a seleção"** (também no menu de contexto do editor): abre ou traz à
  frente a sessão e põe na entrada um chip com o arquivo e o intervalo de linhas.
- Na entrada, `@` abre o seletor de arquivos do projeto (o mesmo do Ctrl+P); o escolhido vira chip.
- Ao enviar, cada chip vira `ContentBlock::ResourceLink` (todo agente v1 aceita) ou, se o agente anunciou
  `promptCapabilities.embeddedContext`, `ContentBlock::Resource` com o texto **do buffer** (inclusive não salvo).
- Imagem colada só se `promptCapabilities.image`.

### 9. `fs`: o agente lê e escreve pelo editor

**[Recomendação]** Anunciar `fs.readTextFile` e `fs.writeTextFile`.

- `fs/read_text_file` de um arquivo aberto devolve o buffer, **com** as mudanças não salvas; os outros, do disco.
- `fs/write_text_file` de um arquivo aberto troca o conteúdo do buffer como **uma** edição desfazível e salva;
  de um arquivo fechado, grava no disco. A aba do arquivo pisca a faixa de "mudado pelo agente".
- Só caminhos dentro da raiz do projeto; fora dela, erro (a permissão do agente não alarga isso).

**[Hipótese]** Hoje, com o agente gravando direto no disco, um buffer aberto **com mudanças não salvas** e o
arquivo mudado por fora entram no fluxo de conflito do editor; com `fs`, isso deixa de acontecer para as escritas
da sessão. Testar com o Claude pelo adaptador, que usa `fs` quando o cliente anuncia **[Referência]**.

### 10. Comandos `/`, plano, modo e modelo

- **Comandos:** `AvailableCommandsUpdate` alimenta um autocompletar quando a entrada começa com `/`.
- **Plano:** `Plan` vira uma lista de tarefas fixa acima da conversa (pendente, em andamento, feita), recolhível.
  O mascote usa a tarefa em andamento como "trabalhando".
- **Modo:** os modos da `session/new` e `CurrentModeUpdate` viram um seletor na aba (no Claude: padrão, aceitar
  edições, plano…); trocar manda `session/set_mode`. **[Recomendação]** Um modo que aprova sozinho (o
  "bypass" do Claude) aparece com aviso, e a IDE continua respondendo cada `request_permission` que chegar.
- **Modelo e opções:** `configOptions` e `ConfigOptionUpdate` viram um seletor; `session/set_config_option`.

### 11. Arquivos mudados no turno

**[Recomendação]** No fim do turno, uma linha "N arquivos mudados" (a partir dos `Diff` e das escritas via `fs`)
que abre a lista; cada arquivo abre o diff do turno. É o "diffs da sessão" que a RFC 0022 deixou de fora.

## Marco 3 — depois da versão

- **Terminais ACP** (`terminal: true`): o comando que o agente roda aparece numa aba de terminal da IDE, que você
  vê e pode interromper; a linha da ferramenta liga para ela.
- **Retomar sessão:** `session/load` quando o agente anuncia `loadSession`; a conversa da aba guardada em
  `.idetea/` (fora do git) para reabrir depois de fechar a IDE.
- **Agentes configuráveis:** `[[agents.acp]]` no config **do usuário** (nome, programa, args), para Codex, Gemini
  CLI e outros, sem código novo por alvo. O do projeto continua ignorado (RFC 0016).
- **Uso de contexto:** `UsageUpdate` na dica da aba; `Notice` e compactação como nota na conversa.
- **`Ctrl+L` como `session/prompt`**, adiado pela RFC 0022.

## Fora desta RFC

- ACP v2 e as features `unstable_*` da crate (sessões filhas, cursores, operações de plano).
- Auto-aprovação e listas de permissão (decisão 4 da RFC 0022).
- Chat que chama API de modelo (opção A da RFC 0012).

## Decisões

Escolhidas no painel em 2026-11-20, todas na recomendação:

1. **O marco 2a inteiro entra antes da versão.** O item 1 é bug; 2–5 são o que mais pesa no uso.
2. **Do 2b, só o item 8 (contexto) entra antes da versão.** `fs` (9) e os itens 10–11 vêm depois, com tempo para
   testar o fluxo de conflito.
3. **`fs/write_text_file` edita o buffer e salva**, porque o agente espera ler do disco o que escreveu
   (ferramentas de terminal, testes).
4. **O estado na aba (item 6) só aparece com o mascote desligado.**

## Testes

- Stub (`tea-acp-stub`): fila de turnos (1), `Diff`, `locations` e `kind` nas ferramentas (4, 5), `Plan`,
  `AvailableCommandsUpdate`, modos (10), `fs/read_text_file` e `fs/write_text_file` (9) com buffer sujo.
- Painel: Markdown incremental, fila, Recomeçar, chips de contexto.
- Manual com o Claude pelo adaptador e o OpenCode: os passos de cada marco num teste do painel Interfaces.
