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
| `idetea --menu [arquivo]` (o `.desktop`) | o repositório do arquivo, senão o último projeto | o arquivo |

`idetea --install-desktop` / `--uninstall-desktop` põe/tira a IDE do menu de aplicativos (`~/.local/share`), apontando
para o executável que rodou o comando (ou para o AppImage).

Um arquivo que não existe abre numa aba nova e é criado ao salvar.

### Receitas do `just`

| Receita | O quê |
|---|---|
| `just run [args]` | instala o ícone (se preciso) e roda em debug |
| `just run-release [args]` | o mesmo, em release |
| `just desktop` / `just undesktop` | instala/remove o `idetea.desktop` e o ícone em `~/.local/share` |
| `just check` | clippy + testes |
| `just dist [--no-appimage]` | `.tar.gz` e AppImage em `dist/` (perfil `dist`: release sem símbolos) |

**Por que o `just desktop`:** no Wayland, o painel do desktop (COSMIC, GNOME, KDE) acha o ícone de uma janela pelo
app id dela (`idetea`), procurando um `idetea.desktop` com o mesmo nome. Depois de instalado uma vez, até o
`cargo run` direto mostra o ícone. Se já houver um `idetea.desktop` (de uma versão instalada), ele é mantido.

## Lançar uma versão

1. Suba a versão em `[workspace.package]` no `Cargo.toml` (e rode `cargo build` para o `Cargo.lock` acompanhar).
2. Commit, `git tag v<versão>` e `git push origin main v<versão>`.
3. O workflow "Versão" (`.github/workflows/release.yml`) testa, roda `packaging/dist.sh` no Ubuntu 22.04 (glibc mais
   antiga, para rodar em mais distros) e publica a versão com o `.tar.gz` e o AppImage. Tag diferente da versão do
   `Cargo.toml` falha.

O CI (`ci.yml`) roda clippy (avisos viram erro) e os testes a cada push na main e em cada pull request.

## Testes e checagens

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
```

Os testes do git criam repositórios temporários (inclusive um "remoto" local) e pulam sozinhos se não houver `git`.

Arquivos grandes, sem abrir janela:

```sh
# Operações do widget (abrir, digitar, buscar, desfazer) com N linhas.
cargo run --release -p tea-editor --example large_file -- 50000 [--write grande.rs]
# Quadros de verdade (layout, desenho e texto na GPU, sem janela): Rust de 50 MB, JSON de 10 MB numa linha só
# e JS minificado. Rolar até o meio e o fim, rolar devagar e digitar; quadro acima de 1 s é marcado.
cargo run --release -p tea-editor --example large_render -- [rust|json|js|todos] [bytes]
QUADROS=/tmp/quadros cargo run --release -p tea-editor --example large_render -- json   # grava o último quadro em PNG
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
├── tea-declarative-ui/  formato idetea-ui: valida, guarda o que o usuário mudou, desenha, monta a resposta
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
    ├── tea-ext-git/        git (chama o binário git)
    ├── tea-ext-debug/      painel Debug (o núcleo fala com o gdb; o painel recebe CoreEvent::Debug)
    ├── tea-ext-references/ painel Referências (onde um nome é usado, vindo do LSP)
    ├── tea-ext-outline/    painel Estrutura (símbolos do arquivo; a visibilidade vem de tea-app/src/outline.rs)
    ├── tea-ext-calls/      painel Chamadas (quem chama / o que chama, vindo do LSP)
    ├── tea-ext-preview/    painel Prévia (Markdown e SVG do arquivo ativo; recebe CoreEvent::ActiveText)
    └── tea-ext-ui/         painel Interfaces (lê .idetea/ui/, grava as respostas; desligado por padrão)
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

## Textos da interface (idiomas)

A interface fala português, inglês e espanhol ([Fluent](https://projectfluent.org), via `i18n-embed`).
O idioma vem de `[ui] language` (`"auto"` segue o sistema) e é escolhido uma vez, ao abrir.

Cada crate com textos tem:

- `i18n.toml` com `fallback_language = "pt-BR"` e `assets_dir = "i18n"`;
- `i18n/pt-BR/<nome-do-crate>.ftl`, `i18n/en/...` e `i18n/es/...`;
- no topo do `lib.rs`/`main.rs`, antes dos `mod`:

```rust
tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}
```

Para um texto novo: escreva a mensagem nos três `.ftl` e use `fl!("id")` ou `fl!("id", name = valor)`.
O português é a origem: chave que não existe nele não compila. O teste `translations_are_complete`
(em `tea-app`) confere que inglês e espanhol têm as mesmas mensagens e os mesmos valores. Plural:

```fluent
references = { $count ->
    [one] 1 referência
   *[other] { $count } referências
}
```

Cuidados:

- Nada de `fl!` antes de `tea_core::i18n::init` (no `boot`): o primeiro uso fixa o idioma do crate.
  Texto criado antes disso (ex.: avisos da linha de comando) guarda os dados e vira texto depois.
- Não compare textos traduzidos no código (`msg.starts_with("Commit")`): use um tipo.
- Títulos registrados por id (comandos do núcleo, da árvore) usam `LOADER.get(id)`, que a compilação não
  confere; há testes para isso.
- Guia da ajuda: o comando entra como valor com hífens, `{ $palette-open }`, e vira o atalho atual do
  usuário (`get_with_markers`).
- Linha de continuação num `.ftl` não pode começar com `[`, `*` ou `.` (o Fluent lê como variante).

## Convenções

- Textos da interface em português nos `.ftl` (com inglês e espanhol ao lado); mensagens de log, comentários e
  documentação em português.
- Trabalho pesado ou que espera (git, busca, rede) roda fora da thread da interface.
- Cada item do roadmap só é marcado como feito no [SPEC.md](SPEC.md) depois do teste manual na tela.
