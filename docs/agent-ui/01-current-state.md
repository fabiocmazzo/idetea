# 01 — Como as interfaces para agentes funcionam hoje

Mapa do mecanismo na `main` em `0d7eebc` (0.19.0 + #29). Tudo aqui foi conferido no código; o que não existe está
dito como "não existe". O formato para agentes está em [`docs/interfaces/formato.md`](../interfaces/formato.md).

## As peças

| Peça | Onde | O que faz |
| --- | --- | --- |
| Documento | `crates/tea-declarative-ui/src/document.rs` | `parse(bytes, &Limits) -> Result<Document, String>`: valida o JSON (`format: "idetea-ui"`, `version: 1`), monta `Vec<Component>` com `Kind`, junta avisos (campo desconhecido) sem recusar. `Kind::Unknown` mostra um aviso no lugar de um componente que a versão não conhece. |
| Estado | `crates/tea-declarative-ui/src/state.rs` | `State`: o que o usuário mudou (`local: caminho → valor`) por cima do `data` do agente. Numa revisão nova, o agente vence só onde **ele** mudou o valor. `press()` monta os `values` do `context` do botão; `waiting()` diz o que falta para um `requires`. |
| Ponteiros | `crates/tea-declarative-ui/src/pointer.rs` | RFC 6901 (`/usuario/nome`). É o único jeito de ligar um componente a um valor. |
| Desenho | `crates/tea-declarative-ui/src/view.rs` | `view(doc, state, rendered, media, playback) -> Element<UiMessage>`: um componente embaixo do outro; botões seguidos na mesma linha. `UiMessage`: `Set`, `Press`, `Open`, `Link`, `Comment`, `Copy`, `Play`, `Stop`, `SpritePause`, `SpriteStep`, `Shown`, `ToggleSelect`, `SelectText`. |
| Mídia | `crates/tea-declarative-ui/src/media.rs`, `wav.rs` | Lê imagens, spritesheets, tabelas e WAV numa thread fora da interface, conferindo o tamanho pelo cabeçalho antes de decodificar (`MediaLimits`: 8 MB, 4096 px, 64 Mpx somados). |
| Links | `crates/tea-declarative-ui/src/links.rs` | Só quatro destinos: web, arquivo do projeto (`src/x.rs:42`), config da IDE e `idetea:open/<arquivo>` com lista de extensões. |
| Resposta | `crates/tea-declarative-ui/src/response.rs` | `idetea-ui-response`: `action`, `values`, `comment`, `ide` (pid, versão, `outdated`, versão das instruções) e `media` (o que foi mostrado, tocado, aberto). |
| Painel | `crates/ext/tea-ext-ui/src/lib.rs` | Uma thread faz `stat` em `.idetea/ui/` a cada 250 ms; quando algo muda, lê tudo fora da interface e manda. Uma aba por documento. |
| Disco | `crates/ext/tea-ext-ui/src/store.rs` | Um escritor por arquivo: o agente escreve `ui/<nome>.json`; a IDE escreve `ui/responses/<nome>/<seq>-<evento>.json`; o agente apaga a resposta (é a confirmação). Não segue symlinks. |
| Histórico | `crates/ext/tea-ext-ui/src/history.rs` | `.idetea/history/<AAAA-MM>/…json`: cada pergunta, a resposta e o comentário, gravados pela IDE. `idetea history` lê. |
| Linha de comando | `ask.rs`, `check.rs`, `fix.rs`, `teach.rs`, `novo.rs`, `decision.rs` | `idetea ask` faz o ciclo inteiro numa chamada; `idetea ui check` valida como o painel; `fix` arruma restos; `teach` grava as instruções no `AGENTS.md`/`CLAUDE.md`; "Registrar como decisão" vira ADR. |
| Som | `crates/ext/tea-ext-ui/src/player.rs` | Tocador do sistema (`pw-play`, `paplay`, `aplay`; `afplay` no macOS), um som por vez, num grupo de processos que morre inteiro. |
| Plugins | `crates/tea-plugins` | Componentes WASM (wasmtime 50, WIT em `wit/plugin.wit`), prazo por época (500 ms por chamada), `host.run` com prazo e saída limitada. O painel de um plugin é um documento `idetea-ui`. Feature `plugins` fora do padrão; plugins só de `$XDG_DATA_HOME/idetea/plugins/`, nunca da pasta do projeto. |
| Terminais pedidos | `crates/ext/tea-ext-terminal/src/agents.rs` | `.idetea/agents/<nome>.json`: um comando que o usuário aprova vendo tudo; roda numa aba, com worktree opcional e `IDETEA_ANSWER`. |

## Os componentes

`text`, `markdown`, `select`, `checkbox`, `input`, `button` (com `requires`), `steps`, `file`, `image`,
`gallery` (`select`: `none`, `one`, `many`), `sprite`, `audio` e `table`. Limites do documento: 256 KB, 200
componentes, 16 KB por texto, 500 opções, 32 imagens, 32 sons, 8 tabelas, 64 KB e 16 níveis em `data`.

## O que não existe

- **Expressões, valores calculados, condições entre campos.** Nada no código. A
  [RFC 0003](../rfc/0003-interfaces-pelo-uso.md) decidiu, no painel, "simulações como plugins, sem expressões no
  JSON" (decisão 4).
- **Campos numéricos e sliders.** Um número hoje é um `input` de texto ou um `select`.
- **Layout.** Tudo em coluna; só botões seguidos ficam lado a lado. `gallery` é a única comparação lado a lado.
- **Anexos na resposta.** O usuário não manda imagem nem arquivo de volta; a resposta leva só valores e o
  comentário.
- **Código do agente rodando na IDE.** Por desenho: plugins não vêm do projeto, e um comando passa pela confirmação
  de terminal.
- **Som no Windows.** A lista de tocadores fora do macOS é `pw-play`, `paplay`, `aplay`; no Windows nenhum existe e
  o botão fica desligado com o motivo.
- **Seek, pausa e volume no som; zoom e pan dentro do painel** (um clique abre a imagem na aba, que tem zoom).
- **Vídeo e 3D.** Ficaram fora da [RFC 0004](../rfc/0004-midia-no-painel.md); `idetea:open` abre no app do sistema.

## O que o uso mostra

Contagem dos documentos guardados em `.idetea/history/` de três projetos (idetea, tobby, coralynx; 241
documentos):

| componente | usos |
| --- | --- |
| `button` | 563 |
| `markdown` | 251 |
| `select` | 147 |
| `steps` | 140 |
| `checkbox` | 62 |
| `file` | 35 |
| `input` | 30 |
| `image`, `table`, `text`, `gallery` | 2, 2, 2, 1 |
| `sprite`, `audio` | 0 |

- O painel é usado para **decidir** (select + botões) e para **entregar teste manual** (steps + três botões). A
  mídia, construída nas RFCs 0004 e 0020, quase não foi usada: nenhum dos três projetos produz imagens ou sons.
- **49 das 241 respostas (20%) trazem comentário**, e no idetea cerca de **12 dizem "tirei um print"**: o usuário
  salva a captura em `~/Pictures` e escreve onde está, porque a resposta não leva imagem.
- Outros comentários pedem uma tarefa nova ("e após, pode verificar…"), fazem uma pergunta de volta ou dão uma
  ressalva. O comentário funciona; o que falta é anexar o que se viu.

## Limitações e acoplamentos

- **`Kind` é um enum fechado** (`document.rs`): cada componente novo mexe no parser, no `view`, no schema JSON, no
  `formato.md`, no `check` e nas instruções do `teach`. Não é um problema a resolver com um sistema de registro: são
  poucos componentes e o enum dá exaustividade. É o custo a contar em cada etapa.
- **`State` só conhece valores do usuário e do agente.** Um valor calculado precisaria de um terceiro dono, que não
  pode ser editado nem enviado como se fosse do usuário.
- **`view` é uma função grande por `Kind`.** Um componente com estado de desenho próprio (zoom, posição de um
  slider de comparação) precisa de espaço em `Rendered`, como o texto selecionável já tem.
- **A mídia é lida uma vez por revisão e quando o arquivo muda** (`media_stamps`). Um anexo do usuário seria o
  primeiro arquivo que a **IDE** escreve para o agente ler além da resposta.
