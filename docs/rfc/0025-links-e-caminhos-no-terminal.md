# RFC 0025 — Links e caminhos clicáveis no terminal

- **Status:** aprovada no painel em 2026-11-24, com as quatro recomendações (ver Decisões).
- **Continua:** [RFC 0024](0024-terminais-de-agente-muitos-e-retomaveis.md) (o terminal como peça central do
  trabalho com agentes) e a [ADR 0004](../decisions/0004-iced-term-no-workspace.md) (o `iced_term` no workspace,
  com as mudanças em `crates/iced_term/UPSTREAM.md`).

Marcadores: **[Fato]** conferido no código ou na fonte da crate (`iced_term` 0.8.0 no workspace,
`alacritty_terminal` 0.25.1, `vte` 0.15.0); **[Hipótese]** provável, não testado; **[Recomendação]** proposta,
ainda não decidida.

## Resumo

Quem trabalha com um agente no terminal vive lendo caminhos e endereços: o erro do `cargo` em
`src/main.rs:42:5`, o arquivo que o agente diz ter editado, a URL do PR que o `gh` imprime, o `localhost:3000` do
servidor de desenvolvimento. Hoje, para abrir qualquer um deles, é copiar e colar.

Esta RFC tem duas partes:

1. **Consertar os links que já existem.** O `Ctrl+clique` em URL já funciona, mas derruba a IDE quando a abertura
   falha, manda qualquer esquema (`ssh:`, `magnet:`) para o sistema e ignora os links OSC 8.
2. **Caminhos de arquivo abrindo na IDE.** `Ctrl+clique` em `src/main.rs:42:5` abre o arquivo numa aba, na linha e
   na coluna, só quando o arquivo existe.

## O que existe hoje

- **[Fato]** `crates/iced_term/src/backend.rs`: `URL_REGEX` reconhece `https://`, `http://`, `file://`, `mailto:`,
  `ssh:`, `git://`, `ftp://`, `magnet:`, `ipfs:`, `ipns:`, `gemini://`, `gopher://` e `news:`. Com `Ctrl`
  (`Cmd` no macOS) apertado, o link sob o mouse fica sublinhado (`LinkAction::Hover`), e `Ctrl+clique`
  (`BindingAction::LinkOpen`) chama `open_link`.
- **[Fato]** `open_link` passa o texto para `open::that(url)` e, se falhar, faz
  `panic!("link opening is failed")`: sem um programa associado ao esquema (ou sem `xdg-open`), **a IDE cai**.
- **[Fato]** Nenhum esquema é filtrado: `ssh:`, `magnet:` ou `file://` vão para o programa padrão do sistema.
- **[Fato]** OSC 8 (texto clicável que aponta para outro endereço) chega ao `alacritty_terminal`: `Term` grava o
  link em cada célula (`set_hyperlink`, `Cell::hyperlink()`). O `iced_term` não lê isso: o texto aparece, sem link.
- **[Fato]** OSC 7 (o shell avisando a pasta atual) é ignorado pelo `vte`/`alacritty_terminal` 0.25.
- **[Fato]** O processo do shell é `tty::Pty::child()` no `alacritty_terminal`; o `iced_term` entrega o `Pty` ao
  laço de eventos sem guardar o pid.
- **[Fato]** A extensão do terminal já pode abrir arquivos na IDE (`Context::open_file_at(path, line, column)`, o
  `Request::OpenFileAt` de `tea-core`), e `tea_core::os::open_url` abre um endereço no programa do sistema (o
  preview do Markdown já usa).
- **[Fato]** Nada disso está documentado: `docs/features.md` não menciona o `Ctrl+clique` no terminal.

## Por que `Ctrl+clique`, e não o clique simples

O clique simples já tem dono: posiciona e começa a seleção de texto e, em programas com mouse (vim, htop, a TUI de
alguns agentes), vai para o programa. Também é o clique acidental mais fácil. Todo terminal conhecido (VS Code,
Kitty, WezTerm, iTerm2, o próprio Alacritty) usa um modificador para seguir links, e o `iced_term` já usa `Ctrl`.
**[Recomendação]** manter.

