# RFC 0029 — Aprovação em lote dos pedidos de terminal

- **Status:** aprovada em 2026-11-27 (pelo painel); ver [Decisões](#decisões). **Implementada** (ver
  [Andamento](#andamento)).
- **Continua:** [RFC 0005](0005-agentes-e-projetos.md) (nada roda sem você ver o comando) e
  [RFC 0006](0006-ide-como-cli-para-agentes.md) (`idetea run`).

## Resumo

Um agente que quer rodar vários comandos separados (o servidor, os testes, um ajudante num worktree) faz vários
pedidos de terminal. Até a 0.21.0, a aba **Pedido de terminal** mostrava um por vez, com "mais N depois deste": um
clique por pedido, sem ver o conjunto. A pergunta veio de um usuário no Product Hunt: aprovar um lote e ainda ver
exatamente o que rodou.

Uma cadeia num pedido só (`cargo fmt && cargo test`) já resolvia parte disso; o lote cobre comandos que precisam de
abas separadas.

## Proposta

1. Com **dois ou mais** pedidos esperando, a aba lista todos. Cada um aparece inteiro, como na tela de um pedido só
   (quem pede, título, comando exato, pasta, worktree, campos ignorados), com os seus botões. Em cima ficam o aviso
   (uma vez), **Permitir todos (N)** e **Recusar todos**.
2. O clique leva as cópias que estavam na tela, como o **Permitir e rodar** de um pedido só: um pedido regravado
   depois do que você viu, ou um que chegou depois, não roda e continua esperando.
3. Cada pedido permitido roda na sua aba, como hoje. O ponto da aba já mostra o fim e o código (verde ou vermelho).
4. `idetea run --wait a b c` espera um lote: cada nome na ordem, com um `--timeout` só para todos. Ele diz o fim de
   cada um no stderr, põe a resposta de cada um no stdout depois de `==> nome <==` e sai com o primeiro código
   diferente de 0, ou 0 se todos deram 0. Com `--no-wait` para pedir cada um, o agente monta o lote sem campo novo
   no JSON.

## Opções que ficaram de fora

- **Um campo `batch` no pedido**, para só juntar pedidos do mesmo lote. Não mistura pedidos de agentes
  diferentes, mas pede um formato novo; a tela já diz quem pede cada um.
- **Uma tela de resumo do lote** com o estado e o código de cada um depois de permitir. As abas já mostram isso.

## Decisões

Aprovadas no painel em 2026-11-27:

1. **O que entra no lote:** todos os pedidos esperando.
2. **Depois de permitir:** cada um na sua aba, sem tela de resumo.
3. **Linha de comando:** `idetea run --wait` aceita vários nomes.

## Andamento

**2026-11-27:** implementada como proposta.

- `tea-ext-terminal`: `Message::AllowAll` e `Message::RefuseAll`; a tela de um pedido virou um cartão
  (`request_card`) que a lista repete. A frase "mais N depois deste" saiu (a lista mostra todos).
- `idetea run --wait` lê os nomes até a próxima opção (também aceita `--wait` repetido).
- As instruções para os agentes (versão `13e43f45`) explicam como montar um lote.
