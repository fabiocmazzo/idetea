# RFC 0016 — Lançador de agentes e memória do projeto

- **Status:** aprovada em 2026-11-13 (pelo painel), com as decisões do fim. Base: `main` em `f03455a` (0.7.0).
- **Depende de:** [RFC 0005](0005-agentes-e-projetos.md) (terminais de agente, worktrees), [RFC 0006](0006-ide-como-cli-para-agentes.md)
  (`idetea ask`, `run`, `status`), [RFC 0007](0007-historico-das-interfaces.md) (histórico das interfaces) e
  [RFC 0012](0012-chat-multimodelo-e-mascote.md), que já recomendava um lançador de agentes e o deixou para depois.
- **Origem:** pedido do usuário no chat (2026-11-13): "talvez falte duas coisas na IDE […] 1. Uma forma de iniciar os
  agentes mesmo que vão para o terminal, pois hoje o usuário tem abrir o terminal, digitar claude, codex, etc e pedir
  para ler o AGENT e se comunicar pela IDE. 2. Uma memória do projeto para os agentes, ao menos definições de como
  armazenar em arquivos dentro do projeto, algum padrão aberto talvez".

Marcadores: **[Fato]** conferido nesta sessão no código do IDE TEA ou na máquina de desenvolvimento;
**[Referência]** comportamento documentado de ferramentas externas, não conferido aqui; **[Hipótese]** provável, não
testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

As duas faltas são reais, mas não pesam igual.

1. **Lançador de agentes: vale fazer já.** O atrito de hoje não é o agente "ler o AGENTS", porque os agentes já leem
   sozinhos o arquivo de instruções do projeto. O atrito está em abrir um terminal, lembrar o comando, montar uma
   worktree à mão e copiar contexto do editor para o agente. Quase tudo de que o lançador precisa já existe: terminais
   com título, worktrees dos terminais de agente, tarefas e a lista `[agents] allow`. Falta juntar essas peças numa
   entrada só e num atalho que mande contexto.
2. **Memória do projeto: vale fazer, com cuidado.** O risco é inventar mais um formato que nenhum agente lê. A proposta
   é **não criar formato novo**: Markdown com frontmatter, um fato por arquivo, numa pasta versionada do projeto,
   decisões no formato ADR (MADR), e o bloco de instruções que o "ensinar os agentes" grava diz onde ler e como gravar.
   A IDE entra com uma tela para ver, editar e podar, e com o caminho do histórico do painel até uma decisão
   registrada. Sem banco vetorial, sem servidor MCP, sem formato binário.

Ordem sugerida: lançador primeiro (etapas 1–2), memória depois (etapas 3–4).

## Parte 1 — Lançador de agentes

### O que existe hoje

- **[Fato]** Tarefas no config (`[tasks]`, `crates/tea-core/src/config.rs`), rodadas com `Ctrl+Shift+B` num
  terminal com nome. Um projeto pode ter `"agent" = "claude"` e já abre o agente numa aba, mas é preciso saber disso
  e escrever a tarefa.
- **[Fato]** Terminais pedidos por agentes (`.idetea/agents/<nome>.json`) com `worktree`: a IDE cria a worktree em
  `.idetea/worktrees/<nome>` e roda lá (`crates/ext/tea-ext-terminal/src/worktree.rs`). Hoje só um agente pede isso;
  a pessoa não tem um botão para fazer o mesmo.
- **[Fato]** `[agents] allow` vale só no config do usuário: o do projeto é ignorado, com aviso.
- **[Fato]** "Interfaces: ensinar os agentes deste projeto" (`interfaces.teach`, `crates/ext/tea-ext-ui/src/lib.rs`)
  grava o bloco de instruções em `AGENTS.md` e `CLAUDE.md` e sabe dizer se o bloco no disco está desatualizado.
- **[Fato]** O terminal é o `iced_term` 0.8, que aceita `BackendCommand::Write(Vec<u8>)`: dá para escrever no pty de
  uma aba a partir da IDE.
- **[Fato]** Nesta máquina estão no PATH o `claude` e o `codex`; `gemini`, `opencode`, `aider` e `cursor-agent`, não.
- **[Referência]** O Claude Code lê `CLAUDE.md`, o Codex e o OpenCode leem `AGENTS.md`, e o Gemini CLI lê
  `GEMINI.md` (com opção para ler também `AGENTS.md`). Ou seja, depois do "ensinar os agentes", pedir para ler o
  arquivo é desnecessário na maioria dos casos.

