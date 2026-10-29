# IDE TEA — Especificação

> **IDE TEA. Just code. Add ice.**
> (Trocadilho com *Iced Tea*: a IDE é feita com o [iced](https://github.com/iced-rs/iced).)

IDE rápida, customizável, em Rust puro (o máximo possível) e sem exageros.
O núcleo só sabe editar arquivos. Todo o resto (árvore de arquivos, terminal, LSP...) é extensão.

## Objetivos

- **Rápida:** inicia em menos de 200 ms e não trava com arquivos grandes.
- **Núcleo pequeno:** o núcleo não conhece nenhuma extensão específica.
- **Desacoplada:** tudo pode ser compilado junto, mas cada parte vive em uma crate separada e pode ser desligada.
- **Extensível depois:** plugins de terceiros sem recompilar a IDE (fase 4).

## Não-objetivos (por enquanto)

- Paridade com VS Code ou JetBrains.
- Marketplace de extensões.
- Edição colaborativa ou remota.

## Decisões de arquitetura

### Modelo de extensões: híbrido em camadas

O ponto difícil não é carregar código de plugin, e sim plugin desenhar UI. O iced usa tipos
`Message` e widgets genéricos, que não atravessam uma fronteira dinâmica (`.so` ou WASM).
Por isso as extensões ficam em três camadas:

| Camada | Mecanismo | Pode desenhar UI? | Exemplos |
|---|---|---|---|
| **Extensões internas** | Crates do workspace que implementam a trait `Extension`, ligadas por feature do cargo e pela config | Sim, com widgets iced completos | Árvore de arquivos, terminal, abas, barra de status |
| **Linguagens** | LSP (processo separado, JSON-RPC) | Não (só dados) | rust-analyzer, pyright, gopls |
| **Plugins de terceiros** (fase 4) | WASM via `wasmtime` com interfaces WIT versionadas (mesmo modelo do Zed) | Só por protocolo declarativo: árvore, lista, texto, botão, input, que o host renderiza | Comandos, formatadores, painéis simples |

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
│   ├── tea-lsp/         # cliente LSP (processo, JSON-RPC, posições), sem UI
│   ├── tea-app/         # binário: janela iced, layout de slots, carrega extensões, gerencia LSP
│   └── ext/
│       ├── tea-ext-filetree/   # árvore de arquivos
│       ├── tea-ext-statusbar/  # barra de status
│       ├── tea-ext-search/     # busca no projeto
│       ├── tea-ext-problems/   # erros e avisos
│       ├── tea-ext-terminal/   # terminal (alacritty via iced_term)
│       └── tea-ext-git/        # git (chama o binário git)
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

## Inicialização

### Linha de comando

`idetea [caminho...]`

- A **pasta do projeto** (raiz da árvore de arquivos) é sempre a **pasta atual** de onde a IDE foi chamada.
  A exceção é quando o argumento é uma pasta: aí ela vira a raiz.
- **Todo arquivo passado na linha de comando é sempre aberto** numa aba, mesmo que esteja fora da pasta atual.
  Com vários arquivos, abre todos e o último fica ativo.
- A árvore de arquivos mostra a raiz **e** revela (expande e destaca) o arquivo ativo, quando ele está dentro dela.
- Um caminho que não existe abre uma aba nova com esse nome, e o arquivo é criado ao salvar.

| Comando | Raiz da árvore | Abas abertas |
|---|---|---|
| `idetea` | pasta atual | nenhuma (aba vazia) |
| `idetea src/main.rs` | pasta atual | `src/main.rs` |
| `idetea ~/outro/projeto` | `~/outro/projeto` | nenhuma |
| `idetea a.rs b.rs` | pasta atual | `a.rs`, `b.rs` (ativa) |

(Na Fase 0, `idetea arquivo` usava a pasta do arquivo como raiz. Isso muda na Fase 1.)

### Splash screen

- Splash elegante ao abrir: logo/nome "IDE TEA", versão e uma animação discreta (fade).
- Fica na tela **no mínimo 2 segundos** e some com fade. Se o carregamento demorar mais, ela fica até tudo estar pronto
  (config lida, extensões ativadas, arquivos da linha de comando abertos).
- O carregamento acontece **por trás** da splash. Os 2 segundos não somam ao tempo de boot, e a IDE já está pronta quando a splash some.
- Pode ser desligada, para testes e desenvolvimento:
  - flag `--no-splash`;
  - variável de ambiente `IDETEA_NO_SPLASH=1`;
  - config: `[ui] splash = false`.
- O padrão é ligada.

## Configuração

- Camadas: padrões embutidos (`tea_core::config::DEFAULT_CONFIG`) → global → projeto.
  - Global: `$XDG_CONFIG_HOME/idetea/config.toml` (ou `~/.config/idetea/config.toml`).
    O comando "Config: abrir config global" cria o arquivo já com os padrões comentados.
  - Projeto: `<raiz>/.idetea/config.toml`.
- O merge é profundo: `filetree = { slot = "right" }` no projeto mantém `enabled` do global.
- Recarrega ao salvar qualquer um dos dois pela IDE (tema, fonte e atalhos). Ligar ou desligar extensão só vale ao reiniciar.
- Um erro no config não impede a IDE de abrir: a parte válida é usada e os problemas aparecem na barra de status.

```toml
[ui]
splash = true
theme = "dark"        # "dark", "light" ou nome de tema do iced ("Nord", "Dracula"...)
font_size = 14

[extensions]
filetree = { enabled = true, slot = "left" }
statusbar = { enabled = true }
terminal = { enabled = true, slot = "bottom" }

[keys]
"ctrl+shift+p" = "palette.open"
"ctrl+b" = "filetree.toggle"
"ctrl+s" = ""          # texto vazio desliga um atalho padrão
```

### Atalhos: quem tem prioridade

1. **Palette aberta:** `Esc` fecha, `↑`/`↓` navegam e `Enter` executa. O atalho da palette também fecha.
2. **Widget com foco** (editor, campos de texto): se ele usou a tecla, a IDE não faz nada.
   O editor já trata `Ctrl+F/H/G` (buscar, substituir, ir para linha), `Ctrl+Z/Y`, `Ctrl+D`, `Ctrl+K/J` (dobrar)
   e `Ctrl+S` (emite `WriteRequested`, que a IDE trata como `file.save`, assim como o `:w` do modo vim).
3. **Mapa de atalhos da IDE** (`[keys]`) para o resto.

**Painéis que usam o teclado todo (terminal):** com o foco num painel, `Ctrl+letra`, F-teclas, setas etc. são do painel
(ex.: `Ctrl+C`, `Ctrl+B`, `Ctrl+W`, `F8` vão para o shell). Os atalhos da IDE com `Ctrl+Shift`
(menos `Ctrl+Shift+C/V`, que copiam e colam) e `Ctrl+PageUp/PageDown` continuam valendo — por isso a palette (`Ctrl+Shift+P`)
sempre funciona. O app aplica a regra em todos os painéis laterais e inferiores (`Keymap::wins_over_panels`).

`Ctrl+Tab` é usado pelo editor (insere tab), então a troca de abas usa `Ctrl+PageUp`/`Ctrl+PageDown`.

### Comandos do núcleo

`palette.open`, `file.{new,save,save_as,close}`, `tab.{next,prev}`,
`editor.{find,replace,goto_line,undo,redo,select_all,fold_all,unfold_all,toggle_vim}`,
`theme.toggle`, `config.{open,open_project,reload}`, `view.toggle_{left,right,bottom}`, `app.quit`.
Extensões registram os delas com o próprio prefixo (`filetree.*`).

### Segurança contra perda de dados

- Fechar aba com alterações: diálogo **Salvar** (Enter) / Descartar / Cancelar.
- Sair (`Ctrl+Q` ou o X da janela) com abas alteradas: diálogo com a lista delas, **Salvar tudo e sair** (Enter) / Sair sem salvar / Cancelar.
  Se alguma não puder ser salva (erro ou aba sem nome), a IDE não sai.
- Apagar na árvore: diálogo de confirmação e **move para a lixeira** (crate `trash`), sem apagar de vez.
- Arquivo apagado ou renomeado pela árvore: as abas são atualizadas. Aba com alterações nunca é fechada sozinha.
- CRLF é preservado ao salvar. Arquivos que não são UTF-8 não abrem (com aviso), em vez de serem corrompidos.

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
- [x] Abrir e salvar arquivos, várias abas, indicador de "modificado", salvar como, aviso ao fechar com alterações.
- [x] Syntax highlight e temas claro e escuro (`theme.toggle`, `[ui] theme`).
- [x] Buscar e substituir no arquivo (do widget, exposto como comandos).
- [x] Command palette (todos os comandos registrados, incluindo os das extensões, com busca aproximada e atalho ao lado).
- [x] Config TOML em camadas e keybindings configuráveis, recarregados ao salvar.
- [x] Extensão `filetree` (navegar, abrir, criar, renomear, apagar para a lixeira).
- [x] Extensão `statusbar` (notificação, modificado, linha/coluna, linguagem, encoding, LF/CRLF).
- [x] Linha de comando: raiz = pasta atual e arquivos passados sempre abertos (ver "Inicialização").
- [x] Árvore de arquivos revela o arquivo ativo.
- [x] Splash screen com fade, mínimo de 2 s e desligável por `--no-splash`, `IDETEA_NO_SPLASH=1` ou config.
- [x] Teste manual de tudo acima na tela: aprovado.

Fora do MVP (anotado para depois): abrir arquivo rápido por nome (`Ctrl+P`), observar mudanças no disco (`notify`),
arrastar painéis, encodings além de UTF-8.

### Fase 2 — IDE
- [x] Busca no projeto (extensão `search`, painel inferior): busca enquanto digita, respeita `.gitignore`,
      opções Aa / palavra inteira / regex, paralela (85 mil arquivos em ~0,2 s), clique abre na linha.
- [x] LSP com cliente próprio (`tea-lsp`): diagnósticos, hover, autocomplete, ir para definição, formatar.
- [x] Extensão `problems` (painel inferior) + contagem de erros/avisos na barra de status.
- [x] Teste manual do LSP na tela (hover, menu de autocomplete, Ctrl+Clique): aprovado.
- [x] Extensão `terminal` (painel inferior, crate `iced_term` = `alacritty_terminal` + widget iced):
      `Ctrl+Shift+T` abre e foca, "Terminal: reiniciar o shell". Usa `$SHELL` na pasta do projeto,
      cores e fonte seguem o tema. O shell só sobe na primeira vez que o painel abre (não pesa no boot).
      Digitar `exit` fecha o painel.
- [x] Teste manual do terminal na tela: aprovado.

#### Decisões do terminal

- **Extensões ganharam `subscription()`** (fontes contínuas de eventos, como a saída do shell) e o evento
  `CoreEvent::Appearance { theme, font_size }` (enviado na ativação e quando tema/fonte mudam).
- **Foco:** mensagens de painel não tiram mais o foco do editor sozinhas (o terminal manda "mudei de tamanho" quando o layout muda);
  o foco sai por clique no painel ou por comando. Esconder o painel inferior (`Ctrl+Shift+J`) devolve o foco ao editor.
- `theme.toggle` perdeu o atalho padrão (`Ctrl+Shift+T` agora é do terminal); continua na palette.

#### Decisões do LSP

- **Cliente próprio em vez do embutido no widget**: o do `iced-code-editor` ignora `publishDiagnostics`.
  O nosso implementa a trait `LspClient` do widget, então hover, autocomplete e definição continuam usando a UI pronta dele.
- **Camadas**: `tea-lsp` (processo + JSON-RPC, sem UI) → `tea-editor` (ponte com o widget, overlay, menu)
  → `tea-app` (um servidor por linguagem, iniciado no primeiro arquivo) → extensões recebem `CoreEvent::Diagnostics`/`Activity`.
  O LSP não é uma extensão: ele precisa do editor por dentro. Só a parte visual (problemas, status) é extensão.
- **Posições**: pedimos `utf-32` ao servidor (rust-analyzer e clangd aceitam), assim colunas = caracteres.
  Se ele só falar UTF-16, o cliente converte usando uma cópia do texto.
- **Menu de autocomplete**: o widget não sabe que o menu está aberto. Um widget `KeyGate` intercepta ↑/↓/Enter/Tab/Esc só nessa hora.
- **Hover**: pedido depois de 450 ms com o mouse parado. Os quadros de animação (`window::frames`) só ficam ativos enquanto isso.
- **Falhas**: um servidor que não existe ou morre gera **um** aviso com o motivo (ex.: última linha do stderr) e não é reiniciado sozinho.
  "LSP: reiniciar servidores" tenta de novo.
- Servidores embutidos: rust-analyzer, clangd, pyright, typescript-language-server, gopls, lua-language-server.
  Configuráveis em `[lsp.servers.<nome>]` (trocar comando, desligar, adicionar novos).
- `IDETEA_LSP_LOG=1` mostra o log dos servidores no terminal.
- Fora por enquanto: sublinhado de erro no próprio editor (o widget não suporta: precisaria de PR upstream ou fork), formatar ao salvar.

### Fase 3 — Git e polimento

Os plugins WASM ficaram para depois do polimento: a API das extensões ainda vai mudar, e congelá-la antes seria cedo demais.

- [x] Integração com git (extensão interna `git`, usando o `git` instalado): branch/↑↓/nº de mudanças na barra de status;
      painel "Git" (`Ctrl+Shift+G`) com mudanças no stage e fora dele, `+`/`−` por arquivo ou tudo, commit com mensagem,
      diff colorido do arquivo selecionado (clique numa linha abre o arquivo nela). Atualiza ao salvar e quando o `.git` muda
      (commit/checkout no terminal), olhando a data de `index`, `HEAD`, `logs/HEAD` e `FETCH_HEAD` a cada segundo.
- [x] Teste manual do git na tela: aprovado.
      Fora por enquanto: push/pull (use o terminal), descartar mudanças, marcas de diff no editor (o widget não suporta),
      cores de status na árvore de arquivos.

Polimento, nesta ordem (o diálogo vem primeiro porque menu de contexto e configurações usam ele):

- [x] **Diálogo de confirmação** desenhado pela IDE (camada sobre a janela, como a palette; segue o tema).
      Usado em: apagar arquivo, fechar aba com alterações, sair com alterações.
      Extensões usam `ctx.show_dialog(Dialog::new(..).button(rótulo, ButtonKind::Danger, Message::X).cancel(..))`;
      o botão escolhido manda a mensagem de volta para a extensão que abriu.
      Teclado: Enter aciona o botão em destaque (o primeiro `Primary`/`Danger`), ←/→/Tab trocam, Esc ou clique fora cancelam.
      Com o diálogo aberto nenhum outro atalho funciona e nada embaixo fica com o foco.
- [x] Teste manual do diálogo na tela: aprovado.
- [x] **Barra de ícones** (borda esquerda, `[ui] activity_bar`, liga/desliga nas configurações): um ícone por painel
      (a extensão escolhe com `ctx.panel_icon`), laterais em cima e inferiores embaixo; clique mostra/oculta, o aberto fica
      destacado, dica com o atalho. Ao abrir pela barra o dono recebe `CoreEvent::PanelShown` (terminal sobe o shell e foca,
      busca foca o campo, git atualiza).
- [x] **Redimensionar painéis com o mouse:** divisor arrastável entre lateral/inferior e o editor (cursor de redimensionar,
      destaque ao passar o mouse, duplo clique volta ao padrão). Sempre sobram 240 px para o editor. Tamanhos salvos em
      `$XDG_STATE_HOME/idetea/state.toml` (estado da IDE, não config).
- [x] Teste manual da barra de ícones e do redimensionar: aprovado.
- [ ] Diálogo nativo (`rfd`) para escolher arquivo ou pasta ("Salvar como", "Abrir pasta").
- [x] **Ícones:** fonte Codicons embutida (80 KB, CC BY 4.0, ver `crates/tea-core/assets/NOTICE.md`) e módulo
      `tea_core::icons` (`icon(Icon::X)`, `icon_button(Icon::X, "dica", msg)` com tooltip após 450 ms).
      Árvore: barra com ícones (novo arquivo/pasta, renomear, apagar, atualizar, recolher), setas e ícones de pasta/arquivo nas linhas.
      Git e barra de status também usam os ícones.
      Escolhida no lugar da Lucide (944 KB inteira): é pequena e já traz ícones de git, terminal e debug.
- [x] Teste manual dos ícones na tela: aprovado.
- [x] **Menu do botão direito** (`tea_core::menu`): a extensão envolve o conteúdo em `context_area(..)`, que transforma
      o clique direito em mensagem com a posição, e responde com `ctx.show_menu(Menu::new(pos).item(..).command(..).danger(..))`.
      Itens mandam uma mensagem de volta para a extensão ou rodam um comando (mostrando o atalho dele). O app desenha o menu
      no ponto do clique, virando para a esquerda/para cima perto da borda; ↑/↓/Enter/Esc e clique fora fecham.
      Áreas aninhadas: vale a mais interna (linha da árvore > espaço vazio da árvore).
      Árvore: novo arquivo, nova pasta, renomear, apagar, copiar caminho (absoluto e relativo), abrir no gerenciador de arquivos;
      no espaço vazio: itens da raiz + atualizar e recolher tudo.
- [x] Teste manual do menu na tela: aprovado.
- [x] Menu de contexto nas abas: fechar, fechar outras, fechar à direita, fechar salvas, fechar todas
  (um diálogo só para as abas com alterações: salvar e fechar / descartar / cancelar), copiar caminho,
  copiar caminho relativo, revelar na árvore, abrir no gerenciador de arquivos. Clique do meio fecha a aba.
- [x] Menu de contexto nos arquivos do git: abrir, ver diff, adicionar/tirar do stage, descartar
  alterações (com confirmação; arquivo novo vai para a lixeira), copiar caminho.
- [x] Arquivo mudado no disco por uma extensão (ex.: descartar no git): aba sem alterações recarrega,
  aba com alterações fica e avisa. Evita salvar por cima e desfazer o descarte.
- [x] Teste manual dos menus das abas e do git na tela: aprovado.
- [x] **Help** (`F1` / `help.open`, janela sobre a IDE, Esc fecha):
      **Atalhos** — todos os comandos agrupados (núcleo e extensões, com id e teclas atuais) + atalhos fixos do editor, com filtro;
      **Guia** — primeiros passos, arquivos, árvore, painel inferior, terminal, git, LSP e config. O texto cita comandos como
      `{file.save}` e a tela mostra o atalho atual do usuário (teste garante que todo comando citado existe);
      **Sobre** — versão, caminhos dos configs e créditos (inclui a atribuição dos Codicons).
      `F1` passou a abrir a ajuda; a paleta continua em `Ctrl+Shift+P`.
- [x] Teste manual da ajuda na tela: aprovado.
- [x] **Configurações** (`Ctrl+,` / `settings.open`, janela sobre a IDE): abas Aparência (tema, fonte, splash),
      Extensões (liga/desliga e lado do painel), Atalhos (filtro, `+` grava a próxima tecla, clicar numa tecla remove)
      e LSP (liga/desliga geral e por servidor, mostra se o comando está no PATH, reiniciar servidores).
      Grava no config **global ou do projeto** (escolha no topo) com `toml_edit`, preservando comentários e formatação;
      cada mudança vale na hora, inclusive mudar um painel de lado. Ligar/desligar extensão vale ao reiniciar.
      Remover atalho: tira a linha do arquivo; se o padrão ou o global ainda ligarem a tecla ao comando, grava `""`.
      Gravar uma tecla já usada troca o dono e avisa ("Ctrl+X era de …").
- [x] Teste manual das configurações na tela: aprovado.
      Com a ajuda ou as configurações abertas, qualquer atalho da IDE fecha a janela e roda o comando (só a gravação de tecla pega tudo).

#### Decisões da Fase 3

- **Git chama o binário `git`** (não `gix`/`libgit2`): sem dependência pesada e com o mesmo comportamento do git do usuário
  (config, hooks, credenciais). Comandos rodam em threads, fora da interface. Sem `git` no PATH ou fora de um repositório, a extensão fica quieta.
  O `status` usa `--no-optional-locks` para não disputar o index com o git do terminal (nem disparar o próprio observador).
- Itens da barra de status não mudam de slot pelo config: o `slot` de `[extensions]` move só os outros painéis da extensão.

### Fase 4 — Plugins externos
- [ ] Host WASM (`wasmtime`) + WIT versionado.
- [ ] Protocolo de UI declarativa para painéis de plugins.
- **Plugins WASM têm confiança total:** um plugin instalado pode ler e escrever no disco e rodar processos, como extensões do VS Code.
  Não há sandbox de permissões (decisão do usuário). Instale só plugins em que você confia.

### Fase 5 — Debug
- [ ] Spike: o `iced-code-editor` consegue mostrar breakpoints na margem e destacar a linha atual sem fork?
- [ ] Cliente DAP próprio (como o do LSP): breakpoints, continuar/próximo/entrar/sair, pilha e variáveis.
      Adaptadores: `codelldb` (Rust/C/C++), `debugpy`, `delve`...

## Riscos

- **`iced-code-editor` é jovem** (v0.5, um único mantenedor, evolui rápido: 0.3 → 0.5 em ~6 meses). É mitigado pela fachada
  `tea-editor`. A performance foi validada na Fase 0.
- **`syntect` vs tree-sitter:** o `syntect` é bom para highlight, mas não serve para recursos estruturais. Reavaliar na fase 2.
- **API de extensão instável no início:** aceitável enquanto só houver extensões internas. Congelar e versionar antes da fase 4.
- **WASM aumenta o binário** (~10–20 MB) e o tempo de build. Fica atrás de uma feature `wasm-plugins`.
