# RFC 0006 — A IDE como CLI para agentes

- **Status:** aprovada em 2026-11-03 (pelo painel), com as decisões do fim. Base: `main` em `9155f57`.
- **Depende de:** [RFC 0005](0005-agentes-e-projetos.md) (terminais de agente, worktrees, instruções dos agentes) e
  [RFC 0001](0001-interfaces-declarativas.md) (painel Interfaces).
- **Origem:** comentários do usuário (2026-11-03). No painel: "uma RFC do idetea usar o LSP do serena (o LSP é
  licença MIT) […], e o idetea ter funcionalidades para agente como cli. Tipo, com replace, rename e tudo que o LSP
  prover e mais coisas úteis para o projeto". No chat, depois da primeira versão: "precisa do LSP para isso? Um
  idetea se comportar apenas como um cli que facilita muita coisa para um agente, para que tenha que raciocinar
  menos para algumas tarefas, é vantajoso?". Junta o item 7 do `melhorias.md` ("idetea como CLI para agentes").

Marcadores: **[Fato]** conferido no código, no uso ou na fonte citada; **[Hipótese]** provável, não testado;
**[Recomendação]** proposta, ainda não decidida.

## Resumo

**Sim, vale, e a maior parte não precisa de LSP.** O que mais faz o agente "raciocinar" hoje não é o código: é o
protocolo. Para perguntar uma coisa no painel ele escreve um temporário, confere, renomeia, espera num laço,
lê, apaga três arquivos e controla a revisão. Para pedir um terminal, outro tanto. Cada passo é uma chance de errar.
Um comando que faz tudo isso numa chamada (`idetea ask`, `idetea run`) tira esse peso, e só a IDE TEA pode oferecê-lo.

O LSP entra depois, para o que só ele dá com segurança: referências, renomear em todo o projeto, símbolos por nome,
trocar o corpo de uma função. Ali o diferencial da IDE TEA é menor (os harnesses começam a trazer LSP próprio), mas
existe: o servidor já aquecido, o texto que o usuário não salvou e a edição que aparece no editor, com desfazer.
O Serena (SolidLSP, MIT) serve de referência para configurar os servidores.

## 1. O peso de hoje [Fato]

Passos que um agente faz hoje, seguindo as instruções do `AGENTS.md`/`CLAUDE.md`:

| Para… | Passos |
|---|---|
| perguntar algo no painel | apagar resposta velha; escrever `.idetea/ui/.<nome>.json.tmp`; `.idetea/ui/check`; `mv`; laço de espera tocando `<nome>.wait`; ler a resposta; conferir `revision` e `ide.outdated`; apagar a resposta; mais tarde, apagar o documento; lembrar a maior revisão já usada |
| rodar algo num terminal da IDE | escrever o pedido (temporário e `mv`); laço lendo o `state` do status; ler a resposta (`answer`); apagar pedido, status e resposta, nessa ordem |
| saber o que o usuário está vendo | não dá |
| mostrar um arquivo ao usuário | um documento só com um `file` com `open: true` |

Erros que aconteceram nesta mesma sessão, todos de protocolo e não de código: um laço de espera que disparou com a
resposta de uma revisão velha (o agente esqueceu de apagá-la antes); um pedido de terminal que a IDE não perguntou
de novo porque o arquivo foi apagado e regravado em menos de 500 ms; documentos com a revisão errada. As
instruções do protocolo ocupam hoje umas 150 linhas no `CLAUDE.md` de cada projeto.

## 2. Etapa 1: ajudantes do protocolo, sem LSP [Recomendação]

Comandos que embrulham os protocolos que já existem. Os arquivos continuam sendo o protocolo (quem não tem o
`idetea` no PATH segue funcionando); o comando é o jeito fácil.

### `idetea ask`

```sh
idetea ask pergunta.json                 # o documento inteiro (ou pelo stdin: idetea ask -)
idetea ask --title "Próximo passo" --text "O que fazer agora?" --options "lacunas|melhorias|outra"
idetea ask --title "Apagar 3 arquivos?" --confirm
```

- Escolhe o nome (ou `--name`), cuida da revisão (sempre maior que a última usada com esse nome), apaga resposta
  velha antes de publicar, confere como o `check`, grava pelo temporário, espera tocando o `.wait`, lê a resposta,
  confere `revision` e avisa `ide.outdated`, e apaga a resposta.
- Imprime a resposta (JSON) ou só um valor (`--value /option`). `comment` vem junto, para não se perder.
- Formas curtas: `--options` vira um `select` com "Confirmar"; `--confirm` vira "Sim"/"Não"; `--input "Nome"` um campo
  de texto. Para formulários de verdade, o JSON.
