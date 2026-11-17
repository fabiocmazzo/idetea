# RFC 0017 — `iced-code-editor`: dependência, fork ou código próprio

- **Status:** aprovada em 2026-11-16 (pelo painel): opção A, com as três etapas. Decisão registrada em
  [ADR 0001](../decisions/0001-iced-code-editor-como-dependencia.md). Base: `main` em `9b34b36` (0.8.0).
- **Depende de:** [RFC 0009](0009-ergonomia-do-editor.md) (caminho "PR no upstream + `[patch]`"),
  [RFC 0011](0011-basico-da-ide.md) (sublinhados de diagnóstico), [RFC 0015](0015-ilhas-e-acabamento-da-ui.md)
  (barra em pílula) e [RFC 0016](0016-lancador-de-agentes-e-memoria-do-projeto.md) (seleção pública para o `Ctrl+L`).
- **Origem:** pedido do usuário no chat (2026-11-16): "Pode criar uma RFC das limitações que o iced-code-editor
  traz e quais seriam as vantagens/desvantagens desse código vir para o projeto?"

Marcadores: **[Fato]** conferido nesta sessão no código do IDE TEA, no fork (`~/NewProjects/Belabs/iced-code-editor`)
ou no upstream (`LuDog71FR/iced-code-editor`, `main` em `48f9e89`); **[Referência]** dito por documentação externa,
não conferido; **[Hipótese]** provável, não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

O widget de edição é a peça mais importante da IDE e a única grande que não é nossa. Ele limita três itens do
roadmap: inlay hints, semantic tokens e valores de variáveis no texto durante o debug. Os três precisam desenhar
dentro do texto, e isso só o widget faz. As outras faltas do comparativo (code actions, snippets, seleção por AST,
encodings) dá para fazer na fachada `tea-editor`, sem mexer no widget.

