# RFC 0019 — Semantic tokens do LSP

- **Status:** implementada em 2026-11-17 (opção A); widget no PR [#48](https://github.com/LuDog71FR/iced-code-editor/pull/48) do upstream.
- **Depende de:** [RFC 0017](0017-iced-code-editor-dependencia-ou-codigo-proprio.md) (API de decorações, aceita na
  [issue #41](https://github.com/LuDog71FR/iced-code-editor/issues/41)) e [RFC 0018](0018-inlay-hints.md) (texto
  virtual, que deixou o desenho do texto em pedaços por coluna).
- **Origem:** lacuna 3 do `docs/comparativo-ides.md` ("Sem Semantic Tokens do LSP"), escolhida no chat em
  2026-11-17.

Marcadores: **[Fato]** conferido nesta sessão no código da IDE ou do widget (`upstream/main` em `437802a`, fork
`idetea-main` em `e8a355d`); **[Referência]** dito por documentação externa, não conferido; **[Hipótese]** provável,
não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

Hoje a cor do código vem só da gramática (`syntect`), que olha o texto e não o programa: ela não sabe se `total` é
uma variável mutável ou um parâmetro, se `Foo` é uma struct ou um trait, se `vec!` é uma macro ou uma função. O
servidor de linguagem sabe, e o LSP tem um pedido para isso: `textDocument/semanticTokens`.

O trabalho tem duas metades, como os inlay hints: o cliente LSP e a fachada (que ficam na IDE) e um **estilo por
intervalo** no widget, que é a terceira peça da API de decorações da issue #41.

**[Recomendação]** O widget recebe cores prontas por intervalo (`set_range_styles`, uma camada por fonte) e expõe a
cor que o tema dele dá a um escopo TextMate (`scope_color`). A IDE traduz cada tipo do LSP para um escopo
(`function` → `entity.name.function`) e pede a cor ao widget. Assim as cores semânticas saem do mesmo tema que a
gramática usa, e o widget não precisa saber nada de LSP. Ligado por padrão, com `[lsp] semantic_tokens` para
desligar.

## O que existe hoje

- **[Fato]** O widget escolhe o tema do `syntect` pela claridade do fundo: `base16-ocean.dark` ou
  `base16-ocean.light` (`render/highlighting.rs`). `TokenColors` troca a cor de 6 grupos de escopo (chave, string,
  número, literal, comentário, pontuação), não de intervalos.
- **[Fato]** Cada linha vira uma lista `(cor, texto)` guardada em cache (`highlighted_line_cached`). No fork, o
  desenho já parte essas listas em pedaços `(coluna inicial, coluna final, cor)` para pôr o texto virtual entre
  eles (PR #46). Um estilo por intervalo é partir os mesmos pedaços nas colunas dos tokens e trocar a cor.
- **[Fato]** O cliente LSP da IDE (`crates/tea-lsp`) não pede `semanticTokens`. Os inlay hints deixaram pronto o que
  se repete aqui: limitar o intervalo ao documento, descartar respostas velhas por token, o `refresh` do servidor
  e mover marcas junto com o texto até a resposta nova (`Editor::follow_edits`).
- **[Fato]** O mantenedor do widget aceitou a API de decorações na issue #41 e já juntou o que mandamos (#42 a #45,
  #47). O PR do texto virtual (#46) está aberto.

## O protocolo

**[Referência]** LSP 3.17, `textDocument/semanticTokens`:

- O servidor anuncia uma **legenda** no `initialize`: a lista de tipos (`namespace`, `type`, `struct`, `enum`,
  `interface`, `typeParameter`, `parameter`, `variable`, `property`, `enumMember`, `function`, `method`, `macro`,
  `keyword`, `comment`, `string`, `number`, `operator`...) e de modificadores (`declaration`, `readonly`, `static`,
  `deprecated`, `mutable` no rust-analyzer, `defaultLibrary`...). Os servidores acrescentam os seus
  (`lifetime`, `selfKeyword`, `builtinType` no rust-analyzer).
- `semanticTokens/full` devolve o arquivo todo; `full/delta` só o que mudou desde a resposta anterior;
  `semanticTokens/range` um trecho. Cada token são 5 inteiros relativos ao anterior (linha, coluna, comprimento em
  UTF-16, tipo, modificadores em bits).
- `workspace/semanticTokens/refresh`: o servidor pede para reconsultar (o rust-analyzer faz isso depois de
  indexar).
- **[Referência]** O rust-analyzer responde `full`, `full/delta` e `range`; o clangd, ao menos `full` e
  `full/delta`; gopls, pyright e o typescript-language-server também respondem. Os testes do passo 1 conferem
  os dois primeiros.

## O que muda no widget

| Peça | O que muda |
|---|---|
| API | `set_range_styles(camada, Vec<RangeStyle>)`, `clear_range_styles(camada)`; `RangeStyle { line, start, end, color }`. Mesma forma do texto virtual: uma lista por camada, guardada por linha. |
| Desenho | os pedaços `(início, fim, cor)` de cada linha são partidos nas colunas dos estilos; a cor do estilo ganha da gramática. O texto virtual continua entre os pedaços. |
| Cor de um escopo | `scope_color("entity.name.function") -> Option<Color>`: a cor que o tema do `syntect` em uso (e o `TokenColors`) dá ao escopo. |
| Cache | nenhum novo: os estilos entram no desenho, não no cache da gramática. Trocar a camada limpa o cache do canvas. |
| Edição | o widget não move os estilos; quem é dono da camada troca. Um estilo além do fim da linha é cortado. |

**[Hipótese]** De 400 a 700 linhas no widget com testes. É bem menor que o texto virtual: não muda nenhuma
largura, então cursor, clique, seleção e quebra de linha ficam como estão.

Negrito e itálico ficam fora: o widget desenha cada pedaço com uma fonte só (a do código, ou a padrão para CJK), e
trocar o peso por pedaço muda a largura dos caracteres em fontes que não são monoespaçadas em todos os pesos.

## O que muda na IDE

- **tea-lsp:** guardar a legenda do `initialize`; `Client::semantic_tokens(path, token)` com `full` (e `full/delta`
  quando o servidor tiver, guardando o `resultId`); decodificar os 5-inteiros para `SemanticToken { range, kind: String,
  modifiers: Vec<String> }` em caracteres; `Event::SemanticTokens` e
  `Event::SemanticTokensRefresh`.
- **tea-editor:** pedir quando o servidor termina de analisar (o mesmo gatilho dos inlay hints); mover os tokens
  com o texto até a resposta nova; traduzir tipo e modificadores para um escopo e pedir a cor ao widget.
- **Mapa tipo → escopo.** **[Referência]** Inspirado no que o VS Code usa como reserva quando o tema não define
  cores semânticas (não conferido item a item):

  | Tipo LSP | Escopo TextMate |
  |---|---|
  | `function`, `method` | `entity.name.function` |
  | `macro` | `entity.name.function.macro` |
  | `struct`, `class`, `enum`, `interface`, `type`, `typeParameter` | `entity.name.type` |
  | `parameter` | `variable.parameter` |
  | `variable` | `variable.other.readwrite` |
  | `property` | `variable.other.property` |
  | `enumMember` | `variable.other.enummember` |
  | `namespace` | `entity.name.namespace` |
  | `keyword`, `selfKeyword` | `keyword` |
  | `lifetime` | `storage.modifier.lifetime` |
  | `comment`, `string`, `number` | ignorados: a gramática já pinta |

  Modificadores: `mutable` → `variable.other.mutable` (quando o tema tem); `deprecated` não muda a cor nesta
  versão.
- **Config:** `[lsp] semantic_tokens = true`; sem atalho (diferente dos hints, ninguém costuma alternar).

## As opções

### A. Cores prontas por intervalo, a IDE escolhe pelo tema do widget (recomendada)

O widget só pinta intervalos com a cor dada e diz a cor de um escopo; a IDE faz o mapa do LSP.

- A favor: o widget fica genérico (serve para destacar referências, busca semântica, o que for); as cores saem do
  mesmo tema da gramática; é a forma que a issue #41 propôs (`RangeStyle`).
- Contra: a IDE precisa do `scope_color` do widget, mais uma API pública.

### B. O widget recebe escopos, não cores

`RangeStyle { scope: "entity.name.function" }`, e o widget resolve a cor no desenho.

- A favor: troca de tema sem a IDE mandar de novo.
- Contra: o widget passa a carregar a ideia de escopo nas decorações, e quem não usa `syntect` não tem como pôr
  uma cor qualquer. A troca de tema já faz a IDE mandar tudo de novo (`set_theme` repõe os inlay hints).

### C. Uma paleta própria por tipo LSP, fora do tema

Cada tipo com uma cor fixa por tema claro/escuro.

- A favor: controle total, sem depender da cobertura do `base16-ocean`.
- Contra: mais uma paleta para manter, e as cores semânticas brigam com as da gramática.

**[Hipótese]** Com o `base16-ocean`, a diferença visível é moderada: ele tem uns 8 tons, e funções, tipos e
macros já saem diferentes pela gramática em Rust. O ganho maior é em parâmetro x variável x campo, em tipos que a
gramática não reconhece (um `Foo` usado como valor) e em linguagens com gramática fraca.

## Recomendação

**[Recomendação]** Opção **A**, nesta ordem:

1. tea-lsp: legenda, `semanticTokens/full` (o `full/delta` depois, se o tamanho da resposta pesar), testes com
   clangd e rust-analyzer;
2. widget: `set_range_styles` e `scope_color` no fork, PR no upstream citando a issue #41 (depois do #46, para não
   conflitar no desenho);
3. fachada e app: pedir, mover com o texto, mapa tipo → escopo, config;
4. teste manual no painel e documentação.

## Perguntas abertas

1. **Ligado por padrão?** (recomendo sim, como os inlay hints)
2. **Variável mutável:** só cor (quando o tema tem `variable.other.mutable`), ou sublinhado como o VS Code faz com
   o rust-analyzer? O sublinhado precisaria de mais um campo no `RangeStyle`.
3. **Esperar o #46 entrar** antes de mandar o PR dos estilos, ou mandar já em cima dele?

## Decisões

1. **Opção A** (2026-11-17, pelo painel), na ordem da recomendação.
2. **Ligado por padrão**, com `[lsp] semantic_tokens = false` para desligar.
3. **Variável mutável:** só a cor, quando o tema tiver `variable.other.mutable`; sem sublinhado.
4. **PR do widget já**, em cima do #46 (texto virtual).

## Andamento

- **2026-11-17, passo 1 (tea-lsp):** legenda das capacidades (`semantic_legend`, só com `full`),
  `Client::semantic_tokens`, `Event::SemanticTokens`/`SemanticTokensRefresh`, decodificação em `protocol.rs`.
  Testes com clangd (parâmetro x variável x função) e rust-analyzer (variável mutável com acentos antes, em
  caracteres) em `tests/semantic_tokens.rs`. **[Fato]** A primeira resposta do rust-analyzer pode vir antes de o
  projeto carregar: sem `mutable` na declaração e com os usos da variável como `const`; o `refresh` dele traz a
  certa.
- **2026-11-17, passo 2 (widget):** `RangeStyle`, `set_range_styles`/`clear_range_styles`/`range_styles` e
  `scope_color` no fork, ramo `pr/range-styles` (em cima do `pr/virtual-text`). O desenho parte os pedaços da linha
  nas colunas dos estilos (`apply_range_styles`); nenhuma largura muda.
- **2026-11-17, passo 3 (fachada e app):** pedidos nos mesmos momentos dos inlay hints, tokens andando com o texto
  (`follow_edits`), camada `"semantic-tokens"`, `[lsp] semantic_tokens`.
- **[Fato] O tema padrão pinta todo `variable.*` da mesma cor** (`#BF616A` no `base16-ocean.dark`): parâmetro,
  variável, variável mutável, campo e membro de enum. Seguindo o mapa da tabela, todas as variáveis do arquivo
  ficariam vermelhas sem distinguir nada. Mudou:
  - `variable` comum: sem escopo (fica com a cor da gramática);
  - `variable.other.mutable` só vale quando o tema dá a ele uma cor diferente da de `variable.other.readwrite`
    (decisão 3: "só a cor, quando o tema tiver"); no `base16-ocean`, não dá;
  - parâmetros e campos continuam com `variable.parameter`/`variable.other.property`, e assim se destacam das
    variáveis locais;
  - `entity.name.type` e `entity.name.namespace` não têm regra no `base16-ocean`: tipos e módulos ficam com a cor
    da gramática.
- **2026-11-17, teste manual:** parâmetros, campos e edição passaram. As chamadas de função saem com a cor da
  definição (`#8FA1B3`), mas no `base16-ocean` ela é próxima da que a gramática já dava (`#96B5B4`): a diferença é
  sutil. **[Fato]** O rust-analyzer não manda tokens para os argumentos dentro de um `println!`; o que está ali fica
  com a cor da gramática. Um tema de sintaxe com mais contraste fica para outra RFC.
- **2026-11-17, upstream:** PR [#48](https://github.com/LuDog71FR/iced-code-editor/pull/48) (em cima do #46); a IDE
  usa o fork em `cf623d8` (`idetea-main`).
