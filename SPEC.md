# IDE TEA — Especificação

IDE rápida, customizável, em Rust puro (o máximo possível) e sem exageros.
O núcleo só sabe editar arquivos. Todo o resto (árvore de arquivos, terminal, LSP...) é extensão.

## Objetivos

- **Rápida:** inicia em menos de 200 ms e não trava com arquivos grandes.
- **Núcleo pequeno:** o núcleo não conhece nenhuma extensão específica.
- **Desacoplada:** tudo pode ser compilado junto, mas cada parte vive em uma crate separada e pode ser desligada.
- **Extensível depois:** plugins de terceiros sem recompilar a IDE (fase 3).

## Não-objetivos (por enquanto)

- Paridade com VS Code ou JetBrains.
- Marketplace de extensões.
- Edição colaborativa ou remota.
- Debugger (DAP), que pode ser reavaliado depois da fase 3.

## Decisões de arquitetura

### Modelo de extensões: híbrido em camadas

O ponto difícil não é carregar código de plugin, e sim plugin desenhar UI. O iced usa tipos
`Message` e widgets genéricos, que não atravessam uma fronteira dinâmica (`.so` ou WASM).
Por isso as extensões ficam em três camadas:

| Camada | Mecanismo | Pode desenhar UI? | Exemplos |
|---|---|---|---|
| **Extensões internas** | Crates do workspace que implementam a trait `Extension`, ligadas por feature do cargo e pela config | Sim, com widgets iced completos | Árvore de arquivos, terminal, abas, barra de status |
| **Linguagens** | LSP (processo separado, JSON-RPC) | Não (só dados) | rust-analyzer, pyright, gopls |
| **Plugins de terceiros** (fase 3) | WASM via `wasmtime` com interfaces WIT versionadas (mesmo modelo do Zed) | Só por protocolo declarativo: árvore, lista, texto, botão, input, que o host renderiza | Comandos, formatadores, painéis simples |

**Descartado:** `.so`/`.dll` dinâmicas em Rust. A ABI é instável, exige `unsafe`, um crash derruba a IDE e o suporte multiplataforma é ruim.
**Possível no futuro:** scripting leve (Lua via `mlua`) só para config e macros, se houver demanda.

Regra de ouro: **as extensões internas usam exatamente a mesma API pública que um plugin usaria.**
Se a árvore de arquivos precisar de algo que a API não oferece, a API está incompleta.

### Editor

- Widget: `iced-code-editor` 0.5 (canvas, highlight via `syntect`, undo/redo, clipboard,
  busca e substituição, folding, multi-cursor, modo vim, cliente LSP).
- Fica **isolado atrás da fachada `tea_editor::Editor`**, com mensagem opaca (`EditorMessage`).
  Nenhuma outra crate importa `iced_code_editor`. Trocar o widget mexe só em `tea-editor`.
  (Na Fase 0 a fachada concreta se mostrou suficiente. Uma trait `EditorView` só vale a pena se houver dois widgets.)
- O buffer de texto fica dentro do widget por enquanto. Um buffer próprio (`ropey`) só entra se
  LSP ou extensões precisarem ler texto sem passar pelo widget.

### Layout

- A janela é dividida em **slots**: `left`, `right`, `bottom` e `center` (editores/abas).
- Cada extensão registra painéis e diz o slot preferido. O usuário pode mudar pela config.

## Estrutura de crates

```
idetea/
├── crates/
│   ├── tea-core/        # buffers, comandos, keybindings, config, eventos, trait Extension
│   ├── tea-editor/      # fachada sobre o iced-code-editor
│   ├── tea-app/         # binário: janela iced, layout de slots, carrega extensões
│   └── ext/
│       ├── tea-ext-filetree/
│       ├── tea-ext-statusbar/
│       ├── tea-ext-terminal/   # fase 2
│       └── tea-ext-lsp/        # fase 2
└── Cargo.toml           # workspace + features (filetree, terminal, lsp...)
```

## API de extensão (implementada em `tea-core`)

```rust
pub trait Extension: 'static {
    type Message: Clone + Debug + Send + Sync + 'static;   // cada extensão tem seu tipo
    fn id(&self) -> &'static str;                            // também é o prefixo dos comandos
    fn activate(&mut self, ctx: &mut Context) -> Task<Self::Message>;
    fn on_event(&mut self, ev: &CoreEvent, ctx: &mut Context) -> Task<Self::Message> { .. }
    fn on_command(&mut self, cmd: &str, ctx: &mut Context) -> Task<Self::Message> { .. }
    fn update(&mut self, msg: Self::Message, ctx: &mut Context) -> Task<Self::Message>;
    fn view(&self, panel: PanelId) -> Element<'_, Self::Message>;
}
```

- **Tipada para quem escreve a extensão, apagada para o app:** uma impl automática converte
  qualquer `Extension` em `AnyExtension` (object safe), e a mensagem vira `ExtMessage`
  (`Arc<dyn Any>`). Só a extensão dona faz o downcast.
