# Desenvolvendo a IDE TEA

Guia para quem vai compilar, testar ou mexer no código. A arquitetura completa, as decisões e o roadmap
detalhado estão no [SPEC.md](SPEC.md).

## Compilar e rodar

Precisa do Rust estável (edition 2024). Por enquanto só o Linux é testado. A janela de arquivos (abrir, salvar como)
usa o portal do desktop (xdg-desktop-portal) e, sem ele, o `zenity`.

```sh
cargo run --release -- [--no-splash] [pasta|arquivo...]
cargo build --release          # binário em target/release/idetea
```

Linha de comando:

| Comando | Raiz do projeto | Abas abertas |
|---|---|---|
| `idetea` | pasta atual | nenhuma (aba vazia) |
| `idetea src/main.rs` | pasta atual | `src/main.rs` |
| `idetea ~/outro/projeto` | `~/outro/projeto` | nenhuma |
| `idetea a.rs b.rs` | pasta atual | `a.rs`, `b.rs` (ativa) |

Um arquivo que não existe abre numa aba nova e é criado ao salvar.

### Receitas do `just`

| Receita | O quê |
|---|---|
| `just run [args]` | instala o ícone (se preciso) e roda em debug |
| `just run-release [args]` | o mesmo, em release |
| `just desktop` / `just undesktop` | instala/remove o `idetea.desktop` e o ícone em `~/.local/share` |
| `just check` | clippy + testes |

**Por que o `just desktop`:** no Wayland, o painel do desktop (COSMIC, GNOME, KDE) acha o ícone de uma janela pelo
app id dela (`idetea`), procurando um `idetea.desktop` com o mesmo nome. Depois de instalado uma vez, até o
`cargo run` direto mostra o ícone. O `.desktop` fica fora do menu de aplicativos (`NoDisplay`): aberto pelo menu,
a raiz do projeto seria a pasta home.

## Testes e checagens

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
```

Os testes do git criam repositórios temporários (inclusive um "remoto" local) e pulam sozinhos se não houver `git`.

Benchmark do editor com arquivo grande, sem abrir janela:

```sh
cargo run --release -p tea-editor --example large_file -- 50000 [--write grande.rs]
```

## Variáveis de ambiente

| Variável | Efeito |
|---|---|
| `IDETEA_NO_SPLASH=1` | não mostra a splash (o mesmo que `--no-splash` ou `[ui] splash = false`) |
| `IDETEA_LSP_LOG=1` | mostra no terminal o log dos servidores LSP |
| `IDETEA_DAP_LOG=1` | mostra no terminal os eventos do debugger (DAP) |

## Onde ficam os arquivos

| Arquivo | Conteúdo |
|---|---|
| `~/.config/idetea/config.toml` | config global (`$XDG_CONFIG_HOME`) |
| `<projeto>/.idetea/config.toml` | config do projeto (por cima da global) |
| `~/.local/state/idetea/state.toml` | estado da interface: tamanho dos painéis (`$XDG_STATE_HOME`) |

Os padrões ficam em `tea_core::config::DEFAULT_CONFIG`; o comando "Config: abrir config global" cria o arquivo
com eles comentados.

## Estrutura

```
crates/
├── tea-core/      API das extensões (trait Extension, Context, eventos), config, atalhos, ícones, menu
├── tea-editor/    fachada sobre o iced-code-editor (nenhuma outra crate usa o widget direto)
├── tea-lsp/       cliente LSP (processo + JSON-RPC), sem UI
├── tea-dap/       cliente DAP (debug), sem UI
├── tea-app/       o binário: janela, layout, paleta, diálogos, ajuda, configurações, LSP
└── ext/
    ├── tea-ext-filetree/   árvore de arquivos
    ├── tea-ext-statusbar/  barra de status
    ├── tea-ext-search/     busca no projeto
    ├── tea-ext-problems/   erros e avisos
    ├── tea-ext-terminal/   terminal (alacritty via iced_term)
    └── tea-ext-git/        git (chama o binário git)
```

Regra de ouro: **as extensões internas usam só a API pública de `tea-core`**, a mesma que um plugin usaria.
Se uma extensão precisa de algo que a API não tem, a API está incompleta: amplie a API, não abra um atalho.

## Escrevendo uma extensão

1. Crie a crate em `crates/ext/tea-ext-<nome>/`, dependendo de `iced` e `tea-core`.
2. Implemente a trait `tea_core::Extension`:

   ```rust
   impl Extension for MinhaExtensao {
       type Message = Message;
       fn id(&self) -> &'static str { "minha" }               // prefixo dos comandos: "minha.algo"
       fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
           ctx.register_panel(PANEL, "Minha", Slot::Left);    // painel e slot preferido
           ctx.panel_icon(PANEL, Icon::Files);                // ícone na barra de ícones
           ctx.register_command("minha.algo", "Minha: fazer algo");
           Task::none()
       }
       fn on_event(&mut self, ev: &CoreEvent, ctx: &mut Context) -> Task<Message> { /* arquivo salvo, tema... */ }
       fn on_command(&mut self, cmd: &str, ctx: &mut Context) -> Task<Message> { /* "minha.algo" */ }
       fn update(&mut self, msg: Message, ctx: &mut Context) -> Task<Message> { /* mensagens da própria UI */ }
       fn view(&self, panel: PanelId) -> Element<'_, Message> { /* widgets iced */ }
   }
   ```

3. Ligue no app: uma feature `ext-<nome>` em `crates/tea-app/Cargo.toml` (e no `default`, se for padrão)
   e uma linha em `builtin_extensions()` em `crates/tea-app/src/main.rs`.

O que o `Context` oferece (pedidos que o app executa depois que a chamada retorna): abrir arquivo (e numa linha),
registrar painéis e comandos, rodar comandos, notificar, mostrar/esconder painel, diálogo de confirmação
(`show_dialog`), menu de contexto (`show_menu` + `context_area`), avisar que arquivos mudaram no disco e publicar
status de arquivos. Os eventos (`CoreEvent`) contam o que aconteceu: arquivo aberto/salvo/ativo, diagnósticos,
tema e fonte, status do git, painel aberto pela barra de ícones...

Extensões não se chamam diretamente: conversam só por **comandos** e **eventos**. Toda extensão pode ser desligada
em `[extensions] <id> = { enabled = false }` (vale ao reiniciar) ou mudar de lado com `slot = "right"`.

## Convenções

- Textos da interface, mensagens e documentação em português.
- Trabalho pesado ou que espera (git, busca, rede) roda fora da thread da interface.
- Cada item do roadmap só é marcado como feito no [SPEC.md](SPEC.md) depois do teste manual na tela.
