# Memória do projeto

- [Sem cargo fmt](sem-cargo-fmt.md) — o código não segue o rustfmt padrão; `cargo fmt --all` reformata o projeto inteiro
- [Fork do iced-code-editor](fork-iced-code-editor.md) — onde fica, como o `[patch]` aponta e a pegadinha da versão
- [Mudanças do widget para o LSP](widget-mudancas-lsp.md) — linhas inteiras reescritas, não o trecho exato; para saber o que mudou, compare os textos
- [Revisão do texto](revisao-do-texto.md) — `text_revision` só anda com um `LspClient` no widget; sem servidor, use o `lsp::Counter`
- [Teste com duas IDEs](teste-com-duas-ides.md) — pedidos de terminal podem cair na IDE antiga durante o teste manual; confira a janela
- [Texto CJK e o recorte da fonte](texto-cjk-e-o-recorte-da-fonte.md) — ideograma novo em ja/zh-CN precisa estar no recorte embutido; rode `cargo test -p tea-core`