## Parte 1 — Os links que já existem

### 1.1 Abrir sem derrubar a IDE

`open_link` deixa de abrir. O backend devolve uma ação nova, `Action::OpenLink(Link)`, e a extensão do terminal
decide o que fazer. Uma falha vira aviso ("não deu para abrir: …"), nunca `panic`.

### 1.2 Quais esquemas abrem

| esquema | o que acontece |
| --- | --- |
| `http://`, `https://` | abre no navegador (`tea_core::os::open_url`) |
| `mailto:` | abre no programa de e-mail |
| `file://` | abre **na IDE**, como os caminhos da parte 2 (a URL decodificada vira caminho) |
| os outros (`ssh:`, `magnet:`, `git://`, `ftp://`, `ipfs:`…) | não abrem. Continuam sublinhados, e o `Ctrl+clique` copia o endereço e avisa |

**[Recomendação]** O texto do terminal vem de programas e de agentes: um esquema que dispara um programa local
(cliente de torrent, `ssh`) não deveria rodar com um clique sem o usuário ver o endereço inteiro.

### 1.3 OSC 8

`ls --hyperlink`, `gcc`, `systemd`, `gh` e outros escrevem links OSC 8: o texto da tela (`PR #24`) aponta para
outro endereço (`https://github.com/...`). O `iced_term` passa a ler o link da célula sob o mouse (OSC 8 primeiro;
sem ele, a regex). Com `Ctrl` apertado, a aba mostra **o destino** num tooltip, porque, com OSC 8, o texto visível
pode não ter nada a ver com o destino. O destino segue as regras de 1.2.

### 1.4 Documentar

`docs/features.md` (6 línguas): `Ctrl+clique` em links e caminhos.

## Parte 2 — Caminhos de arquivo abrindo na IDE

### 2.1 O que vira link

Com `Ctrl` apertado, o texto sob o mouse é candidato quando bate com um destes formatos:

| formato | exemplo | de onde vem |
| --- | --- | --- |
| `caminho[:linha[:coluna]]` | `src/main.rs:42:5`, `./README.md`, `/etc/hosts` | gcc, clang, Go, Zig, TypeScript, ESLint, o próprio agente |
| `--> caminho:linha:coluna` | `--> crates/tea-core/src/lib.rs:208:5` | rustc, cargo, clippy |
| `File "caminho", line N` | `File "app/main.py", line 12, in main` | tracebacks de Python |
| `at … (caminho:linha:coluna)` | `at render (src/App.tsx:8:2)` | pilhas de Node e Deno |
| `caminho(linha,coluna)` | `src/App.cs(10,5)` | MSBuild, dotnet, tsc com `--pretty false` |