- Espera até `--timeout` (padrão: 10 minutos). Sem resposta, sai com código **4** e deixa o documento no painel;
  `idetea ask --resume <nome>` volta a esperar a mesma pergunta. `--keep` deixa o documento depois da resposta (para
  mostrar o resultado numa revisão seguinte); `idetea ask --close <nome>` tira.
- Saída: 0 respondido; 2 documento inválido (com o que está errado); 4 sem resposta no prazo.

### `idetea run`

```sh
idetea run --title "Servidor" -- npm run dev
idetea run --title "Revisor" --worktree agente/revisao -- claude -p 'Review the last commit. Write to $IDETEA_ANSWER.'
```

- Escreve o pedido de terminal (RFC 0005), espera o fim, imprime o estado, o código e o conteúdo da resposta
  (`IDETEA_ANSWER`), e limpa pedido, status e resposta (`--keep` deixa).
- O código de saída é o do comando. Os outros fins viram códigos próprios: **5** recusado, **6** fechado pelo
  usuário ou perdido (a IDE fechou), **2** pedido inválido (com o erro).
- `--no-wait` só pede e devolve o nome; `idetea run --wait <nome>` espera depois; `idetea run --stop <nome>` encerra
  (apaga o pedido).
- A confirmação do usuário continua igual: o comando não pula nada que o `[agents] allow` não pule.

### `idetea status` e `idetea open`

- **`idetea status`:** quais arquivos estão abertos, quais têm mudança não salva, o arquivo da aba ativa, a linha e a
  seleção do usuário, e se a IDE está aberta neste projeto. A IDE grava isso em `.idetea/state.json` (pelo
  temporário, quando muda, no máximo a cada 200 ms; fora do git como o resto de `.idetea/`), e o comando lê.
  A seleção só vai até um tamanho (por exemplo 4 KB); o agente pede o resto pelo arquivo.
- **`idetea open arquivo:linha[:coluna]`:** mostra algo ao usuário, sem pedir resposta: a IDE aberta abre a aba e põe
  o cursor. Pedido por arquivo (`.idetea/requests/open-<n>.json`, que a IDE lê e apaga), como os outros.
- Sem a IDE aberta: `status` diz isso (código 3); `open` também.

### Instruções dos agentes

O bloco do `AGENTS.md`/`CLAUDE.md` passa a ensinar primeiro os comandos (umas poucas linhas por comando), com o
formato dos arquivos como referência para quem não tiver o `idetea`. Uma versão nova das instruções; o "ensinar"
atualiza os projetos como hoje.

**Custo:** pequeno a médio. Tudo já existe por arquivo; os comandos são Rust puro, sem rede e sem LSP. O que é novo
na IDE: gravar o `state.json` e atender o `open`.

## 3. Etapas seguintes: o código pelo LSP

### 3.1 O Serena e a licença [Fato]

