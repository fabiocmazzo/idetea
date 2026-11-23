# RFC 0024 — Terminais de agente: muitos, eficientes e retomáveis

- **Status:** aprovada no painel em 2026-11-23: M1 e M2 nesta versão, M3 depois do uso. Base: `main` em `2e08d33`
  (ACP revertido).
- **Substitui:** as RFCs 0022 (cliente ACP) e 0023 (acabamento da sessão ACP), que entraram na `main` em `7ba0695`
  e `bb25311` e saíram no revert `2e08d33`. A decisão está na [ADR 0003](../decisions/0003-agentes-no-terminal-sem-acp.md).
- **Continua:** [RFC 0005](0005-agentes-e-projetos.md) (terminais pedidos por agentes),
  [RFC 0016](0016-lancador-de-agentes-e-memoria-do-projeto.md) (lançador e `Ctrl+L`) e
  [RFC 0012](0012-chat-multimodelo-e-mascote.md) (mascote e hooks).

Marcadores: **[Fato]** conferido nesta sessão no código ou na fonte da crate (`iced_term` 0.8.0,
`alacritty_terminal` 0.25.1); **[Referência]** dito pela ajuda ou documentação de ferramenta externa, não testado
aqui; **[Hipótese]** provável, não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

O agente roda no terminal, com a interface dele. A IDE não refaz a conversa do agente: ela dá **a tela, o teclado,
o contexto (`Ctrl+L`), o painel Interfaces e a confirmação** de cada terminal pedido. Com isso decidido, o terminal
vira a peça central do trabalho com agentes, e hoje ele foi feito para um ou dois shells: abas sem estado, uma aba
visível por vez, só no painel de baixo, e tudo some ao fechar a IDE.

**[Recomendação]** Três marcos, nesta ordem:

- **M1 — muitos terminais, sem se perder:** cada aba diz em que estado está (rodando, saída nova, **esperando
  você**, terminou com código), tem nome, atalho e lugar na paleta; dois terminais lado a lado; um terminal pode ir
  para a área do editor; e trocar de aba não custa mais com dez terminais abertos do que com um.
- **M2 — reabrir como estava:** a IDE guarda as abas de terminal na sessão do projeto e, ao reabrir, oferece
  **retomar os agentes na mesma conversa** pelo próprio agente (`claude --resume <id>`, `codex resume …`). A
  "memória" do agente é a conversa que ele mesmo grava em disco; o processo não precisa sobreviver para ela voltar.
  Opcionalmente, o texto que estava na tela volta acima do prompt novo.
- **M3 — o processo sobrevive à IDE (experimental, decidir depois do M2):** um processo pequeno da própria IDE
  (`idetea pty-host`) segura os PTYs; a IDE fecha, trava ou é atualizada, e reconecta neles.

## Por que no terminal (e não ACP)

- **[Fato]** As RFCs 0022/0023 foram até a sessão ACP com várias abas, permissão explícita e estado no mascote. A
  RFC 0023 já listava o que faltava para ficar usável: Markdown, copiar, entrada multilinha, permissão e ferramentas
  legíveis, contexto do editor. Era refazer, dentro da IDE, a interface que cada agente já tem.
