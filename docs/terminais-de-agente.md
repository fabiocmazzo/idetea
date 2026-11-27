# Terminais pedidos por agentes

Um agente pode pedir à IDE um terminal para rodar algo que o usuário precisa **ver e poder intervir**: outro
agente, um servidor de desenvolvimento, uma ferramenta interativa. O agente principal já roda processos sozinho; o
que o terminal da IDE dá é a tela, o teclado e o botão de fechar (RFC 0005, etapa 4).

## Pedido

O agente grava `.idetea/agents/<nome>.json` (nome: letras minúsculas, dígitos, `_` e `-`, até 64), por um
temporário e `mv`:

```json
{"format": "idetea-terminal", "version": 1, "title": "Revisor", "source": "claude",
 "command": "claude 'revise o último commit'", "cwd": "."}
```

- `command` (obrigatório, até 4000 caracteres) roda com `/bin/sh -c`: `&&`, pipes e variáveis funcionam.
  Caracteres de controle (menos quebra de linha e tab) são recusados: poderiam esconder parte do comando na tela.
- `cwd`: relativo à raiz do projeto (padrão: a raiz); precisa existir e ficar dentro do projeto.
- `title` (até 80) e `source` (até 40) só aparecem na tela.
- `ide` (desde a 0.17.0): o pid da IDE de onde o pedido saiu. Enquanto ela estiver aberta, as outras IDEs do mesmo
  projeto deixam o pedido para ela; se ela fechar sem tratar, qualquer uma atende. Quem roda numa aba da IDE acha o
  pid em `IDETEA_TERMINAL` (`<pid>-<id>`), e a raiz do projeto dessa IDE em `IDETEA_ROOT` (desde a 0.18.0). Só
  grave `ide` quando o pedido é para esse mesmo projeto: o `idetea run` e o `idetea agent` já fazem isso sozinhos.
- Campos desconhecidos (desde a 0.17.0) não invalidam o pedido: são ignorados, e a tela de confirmação diz quais.
  Até a 0.16.0, um campo desconhecido virava `invalid`.

## Confirmação

Cada pedido aparece inteiro no painel do terminal, numa aba **Pedido de terminal**: quem pede, o título, o
comando exato e a pasta, com **Permitir e rodar** e **Recusar**. O painel abre sozinho e um aviso aparece, mas
nada pega o teclado: um Enter digitado em outro lugar não permite nada.

O que aparece e o que roda saem da mesma cópia em memória. Se o arquivo é regravado enquanto espera, a tela mostra
a versão nova (e um clique na versão antiga não roda nada); regravar depois de permitido não muda o que roda.

Com **dois ou mais** pedidos esperando (desde a 0.22.0, [RFC 0029](rfc/0029-aprovacao-em-lote-dos-terminais.md)), a
aba lista todos, cada um inteiro e com os seus botões, e em cima **Permitir todos (N)** e **Recusar todos**. O
clique leva as cópias que estavam na tela: um pedido regravado depois, ou um que chegou depois, fica esperando. Cada
pedido permitido roda na sua aba, e o ponto da aba mostra o fim e o código.

Não há lista de comandos permitidos: todo pedido é confirmado (decisão da RFC 0005).

## Status

A IDE grava `.idetea/agents/<nome>.status.json`:

```json
{"format": "idetea-terminal-status", "version": 1, "state": "exited", "ide": 12345, "code": 0}
```

| `state` | quando |
| --- | --- |
| `pending` | a IDE viu o pedido e espera o usuário |
| `running` | permitido; rodando numa aba **Agente: <título>** |
| `exited` | terminou; `code` é o código de saída (gravado pelo script do terminal, antes de esperar o Enter) |
| `refused` | o usuário recusou |
| `closed` | o usuário fechou a aba antes do fim |
| `invalid` | pedido com problema; `error` diz qual. Corrigir e gravar de novo |
| `lost` | rodava numa IDE que fechou |

`ide` é o pid da IDE que gravou. Com duas IDEs no mesmo projeto, a que viu primeiro fica com o pedido (a não ser
que o pedido diga a sua `ide`, acima); um
`pending` ou `running` de uma IDE que já fechou é tratado de novo (pergunta outra vez, ou vira `lost`).

## Pela linha de comando: `idetea run`

O jeito fácil ([RFC 0006](rfc/0006-ide-como-cli-para-agentes.md)): uma chamada escreve o pedido, espera o fim e limpa.