Fontes: [repositório](https://github.com/oraios/serena), [LICENSE](https://raw.githubusercontent.com/oraios/serena/main/LICENSE),
[discussão #1986](https://github.com/oraios/serena/discussions/1986).

- O Serena é um servidor MCP em **Python** ("the IDE for your agent"): achar símbolo, visão geral dos símbolos de um
  arquivo, referências, declaração, implementações, diagnósticos; renomear; trocar o corpo de um símbolo, inserir
  antes/depois, apagar com segurança. Mover, *inline* e depurador só no backend pago do JetBrains.
- **Licença por componente:** o **SolidLSP** (`src/solidlsp/`, testes e `test/resources/`) é **MIT** (© Jain &
  Panchenko IT-Berater Partnerschaft / Oraios AI e colaboradores, 2025–2026). O **aplicativo Serena** voltou a ser
  **GPL-3.0-or-later** na v2. As ferramentas do agente não podem ser copiadas para a IDE TEA (Apache 2.0); a camada de
  LSP pode, com o aviso da MIT.
- O SolidLSP tem um arquivo por servidor (mais de 70): como achar o executável, os parâmetros de `initialize`, as
  pastas a ignorar e os desvios. Exemplo: o `gopls` começa o intervalo de `type`/`var`/`const` depois da palavra-chave
  e, sem o conserto, trocar o corpo gera `type type Foo`.

### 3.2 O que a IDE TEA já tem [Fato]

- `crates/tea-lsp`: cliente próprio, em threads, com hover, assinatura, completar, definição, implementação, tipo,
  referências, símbolos do arquivo e do projeto, hierarquia de chamadas, formatar, renomear e diagnósticos.
- Servidores embutidos: rust-analyzer, clangd, pyright, typescript-language-server, gopls, lua-language-server
  (mais pelo `[lsp]`). Quem os mantém é o app (`crates/tea-app/src/lsp.rs`), enquanto a janela está aberta.
- **Sem LSP não há estrutura de código em Rust puro:** o realce é do `syntect` (expressões regulares, não um
  parser); o tree-sitter resolveria parte (símbolos, corpo de função) mas é C. Para "trocar o corpo desta função"
  com segurança, é o LSP.
- O Claude Code já traz uma ferramenta de LSP própria **[Fato: visível nas ferramentas desta sessão]**: navegar pelo
  código, em parte, o agente já consegue sem a IDE.

### 3.3 Proposta [Recomendação]

- **Etapa 2, leitura:** `idetea code symbols | find | refs | def | impl | calls | hover | diagnostics`, falando com a
  **IDE aberta** por um socket Unix (`$XDG_RUNTIME_DIR/idetea/<hash da raiz>.sock`, `0600`). Servidor aquecido, e o
  texto do editor vale mais que o disco quando o arquivo tem mudança não salva.
- **Etapa 3, edições:** `idetea code rename | replace-body | insert-before | insert-after | delete | format |
  actions`, aplicadas ao editor (buffer, desfazer, aviso de quem pediu) quando o arquivo está aberto, e ao disco
  quando não está; `--dry-run` devolve o diff. Edições só dentro da raiz (ou do worktree do agente).
- **Etapa 4, sem janela:** `idetea serve` sobe só os servidores LSP quando a IDE não está aberta, no mesmo socket.
- **Etapa 5, mais linguagens:** portar do SolidLSP o conhecimento de cada servidor que o uso pedir, em
  `crates/tea-lsp/src/servers/<servidor>.rs` com `SPDX-License-Identifier: MIT` e o aviso de copyright num `NOTICE`.
  Não rodar o Python do SolidLSP ao lado (duas camadas de LSP, servidores em dobro, Python como dependência, e as
  edições não passariam pelo editor). Instalar servidores sozinha: não; a IDE só usa a rede quando o usuário pede.
- **MCP:** um `idetea mcp` fino, repassando os mesmos comandos, só se o uso pedir. A CLI funciona em qualquer agente.

## 4. Segurança [Recomendação]

- Nada novo roda processo sem passar pelo pedido de terminal (com confirmação, ou o `[agents] allow` do usuário).
- `ask` e `run` só fazem o que o agente já podia fazer escrevendo os arquivos; não ganham poder novo.
- O `state.json` diz o que o usuário está vendo, mas só dentro do projeto e para quem já lê o projeto; a seleção tem
  limite de tamanho. Fica fora do git.
- O socket (etapa 2) é do usuário (`0600`); edições só dentro da raiz; cada edição do agente vira aviso e desfazer.

## 5. Para decidir

1. Começar pela etapa 1 (`ask`, `run`, `status`, `open`, sem LSP)? (recomendado: sim)
2. `status` e `open` por arquivo em `.idetea/` (recomendado), ou já pelo socket?
3. As instruções dos agentes passam a ensinar os comandos primeiro, com os arquivos como referência (recomendado), ou
   continuam como estão e os comandos entram como opção?
4. `idetea ask` espera 10 minutos e sai com código 4 deixando a pergunta no painel (recomendado), ou espera sem prazo?
5. As etapas 2 a 5 (LSP) ficam como planejado aqui: socket na IDE aberta, edições no editor, SolidLSP como
   referência portada, MCP só se pedir? (recomendado: sim, decidindo os detalhes quando chegar a vez)

## Decisões (pelo painel, 2026-11-03)

1. **Começar pela etapa 1** (`idetea ask`, `run`, `status`, `open`), sem LSP.
2. **`status` e `open` por arquivo** em `.idetea/` (`state.json` e pedidos de `open`), como o resto do protocolo.
3. **As instruções dos agentes ensinam os comandos primeiro**; o formato dos arquivos fica como referência.
4. **`idetea ask` espera 10 minutos** (`--timeout` muda) e sai com código 4, deixando a pergunta no painel.
5. **As etapas 2 a 5 (LSP) ficam como planejado**: socket na IDE aberta, edições no editor, SolidLSP como referência
   portada para Rust com o aviso MIT, MCP só se o uso pedir. Os detalhes se decidem quando chegar a vez.

## Fontes

- Serena: https://github.com/oraios/serena (README, `src/solidlsp/`, `src/solidlsp/language_servers/gopls.py`)
- Licença por componente: https://raw.githubusercontent.com/oraios/serena/main/LICENSE
- Serena v2 de volta à GPL, SolidLSP continua MIT: https://github.com/oraios/serena/discussions/1986
- Código da IDE TEA: `crates/tea-lsp/src/client.rs`, `crates/tea-lsp/src/lib.rs`, `crates/tea-app/src/lsp.rs`,
  `crates/tea-app/src/cli.rs`, `crates/ext/tea-ext-ui/agents/instructions.md`.
