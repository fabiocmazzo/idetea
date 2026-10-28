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

- Widget: `iced-code-editor` (canvas, highlight via `syntect`, undo/redo, clipboard).
- Fica **isolado atrás de uma trait própria** (`EditorView`) para poder ser trocado depois
  (por exemplo por um editor próprio com tree-sitter) sem quebrar as extensões.
- Buffer de texto do núcleo: `ropey` (independente do widget).

### Layout

- A janela é dividida em **slots**: `left`, `right`, `bottom` e `center` (editores/abas).
- Cada extensão registra painéis e diz o slot preferido. O usuário pode mudar pela config.

## Estrutura de crates

```
idetea/
├── crates/
│   ├── tea-core/        # buffers, comandos, keybindings, config, eventos, trait Extension
│   ├── tea-editor/      # wrapper do iced-code-editor (implementa EditorView)
│   ├── tea-app/         # binário: janela iced, layout de slots, carrega extensões
│   └── ext/
│       ├── tea-ext-filetree/
│       ├── tea-ext-statusbar/
│       ├── tea-ext-terminal/   # fase 2
│       └── tea-ext-lsp/        # fase 2
└── Cargo.toml           # workspace + features (filetree, terminal, lsp...)
```

## API de extensão (rascunho)

```rust
pub trait Extension: Send {
    fn id(&self) -> &'static str;
    fn activate(&mut self, ctx: &mut ExtensionContext);   // registra comandos, painéis, keybindings
    fn on_event(&mut self, ev: &CoreEvent, ctx: &mut ExtensionContext) {}
    fn update(&mut self, msg: ExtMessage, ctx: &mut ExtensionContext) -> Task<ExtMessage>;
    fn view(&self, panel: PanelId) -> Element<'_, ExtMessage>;
}
```

- `ExtensionContext`: abrir e salvar arquivos, ler o workspace, registrar comandos, mostrar notificações, ler config.
- `CoreEvent`: arquivo aberto, salvo ou alterado, aba ativa mudou, config recarregada...
- `ExtMessage`: mensagem opaca roteada pelo app de volta à extensão dona (`Box<dyn Any>` + id).
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
- [ ] Janela iced com `iced-code-editor` abrindo um arquivo grande (≥ 50k linhas) e medindo a performance.
- [ ] Protótipo da trait `Extension` com um painel lateral trivial.

### Fase 1 — MVP
- [ ] Abrir e salvar arquivos, várias abas, indicador de "modificado".
- [ ] Syntax highlight e temas claro e escuro.
- [ ] Buscar e substituir no arquivo.
- [ ] Command palette (todos os comandos registrados, incluindo os das extensões).
- [ ] Config TOML e keybindings configuráveis.
- [ ] Extensão `filetree` (abrir pasta, navegar, criar, renomear e apagar).
- [ ] Extensão `statusbar` (linha/coluna, linguagem, encoding).

### Fase 2 — IDE
- [ ] Extensão `lsp`: diagnósticos, autocomplete, hover, ir para definição.
- [ ] Extensão `terminal` (`alacritty_terminal` + renderização iced).
- [ ] Busca no projeto (`grep`/`ignore` crates).

### Fase 3 — Plugins externos
- [ ] Host WASM (`wasmtime`) + WIT versionado.
- [ ] Protocolo de UI declarativa para painéis de plugins.
- [ ] Integração com git (como plugin ou extensão interna).

## Riscos

- **`iced-code-editor` é jovem** (v0.3.x, um único mantenedor). É mitigado pela trait `EditorView` e pelo spike da fase 0.
- **`syntect` vs tree-sitter:** o `syntect` é bom para highlight, mas não serve para recursos estruturais. Reavaliar na fase 2.
- **API de extensão instável no início:** aceitável enquanto só houver extensões internas. Congelar e versionar antes da fase 3.
- **WASM aumenta o binário** (~10–20 MB) e o tempo de build. Fica atrás de uma feature `wasm-plugins`.
