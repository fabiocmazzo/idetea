# Sessão ACP

A IDE pode falar [ACP](https://agentclientprotocol.com/) (Agent Client Protocol) com um agente de código, em
stdio. O agente continua com a conta, o modelo e as ferramentas dele. A IDE inicia o processo, mostra o texto
e pede permissão antes de cada ação. Não é um chat que chama a API do modelo, e não lê o texto do terminal.

A decisão está na [RFC 0022](rfc/0022-cliente-acp.md). O lançador de aba (`Ctrl+Shift+A`, `idetea agent`), o
painel Interfaces, os terminais pedidos em `.idetea/agents/` e o `idetea mascot setup-hooks` continuam como
estão.

## Protocolo

O cliente fala **ACP v1** (crate `agent-client-protocol` 5.4.0, fixada). O rascunho v2 fica de fora: o
OpenCode ainda fala v1, e o adaptador do Claude só serve v2 quando o cliente pede a versão 2. A feature
`unstable_protocol_v2` não entra neste marco.

## Dois alvos

Na paleta de comandos:

- **Agente (ACP): OpenCode** — roda `opencode acp`.
- **Agente (ACP): Claude Code** — roda `claude-agent-acp`, do pacote
  `@agentclientprotocol/claude-agent-acp@0.109.0`.

O Claude pelo npm precisa de **Node.js 22 ou mais novo**:

```sh
npm install -g @agentclientprotocol/claude-agent-acp@0.109.0
```

Cada release do adaptador também traz um arquivo só, com o `claude-agent-acp` e um `claude` junto, sem Node.
Se ele não estiver no PATH, aponte o caminho no config **do usuário** (o do projeto é ignorado, como o resto
de `[agents]`):

```toml
[agents]
claude_acp = "/caminho/claude-agent-acp"
```

A IDE não passa `--hide-claude-auth`. Um login que você já fez com `claude`, ou `ANTHROPIC_API_KEY` no
ambiente, vale: o processo herda `HOME` e `PATH`.

Se faltar o binário, o Node 22, ou a conta, a IDE explica e não deixa a sessão pendurada. No caso da conta,
os métodos de terminal que o adaptador anuncia (`claude-ai-login`, `console-login`, ou `claude-login` num
ambiente remoto) ganham um botão que abre o login numa aba de terminal. A IDE não chama `authenticate` para
esses métodos: o adaptador recusa.

## Várias sessões

Cada comando da paleta abre uma sessão **nova**, numa aba do painel; as outras continuam rodando. Duas do
mesmo agente ficam numeradas ("Claude Code", "Claude Code 2"). Os botões `+ OpenCode` e `+ Claude Code` ao
lado das abas fazem o mesmo. Fechar a aba (✕) encerra só aquela sessão. Quando o processo acaba, a aba fica
com a conversa, marcada como encerrada, até você fechar. "Cancelar o turno" vale para a aba à frente.

## Painel e mascote

A aba guarda a conversa: o que você mandou, a resposta do agente e uma linha por ferramenta, que muda no
lugar quando o status muda (não aparece uma linha nova a cada atualização). A conversa rola sozinha até o fim
enquanto o texto chega; se você subiu para ler, ela fica onde você está até você voltar ao fim, trocar de aba
ou mandar outra mensagem.

O estado vai para o mascote, com o nome da aba: "pensando…", "escrevendo…", a ferramenta em andamento, o
pedido de permissão (com prioridade, até você escolher), o fim do turno, a recusa, o erro, e os apartes
"pronto" e "encerrou" (estes dois somem em `chatty = "quiet"`). O pensamento do agente não entra na aba. Um
clique no balão abre o painel da sessão. Com uma pergunta aberta no painel Interfaces (ou um terminal esperando
confirmação), cada ação nova da sessão aparece por 4 s e o mascote volta para a atenção.

Se os hooks do Claude Code (`idetea mascot setup-hooks`) também rodarem dentro da sessão ACP, o estado que
eles gravam em `.idetea/mascot.json` tem prioridade sobre o da sessão; o mascote mostra um só.

## Permissão

Quando o agente manda `session/request_permission`, a IDE mostra um diálogo e o mesmo pedido no painel.
Permitir ou recusar é um clique na opção que o agente ofereceu. Esc, clique fora e fechar **não** permitem a
ação: o pedido continua no painel até uma escolha. Cancelar o turno manda `session/cancel` e responde
`cancelled`. Não há aprovação automática. O diálogo diz de qual sessão vem o pedido; se ela não está à
frente, a aba ganha um `●` até você escolher.

## Testes

O CI fala com um agente de mentira (`tea-acp-stub`), em stdio, sem rede e sem conta. Um Claude de verdade
precisa da sua conta ou de `ANTHROPIC_API_KEY`; isso não roda no CI.

```sh
cargo test -p tea-acp
```
