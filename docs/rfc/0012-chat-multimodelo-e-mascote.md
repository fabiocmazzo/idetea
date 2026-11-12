# RFC 0012 — Chat multimodelo na IDE e mascote reativo (estilo Masko)

- **Status:** aprovada em 2026-11-08 (pelo painel), com as decisões do fim. Base: `main` em `b28a858` (0.4.0).
- **Depende de:** [RFC 0001](0001-interfaces-declarativas.md) (painel Interfaces), [RFC 0004](0004-midia-no-painel.md)
  (spritesheets e áudio no painel), [RFC 0005](0005-agentes-e-projetos.md) (terminais de agente e batimento `.wait`) e
  [RFC 0006](0006-ide-como-cli-para-agentes.md) (`idetea ask`, `idetea run`, `idetea status`).
- **Origem:** pedido do usuário no painel (2026-11-08): *"A ideia do idetea é uma IDE com capacidades para agentes,
  também ser uma IDE boa. O que eu preciso: Que você avalie além desse esquema de agentes rodando e falando por aqui
  com linguagem declarativa, se vale a pena ter um chat multimodelo dentro da IDE, e pensei também em ter uma janela
  com um mascote que reage ao que está fazendo meio na linha do https://masko.ai/integrations/claude-code, mas dentro
  de uma janela na ide. Pode criar uma RFC?"*

Marcadores: **[Fato]** conferido no código, no uso ou na fonte citada; **[Referência]** comportamento de ferramentas
externas; **[Hipótese]** provável, não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

As duas ideias apontam para lados opostos da filosofia do IDE TEA:

1. **Chat multimodelo próprio dentro da IDE (falando direto com APIs de LLMs):** **não vale a pena construir um
   harness próprio** **[Recomendação]**. Isso colocaria o IDE TEA para competir onde Claude Code, Codex, Gemini CLI e
   OpenCode já gastam todo o seu esforço (loop de ferramentas, compactação de contexto, cache de prompt, assinaturas
   OAuth como Claude Max/ChatGPT Plus que não funcionam por API key avulsa) e adicionaria pilha HTTP/TLS ao binário.
   **O que vale a pena no lugar:** tratar o **multimodelo como orquestração de agentes locais e modelos rápidos**:
   (a) um lançador/seletor de agentes no terminal integrado com envio de contexto do editor (`Ctrl+L`: manda seleção,
   arquivo ou erro do LSP para o agente ativo), e (b) um comando/skill de **"Segunda opinião multimodelo"** que
   consulta um segundo agente/modelo em paralelo (`claude -p`, `codex`, `gemini`, `ollama`) e entrega a comparação no
   próprio painel **Interfaces**.
2. **Mascote reativo dentro da IDE (na linha do Masko.ai):** **vale muito a pena e encaixa como uma luva no que já
   existe** **[Recomendação]**. Enquanto o Masko precisa de servidor MCP na nuvem e hooks para adivinhar o que o
   Claude Code está fazendo num terminal solto, o IDE TEA **já sabe nativamente** quando um agente fez uma pergunta
   (`.idetea/ui/*.json`), quando está esperando (`.wait`), quando pediu um terminal (`.idetea/agents/*.status.json`),
   quando uma tarefa/build falhou ou passou e quando o LSP achou erros. Além disso, o recorte e animação de
   spritesheets em pixel art (`tea-declarative-ui/src/media.rs`) e o tocador de áudio (`tea-ext-ui/src/player.rs`) **já
   estão prontos na árvore desde a RFC 0004** **[Fato]**.

---

## Parte 1 — Avaliando um chat multimodelo dentro da IDE

### 1.1 O que o IDE TEA tem hoje [Fato]

- **Zero pilha de rede no binário:** o `Cargo.toml` não traz `reqwest`, `ureq`, `hyper`, `rustls` nem `openssl`. A
  única conversa com a rede é invocando o `git` do sistema. Isso mantém a compilação rápida e o AppImage livre de
  bibliotecas do sistema.
- **Agentes conversam por arquivos e CLI:** qualquer agente (Claude Code, Codex, Gemini CLI, OpenCode, Antigravity)
  pergunta e mostra resultados ricos pelo painel **Interfaces** (`idetea ask`), pede terminais supervisionados
  (`idetea run`, com suporte a `worktree` e `IDETEA_ANSWER`) e lê o que o usuário está vendo no editor (`idetea status`,
  gravado em `.idetea/state.json`).

