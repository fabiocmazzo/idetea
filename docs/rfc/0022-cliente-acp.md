# RFC 0022 — Cliente ACP: a sessão do agente na IDE

- **Status:** aprovada com a issue 16 (2026-11-20). O marco 1 está implementado. Base: `main`.
- **Emenda:** [RFC 0012](0012-chat-multimodelo-e-mascote.md) (opção B, adiada) e
  [RFC 0016](0016-lancador-de-agentes-e-memoria-do-projeto.md) (o que ficou de fora: ler a saída do agente).
  Não reabre a opção A da 0012.
- **Depende de:** [RFC 0005](0005-agentes-e-projetos.md), [RFC 0006](0006-ide-como-cli-para-agentes.md) e
  [RFC 0012](0012-chat-multimodelo-e-mascote.md) (mascote e hooks).

Marcadores: **[Fato]** conferido no código ou na fonte citada; **[Referência]** comportamento documentado de
ferramenta externa; **[Hipótese]** provável, não testado aqui; **[Recomendação]** proposta.

## Resumo

A IDE passa a ser cliente do [Agent Client Protocol](https://agentclientprotocol.com/) (ACP). O agente continua
com o runtime, a autenticação, as ferramentas e o modelo dele. A IDE só inicia o processo, mostra a sessão e
responde `session/request_permission`. Não é um chat que chama API de modelo, e não é uma leitura do texto do
terminal.

O protocolo falado neste marco é o **v1 estável** da crate `agent-client-protocol` **5.4.0** (fixada, feature
`process`, sem `unstable_protocol_v2`). O transporte é JSON-RPC 2.0, uma mensagem por linha, em stdin/stdout.
O stderr fica com o agente. Não entra pilha HTTP/TLS no binário.

## O que as RFCs anteriores deixaram de fora

- **[Fato]** A RFC 0012 recusou a opção A (chat HTTP próprio, chaves de API, loop de ferramentas na IDE) e
  descreveu a opção B (painel nativo sobre ACP ou stdio JSON) como cara, ao lado do TUI no terminal e do painel
  Interfaces. A decisão de 2026-11-08 foi deixar o multimodelo de lado e fazer o mascote.
- **[Fato]** A RFC 0016 fez o lançador (`Ctrl+Shift+A`, `idetea agent`) e disse, de fora: não ler a saída do
  agente pelo texto do terminal, porque o formato muda. O canal estruturado continuou sendo o painel e os
  pedidos de terminal.
- **[Fato]** O mascote e `idetea mascot setup-hooks` gravam os hooks do Claude Code em
  `.claude/settings.local.json` sem apagar os que já existem (`crates/tea-app/src/mascot/hooks.rs`).

Esta RFC adota a opção B **como protocolo de sessão**, não como harness de modelo. A opção A continua fora.
Os hooks do mascote continuam: são efeito colateral (o mascote, a política do projeto), não o protocolo da sessão.

## Por que v1, e não o rascunho v2

**[Fato]** A documentação pública do ACP já descreve o v2 (`session/new` com outra forma, `auth/login`, estado
do turno em `state_update`). A crate 5.4.0 guarda esse rascunho atrás de `unstable_protocol_v2`, e o próprio
texto da crate diz que essa superfície pode mudar numa versão menor.

**[Fato]** O OpenCode (`opencode acp`), o alvo nativo deste marco, ainda fala v1. O suporte a v2 no OpenCode
é um pedido em aberto, não o padrão do binário. **[Referência]** O adaptador
`@agentclientprotocol/claude-agent-acp` **0.109.0** serve v2 só quando o cliente pede a versão 2 ou mais no
`initialize`; nos outros casos negocia v1. Pedir v1 é o caminho que os dois alvos falam hoje, com a API
estável da crate.

O que o marco 1 precisa — `initialize`, `session/new`, `session/prompt`, `session/update`,
`session/request_permission`, `session/cancel` — existe no v1. Quando o OpenCode (ou outro alvo nativo) falar
v2 por padrão, a troca fica para uma RFC seguinte, com a feature instável ou com a versão da crate que a
estabilizar. Não se fala os dois ao mesmo tempo neste marco.

## Como fica

Dois alvos, o mesmo cliente (`crates/tea-acp`, sem UI; o painel é `crates/ext/tea-ext-acp`):

1. **OpenCode**, nativo: `opencode acp`.
2. **Claude Code**, pelo adaptador oficial `claude-agent-acp`, pacote
   `@agentclientprotocol/claude-agent-acp@0.109.0`. Não se passa `--hide-claude-auth`: a assinatura e o
   Console continuam nos métodos que o adaptador anuncia. O processo herda o ambiente da IDE (`HOME`,
   `PATH`), para achar um login que já existe.

O cliente anuncia a capacidade `auth.terminal`. **[Referência]** Com ela, o adaptador do Claude oferece
`claude-ai-login` e `console-login` (ou `claude-login` quando o ambiente é remoto). Esses métodos são de
terminal: o cliente **não** os manda em `authenticate` / `auth/login`. A IDE oferece abrir o comando numa aba
de terminal. Se a sessão volta `authRequired` (`-32000`), a IDE mostra a mensagem e os métodos, e a tentativa
termina. Não fica pendurada.

Permissão: `session/request_permission` abre um diálogo e um cartão no painel. A resposta é a opção que a
pessoa clicou (`allow_once`, `reject_once`, ou outra que o agente mandou). Esc, clique fora e fechar **não**
escolhem opção nenhuma: o pedido continua aberto no painel. Cancelar o turno manda `session/cancel` e só
então a resposta `cancelled`. Não há lista de auto-aprovação neste marco. O `[agents] allow` continua valendo
só para terminais pedidos por arquivo, como hoje.

O que não muda:

- o lançador de aba de terminal (`Ctrl+Shift+A`, `idetea agent`);
- o painel Interfaces (`idetea ask`);
- os terminais pedidos em `.idetea/agents/`;
- `idetea mascot setup-hooks` e o comando `idetea mascot`.

### Gancho do mascote

Os eventos da sessão (`tea_acp::session::mascot_cue`) já dizem quando a sessão pede atenção, quando uma
ferramenta está em andamento, quando o turno acaba bem e quando falha. São os mesmos estados do mascote
(`attention`, `working`, `celebrating`, `error`). Neste marco a IDE **não** grava `.idetea/mascot.json` a
partir da sessão: o Claude Code, quando roda, continua disparando os hooks. Ligar os dois ao mesmo tempo
duplicaria o mascote. O gancho fica para um marco seguinte, que pode usar `mascot_cue` sem raspar o TUI.

Fora deste marco, como na issue: Codex e outros adaptadores, retomar sessão, terminais ACP, diffs da sessão,
`Ctrl+L` virando `session/prompt`, e auto-aprovação.

## Onde mora

| Peça | Onde |
| --- | --- |
| Protocolo, alvos, permissão, gancho do mascote | `crates/tea-acp` |
| Agente de mentira dos testes (stdio, sem rede) | binário `tea-acp-stub` |
| Painel, diálogo, textos | `crates/ext/tea-ext-acp` |
| Caminho do binário standalone do Claude, só no config do usuário | `[agents] claude_acp` |

O `[agents]` do projeto continua ignorado, como na RFC 0016: um repositório de outra pessoa não aponta um
binário.

## Decisões

1. **Opção B da RFC 0012, como protocolo de sessão.** A opção A (chat que chama API de modelo) continua fora.
   Não se raspa a saída do TUI (RFC 0016).
2. **ACP v1 estável**, crate `agent-client-protocol` =5.4.0, sem `unstable_protocol_v2`, até um alvo nativo
   falar v2 por padrão.
3. **Dois alvos no marco 1:** `opencode acp` e `claude-agent-acp` 0.109.0. Sem `--hide-claude-auth`.
4. **Permissão explícita.** Fechar o diálogo não aprova. Sem auto-aprovação.
5. **Hooks do mascote intactos.** `mascot_cue` é o gancho; a sessão ainda não dirige o mascote.

## Andamento

- **2026-11-20 — marco 1.** Cliente v1 em `tea-acp`, painel em `tea-ext-acp`, testes de permissão
  (permitir, recusar, dispensar, cancelar) e uma sessão contra o stub em stdio. Texto da interface nos oito
  idiomas. Como experimentar: [docs/acp.md](../acp.md).