- **[Fato]** Decisão do usuário em 2026-11-22: o ACP saiu da `main` (#20); os agentes rodam no terminal.
- **[Hipótese]** O que pesa: cada agente implementa uma parte diferente do protocolo, e os comandos próprios
  (slash commands, `/compact`, modos, retomada, subagentes) chegam primeiro, ou só, na interface de terminal dele.
  No terminal, a IDE ganha cada recurso novo do agente sem escrever nada.
- **[Fato]** O canal estruturado entre agente e IDE continua sendo de arquivos: painel Interfaces, pedidos de
  terminal, `idetea ask/run/agent/status/open`, hooks do mascote. Nada disso depende de ler o texto do terminal
  (RFC 0016: o formato muda).

## O que existe hoje

- **[Fato]** `tea-ext-terminal` (`crates/ext/tea-ext-terminal/src/lib.rs`) usa `iced_term` 0.8 sobre
  `alacritty_terminal`. Três tipos de aba: `Shell` ("Terminal N"), a `TaskTerm` (uma só) e `AgentTerm` (pedidas por
  agentes ou abertas pelo lançador). Os pedidos esperando o usuário ficam numa aba de pedido.
- **[Fato]** Uma aba visível por vez (`Shown`), no painel de baixo. A barra mostra só o rótulo: nada diz se o
  terminal escondido mudou, terminou ou espera uma resposta.
- **[Fato]** O título que o programa manda (OSC 0/2) chega do `iced_term` como `Action::ChangeTitle` e é ignorado.
- **[Fato]** Os agentes abertos pelo lançador rodam o script da tarefa sem arquivo de status: a IDE não sabe quando
  nem como terminaram (e o `Ctrl+L` continua colando neles depois do fim).
- **[Fato]** A cada troca de aba ou quando a barra aparece/some, `redraw_if_moved` manda `ChangeFont` para **todos**
  os terminais, para limpar o cache de desenho do `iced_term`. Com N terminais, cada troca recalcula N grades.
- **[Fato]** O `iced_term` cria o backend com `term::Config::default()`: o histórico (scrollback) é o padrão do
  alacritty e não é configurável; a busca por regex do alacritty (`RegexSearch`) existe no backend, mas só é usada
  para achar URLs.
- **[Fato]** `AgentTerm` já guarda `last_output` (para a colagem do `Ctrl+L` esperar o agente subir).
- **[Fato]** A sessão do projeto (`crates/tea-app/src/session.rs`, em `$XDG_STATE_HOME/idetea/sessions/`) guarda
  abas do editor, painéis e o editor dividido; nada do terminal. Ao fechar a IDE, todos os processos morrem, e um
  pedido que rodava vira `lost`.
- **[Fato]** Os hooks do mascote (`idetea mascot setup-hooks`) mandam `working` no `PreToolUse`, `attention` no
  `Notification` e `celebrating` no `Stop` do Claude Code. O estado é do projeto, não de uma aba.
- **[Fato]** `claude --help` (instalado aqui) tem `--session-id <uuid>`, `-r/--resume [id]` e `-c/--continue`;
  `codex --help` tem `codex resume` (escolha, ou `--last`).

## M1 — muitos terminais, sem se perder

### 1. Estado em cada aba

Cada aba ganha um ponto à esquerda do nome:

| estado | quando | como a IDE sabe |
| --- | --- | --- |
| rodando | saiu algo nos últimos ~2 s | `last_output` (já existe; passa a valer para todo tipo de aba) |
| saída nova | saiu algo desde a última vez que a aba apareceu | idem, comparado com a hora em que foi mostrada |
| **esperando você** | o agente pediu atenção | hook `Notification` (ver abaixo), BEL (`\a`) ou OSC 9 / OSC 777 *notify* |
| ocioso | o agente terminou o turno | hook `Stop` |
| terminou | o processo saiu | o código, em verde ou vermelho |

**[Recomendação]** O estado "esperando você" vem dos hooks que o mascote já instala, com um endereço: a IDE põe
`IDETEA_TERMINAL=<id>` no ambiente de cada aba, e `idetea mascot attention` (e os outros) passa a gravar também
`.idetea/terminals/<id>.state`. Sem o `IDETEA_TERMINAL`, nada muda (o mascote continua como hoje). Um agente sem
hooks ainda dá o estado pelo BEL ou pela notificação OSC. **[Fato]** O `iced_term` 0.8 só trata `Exit`, `Title` e
`PtyWrite` dos eventos do alacritty, mas entrega todos à extensão antes (`BackendCall(id, ProcessAlacrittyEvent(..))`):
o `Bell`, o `Wakeup` (saída) e o `Title` chegam sem patch.

O mascote, que hoje diz "um agente precisa de você", passa a dizer qual, e um clique leva à aba.

### 2. Nome, atalhos e paleta

- O título OSC vira o nome da aba quando o usuário não deu um (o Claude Code e o Codex mudam o título com a tarefa
  **[Hipótese]**). Duplo clique renomeia.
- `Ctrl+Shift+PgDn/PgUp` anda entre as abas, também com o foco no terminal. (Na implementação: `Alt+1…9` ficou de
  fora porque as teclas com Alt vão para o shell, que as usa, e `Ctrl+PgUp/PgDn` já é a troca de aba do editor.)
- Paleta: **Terminal: ir para…** lista as abas com o estado, filtrando pelo nome (a mais útil com muitos agentes);
  **Terminal: próxima esperando você**.
- Fechar uma aba de agente ou de tarefa que ainda roda pergunta antes (hoje o `×` encerra direto). Um shell fecha
  direto: a IDE não sabe se há um comando rodando dentro dele.

### 2b. O pedido vai para a IDE de onde saiu

**[Fato]** Com duas IDEs abertas no mesmo projeto (o caso de todo teste manual no dogfooding), um pedido de terminal
fica com a que o vê primeiro: um `idetea run` digitado na janela de teste pode abrir na outra. **[Recomendação]** O
`idetea run`/`idetea agent` que roda numa aba da IDE tem `IDETEA_TERMINAL=<pid>-<id>`; a CLI grava o pid no pedido
(`"ide": <pid>`) e as outras IDEs, enquanto esse pid estiver vivo, deixam o pedido para ela. **[Fato]** Cuidado: o
pedido é lido com `deny_unknown_fields`, então uma IDE antiga marca como `invalid` um pedido com campo novo. Antes
de a CLI gravar o campo, a leitura precisa passar a aceitar e ignorar campos desconhecidos, e isso tem que estar
numa versão publicada.

### 3. Lado a lado e na área do editor

- **Dividir** (`Ctrl+Shift+5`, como no VS Code): duas colunas no painel de baixo, cada uma com a sua aba. Uma
  divisão só (duas colunas) no começo; grade fica para quando o uso pedir.
- **Mover para o editor**: a aba de terminal vira uma aba da área do editor, ao lado do código e com toda a altura.
  É o lugar natural de um agente com quem se conversa por muito tempo. O processo é o mesmo (só muda onde aparece),
  e volta ao painel pelo mesmo comando. **[Hipótese]** É a parte mais cara do M1: o editor hoje só abre arquivos
  (`tea-editor`), e a aba precisa de um tipo novo que delega a `view` para a extensão do terminal.
- **[Recomendação]** O lançador (`Ctrl+Shift+A`) ganha a escolha "abrir no editor" e `[agents] open_in = "editor"`.

### 4. Desempenho

- `redraw_if_moved` reaplica a fonte só no terminal que vai aparecer e marca os outros como "sujos"; cada um se
  redesenha quando voltar. Trocar de aba passa a custar 1 grade, não N.
- Terminais escondidos não desenham (já é assim: só o atual entra na `view`) e continuam lendo o PTY (precisam,
  senão o processo trava com o buffer cheio).
- `[terminal] scrollback = 10000` (linhas, por aba). Dez agentes com 100 mil linhas cada são centenas de MB.
  Precisa que o `iced_term` aceite um `term::Config`: patch pequeno, mandado como PR para o upstream, com o mesmo
  tratamento do fork do `iced-code-editor` (ADR 0001) enquanto não entra.
- **Buscar no terminal** (`Ctrl+Shift+F` com o terminal em foco): o `RegexSearch` do alacritty já está no backend;
  falta o `iced_term` expor busca e rolagem até o resultado. Mesmo patch.

## M2 — reabrir como estava

### 5. As abas voltam

A sessão do projeto ganha a lista de terminais, na ordem, com a ativa e a divisão:

```toml
[[terminals]]
kind = "agent"             # shell | agent | task (a tarefa não volta)
title = "Revisor"
cwd = "."                  # relativo à raiz; OSC 7 atualiza a pasta de um shell
launcher = "claude"        # nome no lançador (KNOWN ou [agents.launchers])
worktree = "agente/revisao"
agent_session = "6f1c…"    # quando a IDE sabe (ver 6)
where = "editor"           # panel | editor
```

Ao reabrir:

- **Shells** voltam sozinhos, na pasta onde estavam.
- **Agentes** não rodam sozinhos: a IDE mostra "3 agentes estavam abertos: Retomar todos / Escolher / Não". Um
  agente roda comandos, e o princípio da RFC 0005 vale aqui: nada roda sem você ver o comando. O `[agents] allow`
  do usuário vale como sempre.
- **Pedidos de agentes** (`.idetea/agents/`) continuam virando `lost`; quem pediu decide se pede de novo. (Com o M3,
  eles sobrevivem.)

### 6. A memória do agente: retomar a conversa

O que se quer guardar de um agente não é o processo, é a conversa: o que ele leu, decidiu e fez. Os agentes já a
gravam em disco, e cada um tem o seu jeito de voltar:

| agente | abrir sabendo o id | retomar |
| --- | --- | --- |
| Claude Code | `claude --session-id <uuid>` **[Fato]** | `claude --resume <uuid>` **[Fato]** |
| Codex | não há **[Fato]** | `codex resume --last` na mesma pasta **[Fato]**, ou o id, se descobrirmos de onde lê-lo |
| Gemini CLI, OpenCode, Aider, Cursor Agent | **[Referência]** a conferir em cada um | idem |

**[Recomendação]**

- O lançador gera o uuid e abre o Claude Code com `--session-id`: a IDE sabe o id desde o começo e o retoma com
  `--resume <id>`, sem adivinhar.
- Para agentes sem id conhecido, "a última conversa nesta pasta". Como cada agente do lançador numa worktree tem
  a sua pasta (`.idetea/worktrees/launch-<branch>`), "a última" é a certa nesse caso; dois agentes na raiz podem
  trocar de conversa, e a tela de retomada avisa.
- `[agents.launchers]` ganha `resume = "meu-agente --resume {session}"` e `session_arg = "--session-id {session}"`,
  para os agentes que o usuário configura. Os conhecidos vêm com a tabela acima em `tea-core::coding_agents`.
- O comando de retomada muda entre versões dos agentes: se o agente sair com erro logo ao retomar, a aba oferece
  "abrir uma conversa nova".

### 7. O texto da tela (opcional, desligado por padrão)

`[terminal] keep_scrollback = true` grava, ao fechar, as últimas linhas de cada aba em
`$XDG_STATE_HOME/idetea/terminals/<projeto>/<id>.txt` (fora do projeto, permissão 0600), e ao reabrir mostra esse
texto acima do prompt novo, com uma linha "— sessão anterior, 23/11 14:02 —". Desligado por padrão porque o
terminal tem senhas, tokens e saídas de `env`; a IDE não sabe o que é segredo. Para agentes com retomada, o próprio
agente redesenha a conversa, e isto quase não faz falta.

### Como o M2 ficou (2026-11-23)

Os três itens entraram, cada um aprovado em teste manual. Onde o feito difere do proposto acima:

- **As abas não ficam na sessão do editor**, e sim num arquivo da extensão do terminal,
  `$XDG_STATE_HOME/idetea/terminals/<pasta>-<hash da raiz>.toml` (o mesmo nome que o da sessão, por
  `tea_core::os::project_state_file`). A API das extensões não tem gancho de sessão, e o arquivo é gravado a cada
  mudança das abas, então sobrevive a uma IDE que trava. Guarda o tipo, o nome dado, o título e o comando do
  agente, a worktree e o id da conversa. **Ainda não guarda** a pasta de cada shell (eles voltam na raiz), a aba
  ativa nem a divisão.
- **Retomar:** "Retomar todos / Escolher… / Não", e a paleta "Terminal: retomar agentes da sessão anterior" para
  os que ficaram. Claude Code e Codex como na tabela (`tea_core::coding_agents::{with_new_session,
  resume_command}`); sem id, `claude --continue`. Os outros abrem uma conversa nova. **Não entrou** o `resume` /
  `session_arg` em `[agents.launchers]`. Retomada que sai com erro em até 15 s vira um aviso que aponta o
  lançador (não um botão na aba).
- **Texto da tela:** os arquivos ficam em `terminals/<pasta>-<hash>/<posição>.txt` (pasta 0700, arquivos 0600),
  com as últimas 2 000 linhas, e são lidos uma vez só. A data da linha de separação é `AAAA-MM-DD HH:MM`. Para
  gravar ao fechar, as extensões recebem `CoreEvent::Closing`; o `iced_term` ganhou `Terminal::text` e
  `BackendSettings::preamble` (ver `crates/iced_term/UPSTREAM.md`).

## M3 — o processo sobrevive à IDE (experimental)

Para o servidor de desenvolvimento, o build longo e o agente no meio de um turno, que não podem morrer quando a
IDE fecha, trava ou é reinstalada.

| opção | como | prós | contras |
| --- | --- | --- | --- |
| A. tmux / zellij por baixo | cada aba é uma sessão do tmux | pronto, maduro | o tmux tem a própria tela, rolagem, mouse e atalhos, que brigam com os da IDE; uma dependência a mais |
| B. dtach / abduco | só segura o PTY | pequeno, transparente | sem histórico ao reconectar; dependência externa, ausente na maioria das máquinas |
| C. `idetea pty-host` | um processo da IDE segura os PTYs, com um buffer das últimas saídas, num socket em `$XDG_RUNTIME_DIR/idetea/` (0700) | sem dependência, mesmo binário, histórico ao reconectar; é como o VS Code faz os terminais persistentes **[Referência]** | precisa de um backend do `iced_term` que leia de um socket em vez de abrir o PTY (fork ou PR maior); processos órfãos a administrar; Windows fica de fora no começo |

**[Recomendação]** C, mas só depois de usar o M2 por um tempo: se retomar a conversa do agente bastar, o M3 vira
só para servidores e builds longos, e talvez nem precise. Se for feito: o host sai sozinho quando não sobra nenhum
PTY; a IDE lista os órfãos na abertura ("2 terminais de uma sessão anterior continuam rodando: reconectar /
encerrar"); o `idetea status` mostra quais estão vivos.

## O que fica de fora

- ACP, ou qualquer chat próprio que fale com modelos (ADR 0003, RFC 0012 opção A).
- Ler o texto do terminal para entender o agente: o estado vem de hooks, BEL/OSC e da saída do processo, não do
  conteúdo da tela.
- Grade com mais de duas colunas, terminais em janela separada e "mandar a mesma tecla para todos".

## Riscos

- **Patches no `iced_term`** (scrollback, busca, backend por socket): mais um fork a manter. Mandar cada um
  como PR pequeno ao upstream e manter o `[patch]` só até entrar.
- **Segredos:** o histórico gravado (item 7) e o buffer do `pty-host` (M3) guardam o que passou na tela. Fora do
  projeto, 0600, desligado por padrão, e nunca no git.
- **Retomada que quebra:** as opções de linha de comando dos agentes mudam. A tabela fica num lugar só
  (`coding_agents.rs`), coberta por teste, e o config do usuário pode sobrescrever.
- **Hooks são do Claude Code:** os outros agentes dão menos estado ("rodando" e "terminou" sim, "esperando você"
  só se tocarem o sino ou mandarem OSC).

## Testes

- Unitários: transições de estado da aba (saída, hook, BEL, saída do processo); a sessão grava e lê os terminais;
  montagem do comando de retomada para cada agente conhecido e para `[agents.launchers]`.
- `iced_test`: a barra mostra o estado; trocar de aba reaplica a fonte só no terminal que aparece.
- Manual (painel): três agentes no lançador, um em cada estado; fechar e reabrir a IDE; retomar; conferir que a
  conversa é a mesma.

## Decisões (painel, 2026-11-23)

1. **Escopo:** M1 e M2 nesta versão; o M3 é decidido depois de usar o M2.
2. **Terminal na área do editor:** entra no fim do M1, depois da divisão no painel.
3. **Retomada:** os agentes só voltam depois de uma pergunta, mesmo os que estão no `[agents] allow`.
4. **Texto da tela (`keep_scrollback`):** existe, desligado por padrão.

5. **Terminal na área do editor** (painel, 2026-11-23): adiado. As abas do editor são sempre um `Editor` de arquivo,
   e generalizar isso toca sessão, editor dividido, salvar e fechar. Enquanto isso, o painel do terminal já pode
   ficar à direita, com toda a altura, por `[extensions] terminal = { slot = "right" }` (vale ao recarregar o
   config).

## Ordem de implementação

1. M1.4 desempenho: fonte reaplicada só no terminal que aparece.
2. M1.1 estado em cada aba (saída, fim, título OSC; depois hooks com `IDETEA_TERMINAL` e sino).
3. M1.2 nome, atalhos, paleta e confirmação ao fechar; 2b (pedido para a IDE de onde saiu) em dois passos: aceitar
   campos desconhecidos numa versão, gravar o campo na seguinte.
4. Patch do `iced_term`: scrollback e busca.
5. M1.3 divisão no painel; depois o terminal na área do editor.
6. M2: abas na sessão, retomada dos agentes, `keep_scrollback`.