### 1.2 As três formas de "chat multimodelo na IDE"

Quando se fala em "chat multimodelo dentro da IDE", há três arquiteturas bem diferentes:

| Arquitetura | Como funciona | Prós | Contras para o IDE TEA |
|---|---|---|---|
| **A. Chat HTTP próprio (estilo Cursor / Continue)** | A IDE guarda API keys (OpenAI, Anthropic, Gemini, OpenRouter, Ollama), faz chamadas SSE e implementa o loop de ferramentas (`read_file`, `edit_file`, `run`) | Não depende de CLI externa instalada; troca de modelo num dropdown por mensagem | **Alto custo e retrabalho [Fato]:** exige HTTP+TLS no binário, cofre de chaves, manutenção de APIs que mudam todo mês, UI de chat com streaming Markdown, e **não aproveita assinaturas fixas** (Claude Pro/Max, ChatGPT Plus) quando elas exigem o cliente oficial. Compete de frente com os próprios agentes que usam o IDE TEA. |
| **B. Painel de chat nativo sobre protocolo de agente (ACP / stdio JSON)** | A IDE desenha bolhas de chat no `iced`, mas conversa por trás com `claude`, `gemini` ou `opencode` via protocolo (como o *Agent Client Protocol* do Zed) | Aproveita autenticação, ferramentas e contexto dos agentes instalados | **Médio/Alto custo:** exige reimplementar no `iced` uma interface de streaming de chat e aprovação de tool calls que o próprio TUI do agente já desenha no terminal integrado (`iced_term`), além de já termos o painel **Interfaces** para interações estruturadas. |
| **C. Orquestração multimodelo sobre o que já existe (Abas de Agente + Contexto + "Segunda Opinião")** | Os agentes rodam em abas do terminal integrado (ou fora); a IDE ganha atalhos para injetar seleção/erros no agente ativo e um modo de disparar consultas a múltiplos modelos devolvendo a resposta no painel **Interfaces** | **Custo baixo**, zero dependência HTTP, funciona com qualquer assinatura/CLI e reforça a linguagem declarativa da IDE | Não é uma janela de bolhas de chat separada do terminal; para conversar em texto livre dentro da IDE, usa-se a aba do agente no terminal. |

### 1.3 Onde a falta de um "chat integrado" realmente dói hoje [Hipótese]

Analisando o fluxo de dogfooding, quem pensa em abrir um chat dentro da IDE normalmente quer resolver três dores
práticas:

1. **Tirar uma dúvida rápida sobre um trecho de código ou erro do LSP** sem ter que copiar o caminho do arquivo, a
   linha e a mensagem de erro para outra janela.
2. **Começar ou alternar entre agentes de modelos diferentes** (ex.: usar Claude Code para refatorar, Codex para
   revisar um algoritmo, Gemini para analisar um contexto grande, ou um modelo local no Ollama) sem decorar flags de
   CLI.
3. **Pedir uma "segunda opinião" a outro modelo** sobre um plano, RFC ou diff e comparar as respostas lado a lado.

### 1.4 Proposta para o Multimodelo: Opção C em 3 peças pequenas [Recomendação]

Em vez de construir um cliente HTTP de LLMs (Opção A) ou duplicar o TUI dos agentes num painel de bolhas (Opção B):

1. **Perfis de Agentes / Modelos (`[agents.profiles]` no config):**
   - Lista de agentes/modelos conhecidos detectados no `PATH` (Claude Code, Codex, Gemini CLI, OpenCode, Ollama) ou
     configurados pelo usuário.
   - No painel do Terminal, ao lado do `+` de novo shell, um botão/comando **"Novo Agente…"** (`agents.new`) que abre
     uma aba já rodando o agente/modelo escolhido.
2. **"Mandar para o agente" (`Ctrl+L` / menu de contexto no editor e no painel Problemas):**
   - Com um trecho selecionado no editor (ou sobre um erro do LSP), `Ctrl+L` envia a referência formatada
     (`@arquivo:linha-fim` ou o diagnóstico) direto para a aba do agente ativo no terminal (ou copia pronto se o
     agente estiver rodando fora da IDE, já que o `idetea status` também expõe a seleção).