### Proposta

**[Recomendação]**

1. **"Agentes: iniciar…"** na paleta, com um atalho (sugestão: `Ctrl+Shift+A`, livre hoje) e um item na barra de
   atividades. Abre uma lista curta:
   - os agentes conhecidos **que estão no PATH**: `claude`, `codex`, `gemini`, `opencode`, `aider`, `cursor-agent`
     (a lista fica num lugar só do código, fácil de crescer);
   - os do config, em `[agents.launchers]`;
   - para cada um, duas formas: **"aqui"** (na pasta do projeto) e **"numa worktree nova"** (pede o nome do branch e
     usa a mesma criação de worktree dos terminais de agente).
2. **Aba com nome e estado.** A aba se chama como o agente ("Claude", "Codex · fix-login"). O mascote e a barra de
   status já reagem a perguntas abertas no painel; o lançador só precisa marcar a aba como "de agente", para o
   `Ctrl+L` saber para onde mandar.
3. **Instruções em dia antes de abrir.** Se o bloco do "ensinar os agentes" estiver ausente ou desatualizado, o
   lançador pergunta antes ("O projeto ainda não ensina os agentes a usar o painel. Gravar agora?"), com
   "Gravar e abrir", "Só abrir" e "Não perguntar neste projeto".
4. **Mandar contexto: `Ctrl+L`.** No editor, manda para a aba de agente mais recente o caminho do arquivo e a seleção
   como `caminho:linha-linha`, mais o trecho selecionado em bloco de código, por *bracketed paste* (o agente vê como
   texto colado, não como teclas, e não envia sozinho). Sem aba de agente aberta, abre o lançador primeiro.
   Do painel de problemas, o mesmo atalho manda o diagnóstico com o local.
5. **Config.** No config do usuário:
   ```toml
   [agents.launchers]
   # nome mostrado = comando
   "Claude (Opus)" = "claude --model opus"
   "Codex rápido" = "codex --profile fast"

   [agents]
   default = "claude"       # o que Ctrl+Shift+A, Enter abre
   ```
   Como `[agents] allow`, os lançadores **só valem no config do usuário**: o config do projeto viria do repositório de
   outra pessoa e poderia abrir um comando qualquer com um atalho inocente.
6. **CLI.** `idetea agent [nome] [--worktree <branch>]` faz o mesmo de um terminal, para scripts e para a tela
   inicial. **[Hipótese]** Útil também para um agente abrir outro com o mesmo caminho da pessoa.

### O que fica de fora

- **Chat próprio falando com APIs de modelos.** A RFC 0012 já decidiu não fazer.
- **Instalar os agentes.** A lista mostra só o que está no PATH, com um link "Como instalar" por agente conhecido.
- **Ler a saída do agente** (entender o que ele está fazendo pelo texto do terminal). É frágil e muda a cada versão
  dos CLIs; o canal estruturado continua sendo o painel e os pedidos de terminal.

### Riscos

- **Bracketed paste em agentes TUI.** **[Hipótese]** Claude Code e Codex tratam colagem como texto; outros podem
  não tratar. Mitigação: por agente conhecido, um modo `paste` ou `type`, e o padrão mais conservador para os do
  config.
- **Worktrees esquecidas.** Já existe a regra dos terminais de agente (remove sem nada pendente, avisa no resto); o
  lançador usa a mesma.

## Parte 2 — Memória do projeto

### O que existe hoje

- **[Fato]** O histórico do painel (`.idetea/history/AAAA-MM/*.json`, formato `idetea-ui-history`) guarda cada
  pergunta, a resposta e o comentário. É memória de decisões, mas nenhum agente o lê, e o `idetea history` só lista.
- **[Fato]** Neste repositório, `.idetea/` não é versionado: o que fica lá não vai para o resto do time.
- **[Referência]** Cada agente tem a sua memória, que os outros não veem. O Claude Code tem o `CLAUDE.md` (com
  `@caminho` para incluir outros arquivos) e uma memória automática fora do projeto, em `~/.claude/projects/…`. O
  Codex tem o `AGENTS.md`, em hierarquia de pastas. O Cursor tem `.cursor/rules/`.
