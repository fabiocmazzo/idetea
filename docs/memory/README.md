# Memória do projeto

- [Sem cargo fmt](sem-cargo-fmt.md) — o código não segue o rustfmt padrão; `cargo fmt --all` reformata o projeto inteiro
- [Fork do iced-code-editor](fork-iced-code-editor.md) — onde fica, como o `[patch]` aponta e a pegadinha da versão
- [Mudanças do widget para o LSP](widget-mudancas-lsp.md) — linhas inteiras reescritas, não o trecho exato; para saber o que mudou, compare os textos
- [Revisão do texto](revisao-do-texto.md) — `text_revision` só anda com um `LspClient` no widget; sem servidor, use o `lsp::Counter`