3. **Conselho Multimodelo ("Segunda opinião" no painel Interfaces):**
   - Aproveitando o que a RFC 0005/0006 já criou (`idetea run` + `IDETEA_ANSWER`), um comando `idetea council
     "pergunta"` (ou pela paleta: **"Agentes: pedir segunda opinião…"**) que dispara a mesma pergunta em modo
     não-interativo para 2 ou mais modelos configurados (`claude -p`, `codex exec`, `gemini -p`, `ollama run`) e monta
     automaticamente um documento no painel **Interfaces** com a resposta de cada modelo e botões para escolher qual
     seguir.

---

## Parte 2 — Janela / Painel do Mascote Reativo (na linha do Masko.ai)

### 2.1 Como o Masko funciona e por que dentro da IDE é muito mais simples [Referência / Fato]

O **Masko Code** (`masko.ai/integrations/claude-code`) coloca um personagem animado na área de trabalho do macOS que
acompanha os agentes em tempo real **[Referência]**:
- Reage visualmente ao estado do agente (escrevendo código, pensando, esperando permissão/resposta, comemorando quando
  termina).
- Mostra balões de fala quando o agente precisa de aprovação ou resposta, com atalhos rápidos para permitir/negar ou
  focar o terminal certo.
- Conecta-se ao Claude Code por um servidor MCP remoto (`https://masko.ai/api/mcp`) e por hooks em
  `~/.claude/settings.json`.

**Por que dentro do IDE TEA isso é muito mais natural e direto [Fato]:**

1. **A IDE já é o centro por onde passam os eventos:** ela não precisa de conta na nuvem nem de servidor MCP HTTP para
   saber o que está acontecendo no projeto:
   - Pergunta aberta no painel `.idetea/ui/<nome>.json` com `.wait` recente → **o agente está esperando você responder**.
   - Pedido em `.idetea/agents/<nome>.status.json` em `pending` → **o agente está pedindo permissão para abrir um terminal**.
   - Pedido em `running` (ou tarefa da IDE rodando) → **trabalhando**.
   - Pedido terminou com `exited` (`code: 0` vs `code != 0`) → **comemorando** ou **assustado com erro**.
   - `.wait` parou de atualizar há mais de 30s com pergunta aberta → **dormindo / agente sumiu**.
   - Diagnósticos do LSP (`tea-lsp`) subiram para > 0 erros ou zeraram → **reação sutil ao estado do código**.
2. **A IDE já renderiza spritesheets animados e toca sons [Fato]:**
   - `crates/tea-declarative-ui/src/media.rs` já decodifica PNGs com a crate `image`, recorta quadros horizontais ou em
     grade (`frame_width`, `frame_height`) e desenha com `FilterMethod::Nearest` (pixel art nítida).
   - A animação já usa `window::frames()` do `iced` apenas quando visível.
   - `crates/ext/tea-ext-ui/src/player.rs` já toca WAV/OGG pelo tocador do sistema.

### 2.2 Estados do Mascote e gatilhos automáticos [Recomendação]

O mascote funciona como uma máquina de estados com prioridade (do mais urgente ao mais calmo), sem exigir que o
agente escreva nada específico para o mascote — mas permitindo que o agente mande estados/falas extras se quiser:

