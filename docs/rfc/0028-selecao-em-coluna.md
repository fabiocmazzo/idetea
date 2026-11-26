# RFC 0028 — Seleção em coluna no editor

- **Status:** aprovada em 2026-11-26 (pelo painel); ver [Decisões](#decisões). **Implementada** (ver
  [Andamento](#andamento)).
- **Onde:** na fork do `iced-code-editor` ([ADR 0001](../decisions/0001-iced-code-editor-como-dependencia.md)), com
  a IDE só ligando os atalhos.

Marcadores: **[Fato]** conferido no código da fork (`rev cf623d8`); **[Referência]** como outro editor faz;
**[Hipótese]** provável, não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

Editar a mesma coluna em várias linhas seguidas (alinhar uma tabela em Markdown, mudar o prefixo de dez linhas,
apagar o começo de um bloco colado de um log) pede uma **seleção em coluna**: arrastar um retângulo e ganhar um
cursor por linha, cada um com a sua parte selecionada. O editor já tem vários cursores; falta o retângulo.

## O que existe hoje

- **[Fato]** Vários cursores (`editing/cursor_set.rs`), cada um com posição e âncora (a seleção). Digitar, apagar,
  copiar e colar agem em todos; colar com tantas linhas quanto cursores põe uma linha em cada
  (`editing/clipboard.rs`). O desenho mostra todos os cursores e seleções (`render/overlays.rs`).
- **[Fato]** Como ganhar cursores: `Alt+clique`, `Ctrl+Alt+↑`/`↓` (um cursor na linha de cima ou de baixo), `Ctrl+D`
  (a próxima ocorrência) e `Alt+Enter` na busca (todas as ocorrências).
- **[Fato]** `calculate_cursor_from_point` converte um ponto da tela em (linha, coluna) levando em conta a quebra de
  linha, a largura dos caracteres CJK (`measure_char_width`) e o texto virtual (inlay hints).
- **[Fato]** `Ctrl+Alt+↑`/`↓` põe o cursor novo na coluna **em caracteres** do cursor principal, cortada pelo tamanho
  da linha (`col.min(line_len)`), e o cursor novo vira o principal. Duas consequências:
  - passando por uma linha curta, a coluna fica cortada para as linhas seguintes (o editor não lembra a coluna que
    se queria);
  - com tabulações ou caracteres largos antes, o cursor novo não fica embaixo do outro na tela.
- **[Fato]** `Alt` + botão esquerdo é o `Alt+clique` (`input/events.rs`): acrescenta um cursor e não começa arraste.
  O botão do meio não é tratado pelo editor.
- **[Fato]** Na IDE, `Shift+Alt+↑`/`↓` já é "duplicar linha" (no editor) e `Alt+↑`/`↓` é "mover linha".
  `Ctrl+Shift+Alt` com setas está livre.

## Como os outros fazem

- **[Referência]** VS Code: `Shift+Alt+arrastar` (ou arrastar com o botão do meio) faz a seleção em coluna;
  `Ctrl+Shift+Alt+setas` no Linux e no Windows a estende pelo teclado. Linhas mais curtas que o começo do retângulo
  ficam de fora; as que terminam dentro dele ganham o cursor no fim.
- **[Hipótese]** O Zed segue de perto o VS Code nos atalhos de vários cursores; não conferi os de seleção em coluna.

## Proposta

1. **Arrastar um retângulo.** Com o modificador (ver Decisões) e o botão esquerdo, o arraste guarda o ponto de
   começo na tela. A cada movimento, para cada linha **do arquivo** entre a do começo e a do mouse:
   - a coluna de começo e a de fim são achadas pelo **x da tela** nessa linha, com a mesma conta do
     `calculate_cursor_from_point` (CJK e texto virtual já entram; as tabulações também, se a largura delas passar
     por ali — **[Hipótese]**, a conferir com teste);
   - a linha ganha um cursor com a âncora no começo e a posição no fim (arrastar para a esquerda inverte);
   - uma linha que acaba antes do começo do retângulo fica de fora; uma que acaba dentro dele ganha o cursor no fim,
     como no VS Code;
   - um retângulo de largura zero (só vertical) dá um cursor por linha, sem seleção: o jeito de digitar em várias
     linhas de uma vez.
2. **Linhas do arquivo, não da tela.** Com quebra de linha ligada, uma linha quebrada conta uma vez, e o x é medido
   no trecho da tela onde o mouse está. Linhas dobradas (fold) ficam de fora. **[Recomendação]** É o mais simples de
   prever; uma seleção por linhas da tela muda conforme a largura da janela.
3. **Lembrar a coluna da tela no `Ctrl+Alt+↑`/`↓`.** O cursor novo vai para a mesma posição **na tela** do primeiro
   cursor da série, não para a coluna em caracteres do anterior: uma linha curta no caminho não estraga as
   seguintes, e uma tabulação antes não desalinha.
4. **Pelo teclado.** O atalho (ver Decisões) estende o retângulo uma linha para cima ou para baixo, ou uma coluna
   para os lados, a partir do cursor principal.
5. **Sair do modo:** `Esc` volta a um cursor só, como hoje com vários cursores; um clique simples também.

Tudo isso fica na fork, com testes no estilo dos que ela já tem (`editing/cursor.rs`). A IDE só liga os atalhos e
documenta.

## Riscos

- **Desempenho no arraste:** um retângulo de milhares de linhas cria milhares de cursores a cada movimento do mouse.
  **[Recomendação]** Recalcular só quando a linha ou a coluna do mouse muda (como o arraste de hoje já faz), e um
  teto (por exemplo 10 000 cursores) com aviso.
- **Tabulações:** se a largura da tabulação na tela não passar pela conta do x, o retângulo fica torto em arquivos
  com tabulação. O primeiro teste a escrever.
- **Vim:** com o modo Vim ligado, `Alt+clique` já vira clique comum; a seleção em coluna fica desligada no Vim, que
  tem o próprio modo visual em bloco (fora desta RFC).
- **Atalho do sistema:** em alguns gerenciadores de janela do Linux, `Alt+arrastar` move a janela; por isso a
  recomendação é `Shift+Alt`, que costuma ficar livre.

## Tamanho

**[Hipótese]** De 300 a 500 linhas na fork, com testes: o arraste em retângulo é a maior parte; lembrar a coluna e o
atalho do teclado são pequenos depois dele. Mais a liberação da fork (`rev` novo no `Cargo.toml`) e a documentação.

## Decisões

Aprovadas no painel em 2026-11-26:

1. **Mouse:** `Shift+Alt+arrastar`, e também arrastar com o **botão do meio**.
2. **Teclado:** `Ctrl+Shift+Alt+setas`.
3. **Quebra de linha:** o retângulo segue as linhas do arquivo.

## Andamento

**2026-11-26:** implementada na fork (`input/update/column_select.rs`), como proposto:

- `Shift+Alt+arrastar` e o botão do meio começam o retângulo; `Ctrl+Shift+Alt+setas` o aumentam (para os lados, ele
  continua crescendo depois do fim da linha do cursor, para as linhas mais longas em volta).
- **Tabulações:** a largura delas na tela é fixa (`TAB_WIDTH` células, não paradas de tabulação), e a conta de x já a
  usava; o risco da RFC não se confirmou. Um teste cobre tabulação, CJK e texto comum na mesma coluna.
- **Quebra de linha:** o x conta do começo da linha do arquivo, somando o trecho da tela onde o mouse está.
- **Linhas curtas:** ficam de fora; a linha do mouse sempre fica com um cursor. Teto de 10 000 cursores, guardando as
  linhas perto do mouse (sem aviso por enquanto).
- **`Ctrl+Alt+↑`/`↓`:** guardam o x da tela do primeiro cursor da série enquanto o cursor principal for o último
  acrescentado.
- Desligada no modo Vim (o `Shift+Alt+clique` vira clique comum, como o `Alt+clique` já fazia).
- Na IDE: a janela de ajuda (F1) lista os atalhos.