```sh
idetea run --title "Servidor" -- npm run dev
idetea run --title "Revisor" --worktree agente/revisao -- "claude -p 'Review the last commit. Write to \$IDETEA_ANSWER.'"
```

- Depois de `--`: um argumento só é a linha de shell inteira; vários são palavras, cada uma posta entre aspas.
- Imprime a resposta (`IDETEA_ANSWER`) no stdout e o que aconteceu no stderr. Sai com o código do comando, ou
  **5** (recusado), **6** (fechado ou perdido), **2** (pedido inválido ou uso errado), **4** (`--timeout` vencido;
  o terminal segue).
- Limpa na ordem certa: o pedido, e só depois o status e a resposta. `--keep` deixa tudo.
- `--no-wait` só pede (imprime o nome); `--wait <nome>` espera depois; `--stop <nome>` encerra.
- Um lote (desde a 0.22.0): um `--no-wait` por comando, depois `--wait a b c`. Os pedidos aparecem juntos na tela;
  ele espera cada um na ordem (com um `--timeout` só para todos), diz o fim de cada um no stderr, põe a resposta de
  cada um no stdout depois de `==> nome <==` e sai com o primeiro código diferente de 0 (ou 0).
- Um nome que já tem pedido ou status é recusado (não troca o pedido de outro agente).

## Um agente pede algo a outro (worktree e resposta)

Etapa 5 da RFC 0005. O pedido aceita `"worktree": "<branch>"`:

```json
{"format": "idetea-terminal", "version": 1, "title": "Revisor", "source": "Claude", "worktree": "agente/revisao",
 "command": "claude -p 'Review the last commit. Write your findings to the file in $IDETEA_ANSWER.'"}
```

- Ao permitir, a IDE cria um worktree do git em `.idetea/worktrees/<nome>` com esse branch: um branch novo a partir
  do HEAD, ou o que já existe. Se a pasta já é um worktree com o mesmo branch, reusa; com outro branch, o pedido
  vira `invalid`. O `cwd` vale dentro do worktree (a mesma pasta relativa). A pasta `.idetea/worktrees/` ganha um
  `.gitignore` com `*`: o projeto não vê os worktrees no `git status`.
- A tela de confirmação mostra o branch e a pasta.
- Todo comando de pedido recebe `IDETEA_ANSWER`: o caminho de `.idetea/agents/<nome>.answer.md`, na raiz do projeto
  (não no worktree). É onde o ajudante escreve a resposta; quem pediu lê depois do `exited`.
- O status (`running` e `exited`) traz `"answer"` e, com worktree, `"worktree"` e `"branch"` (caminhos relativos à
  raiz).
- Apagar o pedido encerra o terminal e tira o worktree com `git worktree remove` (se o comando já terminou, isso
  espera o usuário fechar a aba), **sem** `--force`: com mudanças
  por commitar ou arquivos novos, o git recusa, o worktree fica e a IDE avisa. O branch fica sempre. O código do
  ajudante entra pelo fluxo de sempre (teste manual e aprovação antes do merge).
- Isso só acontece com a IDE aberta: um pedido apagado com a IDE fechada deixa o worktree (tire com
  `git worktree remove .idetea/worktrees/<nome>`).

Quem espera quem (a fila de cada agente no painel) fica para quando o uso pedir.

## Estado da aba e sinais dos agentes

RFC 0024, M1.1. Cada aba do terminal mostra um ponto com o estado: rodando (saída nos últimos 2 s), saída nova não
vista, pediu atenção, e o fim com o código (verde ou vermelho). Na aba que está na tela, só o fim aparece.

"Pediu atenção" vem do sino (`\a`) ou de um sinal do agente:

- Toda aba recebe `IDETEA_TERMINAL=<pid da IDE>-<id do terminal>` no ambiente.
- `idetea mascot attention` e `idetea mascot celebrating` (chamados pelos hooks `Notification` e `Stop` que o
  `idetea mascot setup-hooks` instala para o Claude Code), rodando com essa variável, gravam também
  `.idetea/terminals/<valor>.json`: `{"format": "idetea-terminal-signal", "version": 1, "signal": "asked"}` (ou
  `"turn-done"`).