Trazer o código para o projeto resolve o acesso, mas dobra o tamanho do código Rust que mantemos: o widget no
upstream tem **~43 mil linhas**, quase o mesmo que a IDE inteira (**~44 mil**). E o upstream está vivo. Desde que
o fork parou, saíram 26 commits em dez dias, com correções de IME e fontes CJK, foco, Shift+clique, e uma colagem
5000× mais rápida em linhas longas. Os dois PRs que mandamos (#36 e #37) entraram em dois a três dias.

**[Recomendação]** Não trazer o código agora. Alinhar o fork com o upstream, mandar os três patches que ficaram
só no fork e propor no upstream **uma API de decorações**, que é a única peça estrutural que falta. Trazer o
código só se uma das condições de saída do fim acontecer.

## O que existe hoje

- **[Fato]** `Cargo.toml` depende de `iced-code-editor = "0.5"` com `[patch.crates-io]` apontando para o branch
  `idetea` do fork (`9a6d7b2`). Só o crate `tea-editor` importa o widget (`SPEC.md`, `DESENVOLVIMENTO.md`).
  `tea-editor` tem ~3,1 mil linhas, com features `lsp-process` e `two-face`.
- **[Fato]** A fachada usa ~55 métodos públicos do widget (`cursor_position`, `content`, `lsp_*`,
  `lsp_position_at_point`, `viewport_scroll`, `set_diagnostics`, `selection_range`…).
- **[Fato]** O fork tem 7 commits a mais que a base (`097ce7f`, 2026-11-06): sintaxes extras (`register_syntaxes`),
  sublinhado de diagnósticos, barra em pílula e seleção pública. A seleção pública e a navegação por palavra já
  entraram no upstream (PRs #36 e #37, 2026-11-08). **Sintaxes extras, sublinhados e barra em pílula nunca viraram
  PR.**
- **[Fato]** O upstream está 26 commits à frente do fork (+7 mil / −1,5 mil linhas em 69 arquivos). A versão no
  `main` é a 0.7.0, e o CHANGELOG lista uma mudança que quebra a API (`Style` ganhou `syntax_theme_name` e
  `token_colors`). Pelo histórico do `main`, o upstream tem 1 mantenedor principal (~250 commits) e um colaborador
  frequente (~90 commits). PRs de terceiros entram em 0 a 3 dias.
- **[Fato]** A licença é MIT, e o arquivo `LICENSE` do crate traz o texto de copyright do iced. O IDE TEA é
  Apache-2.0, e MIT pode entrar desde que o aviso venha junto (como os de `licenses/` no pacote).
- **[Fato]** Outro projeto, o *rustman*, também mantém um fork do crate. O upstream acaba de importar parte dele
  (CHANGELOG, "port rustman fork fixes").

## Limitações que o widget impõe

### Contornadas na fachada (funcionam, com custo)

| Limitação | Contorno hoje | Custo |
|---|---|---|
| **[Fato]** Sem API de marcas na margem | `marks.rs`: camada por cima que pergunta a linha de cada faixa (`lsp_position_at_point`, ~50 chamadas por desenho). Breakpoints, linha parada e marcas do git | A largura da margem (`GUTTER_WIDTH` 45 + `FOLD_MARGIN_WIDTH` 14) é privada e está **copiada**; um teste quebra se mudar. O upstream acabou de criar `FoldMarkerPosition`/`FoldMarkers`, que mudam essa geometria. |
| **[Fato]** Faixa da linha parada só por cima do texto | Retângulo translúcido sobre o texto | Fica menos legível do que um fundo atrás do texto, e o sticky scroll pode cobrir a marca nas primeiras linhas (`SPEC.md`, Fase 5). |
| **[Fato]** O cliente LSP do widget ignora `publishDiagnostics` | Cliente próprio (`tea-lsp`) que implementa a trait `LspClient` do widget | Dois modelos de LSP convivendo; a UI de hover/autocomplete é do widget. |
| **[Fato]** O widget não sabe que o menu de autocomplete está aberto | `KeyGate` intercepta ↑/↓/Enter/Tab/Esc | Teclas da IDE (`Alt+←/→`, `Ctrl+Shift+V`, `Ctrl+Shift+A`) também passam por ali. |
| **[Fato]** Sem Shift+clique; o ponteiro não muda com Ctrl | `KeyGate` avisa antes do clique e troca o ponteiro | O upstream já tem Shift+clique (não lançado). |
| **[Fato]** O widget mede a fonte em `set_font` | Carregar a fonte antes de `iced::application` (`docs/licoes-iced.md`) | Uma pegadinha a mais para quem mexer no boot. |
| **[Fato]** Textos do widget (menu de contexto, busca) com i18n próprio (`rust-i18n`, YAML), sem japonês | Japonês cai no inglês | Dois sistemas de tradução (Fluent na IDE, `rust-i18n` no widget). |
| **[Fato]** Dependências que só o widget traz: `iced_aw`, `iced_font_awesome`, `rust-i18n` | — | Peso no build e no binário, e mais uma API para acompanhar a cada versão do iced. |

### Bloqueiam itens do roadmap (só mudando o widget)

| Item | Por que precisa do widget |
|---|---|
| **Inlay hints** (tipos inferidos, nomes de parâmetros) | É texto virtual no meio da linha: muda a medida das colunas, o cursor e o clique. A camada por cima não consegue empurrar o texto. **[Fato]** O upstream não tem nada de texto virtual. |
| **Semantic tokens do LSP** | A cor vem só do `syntect` dentro do widget. É preciso um gancho para sobrepor estilos por intervalo. **[Fato]** O upstream ganhou `token_colors`, mas por escopo TextMate, não por intervalo. |
| **Valores de variáveis no texto durante o debug** | É o mesmo texto virtual dos inlay hints, no fim da linha. |
| **Faixa de fundo atrás do texto** (linha parada, trecho do git aberto) | Precisa ser desenhada antes do texto, dentro do widget. |
| **Diff inline das edições do agente** (Accept/Reject hunk no editor) | Precisa de linhas "fantasmas" entre as linhas reais, ou seja, mexer no layout vertical. |

### Não dependem do widget (dá para fazer na fachada)

- **[Hipótese]** **Code actions (`Ctrl+.`):** `apply_lsp_text_edits` é público, e o menu pode ser uma sobreposição
  como a do hover, posicionada por `cursor_screen_position`.
- **[Hipótese]** **Encodings não UTF-8:** a fachada decodifica ao abrir e codifica ao salvar (`tea-editor/src/lib.rs`
  já lê os bytes); o widget só vê `String`.
- **[Hipótese]** **Seleção por AST e snippets:** precisam de uma API de "definir seleção" e "inserir com tab-stops".
  **[Fato]** Hoje o widget só tem `set_cursor`. São PRs pequenos, não motivo para trazer o código.

## As opções

### A. Seguir como dependência, com o fork só para o que ainda não entrou (hoje)

- **Vantagens:** as correções do upstream chegam de graça (IME, CJK, foco, desempenho). Mantemos ~3 mil linhas,
  não ~46 mil. O caminho já funcionou duas vezes (#36, #37).
- **Desvantagens:** o fork envelheceu. Os três patches que nunca viraram PR prendem a IDE na 0.5.2 enquanto o
  upstream anda. Cada mudança no widget leva 3 passos (branch no fork, rev no `Cargo.toml`, PR). E a fachada
  depende de detalhes privados (a largura da margem).

### B. Trazer o código para o workspace (`crates/tea-code-editor`, fork rígido)

- **Vantagens:**
  - Acesso total: inlay hints, semantic tokens, valores no texto, faixa atrás do texto e diff inline viram trabalho
    normal, sem negociar API.
  - `marks.rs`, o `KeyGate` e a cópia das larguras somem. Breakpoints, marcas do git e linha parada viram desenho
    nativo do widget.
  - Um commit muda o widget e a IDE juntos, com um CI só, testes juntos e nenhum `[patch]`.
  - Dá para tirar o que a IDE não usa: o `rust-i18n` (textos para o Fluent, com japonês), o `iced_aw`, o
    `iced_font_awesome`, o modo Vim do widget se a IDE tiver o dela e o cliente LSP embutido. Isso deixa build e
    binário menores.
  - Ninguém de fora decide o ritmo das versões, o que conta numa troca de versão do iced.
- **Desvantagens:**
  - **~43 mil linhas** a mais para uma pessoa manter, o que quase dobra o código Rust do projeto.
  - Perdemos o upstream: as correções de IME e CJK, que sem usuários dessas línguas a gente não acharia, as de
    desempenho (colar numa linha de 200 mil caracteres: 247 ms → 0,05 ms), as de foco e de menu de contexto. Cada
    correção vira um cherry-pick à mão, e depois de alguns meses de divergência isso deixa de ser viável.
  - Atualizar o iced (0.14 → 0.15) passa a ser trabalho nosso no widget inteiro.
  - O crate tem lints próprios mais rígidos que os nossos (`missing_docs`, `unwrap_used`, `panic` em `deny`). Ou
    mantemos dois regimes de lint no workspace, ou relaxamos e o código vai degradando.
  - Duplicar um projeto vivo e aberto, em vez de contribuir com ele, vai contra o espírito do manifesto. Os PRs
    também dão visibilidade à IDE.

### C. Trazer o código e continuar sincronizando com o upstream (vendor + `git subtree`)

- **Vantagens:** é a B com um caminho de volta. As correções do upstream entram por `git subtree pull`, e as
  nossas sobem por PR.
- **Desvantagens:** na prática é o fork da opção A, só que dentro do repositório. Os conflitos aparecem do mesmo
  jeito, e o histórico do IDE TEA passa a misturar commits do widget. Não resolve nada que a A com disciplina não
  resolva.

### D. Escrever um editor próprio (sobre `iced` canvas / `cosmic-text`)

- Fica de fora: é a B sem o ponto de partida. Só faria sentido junto com uma troca para tree-sitter e rope, e nada
  hoje pede isso.

## Recomendação

**[Recomendação]** Ficar na **A**, mas consertar o que a envelheceu, em três etapas:

1. **Alinhar o fork com o upstream 0.7:** rebase do branch `idetea` sobre `upstream/main`, ajustando a fachada à
   mudança de `Style`. A seleção pública e a navegação por palavra saem do fork, porque já estão no upstream.
   Shift+clique vem do upstream, e o pedaço do `KeyGate` que fazia isso pode sair. Teste manual do editor inteiro
   no painel.
2. **Mandar os três patches que ficaram no fork** como PRs: `register_syntaxes`, sublinhado de diagnósticos e barra
   em pílula (talvez como opção de estilo). Com eles aceitos, o `[patch]` some.
3. **Propor no upstream uma API de decorações**, desenhada com a fachada como primeira usuária e a partir de uma
   issue antes do código:
   - **marcas na margem** por linha (ícone/cor, clicáveis), com a geometria da margem pública;
   - **fundos de linha e de intervalo** desenhados atrás do texto;
   - **estilos por intervalo** sobre o `syntect` (semantic tokens, destaque de referências);
   - **texto virtual** inline e no fim da linha (inlay hints, valores do debug), com o cursor pulando por cima.

   É a peça que destrava 4 dos 5 itens bloqueados. Também serve a quem usa o widget para outras coisas (o
   rustman, por exemplo), o que aumenta a chance de entrar.

### Condições de saída: quando trazer o código (opção B)

**[Recomendação]** Reabrir esta RFC e trazer o código se qualquer um destes acontecer:

- o upstream recusar a API de decorações, ou deixar a issue e o PR sem resposta por mais de 4 semanas;
- o upstream ficar mais de 3 meses sem commits;
- uma mudança de rumo no upstream quebrar a fachada de um jeito que a gente não queira seguir (por exemplo,
  trocar o modelo de LSP);
- precisarmos de tree-sitter ou de uma estrutura de texto (rope) que o upstream não queira.

Se isso acontecer, a ordem seria: copiar o crate em `crates/tea-code-editor` com o histórico (`git subtree add`),
manter o `LICENSE` MIT e o aviso em `licenses/`, tirar `rust-i18n`, `iced_aw` e `iced_font_awesome` primeiro, e só
então mexer nas APIs.

## Correção que esta pesquisa revelou

**[Fato]** As **marcas do git na margem já existem** desde 2026-11-01 (`21d2cce`, `tea-editor/src/quick_diff.rs` e
`marks.rs`), com o trecho antigo e "desfazer" no clique. Mesmo assim, o `docs/comparativo-ides*.md` (matriz,
seção 7 e passo 2 da seção 9) e o roadmap do `docs/features.md` ("Git marks inside the editor") ainda dizem que
falta. Corrigido junto com esta RFC.

**[Fato]** Na mesma revisão apareceram mais duas faltas que já não eram faltas, ambas de 2026-11-01: trocar e
criar branch pela paleta (`231b860`, *Git: trocar de branch*) e o blame da linha do cursor na barra de status
(`4b10105`). A seção 7 do comparativo (6 idiomas) e o `docs/features*.md` (6 idiomas) foram atualizados.

## Decisões

1. **Opção A, com as três etapas** (2026-11-16, pelo painel).
2. As condições de saída ficam como estão acima.
3. O comparativo e o `features.md` foram corrigidos quanto às marcas do git, à troca de branch e ao blame.

## Andamento

- **Etapa 1 (feita em 2026-11-16):** fork rebaseado sobre o upstream 0.7 (branch `idetea-0.7`, `5b5c788`); a IDE
  usa o 0.7 desde a 0.9.0 e o Shift+clique saiu da fachada.
- **Etapa 3 (aberta em 2026-11-16):** proposta da API de decorações no upstream,
  [issue #41](https://github.com/LuDog71FR/iced-code-editor/issues/41). O prazo de 4 semanas das condições de saída
  conta a partir daqui.
- **Etapa 2 (aberta em 2026-11-16):** três PRs independentes, cada um a partir do `upstream/main`:
  [#42](https://github.com/LuDog71FR/iced-code-editor/pull/42) (sintaxes extras),
  [#43](https://github.com/LuDog71FR/iced-code-editor/pull/43) (sublinhado de diagnósticos, muda o `Style`) e
  [#44](https://github.com/LuDog71FR/iced-code-editor/pull/44) (barra em pílula). Quando entrarem, o `[patch]` sai.
- **Correção (2026-11-17):** a troca do documento inteiro deixava uma linha vazia a mais na cópia do servidor; o
  bug é do upstream. Quinto commit no fork (`50c3135`) e PR
  [#45](https://github.com/LuDog71FR/iced-code-editor/pull/45).
- **2026-11-17:** o mantenedor juntou os PRs #42 a #45 e aceitou a API de decorações na issue #41 ("It's Ok for me").
  O fork passa a ser o `upstream/main` mais o texto virtual (branch `idetea-main`), que vai como PR
  ([RFC 0018](0018-inlay-hints.md)).