- **[Referência]** O padrão aberto que pegou é o **AGENTS.md** (agents.md), lido por vários agentes. Ele é feito para
  **instruções**, não para memória acumulada. Para decisões existe a convenção **ADR**, com o modelo **MADR**
  (Markdown, uma decisão por arquivo, numerada), conhecida de times e lida por qualquer ferramenta de texto.

### Proposta

**[Recomendação]** Uma convenção de arquivos, não um formato novo.

1. **Onde.** Numa pasta versionada (decisão 3), por exemplo:
   ```
   docs/memory/        fatos, um por arquivo
   docs/decisions/     ADRs no modelo MADR (0001-usar-sqlite.md, …)
   ```
2. **Um fato por arquivo**, em Markdown com frontmatter curto:
   ```markdown
   ---
   name: testes-sem-rede
   description: os testes não podem acessar a rede; o CI roda offline
   type: constraint        # constraint | convention | gotcha | reference
   updated: 2026-11-13
   ---
   Os testes rodam no CI sem rede. Use os fixtures em `tests/fixtures/`.

   **Por quê:** o runner do CI não tem saída para a internet.
   ```
   E um `docs/memory/README.md` como índice: uma linha por fato, que o agente lê primeiro e usa para decidir o que
   abrir. É o mesmo desenho da memória automática do Claude Code, só que no projeto, para todos os agentes e no git.
3. **Os agentes aprendem pela instrução que já existe.** O bloco do "ensinar os agentes" ganha uma seção: ler o
   índice no começo, gravar um fato quando aprender algo não óbvio sobre o projeto, preferir atualizar a duplicar,
   apagar o que ficou errado, nunca gravar segredo, registrar decisão de arquitetura como ADR. Funciona com qualquer
   agente que leia `AGENTS.md`/`CLAUDE.md`, sem integração.
4. **Tela "Memória do projeto"** num painel lateral: a lista do índice, busca, abrir, editar, apagar, e um aviso para
   fatos sem `updated` há muito tempo ou citando arquivos que não existem mais (**[Hipótese]** um passe simples por
   caminhos entre crases já pega boa parte).
5. **Do histórico para a decisão.** No histórico do painel, "Registrar como decisão" cria um ADR já preenchido com a
   pergunta, as opções, a escolha e o comentário, e abre para editar. Assim o que se decide pelo painel não fica
   preso em `.idetea/`.
6. **`idetea memory`** (`list`, `add`, `show`), para quem prefere a linha de comando e para agentes sem acesso
   direto a arquivos. **[Hipótese]** Talvez dispensável: gravar um Markdown já é a forma mais simples.

### O que fica de fora

- Banco vetorial, embeddings, servidor MCP de memória. Para o tamanho da memória de um projeto, o índice mais um
  `grep` resolvem; o resto é complexidade e dependência.
- Sincronizar com a memória própria de cada agente. Cada um continua com a sua; esta é a do **projeto**.
- Gravação automática pela IDE (resumir sessões, extrair fatos). Quem grava é o agente ou a pessoa; a IDE mostra e
  ajuda a podar.

### Riscos

- **Memória que apodrece.** Fato velho é pior que fato nenhum. Mitigações: `updated`, o aviso da tela e a regra
  "apague o que ficou errado" na instrução.
- **Segredos.** Pasta versionada não pode ter credencial. A instrução proíbe; **[Hipótese]** a tela pode avisar ao
  ver padrões comuns (`AKIA…`, `-----BEGIN`, `sk-…`).
- **Ruído no repositório.** Para quem não quer a memória no git, a decisão 3 tem a opção de pasta fora dele.

## Etapas

1. **Lançador:** paleta, atalho, detecção no PATH, "aqui" e "numa worktree", `[agents.launchers]` e `default`,
   pergunta do "ensinar" antes de abrir. Testes: detecção com PATH falso, lançadores só do config do usuário.
2. **Contexto:** `Ctrl+L` do editor e dos problemas para a aba de agente, com bracketed paste; `idetea agent`.
3. **Memória, convenção:** a pasta, o índice, o modelo de fato e de ADR, a seção nova no bloco do "ensinar os
   agentes" (com versão nova das instruções), documentação em `docs/`.
4. **Memória, tela:** painel "Memória do projeto", avisos de velho/quebrado, "Registrar como decisão" no histórico.