| Prioridade | Estado | Quando entra sozinho (nativo da IDE) | Quando o agente informa (opcional) | Balão de fala / Ação ao clicar |
|---|---|---|---|---|
| 1 (máxima) | `attention` (acenando / ponto de exclamação) | Há pergunta não respondida em `.idetea/ui/` (com `.wait` ativo) **ou** pedido de terminal `pending` em `.idetea/agents/` | `idetea mascot attention --text "…"` | Mostra o título da pergunta ou do terminal pedido. **Clicar no balão abre/foca o painel Interfaces ou a aba do pedido de terminal** (ou botões rápidos *Ver* / *Permitir*). |
| 2 | `error` (fogo / xícara rachada / susto) | Um terminal de agente ou tarefa da IDE acabou de sair com `code != 0` (dura ~8s) | `idetea mascot error --text "…"` | *"O comando X falhou (código N)"* — clicar foca a aba do terminal. |
| 3 | `celebrating` (comemorando / vapor alegre) | Um terminal/tarefa saiu com `code == 0` ou uma pergunta do painel acabou de ser respondida (dura ~5s) | `idetea mascot happy --text "…"` | *"Pronto!"* |
| 4 | `working` (digitando / fervendo o chá) | Há um terminal de agente em `running`, uma tarefa rodando, ou batimento de atividade recente do agente | Hook do agente (`PreToolUse`) ou `idetea mascot working --text "Lendo src/main.rs…"` | Mostra o que está rodando (*"Rodando: Revisor"* ou a ferramenta atual). |
| 5 | `thinking` (olhando para cima / colher mexendo) | Entre chamadas de ferramentas do agente (via hook/CLI) ou enquanto o LSP carrega | `idetea mascot thinking` | Discreto, sem balão ou com reticências `…`. |
| 6 | `idle` (calmo / piscando / vapor suave) | Projeto aberto sem atividade de agente no momento | `idetea mascot idle` | Sem balão (não distrai). |
| 7 | `sleeping` (dormindo / `Zzz`) | Sem atividade do usuário nem do agente há > 5 min, ou pergunta abandonada (`.wait` expirou) | — | Se houver pergunta com `.wait` expirado, avisa: *"O agente parou de esperar"*. |

### 2.3 Integração fina com agentes externos (Hooks de 1 linha) [Recomendação]

Quando o agente principal roda **fora** da IDE (ex.: Claude Code ou Antigravity num terminal externo) e está apenas
lendo arquivos ou pensando (antes de chamar `idetea ask`), como o mascote sabe que ele está trabalhando?

- Um arquivo leve de estado em `.idetea/mascot.json` (fora do git), gravado por `idetea mascot <estado> [--text "…"]
  [--source <agente>] [--ttl <segundos>]`.
- Qualquer estado enviado por ali tem um **TTL curto** (padrão: 30 segundos). Se o agente terminar ou cair sem limpar,
  o mascote volta sozinho para `idle`.
- No Claude Code (e similares que suportam hooks), o comando **"Interfaces: ensinar os agentes"** (ou um comando
  `idetea mascot setup-hooks`) pode oferecer configurar os hooks locais (`.claude/settings.local.json`) para chamar:
  - `PreToolUse` → `idetea mascot working --text "$TOOL_NAME"`
  - `Notification` → `idetea mascot attention`
  - `Stop` → `idetea mascot celebrating --ttl 5`

### 2.4 Onde o mascote mora na interface da IDE [Recomendação]

Três opções de encaixe na janela da IDE (podemos escolher uma ou combinar 1 + 2):

1. **Cantinho fixo na barra lateral / abaixo da Activity Bar (ou rodapé do painel esquerdo):**
   - Um quadrado compacto (ex.: 64×64 ou 80×80 px) sempre visível quando ativado (`[mascot] enabled = true`), com o
     balão de fala curto ao lado ou acima quando há `attention`/`working`/`error`.
   - Vantagem: você vê o mascote reagindo mesmo quando está com a árvore de arquivos ou o Git abertos, sem precisar
     deixar o painel **Interfaces** aberto o tempo todo — e quando o mascote acena (`attention`), um clique nele já
     abre o painel **Interfaces** na pergunta certa!
2. **Painel / Aba dedicados ("Mascote"):**
   - Uma visão maior (ex.: no painel direito ou inferior) com o mascote em destaque, o balão completo, a lista do que
     cada agente ativo está fazendo agora e botões rápidos para ir à pergunta ou ao terminal.
3. **Mini-janela flutuante (overlay ou janela secundária do `iced`):**
   - No `iced` 0.14 na árvore atual, o app usa janela única (`iced::application`, não `iced::daemon` multi-window)
     **[Fato]**. Portanto, um **overlay flutuante no canto inferior direito do editor** (arrastável ou recolhível para
     um ícone na barra de status) entrega exatamente a sensação de "janela do mascote dentro da IDE" sem mexer na
     arquitetura de janela única do `iced`.

### 2.5 Formato de pacote de Mascote (customizável pelo usuário) [Recomendação]