- `Context`: lê a raiz do projeto e o arquivo ativo. Faz **pedidos** (`Request`): abrir arquivo,
  registrar painel ou comando, rodar comando, notificar. O app executa os pedidos depois que a chamada retorna,
  então a extensão nunca segura referência ao estado do app.
- `CoreEvent`: `FileOpened`, `FileSaved`, `ActiveFileChanged` (mais virão: config recarregada, arquivo alterado no disco...).
- Comandos são strings `"<extensão>.<ação>"`. O app roteia pelo prefixo.
- A comunicação entre extensões é só por **comandos** e **eventos**, nunca por chamada direta.

## Configuração

- `~/.config/idetea/config.toml` (global) + `.idetea/config.toml` (projeto).
- Recarrega ao salvar.
- Keybindings, tema, extensões ativas e slot de cada painel.

```toml
[extensions]
filetree = { enabled = true, slot = "left" }
statusbar = { enabled = true }

[keys]
"ctrl+p" = "palette.open"
"ctrl+b" = "filetree.toggle"
```

## Roadmap

### Fase 0 — Spike (validar riscos)
- [x] Janela iced com `iced-code-editor` abrindo um arquivo grande (≥ 50k linhas) e medindo a performance.
- [x] Protótipo da trait `Extension` com um painel lateral trivial (`tea-ext-demo`).
- [x] Teste manual de scroll e digitação fluidos com o arquivo grande: aprovado (fluido e rápido).

**Resultados** (release, `cargo run --release -p tea-editor --example large_file -- N`):

| Operação | 50k linhas (1,7 MB) | 200k linhas (6,9 MB) |
|---|---|---|
| Custo fixo do 1º editor (carga do syntect) | ~9 ms | ~9 ms |
| Abrir (`CodeEditor::new`) | ~2,3 ms | ~9 ms |
| Digitar 1000 caracteres no meio | ~1,6 ms | ~2,6 ms |
| Buscar texto | ~1,1 ms | ~2,8 ms |
| Desfazer tudo | ~3–4 ms | ~11 ms |
| Boot do app já com o arquivo aberto | 12 ms | — |

O highlight é preguiçoso (só linhas visíveis), por isso abrir é barato.
O binário release sem símbolos tem ~22 MB (iced/wgpu + dados do syntect + fontes de ícones do widget).

**Pegadinhas encontradas:**
- O editor só aceita teclado com `request_focus()` **e** `CanvasFocusGained`. Sem isso, abrir um arquivo
  pela lista lateral deixava o editor surdo até levar um clique. Já resolvido em `Editor::focus`.
- `iced::keyboard::listen()` só recebe teclas que nenhum widget consumiu. Atalhos globais usam
  `iced::event::listen_with`.
- O widget traz command palette e diálogos próprios. A palette dele fica desligada, porque a da IDE será global.
- O modo vim emite `WriteRequested` (`:w`). O app precisa tratar isso como `file.save` (Fase 1).

### Fase 1 — MVP
- [ ] Abrir e salvar arquivos, várias abas, indicador de "modificado".
- [ ] Syntax highlight e temas claro e escuro.
- [ ] Buscar e substituir no arquivo (já vem no widget, só precisa ligar a atalhos e comandos).
- [ ] Command palette (todos os comandos registrados, incluindo os das extensões).
- [ ] Config TOML e keybindings configuráveis.
- [ ] Extensão `filetree` (abrir pasta, navegar, criar, renomear e apagar).
- [ ] Extensão `statusbar` (linha/coluna, linguagem, encoding).

### Fase 2 — IDE
- [ ] Extensão `lsp`: diagnósticos, autocomplete, hover, ir para definição.
  Avaliar o cliente LSP que já vem no widget (`LspClient`, feature `lsp-process`) antes de escrever um próprio.
- [ ] Extensão `terminal` (`alacritty_terminal` + renderização iced).
- [ ] Busca no projeto (`grep`/`ignore` crates).

### Fase 3 — Plugins externos
- [ ] Host WASM (`wasmtime`) + WIT versionado.
- [ ] Protocolo de UI declarativa para painéis de plugins.
- [ ] Integração com git (como plugin ou extensão interna).

## Riscos

- **`iced-code-editor` é jovem** (v0.5, um único mantenedor, evolui rápido: 0.3 → 0.5 em ~6 meses). É mitigado pela fachada
  `tea-editor`. A performance foi validada na Fase 0.
- **`syntect` vs tree-sitter:** o `syntect` é bom para highlight, mas não serve para recursos estruturais. Reavaliar na fase 2.
- **API de extensão instável no início:** aceitável enquanto só houver extensões internas. Congelar e versionar antes da fase 3.
- **WASM aumenta o binário** (~10–20 MB) e o tempo de build. Fica atrás de uma feature `wasm-plugins`.