Um caminho precisa ter `/` (ou `\` no Windows) ou terminar numa extensão conhecida, para `ok`, `main` ou `1.2` não
virarem candidatos. `~/` é expandido.

### 2.2 Só o que existe

Um candidato só fica sublinhado se o arquivo **existe**. A conferência no disco acontece ao passar o mouse com
`Ctrl` (um `stat` por movimento, só com `Ctrl` apertado), nunca no texto todo. Sem isso, toda palavra com `/` ou
`.` acenderia, e um clique abriria uma aba vazia de um arquivo que não existe.

Uma pasta não abre na IDE. **[Recomendação]** revelar na árvore, se estiver dentro do projeto; fora dele, nada.

### 2.3 A partir de qual pasta

Um caminho relativo é tentado, em ordem, a partir de:

1. **A pasta atual do shell.** No Linux, `/proc/<pid do shell>/cwd`, com o pid guardado pelo `iced_term` ao criar
   o terminal (mudança no `UPSTREAM.md`). **[Hipótese]** No macOS, `proc_pidinfo`; no Windows, não há jeito
   simples, e o passo é pulado.
2. **A pasta onde a aba abriu** (a worktree, no caso de um agente pedido com `worktree`).
3. **A raiz do projeto.**

O primeiro que existe ganha. O processo do shell, e não o do comando, é o que importa: um `cargo build` roda na
pasta do shell. **[Hipótese]** Um programa que muda de pasta por conta própria (um `make -C sub`) imprime caminhos
relativos à pasta dele; o passo 3 cobre o caso comum (caminho relativo à raiz), e o resto fica de fora.

**[Recomendação]** OSC 7 seria a fonte mais certa da pasta atual (o shell avisa a cada `cd`), mas exige patch no
`vte`/`alacritty_terminal` e configuração no shell. Fica de fora desta RFC.

### 2.4 Abrir

- Dentro ou fora do projeto: abre numa aba, como qualquer arquivo (`Context::open_file_at`). Um arquivo de fora já
  abre hoje por `Ctrl+O`.
- Com linha e coluna: o cursor vai para lá. Linha além do fim do arquivo: o fim.
- Um arquivo binário ou grande demais segue o que o editor já faz (recusa ou aviso).
- Um caminho quebrado em duas linhas, porque não coube na largura: o `alacritty_terminal` marca a linha como
  continuação (`WRAPLINE`), e a busca do candidato atravessa a quebra.

### 2.5 Mostrar

Com `Ctrl` e o mouse sobre um candidato que existe: sublinhado e cursor de mão, como uma URL. O tooltip mostra o
caminho absoluto que vai abrir (com `:linha:coluna`), para não haver dúvida sobre qual pasta ganhou em 2.3.

## O que fica de fora

- O clique simples (sem `Ctrl`).
- OSC 7.
- Detectar símbolos (`Foo::bar`) ou nomes de teste para abrir: é tarefa do LSP, não do terminal.
- Abrir caminhos no programa do sistema (um `.png` abre na IDE, que já mostra imagens; um `.pdf`, no PDF da IDE).

## Riscos

- **Links enganosos.** Um agente ou um script pode imprimir um OSC 8 cujo texto não é o destino. Mitigação: o
  tooltip com o destino (1.3) e só `http(s)`/`mailto` saindo para o sistema (1.2).
- **Falso positivo em caminhos.** Mitigação: só o que existe (2.2) e as regras de 2.1.
- **Custo.** Uma regex e um `stat` por movimento do mouse com `Ctrl` apertado; nada sem `Ctrl`.
- **Diferença do upstream.** As mudanças no `iced_term` crescem; cada uma vai para o `UPSTREAM.md`, e o OSC 8 e o
  fim do `panic` valem mandar para o upstream.

## Testes

- O reconhecimento de candidatos (2.1) como função pura, com uma tabela de linhas reais (cargo, rustc, Python,
  Node, gcc, tsc, MSBuild) e as que **não** podem virar link (`ok`, `1.2.3`, `a/b` sem arquivo).
- A escolha da pasta (2.3) com pastas temporárias.
- Os esquemas (1.2): quais abrem, quais copiam.
- O fim do `panic`: abrir um esquema sem programa associado não derruba nada.
- Manual, pelo painel: `Ctrl+clique` num erro do `cargo build`, num traceback de Python, num `ls --hyperlink`, numa
  URL do `gh`, num caminho depois de `cd` para uma subpasta, num caminho quebrado em duas linhas.

## Ordem de implementação

1. Parte 1.1 e 1.2: a ação nova, sem `panic`, e os esquemas.
2. Parte 2: candidatos, pasta, só o que existe, abrir na IDE (`file://` junto).
3. Parte 1.3: OSC 8 com o destino no tooltip.
4. Documentação (1.4) e o `UPSTREAM.md`.

## Decisões (painel, 2026-11-24)

1. **Esquemas que saem para o sistema:** só `http(s)` e `mailto`. `file://` abre na IDE.
2. **Os outros esquemas** (`ssh:`, `magnet:`, `git://`…): o `Ctrl+clique` copia o endereço e avisa.
3. **Pasta clicada:** revela na árvore, se for do projeto; fora dele, nada.
4. **A pasta atual do shell:** lida do processo no Linux (`/proc/<pid>/cwd`) e no macOS (`proc_pidinfo`); no
   Windows, o passo é pulado e valem a pasta da aba e a raiz.
