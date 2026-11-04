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

## Confirmação

Cada pedido aparece inteiro no painel do terminal, numa aba **Pedido de terminal**: quem pede, o título, o
comando exato e a pasta, com **Permitir e rodar** e **Recusar**. O painel abre sozinho e um aviso aparece, mas
nada pega o teclado: um Enter digitado em outro lugar não permite nada.

O que aparece e o que roda saem da mesma cópia em memória. Se o arquivo é regravado enquanto espera, a tela mostra
a versão nova (e um clique na versão antiga não roda nada); regravar depois de permitido não muda o que roda.

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

`ide` é o pid da IDE que gravou. Com duas IDEs no mesmo projeto, a que viu primeiro fica com o pedido; um
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
