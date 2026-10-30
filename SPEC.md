# IDE TEA — Especificação

> **IDE TEA. Just code. Add ice.**
> (Trocadilho com *Iced Tea*: a IDE é feita com o [iced](https://github.com/iced-rs/iced).)

IDE rápida, customizável, em Rust puro (o máximo possível) e sem exageros.
Livre e gratuita, sempre: licença [Apache 2.0](LICENSE). Visão geral e funcionalidades no [README](README.md);
como compilar, testar e escrever extensões em [DESENVOLVIMENTO.md](DESENVOLVIMENTO.md).
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
│   ├── tea-dap/         # cliente DAP (debug), sem UI
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
      Fora por enquanto: marcas de diff no editor (o widget não suporta).
      Descartar mudanças, cores na árvore e push/pull vieram depois, no polimento.

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
- [x] Diálogo nativo (`rfd` 0.17) para escolher arquivo ou pasta. No Linux usa o portal do desktop
      (xdg-desktop-portal por D-Bus, carregado em tempo de execução, sem GTK) e cai para o `zenity` sem portal.
      "Salvar como" (`Ctrl+Shift+S`) abre a janela do sistema na pasta do arquivo, com o nome atual;
      "Abrir…" (`Ctrl+O`, `file.open`) abre um ou mais arquivos; "Abrir pasta em nova janela…" (`file.open_folder`)
      abre outra janela da IDE com a pasta como raiz (a raiz de uma janela não muda).
      Digitar o caminho continua na paleta: `file.save_as_path`. Só uma janela de arquivos por vez.
- [x] Teste manual das janelas de arquivo e pasta: aprovado.
- [x] **Ícone da janela**: `assets/idetea.svg` (copo de chá gelado). A janela usa o app id `idetea`; no Wayland o
      painel acha o ícone pelo `idetea.desktop` de mesmo nome. `just desktop` instala `.desktop` e ícone em
      `~/.local/share` (o `.desktop` fica fora do menu de aplicativos, `NoDisplay`: aberto pelo menu, a raiz seria a home).
      `just run [args]` instala e roda. Instalar pelo `build.rs` foi descartado: build não deve mexer fora de `target/`.
- [x] Teste manual do ícone no painel do COSMIC: aprovado.
- [x] **Push, pull e fetch** no painel Git (botões no topo, com ↑/↓ na dica) e comandos `git.push`, `git.pull`, `git.fetch`.
      Rodam numa thread, um por vez, sem bloquear stage e commit; a barra de status mostra "Enviando…" etc.
      O git roda numa sessão nova, sem terminal (`setsid`): o ssh nunca fica esperando senha no terminal de onde a
      IDE foi aberta; usa o agente ssh, um `SSH_ASKPASS` gráfico ou um credential helper, ou falha com explicação.
      Tempo limite de 3 min (mata o git e os filhos). Erros comuns viram frases curtas ("o remoto tem commits que
      você não tem: faça pull antes", falha de autenticação, conflito, branches divergentes).
      Branch sem upstream: diálogo "Publicar a branch?" e `push -u` no `origin` (ou no único remoto).
      `pull` respeita a config do usuário (merge/rebase). Depois do pull, abas sem alterações cujo arquivo mudou
      no disco recarregam (ou fecham, se o arquivo sumiu); a árvore e o git atualizam.
      Sem fetch automático: a IDE não fala com a rede sem o usuário pedir.
- [x] Teste manual do push, pull e fetch: aprovado.
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
- [x] **Cores do git na árvore**: modificado/apagado em amarelo, novo/não rastreado em verde, conflito em vermelho,
      ignorado (`.gitignore`) apagado; letra à direita (M, A, U, D, !) como no painel do git.
      Pasta com mudanças dentro pega a cor mais forte de dentro e mostra um ponto; ignorados não sobem.
      O git publica o status (`CoreEvent::FileStatuses`, caminhos absolutos) a cada atualização, só quando muda;
      a árvore não conhece o git. Criar/renomear/apagar pela árvore ou descartar no git avisa
      (`CoreEvent::FilesChanged`) e o git atualiza; "Atualizar" na árvore também.
- [x] Teste manual das cores do git na árvore: aprovado.
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

### Polimento 2

Ordem combinada: abrir pelo nome → perceber mudanças no disco → spike do debug → debug → plugins WASM
(os plugins vão por último porque o debug ainda vai mexer na API das extensões).

- [x] **Abrir arquivo pelo nome** (`Ctrl+P`, `file.quick_open`): modo "arquivos" da paleta. A lista é lida numa thread
      a cada abertura (respeita `.gitignore`, mostra ocultos, nunca entra no `.git`, no máximo 300 mil arquivos) e a
      paleta já abre com a lista anterior. Busca aproximada em que o nome do arquivo vale mais que o caminho; empate vai
      para o caminho mais curto; até 200 resultados (100 mil caminhos filtram em ~11 ms em release).
      Sem busca, as abas abertas vêm primeiro. `nome:42` abre na linha 42; `>` no começo troca para os comandos;
      `Ctrl+P` ⇄ `Ctrl+Shift+P` trocam de modo com a paleta aberta (o atalho do próprio modo fecha).
- [x] Teste manual do abrir pelo nome: aprovado.
- [x] **Perceber mudanças no disco** (`notify` 8, inotify no Linux), no app (`watcher.rs`), não numa extensão:
      um observador **por pasta**, só nas pastas que o `.gitignore` não ignora (nunca `.git`, `target/`, `node_modules/`),
      no máximo 20 mil pastas; pastas novas passam a ser observadas ao aparecer. Rajadas viram um aviso só
      (150 ms de silêncio, no máximo 1 s). Leitura de arquivo e mudança só de permissão/data não contam.
      Ao mudar: abas sem alterações cujo conteúdo no disco ficou diferente recarregam (ou fecham, se o arquivo sumiu);
      abas com alterações ficam e avisam ("salvar sobrescreve"), menos nos 2 s depois de a própria IDE salvar;
      as extensões recebem `CoreEvent::FilesChanged` (árvore relê as pastas abertas, git atualiza o status e as cores).
      Se o sistema recusar observadores (limite `fs.inotify.max_user_watches`) ou a pasta for grande demais, avisa uma vez.
      Mudanças dentro de pastas ignoradas (ex.: `target/` aberto na árvore) só aparecem com "Atualizar".
- [x] Teste manual das mudanças no disco: aprovado.

### Navegação pelo código

Pesquisa: o caminho é o LSP (o que VS Code, Zed e Helix usam); busca por texto só como alternativa sem servidor.
Tree-sitter com regras de escopo (stack graphs do GitHub) e índices (SCIP/LSIF, ctags) não compensam numa IDE com LSP.
Ordem: (1) voltar/avançar + referências → (2) várias definições, implementação e tipo → (3) símbolos na paleta →
(4) quem chama. Prévia dentro do editor fica de fora (o widget não deixa inserir nada entre as linhas).

- [x] **Item 1 — voltar/avançar e onde é usado:**
  - `tea-lsp`: `references(path, posição)` (`includeDeclaration`), resposta ordenada por arquivo e posição, sem
    repetidos. A definição lembra de onde foi pedida: se o resultado cobre essa posição, vem `AtDefinition` em vez de
    `Definition` (estava na declaração), e o app pede as referências, como o Ctrl+clique do VS Code.
    Teste com o clangd de verdade (pula sem ele).
  - Painel "Referências" (extensão `tea-ext-references`, `CoreEvent::References`): agrupado por arquivo, cada uso com
    o número e a linha (sem o recuo), o nome destacado. As linhas vêm da aba aberta (com as alterações) ou do disco.
    Só a declaração: avisa que o nome não é usado em outro lugar.
  - Histórico (`nav.rs`): antes de cada salto (definição, Ctrl+P, cliques nos painéis de referências, problemas e
    busca) guarda onde o cursor estava; paradas do debug não contam. Saltos na mesma linha não contam; arquivos
    apagados são pulados; até 100 lugares. Saltar para outro lugar apaga o "avançar", como num navegador.
  - Atalhos: `Shift+F12` (`lsp.references`), `Alt+←` (`nav.back`), `Alt+→` (`nav.forward`) e os botões laterais
    do mouse. O widget de edição usaria `Alt+←/→` para mover o cursor; o `KeyGate` não deixa essas duas chegarem nele.
  - Primeiro teste falhou: o `rust-analyzer` do PATH era o atalho do rustup sem o componente instalado
    ("Unknown binary 'rust-analyzer'"), o servidor morria ao abrir e `F12`/`Ctrl+clique` ficavam mudos. Agora:
    pedir definição ou referências (inclusive Ctrl+clique) sem servidor avisa o motivo, e esse erro do rustup vem
    com "Instale com: rustup component add rust-analyzer". Com Ctrl apertado, o ponteiro vira mãozinha sobre o
    texto (`KeyGate`; o widget só sublinhava). Teste com o simulador do iced (`iced_test`): Ctrl+clique atravessa
    as camadas (marcas, overlay do LSP) e chega ao widget.
  - [x] Teste manual (aprovado): Ctrl+clique num uso vai para a declaração e `Alt+←` volta; Ctrl+clique na declaração (ou
    `Shift+F12`) abre o painel com os usos; clicar num uso vai até ele; `Alt+→` avança de novo.

- [x] **Item 2 — várias definições, implementação e tipo:**
  - `tea-lsp`: `implementation` e `type_definition`; as três buscas de lista (mais referências) viram
    `Event::Locations { lookup, ... }`, ordenadas e sem repetidos. Definição com mais de um resultado também vira
    lista (com um, continua `Definition`/`AtDefinition`). Servidor sem o recurso: lista vazia ("Nenhuma
    implementação encontrada"). Pedido do usuário que volta como "conteúdo mudou"/"cancelado" (rust-analyzer
    carregando o projeto) avisa "ainda carregando o projeto" em vez de sumir.
  - App: lista vazia avisa; um resultado salta direto (com histórico); vários vão para o painel Referências, que
    diz o que é ("2 implementações de \"Forma\""; `CoreEvent::Locations` com `LocationKind`). O nome no título é a
    palavra de onde se pediu.
  - Conferido com o rust-analyzer: `Ctrl+F12` no trait acha os dois `impl`, no método do trait acha os dois
    métodos, e "ir para o tipo" numa variável vai para a struct.
  - Atalhos: `Ctrl+F12` (`lsp.implementation`); "LSP: ir para o tipo" (`lsp.type_definition`) fica sem atalho,
    na paleta (dá para pôr um em `[keys]`).
  - [x] Teste manual (aprovado): num trait com dois `impl`, `Ctrl+F12` lista os dois no painel; num tipo com um `impl` só,
    salta direto; "ir para o tipo" numa variável vai para a struct; `Alt+←` volta.

- [x] **Item 3 — símbolos na paleta:**
  - `Ctrl+Shift+O` (`nav.file_symbol`): a estrutura do arquivo ativo (a mesma do painel Estrutura, guardada no app;
    se for de outro arquivo, pede e mostra "Lendo a estrutura…") como lista plana com o caminho dos pais
    (`Conta › impl Conta`), ícone de tipo e de visibilidade. Sem busca, na ordem do arquivo; com busca, pelo nome
    (o lugar vale um pouco).
  - `Ctrl+T` (`nav.project_symbol`): `workspace/symbol` no servidor da aba ativa (ou em qualquer um rodando),
    150 ms depois da última tecla; resposta de busca velha é ignorada. Mostra o caminho relativo e a linha.
    Conferido com o rust-analyzer: traz funções e métodos, não só tipos (com o `container`, ex.: `Quadrado`);
    enquanto o projeto carrega volta vazio, e a paleta diz para tentar de novo.
  - Escolher salta com histórico (`Alt+←` volta). Com a paleta aberta, o atalho do próprio modo fecha e o de outro
    modo troca (vale para `Ctrl+P`, `Ctrl+Shift+P`, `Ctrl+Shift+O` e `Ctrl+T`).
  - Ícones de tipo e de visibilidade foram para o `tea-core` (`Kind::icon`, `visibility_badge`), usados pela paleta
    e pelo painel Estrutura.
  - [x] Teste manual (aprovado): `Ctrl+Shift+O` num arquivo Rust lista os símbolos e filtra; Enter vai até ele; `Ctrl+T` e
    digitar o nome de uma função de outro arquivo acha e abre; `Alt+←` volta.

- [x] **Painel Estrutura** (pedido à parte, inspirado no IntelliJ; adianta o item 3):
  - `tea-lsp`: `document_symbols(path)` (`textDocument/documentSymbol`), em árvore; o formato antigo (lista simples
    `SymbolInformation`) é aninhado pelo trecho de cada um. Erro do servidor vira lista vazia (o painel não fica
    esperando). Teste com o clangd.
  - O LSP não informa visibilidade (conferido com o rust-analyzer: `pub`, privado e `pub(crate)` chegam iguais).
    `tea-app/src/outline.rs` olha o texto antes do nome, na linha dele (só a declaração dele, depois do último
    `{`/`,`/`;`), mais a regra de cada linguagem sem palavra: Rust (`pub`, `pub(...)`; variantes e itens de trait
    herdam; métodos de `impl Trait for` são públicos), Java (sem nada = pacote), C# (membro privado, tipo
    internal), Kotlin/PHP (público), Swift (internal), TS/JS (`#nome` e `private` privados; no topo, só `export`
    sai do arquivo), Go (maiúscula = público, senão pacote), Python (`_` privado, `__x__` público), C/C++ (último
    `public:`/`private:`/`protected:` no nível da classe; class privada e struct pública sem rótulo; `static` fora
    de classe = só neste arquivo). Locais (dentro de funções) e linguagens sem regra ficam sem ícone.
  - Quando pede: ao trocar de aba, ao salvar como, quando o servidor fica pronto ou para, e 400 ms depois da última
    edição (o editor conta as edições enviadas ao servidor; `Editor::text_revision`).
  - Painel (extensão `tea-ext-outline`, lateral direita, `Alt+7` como no IntelliJ): árvore com ícone e cor por tipo,
    cadeado aberto (público), fechado (privado) ou chave (restrito, com o detalhe no tooltip), assinatura em cinza,
    símbolo do cursor destacado, abrir/fechar nós (lembrado por arquivo) e "recolher/expandir tudo".
  - [x] Teste manual (aprovado): abrir um arquivo Rust e `Alt+7`; ver público/privado/`pub(crate)`; mover o cursor muda o
    destaque; editar (criar uma função) atualiza a lista; clique leva ao símbolo.

- [x] **Item 4 — quem chama (árvore de chamadas):**
  - `tea-lsp`: `prepare_call_hierarchy` (`textDocument/prepareCallHierarchy`) acha a função no cursor
    (`Event::CallRoots`); `calls(item, direção, token)` pede `callHierarchy/incomingCalls` ou `outgoingCalls`
    (`Event::Calls`). O item cru do servidor vai junto no `CallItem` (os pedidos seguintes precisam dele). Os
    lugares das chamadas (`fromRanges`) ficam no arquivo de quem chama. Teste com o clangd (`soma` chamada duas
    vezes por `main`; `main` chama `soma`); conferido com o rust-analyzer entre dois arquivos.
  - Núcleo: `CoreEvent::Calls` (`calls::Event::Root` / `Children`) e `Context::call_children(nó, direção)`. O app
    (`tea-app/src/calls.rs`) guarda o item do servidor de cada nó; ids nunca se repetem, então resposta atrasada de
    uma árvore antiga é ignorada.
  - Painel "Chamadas" (extensão `tea-ext-calls`, embaixo): a função do cursor como raiz, já aberta; cada nó abre aos
    poucos (pede os filhos só na primeira vez, "…" enquanto espera; sem filhos, some a seta). Botões "Quem chama" /
    "O que chama" trocam a direção e recomeçam a árvore. Cada linha: ícone do tipo, nome, "2×" se chama mais de uma
    vez, assinatura e `arquivo:linha`. Clique em quem chama vai até a chamada; em "o que chama", até a declaração.
  - Atalhos: `Alt+Shift+H` (`lsp.incoming_calls`, como o Shift+Alt+H do VS Code; `Ctrl+Alt+H` do IntelliJ abriria
    o substituir do editor). "LSP: o que esta função chama" (`lsp.outgoing_calls`) fica na paleta.
  - [x] Teste manual (aprovado): numa função Rust chamada de outros lugares, `Alt+Shift+H` abre o painel com quem chama; abrir um
    nível a mais; clicar leva à chamada e `Alt+←` volta; "O que chama" mostra as chamadas feitas por ela.

### Prévias de arquivos

Ordem: (1) painel de prévia para Markdown e SVG → (2) aba de visualização com imagens (`image::viewer` do iced,
zoom e arrastar; hoje toda aba é um editor de texto) → (3) PDF na mesma aba, com o `hayro` (puro Rust,
Apache/MIT; pdfium precisa de biblioteca C, MuPDF é AGPL).

- [x] **Item 1 — prévia de Markdown e SVG:**
  - Núcleo: `Context::watch_text(["md", ...])` e `CoreEvent::ActiveText { path, text }`. O app manda o texto do
    arquivo ativo (só desses tipos) ao trocar de aba, ao salvar como e 250 ms depois da última edição (passar o
    mouse não conta); não manda de novo se nada mudou. Funciona sem servidor de linguagem.
  - Painel "Prévia" (extensão `tea-ext-preview`, lateral direita, `Ctrl+Shift+V` como no VS Code; o `KeyGate`
    não deixa essa tecla chegar ao editor, que colaria): Markdown pelo widget do iced (títulos, listas, tabelas,
    citações, código com cores); imagens locais aparecem (caminho relativo ao arquivo; da web, o texto
    alternativo); links da web abrem no navegador, links para arquivos abrem numa aba. SVG desenhado no tamanho
    do painel, com as dimensões; SVG quebrado mostra o erro e mantém o último desenho válido.
  - Botão no fim da barra de abas (como no Zed), só com um Markdown ou SVG ativo; fica destacado com a prévia
    aberta e o tooltip mostra o atalho. API genérica: `Context::file_action(FileAction { extensions, icon, tip,
    command, panel })`, para outras extensões porem botões por tipo de arquivo.
  - Executável release: 40 → 47 MB (decodificadores de imagem, desenho de SVG, cores no código do Markdown).
  - [x] Teste manual (aprovado): abrir o README.md e `Ctrl+Shift+V` (ou o botão de prévia no fim da barra de abas); editar um título e ver a prévia mudar; clicar num link da
    web e num link para arquivo; abrir um .svg, mudar uma cor e ver o desenho mudar; quebrar o SVG e ver o erro.

### Fase 4 — Plugins externos (melhoria futura)

Adiado: prévias de arquivos (Markdown, SVG, imagem, PDF) vêm antes.

- [ ] Host WASM (`wasmtime`) + WIT versionado.
- [ ] Protocolo de UI declarativa para painéis de plugins.
- **Plugins WASM têm confiança total:** um plugin instalado pode ler e escrever no disco e rodar processos, como extensões do VS Code.
  Não há sandbox de permissões (decisão do usuário). Instale só plugins em que você confia.

### Fase 5 — Debug
- [x] Spike: o `iced-code-editor` consegue mostrar breakpoints na margem e destacar a linha atual sem fork?
      **Sim, com uma camada por cima do widget** (`tea-editor/src/marks.rs`), sem fork:
      - O widget não tem API de marcas, mas `lsp_position_at_point(ponto)` é pública e converte um ponto da tela em
        linha do arquivo já considerando dobras e quebra de linha; `viewport_scroll`, `line_height` e `viewport_height`
        também são públicos. Perguntando a linha de cada faixa visível (~50 chamadas por desenho, o cálculo de linhas
        visuais é cacheado), sai o mapa "faixa da tela → linha". O próprio widget posiciona o hover do LSP assim.
      - A largura da margem (45 px de números + 14 px de dobras) é privada no widget: está copiada e um teste quebra se mudar.
      - A camada só pega o mouse em cima dos números de linha (clique liga/desliga breakpoint, bolinha fraca ao passar);
        no resto devolve `Interaction::None`, senão o `stack` do iced esconde o cursor do editor embaixo.
      - Breakpoints andam com o texto: a cada edição que muda o número de linhas, os de baixo da edição se deslocam
        (a última linha também vem de `lsp_position_at_point`, sem montar o texto).
      - Limites: a faixa da linha parada é desenhada **por cima** do texto (translúcida), não por trás; o cabeçalho do
        sticky scroll pode cobrir uma marca nas primeiras linhas. Se incomodar, o caminho é um PR no widget com uma API
        de decorações de linha (o mantenedor é ativo), não um fork.
      - Protótipo para ver na tela: `F9` (`debug.toggle_breakpoint`) e "Debug (protótipo): simular parada na linha do cursor".
      - Adaptador: o `gdb` 14+ já fala DAP sozinho (`gdb -i dap`), funciona com Rust/C/C++ e já está instalado aqui;
        bom candidato para começar, com `codelldb`/`lldb-dap` como alternativa.
- [x] Teste manual do protótipo das marcas na tela: aprovado.
- [x] **Parte 1 — cliente DAP** (`tea-dap`, sem UI, como o `tea-lsp`; reaproveita o enquadramento `Content-Length`):
      `Session::start(adaptador, programa, breakpoints, sink)`. Sequência conferida com o gdb 18: `initialize` →
      evento `initialized` → `launch` + `setBreakpoints` (caminho absoluto, um por arquivo) → `configurationDone`
      (só aí o programa roda). Ao `stopped`, o cliente já pede a pilha e entrega `Event::Stopped { frames }`.
      `continue`/`next`/`stepIn`/`stepOut`/`pause` agem na última thread parada; `stop` manda `disconnect` e mata o
      gdb se ele não sair em 2 s. Pedidos do adaptador (ex.: `runInTerminal`) são recusados. Teste com o gdb de verdade
      (breakpoint, sair da função, saída do programa, código de saída) pula sozinho sem gdb/rustc.
      No app (`debug.rs`): uma sessão por vez, com número para ignorar eventos de sessões velhas. Ao parar, abre o
      primeiro quadro da pilha que tem código-fonte no disco (passos dentro da std não têm) e mostra a seta;
      breakpoints mudados durante a sessão valem na hora. Comandos e atalhos: `F5` iniciar/continuar (na primeira vez
      escolhe o executável na janela do sistema), `Shift+F5` parar, `F6` pausar, `F10` próxima linha, `F11` entrar,
      `Shift+F11` sair, `F9` breakpoint, "Debug: escolher o programa…". `IDETEA_DAP_LOG=1` mostra os eventos no terminal.
      A saída do programa fica guardada para o painel (parte 3).
- [x] Teste manual da parte 1: aprovado.
- [x] **Parte 2 — como rodar o programa** (`debug_build.rs`):
  - Projeto com `Cargo.toml`: o `F5` roda `cargo build --message-format=json-render-diagnostics` numa thread e lê
    os executáveis (`compiler-artifact` com `kind` `bin`) da saída do cargo. Um só: roda. Vários: pergunta qual
    (e lembra até fechar o app).
  - `[debug]` no config: `program` (relativo à raiz; tem prioridade), `args`, `cwd`, `build` (comando via `sh -c`;
    `build = ""` não compila nem no cargo).
  - Sem nada disso: a janela de arquivos, como antes.
  - Compilação falhou: avisa com a primeira linha de erro e não inicia. A saída da compilação vai para a saída do
    debug (o painel da Parte 3 mostra). `Shift+F5` durante a compilação cancela o início.
  - [x] Teste manual (aprovado): `F5` num projeto Rust compila e para no breakpoint sem pedir o executável; com erro de
    compilação, aparece o erro; com `[debug] program` e `args` no config, roda esse programa com os argumentos.
- [x] **Parte 3 — painel Debug** (extensão `tea-ext-debug`, no painel de baixo):
  - O núcleo continua dono da sessão e conta tudo por `CoreEvent::Debug` (`tea_core::debug`): status
    (sem sessão / compilando / rodando / parado), pilha, grupos de variáveis, variáveis e saída. O painel pede com
    `Context::debug_select_frame` e `Context::debug_variables`; os botões rodam os comandos `debug.*`.
  - `tea-dap` ganhou `scopes(frame)` e `variables(referência)`; falha vira lista vazia, sem aviso.
  - Ao parar, o núcleo já pede os grupos do quadro mostrado. "Arguments" e "Locals" abrem sozinhos; "Registers" e
    "Globals" não (o gdb não os marca como caros, mas são longos).
  - Variáveis abertas são lembradas pelo caminho (`Locals/p/nome`): continuam abertas depois de um passo, mesmo com
    as referências novas. Respostas de uma parada anterior são ignoradas.
  - O painel aparece sozinho quando uma sessão começa (`debug.focus` também abre). Saída: últimas 5000 linhas
    guardadas, 1000 desenhadas, rolagem presa no fim; stderr em vermelho, gdb e compilação apagados.
  - [x] Teste manual (aprovado): `F5` abre o painel; ao parar, pilha e variáveis aparecem; abrir uma variável e dar `F10`
    mantém aberta; clicar num quadro de baixo vai ao arquivo e troca as variáveis; botões e saída funcionam.
      Adaptadores: `codelldb` (Rust/C/C++), `debugpy`, `delve`...

## Riscos

- **`iced-code-editor` é jovem** (v0.5, um único mantenedor, evolui rápido: 0.3 → 0.5 em ~6 meses). É mitigado pela fachada
  `tea-editor`. A performance foi validada na Fase 0.
- **`syntect` vs tree-sitter:** o `syntect` é bom para highlight, mas não serve para recursos estruturais. Reavaliar na fase 2.
- **API de extensão instável no início:** aceitável enquanto só houver extensões internas. Congelar e versionar antes da fase 4.
- **WASM aumenta o binário** (~10–20 MB) e o tempo de build. Fica atrás de uma feature `wasm-plugins`.