Para não ficar preso a um desenho só (e permitir temas divertidos, como uma **xícara de chá em pixel art** padrão do
IDE TEA, um gatinho, um robô, etc.):

- Um mascote embutido no binário (**"Chá" / Teacup em pixel art**, zero configuração) para funcionar de imediato.
- Suporte a pastas em `~/.config/idetea/mascots/<nome>/` ou `.idetea/mascot/` com um `mascot.toml` simples e um ou mais
  spritesheets PNG:
  ```toml
  name = "Tea"
  pixelated = true
  frame_width = 32
  frame_height = 32

  [states.idle]
  path = "tea.png"
  row = 0
  frames = 4
  fps = 4

  [states.working]
  path = "tea.png"
  row = 1
  frames = 6
  fps = 10

  [states.attention]
  path = "tea.png"
  row = 2
  frames = 4
  fps = 8
  sound = "ping.wav" # opcional, desligado por padrão
  ```

---

## 3. Custos e Riscos

| Item | Custo | Riscos e cuidados |
|---|---|---|
| **Mascote reativo (estados nativos + overlay/painel + `idetea mascot`)** | **Baixo a Médio** — reusa o corte de spritesheets de `tea-declarative-ui` e os estados que a IDE já observa em `.idetea/ui/` e `.idetea/agents/`. | **Consumo de CPU/GPU com animação contínua:** no estado `idle`, usar FPS baixo (2–4 fps) ou animar só a cada poucos segundos (piscar/vapor), e pausar `window::frames()` quando a janela estiver minimizada ou sem foco. |
| **Atalho "Mandar seleção/erro para o agente" (`Ctrl+L`) + lançador de perfil** | **Baixo** — usa o `tea-ext-terminal` e a seleção do `tea-editor`. | Cuidar para não enviar Enter automático no PTY do terminal: apenas colar o texto/caminho no prompt do agente para o usuário revisar antes de enviar. |
| **"Segunda opinião" multimodelo (`idetea council`)** | **Baixo a Médio** — script/subcomando que chama os binários configurados e gera um JSON `idetea-ui`. | Como cada CLI (`claude`, `codex`, `gemini`) tem flags diferentes, deixar os comandos em `[agents.profiles]` no config com um template simples (`command = "claude -p {prompt}"`). |

---

## 4. Para decidir

1. **Sobre o Chat Multimodelo:** você concorda em **não** criar um cliente HTTP/API de chat próprio dentro da IDE e, em
   vez disso, seguir a **Opção C** (perfis de agentes no terminal + atalho `Ctrl+L` para enviar seleção/erros ao agente
   + comando de "Segunda opinião" multimodelo respondendo no painel Interfaces)? Ou você faz questão de um painel de
   chat com bolhas de texto próprio dentro da IDE?
2. **Sobre o Mascote — vamos fazer?** E onde você prefere que ele fique na IDE:
   - **(a)** Uma **janelinha/card flutuante (overlay)** num canto da IDE (recolhível na barra de status) com o sprite e
     o balão de fala clicável;
   - **(b)** Um **painel próprio** na barra de atividades (como Interfaces, Git, etc.);
   - **(c)** Fixo num **canto da barra lateral / topo do painel Interfaces**?
3. **Escopo da primeira versão do Mascote:** começamos com o mascote embutido em pixel art (**xícara de chá do IDE
   TEA**) reagindo aos eventos nativos da IDE (perguntas no painel, pedidos de terminal, tasks, erros) + comando
   `idetea mascot <estado> [--text "…"]` (e suporte a `mascot.toml` customizável)?
4. **Ordem de implementação:** se aprovarmos os dois, começamos pelo **Mascote reativo** ou pelas **melhorias de
   fluxo multimodelo**?

## Decisões (2026-11-08, pelo painel)

1. **Chat multimodelo:** deixar o multimodelo de lado por enquanto e focar apenas no **Mascote reativo**.
2. **Onde o Mascote fica na IDE:** numa **janelinha/card flutuante (overlay)** num canto da IDE (recolhível na barra de
   status), com o sprite animado e o balão de fala clicável.