- A IDE dona do pid lê o arquivo, apaga e marca a aba: "pediu sua atenção" ou "terminou a vez e espera você".
- O valor só vale com dígitos (`^\d+-\d+$`); outro valor é ignorado, então o nome do arquivo não sai da pasta. A
  pasta ganha um `.gitignore` com `*`.

Um agente que não usa os hooks do mascote pode gravar o mesmo arquivo (pelo temporário e `mv`), ou tocar o sino.

## Ao reabrir a IDE

RFC 0024, M2. As abas de terminal ficam em `$XDG_STATE_HOME/idetea/terminals/<pasta>-<hash>.toml`, gravado a cada
mudança. Ao reabrir:

- Os shells voltam sozinhos, com o nome dado, na pasta onde estavam (Linux e macOS; no Windows, na raiz). A aba
  que estava na tela e a divisão voltam como estavam.
- Os agentes abertos pelo lançador esperam "Retomar os agentes?" (*Retomar todos*, *Escolher…*, *Não*); a paleta
  "Terminal: retomar agentes da sessão anterior" mostra os que ficaram. Cada um volta na mesma pasta ou worktree.
- O Claude Code abre com `--session-id <uuid>` escolhido pela IDE e volta com `claude --resume <uuid>`; o Codex,
  com `codex resume --last` (a conversa mais recente daquela pasta). Um lançador do config do usuário pode dizer
  como retomar o seu agente, e isso vale também por cima do que a IDE sabe:

  ```toml
  [agents.launchers]
  "Meu agente" = { command = "meu-agente", resume = "meu-agente --continue" }
  ```

  Sem isso, os outros abrem uma conversa nova. Se a retomada sair com erro em até 15 s, a IDE avisa e aponta o
  lançador.
- Tarefas e terminais pedidos por agentes não voltam: o pedido vira `lost`, e quem pediu decide se pede de novo.

Com `[terminal] keep_scrollback = true`, as últimas 2 000 linhas de cada aba são gravadas ao fechar (fora do
projeto, só o usuário lê) e aparecem uma vez acima do prompt novo. Desligado por padrão: o terminal tem senhas e
tokens.

## Comandos que rodam sem confirmar

No config **do usuário** (`~/.config/idetea/config.toml`), uma lista de comandos que não precisam de confirmação:

```toml
[agents]
allow = ["npm run dev", "claude *"]
```

- `"comando exato"`: só esse comando, igual (espaços nas pontas não contam).
- `"começo *"`: o começo igual (palavra inteira) e, depois, só argumentos simples. O resto não pode ter
  `;`, `&`, `|`, `$`, crase, `<`, `>`, parênteses, chaves, `\` nem quebra de linha, então não dá para emendar outro comando, substituir nem
  redirecionar. `"*"` sozinho não libera nada. Cuidado com o que o próprio programa aceita: `"claude *"` também
  deixa passar `claude --dangerously-skip-permissions`.

O pedido permitido assim vai direto para `running`, numa aba "Agente: …", e a IDE avisa qual linha da lista o
deixou passar. A lista é lida de novo a cada pedido novo (não precisa reiniciar).

`[agents]` no config **do projeto** (`.idetea/config.toml`) é ignorado, com aviso: um agente escreve na pasta do
projeto e se daria permissão. O config do usuário também é um arquivo que um agente rodando com o seu usuário
consegue escrever; a lista protege contra um pedido escrito por engano ou por outro processo do projeto, não
contra um agente que já roda comandos livremente.

## Fim

- Apagar o pedido encerra o terminal (o jeito de o agente parar o que pediu). Se o comando já terminou, a aba
  fica com a saída e o código até o usuário fechar.
- Terminado, a aba mostra o código e fecha com Enter. O agente apaga o pedido e depois o status.
- Para pedir de novo com o mesmo nome, apagar os dois antes. Um pedido já tratado também volta a ser perguntado
  quando o status é apagado ou quando o pedido muda (o jeito de corrigir um `invalid`), mesmo que a IDE não tenha
  visto o arquivo sumir; só não volta enquanto roda nesta IDE ou enquanto outra IDE aberta cuida dele.

A pasta ganha um `.gitignore` com `*`: pedidos e status são da máquina, não do projeto.

As instruções para os agentes estão no bloco das interfaces (`crates/ext/tea-ext-ui/agents/instructions.md`,
seção "Running a command in an IDE terminal"), gravado pelo "ensinar os agentes".
