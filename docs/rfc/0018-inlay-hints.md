# RFC 0018 — Inlay hints do LSP

- **Status:** aprovada em 2026-11-17 (pelo painel): opção C. Decisão registrada na
  [ADR 0002](../decisions/0002-texto-virtual-no-fork-antes-do-upstream.md).
- **Depende de:** [RFC 0017](0017-iced-code-editor-dependencia-ou-codigo-proprio.md) e
  [ADR 0001](../decisions/0001-iced-code-editor-como-dependencia.md) (o widget como dependência, a API de decorações
  proposta na [issue #41](https://github.com/LuDog71FR/iced-code-editor/issues/41)).
- **Origem:** passo 4 da seção 9 do `docs/comparativo-ides.md` ("Inlay Hints do LSP (tipos inferidos e nomes de
  parâmetros opcionais): exibir dicas de tipo inline com um atalho rápido para ligar/desligar"), escolhido no painel
  em 2026-11-17.

Marcadores: **[Fato]** conferido nesta sessão no código da IDE ou do fork (`~/NewProjects/Belabs/iced-code-editor`,
`idetea-0.7` em `50c3135`); **[Referência]** dito por documentação externa, não conferido; **[Hipótese]** provável,
não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

Inlay hints são textos que o servidor de linguagem pede para mostrar dentro do código sem fazer parte do arquivo:
o tipo inferido depois de uma variável (`let total`**`: u32`**` = soma(a, b)`), o nome do parâmetro antes de um
argumento (`soma(`**`a: `**`1, `**`b: `**`2)`), o tipo no fim de uma cadeia de chamadas. É o único item de
"profundidade de LSP" do comparativo que falta à IDE.

A parte do LSP é pequena e fica na IDE. A difícil é o desenho: o widget não sabe mostrar texto que não está no
buffer, e um texto no **meio** da linha muda a posição de tudo que vem depois dele (cursor, seleção, clique,
quebra de linha, sublinhados). Isso só o widget pode fazer.

**[Recomendação]** Não esperar a issue #41 para começar, mas fazer no fork, na forma que a issue propõe
(`VirtualText`), em PRs separados para o upstream: primeiro uma refatoração sem mudança de comportamento que
centraliza a conversão coluna ↔ x, depois o texto virtual. Na IDE, os hints vêm desligados por padrão até o
teste manual passar, com um comando e um atalho para ligar e desligar.

## O que existe hoje

- **[Fato]** O cliente LSP (`crates/tea-lsp`) já faz hover, assinatura, completar, definição, referências,
  símbolos, hierarquia de chamadas e code actions, cada um com uma função em `Client` e um `Pending`. Não pede
  `textDocument/inlayHint`.
- **[Fato]** Ele já negocia a codificação das posições (`positionEncodings: ["utf-32", "utf-16"]`, `client.rs`) e
  converte UTF-16 ↔ caractere em `text.rs`.
- **[Fato]** `Editor::follow_edits` (`crates/tea-editor/src/lib.rs`) já move os diagnósticos junto com o texto
  enquanto o servidor não responde, com `tea_lsp::diff` e `shift_range`. Os hints precisam do mesmo.
- **[Fato]** O widget mede o texto como uma grade: cada caractere tem a largura `char_width` (ou `full_char_width`
  para CJK), por `measure_char_width` e `measure_text_width`. Não há texto virtual nem estilo por intervalo.
- **[Fato]** A conversão coluna → x não está num lugar só. `measure_text_width` é chamada 31 vezes em 8 arquivos
  (contando os testes que ficam dentro deles: desenho do texto, cursor, seleção, busca, colchetes, diagnósticos, IME, largura máxima do
  conteúdo) e `measure_char_width` em mais 5 arquivos (incluindo o clique, `calculate_cursor_from_point`, e a quebra de
  linha, `wrapping.rs`). Cada uma soma as larguras dos caracteres de `0` até a coluna.
- **[Fato]** A quebra de linha divide cada linha lógica em `VisualLine { logical_line, start_col, end_col }`, pelas
  larguras dos caracteres.
- **[Fato]** O upstream está ativo (5 commits em 16/11, incluindo um porte de melhorias do fork rustman), mas a
  issue #41 e os PRs #42 a #45 ainda não tiveram resposta (abertos há 1 dia). O prazo de 4 semanas da RFC 0017
  vence por volta de 14/12.

## O protocolo

**[Referência]** LSP 3.17, `textDocument/inlayHint`:

- O pedido leva um intervalo (o que está visível); a resposta é uma lista de `InlayHint { position, label, kind?,
  paddingLeft?, paddingRight?, tooltip?, textEdits? }`. `label` é texto ou uma lista de partes (cada uma com
  `tooltip`, `location` e `command` opcionais). `kind` é `1` (tipo) ou `2` (parâmetro).
- O servidor pode pedir para reconsultar tudo com `workspace/inlayHint/refresh` (o rust-analyzer faz isso depois
  do `cargo check`); o cliente anuncia `workspace.inlayHint.refreshSupport`.
- `inlayHint/resolve` completa o tooltip e os textEdits sob demanda.
- rust-analyzer, clangd (desde a 14), gopls, typescript-language-server e pyright/basedpyright respondem. **[Hipótese]**
  gopls e o typescript-language-server vêm com os hints desligados nas próprias opções do servidor.

## O que muda no widget

Um hint **no fim da linha** (tipo de uma cadeia de chamadas, valor de variável no debug) é fácil: é desenhado
depois do último caractere e não mexe em nenhuma coluna. Só a largura máxima do conteúdo (barra de rolagem
horizontal) precisa contar com ele.

Um hint **no meio** é texto virtual de verdade:

| Peça | O que muda |
|---|---|
| Coluna → x | o x de uma coluna soma também a largura dos hints antes dela. É a mudança daquelas chamadas todas. |
| x → coluna (clique, arrastar) | um clique em cima de um hint vai para a coluna mais próxima, antes ou depois dele. |
| Cursor | pula o hint: ← e → andam um caractere do buffer, nunca param "dentro" do hint. |
| Seleção, busca, colchetes, sublinhados | desenhados nas colunas reais; o fundo da seleção passa por baixo do hint quando ele está dentro dela. |
| Quebra de linha | a largura de um segmento conta os hints; um hint não é quebrado no meio. |
| Copiar, `content()`, LSP | não mudam: o hint nunca entra no buffer. |
| Edição | o widget não sabe quando um hint fica velho; quem o pôs (a IDE) troca a lista. Até lá, os hints da linha editada andam com o texto. |

**[Fato]** A forma que a issue #41 propõe já cobre isso: `VirtualText { position, text, color, side: Inline |
EndOfLine }`, numa camada por fonte (`set_decorations("lsp-inlay", …)`), com o cursor pulando por cima.

**[Hipótese]** O trabalho no widget é de 1,5 mil a 2,5 mil linhas com testes, a maior parte em reescrever essas
chamadas para uma função só (`line_x(line, col)` e o inverso) e nos testes de cursor e quebra de linha.

## As opções

### A. Esperar a issue #41 (o que a ADR 0001 diz hoje)

Nada muda até o mantenedor responder. Se ele aceitar e escrever a API, ou aceitar que a gente escreva, os hints
vêm depois. Se não responder até ~14/12, a RFC 0017 manda reabrir a decisão de trazer o código.

- A favor: zero divergência do upstream; nenhum trabalho jogado fora.
- Contra: semanas sem andar, e a resposta pode ser "não".

### B. Só hints no fim da linha agora, no fork

Desenhar o texto depois do último caractere. Pequeno (algumas centenas de linhas) e não toca o cursor.

- A favor: rápido, e serve depois para os valores de variáveis no debug.
- Contra: o rust-analyzer põe a maioria dos hints no meio (tipo depois do nome, nome do parâmetro antes do
  argumento). Mostrar todos no fim da linha ("`total: u32, a:, b:`") é confuso e não é o que o comparativo pede.

### C. Texto virtual completo no fork, como PRs para o upstream (recomendada)

Em três passos, cada um um commit no fork e um PR a partir do `upstream/main` que cita a issue #41:

1. **Refatoração sem mudança de comportamento:** uma função para coluna → x e outra para x → coluna, usadas por todas
   essas chamadas. Os testes atuais garantem que nada mudou. É útil mesmo que o upstream recuse o resto.
2. **`VirtualText`** inline e no fim da linha, numa camada por fonte, como a issue propõe (só a parte do texto
   virtual da API; marcas, fundos e estilos ficam para depois).
3. **Demo** na aba de exemplo do upstream.

Na IDE, em paralelo (não depende do widget):

- `Client::inlay_hints(path, range)`, `Event::InlayHints`, o `refreshSupport` e o pedido de novo quando o
  servidor manda `workspace/inlayHint/refresh`;
- pedido só do intervalo visível (mais uma margem), com espera curta depois de digitar ou rolar, e cancelado se
  o texto mudou antes da resposta;
- os hints andam com o texto (`follow_edits`) até a resposta nova;
- comando `lsp.toggle_inlay_hints` na paleta e em um atalho, `[lsp] inlay_hints = false|true` na config e em
  Configurações → Servidores de linguagem;
- cor: a do texto apagado do tema (`comment`), com fundo leve; padding dos lados quando o servidor pede;
- tooltip: o `tooltip` do hint no hover, quando houver (fase 2).

- A favor: anda agora; o que se escreve é exatamente o que a issue propõe, então ajuda o PR em vez de competir;
  o passo 1 é pequeno e fácil de aceitar.
- Contra: o fork cresce até os PRs entrarem (hoje 5 commits, viraria 7 a 8), e cada rebase sobre o upstream fica
  mais caro. Se o upstream mudar a renderização (ele acabou de mexer em gutter e tokens), o passo 1 conflita.

### D. Trazer o código para o projeto (opção B da RFC 0017)

Só faz sentido se uma das condições de saída da RFC 0017 acontecer. Os hints não justificam antecipar.

## Recomendação

**[Recomendação]** Opção **C**, nesta ordem:

1. Lado da IDE (cliente LSP, comando, config), com testes contra rust-analyzer e clangd como os de code actions.
   Sem o widget, só dá para testar os dados.
2. Passo 1 do widget (refatoração coluna ↔ x) no fork, PR no upstream.
3. Passo 2 do widget (`VirtualText`) no fork, PR no upstream.
4. Ligar na IDE, teste manual no painel, e depois documentação (`docs/features*.md`, comparativo, guia do LSP em
   8 idiomas).

Os hints vêm **desligados** por padrão na primeira versão; ligar por padrão depois de um tempo de uso.

Se o mantenedor responder à issue #41 antes do passo 2 com outra forma para a API, seguimos a forma dele.

## Perguntas abertas

1. **Ligados ou desligados por padrão** na primeira versão? (recomendo desligados)
2. **Atalho** para ligar e desligar. Livres e sem conflito com a keymap atual: `ctrl+alt+i` ("inlay") ou
   `alt+shift+i`.
3. **Quais tipos:** tipos e nomes de parâmetro (os dois do LSP), ou só tipos no começo?
4. A ADR 0001 diz que os inlay hints "esperam a API de decorações no upstream". A opção C muda isso: precisa de
   uma ADR nova que a complementa (sem substituir o resto da 0001).

## Decisões

1. **Opção C** (2026-11-17, pelo painel), na ordem da recomendação.
2. Os hints vêm **ligados** por padrão já na primeira versão.
3. Atalho **`ctrl+alt+i`** (comando `lsp.toggle_inlay_hints`).
4. **Tipos e nomes de parâmetro** desde o começo.
5. [ADR 0002](../decisions/0002-texto-virtual-no-fork-antes-do-upstream.md) complementa a 0001.

## Andamento

- **2026-11-17:** feito e aprovado no teste manual (painel, os 8 passos).
  - Cliente LSP: `Client::inlay_hints`, `Event::InlayHints`/`InlayHintsRefresh`; o intervalo é limitado ao fim do
    documento (o rust-analyzer recusa o pedido inteiro se passar dele). Testes com clangd e rust-analyzer.
  - Widget: `CodeEditor::set_virtual_text(camada, itens)` num commit só no fork (a refatoração e o texto virtual
    juntos: quase todas as conversões já passavam por `calculate_segment_geometry`), sobre o upstream que já tem os
    PRs #42 a #45.
  - IDE: pede o arquivo inteiro (não só o trecho visível) quando o servidor termina de analisar; os hints andam com
    o texto até a resposta nova.
  - Ficou para depois: a demo no upstream e o tooltip do hint.
  - PR no upstream: [#46](https://github.com/LuDog71FR/iced-code-editor/pull/46) (branch `pr/virtual-text`, o mesmo
    commit `e281d45` que a IDE usa).

