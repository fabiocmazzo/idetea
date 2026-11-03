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

- Apagar o pedido encerra o terminal (o jeito de o agente parar o que pediu).
- Terminado, a aba mostra o código e fecha com Enter. O agente apaga o pedido e depois o status.
- Para pedir de novo com o mesmo nome, apagar os dois antes. Um pedido já tratado também volta a ser perguntado
  quando o status é apagado ou quando o pedido muda (o jeito de corrigir um `invalid`), mesmo que a IDE não tenha
  visto o arquivo sumir; só não volta enquanto roda nesta IDE ou enquanto outra IDE aberta cuida dele.

A pasta ganha um `.gitignore` com `*`: pedidos e status são da máquina, não do projeto.

As instruções para os agentes estão no bloco das interfaces (`crates/ext/tea-ext-ui/agents/instructions.md`,
seção "Running a command in an IDE terminal"), gravado pelo "ensinar os agentes".
