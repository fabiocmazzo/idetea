---
name: widget-mudancas-lsp
description: as mudanças que o iced-code-editor manda ao servidor são linhas inteiras reescritas ou o documento todo, não o trecho exato
type: gotcha
updated: 2026-11-17
---
O `iced-code-editor` não manda ao servidor o trecho exato de cada edição (`LspClient::did_change`, que a IDE recebe
em `crates/tea-editor/src/lsp.rs`):

- na digitação, o trecho são linhas inteiras reescritas (um Enter na linha 0 chega como a troca de `(0,0)–(2,0)`);
- quando ele troca o documento todo (ex.: edições aplicadas com `apply_lsp_text_edits` depois de um Enter), o
  intervalo vai do começo ao fim do documento. Até o commit `50c3135` do fork, esse fim parava antes do último `\n`,
  e o servidor ganhava uma linha vazia a mais a cada troca.

Para saber o que mudou de fato, compare os textos (`tea_lsp::diff`), como faz `Editor::follow_edits` para os
diagnósticos andarem junto com o texto. Não use os intervalos do `did_change`.

**Why:** a primeira versão que movia os sublinhados usava esses intervalos e deixava o erro parado (o trecho
reescrito o continha).