3. **Escopo do Mascote:** mascote embutido em pixel art (**xícara de chá do IDE TEA**) reagindo aos eventos nativos da
   IDE (perguntas abertas no painel Interfaces, pedidos/execução de terminais de agente, tarefas e erros), comando CLI
   `idetea mascot <estado> [--text "…"]` para agentes externos/hooks, e suporte a pacotes customizáveis via
   `mascot.toml`.
4. **Começo:** implementar o Mascote reativo.

## Andamento

- **2026-11-08 — mascote reativo implementado.** Card flutuante no canto inferior direito (`crates/tea-app/src/mascot.rs`),
  recolhível na barra de status e pelo comando `mascot.toggle` na paleta (nos três idiomas). Traz o mascote embutido
  **"Tea"** (xícara de chá em pixel art 24×24, 4 quadros por estado) reagindo automaticamente a perguntas abertas no
  painel Interfaces (`.idetea/ui/` + `.wait`), pedidos e execução de terminais de agente (`.idetea/agents/`), tarefas,
  diagnósticos e atividades do LSP, além do subcomando CLI `idetea mascot <estado> [--text "…"] [--source <nome>]
  [--ttl <segundos>]` e suporte a pacotes customizados via `mascot.toml`.
- **2026-11-08 — expressões interativas ao passar o mouse e clicar (`curious` e `annoyed`).** Passar o mouse sobre o
  mascote ou clicar 1–2 vezes desperta a expressão `curious` (sobrancelha erguida e `?` flutuante); sacudir o mouse
  sobre ele ou clicar 3+ vezes seguidas dispara a expressão `annoyed` (sobrancelhas franzidas, chá fervendo, tremor e
  falas de irritação progressivas).
- **2026-11-12 — mais falas, mais reações, mais idiomas.** Cada situação (humor, hover, toque, duplo clique,
  tarefa, terminal, salvar, sono, cumprimento, dica, build) tem um pequeno conjunto de frases. O sorteio não repete
  a mesma linha duas vezes seguidas. O mascote reage a arquivo salvo, ao build do depurador (sucesso ou falha), a
  um tempo longo parado (adormece e acorda quando a pessoa volta) e à hora do dia, com um convite leve a descansar
  de madrugada. `[mascot] chatty` (`quiet`, `normal`, `chatty`) e `[mascot] tips` controlam o quanto ele fala; as
  dicas citam atalhos que existem no config padrão. Nada disso rouba o foco nem abre o card se ele estiver
  recolhido. Francês, alemão, italiano, japonês e chinês simplificado traduzem todas as falas do mascote; o resto
  da interface desses idiomas cai no inglês. Um recorte pequeno da Droid Sans Fallback (Apache-2.0) cobre os
  glifos dessas falas quando o sistema não tem fonte CJK.

### 2.6 Falas, reações extras e idiomas

- As falas ficam em `mascot-*` nos `.ftl`. O código guarda um pool por situação e escolhe uma chave que não seja a
  anterior. A frase escolhida fica estável até a situação mudar: o relógio da animação não sorteia de novo a cada
  quadro.
- Prioridade continua a da tabela acima. Salvar, cumprimento, acordar e dica são um aparte com TTL curto, visível
  só quando o estado de base é calmo, dormindo ou pensando. Erro, pergunta e terminal não são cobertos.
- `chatty = "quiet"` desliga cumprimento, dica, comentário ao salvar e a fala de acordar. O sono, o erro e a
  pergunta continuam. `tips = false` desliga só as dicas. O padrão é `normal` e `tips = true`.
- Tarefas do terminal (`Ctrl+Shift+B`, teste no cursor) avisam quando começam. O fim delas não é um evento da IDE:
  o script espera o Enter e não devolve o código. O que a IDE já sabe — terminal de agente que saiu, e o build do
  depurador — muda o mascote.
- Idiomas parciais (`fr`, `de`, `it`, `ja`, `zh-CN`) carregam o inglês por baixo. Chinês tradicional não vira
  simplificado.

## Fontes

- Masko Code (integração com Claude Code): https://masko.ai/integrations/claude-code
- Código da IDE TEA: `Cargo.toml`, `crates/tea-declarative-ui/src/media.rs`, `crates/ext/tea-ext-ui/src/player.rs`,
  `crates/ext/tea-ext-ui/src/store.rs`, `crates/ext/tea-ext-terminal/src/agents.rs`.