Cada etapa com teste manual pelo painel antes do commit, como nas RFCs anteriores.

## Decisões (2026-11-13, pelo painel)

1. **Escopo:** as duas partes nesta RFC, lançador primeiro (etapas 1–2), memória depois (etapas 3–4).
2. **Atalhos:** `Ctrl+Shift+A` para o lançador e `Ctrl+L` para mandar contexto.
3. **Pasta da memória:** `docs/memory/` (fatos e índice) e `docs/decisions/` (ADRs no modelo MADR), versionadas.
4. **Instruções desatualizadas ao abrir um agente:** perguntar antes ("Gravar e abrir", "Só abrir", "Não perguntar
   neste projeto").
5. **`idetea memory`:** fica para depois; a etapa 3 entrega só a convenção de arquivos.

## Andamento

- **Etapa 1 (2026-11-13, aprovada no teste manual):** "Agentes: iniciar…" (`Ctrl+Shift+A`, paleta e tela inicial)
  com os agentes conhecidos no PATH e `[agents.launchers]`, `[agents] default` primeiro (as duas chaves só no config
  do usuário); cada agente na pasta do projeto ou numa worktree nova (`.idetea/worktrees/launch-<branch>`), aberto
  pela extensão do terminal via `CoreEvent::LaunchAgent`. Pergunta antes de abrir quando o `AGENTS.md`/`CLAUDE.md`
  não tem as instruções da IDE ("Gravar e abrir", "Só abrir", "Não perguntar neste projeto", que grava
  `.idetea/launcher-skip-teach`). O teste achou dois defeitos, corrigidos: o editor engolia `Ctrl+Shift+A` como
  "selecionar tudo" (agora passa para a IDE, como `Ctrl+Shift+V`), e fechar a aba com Enter depois do fim não tirava
  a worktree (agora sai pelos dois caminhos).
- **Etapa 2 (2026-11-13, aprovada no teste manual):** `Ctrl+L` (`agents.send_context`) manda o lugar do cursor ou da
  seleção (com o trecho num bloco de código, até 32 KB) e os diagnósticos dessas linhas no formato dos compiladores.
  Vai para a aba de agente na tela, senão a usada por último; sem nenhuma, abre o lançador e cola na aba nova quando
  a saída do agente para por 1,2 s (no máximo 20 s). Agentes conhecidos (`tea_core::coding_agents`) recebem
  *bracketed paste*; os outros, o texto puro, como o `Ctrl+Shift+V` do terminal. Clique direito no painel de
  problemas: "Mandar ao agente" (`Context::send_to_agent`). `idetea agent [nome] [--worktree <branch>]` e `--list`
  pedem a aba pelo pedido de terminal (a IDE pergunta, salvo `[agents] allow`). O fork do `iced-code-editor` ganhou
  `selection_range` e `selected_text` públicos.
- **Etapa 3 (2026-11-13, aprovada no teste manual):** seção "Project memory" no bloco do "ensinar os agentes" (versão `531ea1d7` das
  instruções): ler `docs/memory/README.md` no começo, um fato por arquivo com `name`, `description`, `type`
  (`constraint`, `convention`, `gotcha`, `reference`) e `updated`, atualizar em vez de duplicar, apagar o que ficou
  errado, decisões como ADR (MADR) em `docs/decisions/` sem reescrever as aceitas, nada de segredo. A lista de
  comandos ganhou `idetea agent`. Documentação em `docs/project-memory.md` (inglês, é para quem usa a IDE).
- **Etapa 4 (2026-11-13, aprovada no teste manual):** `tea_core::memory` (ler fatos e decisões, avisos, apagar com a
  linha do índice, próximo ADR) e a extensão `tea-ext-memory`: painel "Memória do projeto" à esquerda (ícone de
  livro), com busca, abrir, apagar, avisos (mais de 180 dias sem `updated`, caminho entre crases que sumiu, padrão
  de credencial, fora do índice, índice citando arquivo sumido) e atualização sozinha (olha as pastas a cada
  segundo). No histórico do Interfaces, "Registrar como decisão" cria `docs/decisions/NNNN-….md` (`status:
  proposed`) com o contexto, as opções, a escolha e o comentário, e abre. Os títulos do ADR saem no idioma da IDE.
  No chinês, "记忆" e "决策" não têm glifo na fonte embutida: o painel é "项目记录" e as decisões, "决定".
