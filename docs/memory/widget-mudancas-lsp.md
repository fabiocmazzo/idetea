---
name: widget-mudancas-lsp
description: as mudanças que o iced-code-editor manda ao servidor são linhas inteiras, e a troca do documento todo deixa uma linha vazia a mais
type: gotcha
updated: 2026-11-16
---
O `iced-code-editor` não manda ao servidor o trecho exato de cada edição (`LspClient::did_change`, que a IDE recebe
em `crates/tea-editor/src/lsp.rs`):

- na digitação, o trecho são linhas inteiras reescritas (um Enter na linha 0 chega como a troca de `(0,0)–(2,0)`);
- quando ele troca o documento todo (ex.: edições aplicadas com `apply_lsp_text_edits`), o fim do intervalo para
  antes do último `\n` (`(3,1)` num texto de 4 linhas terminado em `\n`), e o texto novo termina em `\n`. Quem aplica
  a troca numa cópia (o `DocText` do cliente LSP) fica com uma linha vazia a mais no fim.

Para saber o que mudou de fato, compare os textos (`tea_lsp::diff`), como faz `Editor::follow_edits` para os
diagnósticos andarem junto com o texto. Não use os intervalos do `did_change`.

**Why:** a primeira versão que movia os sublinhados usava esses intervalos e deixava o erro parado (o trecho
reescrito o continha), e a cópia do texto saiu da sincronia na troca do documento todo.
