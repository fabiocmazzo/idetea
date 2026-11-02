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
- Aberta de um terminal, a IDE **solta dele** e devolve o prompt (como `code .`); `-f`/`--foreground` fica preso.

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
`theme.toggle`, `config.{open,open_project,reload}`, `view.toggle_{left,right,bottom}`, `app.restart`, `app.quit`.
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

- [x] **Item 2 — imagens numa aba:**
  - Em vez de um segundo tipo de aba (o app usa `editor` em mais de 100 lugares), o `Editor` tem um modo imagem
    (`tea-editor/src/image_file.rs`): `Editor::open` de png, jpg, gif, webp, bmp, ico, tiff, tga e qoi guarda os
    bytes e mostra o `image::viewer` do iced (roda do mouse aproxima, arrastar move) com largura × altura e o
    tamanho do arquivo. Os bytes, não o caminho: arquivo mudado no disco recarrega a aba com a versão nova (pelo
    caminho, o iced usaria a do cache). Arquivo que não decodifica mostra o erro.
  - Nada grava na imagem: salvar não faz nada (gravar o texto vazio apagaria o arquivo), salvar como copia o
    arquivo, F9 e as ações de texto (buscar, desfazer...) não fazem nada; a barra de status mostra "Imagem".
    GIF animado mostra só o primeiro quadro.
  - [x] Teste manual (aprovado): abrir um PNG pela árvore e pelo Ctrl+P; aproximar com a roda e arrastar; `Ctrl+S` não muda o
    arquivo; trocar a imagem por fora (copiar outra por cima) e ver a aba atualizar.

- [x] **Item 3 — PDF numa aba:**
  - Mesmo jeito da imagem: modo PDF no `Editor` (`tea-editor/src/pdf_file.rs`), nada grava no arquivo.
  - `hayro` 0.8 desenha cada página numa imagem, numa thread só dele: a página pedida por último vai primeiro
    (a que o usuário está vendo) e pedidos de um zoom antigo são jogados fora. Cada página na lista tem um
    `sensor` do iced: só as que aparecem (ou estão a 400 px de aparecer) são desenhadas; ficam até 12 na
    memória. Desenha com 1,5× os pixels da tela (texto nítido em tela de alta densidade), no máximo 6000 px
    de lado.
  - Barra: "Página X de N", zoom − / % / + (25% a 500%) e "Ajustar" (acompanha a largura da aba; é o começo).
    Ao mudar o zoom, a imagem anterior fica esticada até chegar a nova.
  - PDF com senha ou danificado mostra o motivo. Sem seleção nem busca de texto.
  - Teste: PDF de 3 páginas gerado no teste (tamanho, texto desenhado, ~25 ms por página); zoom e "Ajustar".
  - Executável release: 47 → 57 MB (fontes padrão e tabelas de caracteres que o `hayro` embute).
  - [x] Teste manual (aprovado): abrir um PDF de várias páginas; rolar até o fim (as páginas aparecem conforme chegam);
    `+`/`−` e "Ajustar"; redimensionar o painel com "Ajustar" ligado; `Ctrl+S` não muda o arquivo.

### Idiomas (i18n)

Português, inglês e espanhol. Fluent (`i18n-embed` + macro `fl!`, o mesmo dos apps do COSMIC): chave conferida
na compilação, plural e valores no texto, e cada crate (ou plugin, no futuro) com os próprios `.ftl`. Descartados:
`rust-i18n` (sem conferência das chaves nem plural) e tabela à mão em Rust. Trocar de idioma vale ao reiniciar
(como VS Code e IntelliJ): os títulos de comandos e painéis são registrados na ativação. Fica de fora: SPEC e
DESENVOLVIMENTO; mensagens dos servidores de linguagem e do git (já seguem o idioma do sistema).

Entregas: (1) base → (2) núcleo e app → (3) extensões → (4) traduções da ajuda e revisão.

- [x] **Entrega 1 — base:**
  - `tea-core/src/i18n.rs`: `Language` (pt-BR, en, es), `resolve` (config vence; `"auto"` segue `LANG`; `pt-PT`
    vira pt-BR, `es-419` vira es; sistema sem idioma conhecido, como `LANG=C`, fica em inglês), `init` (uma vez, no
    `boot`, antes de ativar as extensões), `current` (sem `init`, nos testes: português), macro `i18n_loader!`
    (embute `i18n/` e cria o `LOADER` do crate) e `fl!` reexportada.
  - O português é a origem e o idioma de reserva: `fl!` confere a chave nele na compilação; o teste
    `translations_are_complete` confere em todos os crates que en e es têm as mesmas mensagens e os mesmos
    valores (percorre a árvore do Fluent, inclusive plurais). As marcas Unicode de isolamento em volta dos valores
    ficam desligadas (o iced desenharia quadradinhos).
  - `[ui] language = "auto"`; Configurações → Aparência ganha "Idioma" (Automático (idioma do sistema), Português
    (Brasil), English, Español), avisando que vale ao reiniciar. A aba Aparência já usa os textos traduzidos.
  - O widget de edição (busca, ir para linha) segue o idioma escolhido.
  - Custo: executável +0,5 MB; abertura sem mudança (17 ms).
  - [x] Teste manual (aprovado): Configurações → Aparência → Idioma = English; reiniciar; a aba Aparência fica em inglês e
    `Ctrl+F` no editor também; voltar para Automático.
- [x] **Entrega 2 — núcleo e app:**
  - Textos de tea-app, tea-core e tea-editor em `.ftl` (pt-BR, en, es): títulos dos comandos (paleta e aba
    Atalhos), avisos, diálogos, menus, Configurações inteira, paleta, ajuda (cabeçalho, atalhos fixos e "Sobre";
    o guia fica para a entrega 4), LSP, debug, observador de arquivos, `--help`, nomes dos símbolos, contagens
    do painel Referências (com plural), barra das abas de imagem e PDF, erros do config e dos atalhos.
  - Títulos dos comandos são ids (`cmd-file-save`); o teste `command_titles_exist` confere que todos têm texto.
  - O idioma é escolhido antes de ler o config (`peek_language` lê só `[ui] language`), para os avisos do config
    já saírem no idioma certo; avisos da linha de comando viram texto só depois disso. `--help` segue o sistema.
  - A dica "Com rustup: ..." do rust-analyzer não depende mais do texto do erro.
  - [x] Teste manual (aprovado): Idioma = English (ou `LANG=es_ES.UTF-8 idetea`); reiniciar; conferir paleta (`Ctrl+Shift+P`),
    menus, Configurações, ajuda (`F1`: Atalhos e Sobre), fechar aba com alteração (diálogo), uma imagem e um PDF;
    `idetea --help` no terminal; voltar para Automático e ver tudo em português como antes.
- [x] **Entrega 3 — extensões:**
  - Cada uma das 11 extensões tem os próprios `.ftl` (pt-BR, en, es): nomes dos painéis, comandos, botões,
    menus de contexto, diálogos, avisos e textos vazios. No git, também as explicações dos erros de rede.
  - O git deixou de reconhecer o resultado pelo texto ("Commit ...", "alterações descartadas"): cada ação diz o
    que fazer depois (`After`).
  - [x] Teste manual (aprovado): Idioma = English; reiniciar; abrir cada painel (Arquivos, Git, Busca, Problemas, Terminal,
    Referências, Chamadas, Estrutura, Prévia, Debug) e ver os textos em inglês; na árvore, botão direito → menu e
    Apagar… (diálogo, cancelar); no git, stage de um arquivo, Descartar alterações… (cancelar) e um commit (a
    caixa da mensagem limpa); barra de status ("Ln, Col", "● unsaved"); voltar para Automático.
- [x] **Entrega 4 — guia da ajuda e revisão:**
  - O guia (9 seções, 41 parágrafos) em `.ftl`. Os comandos citados viram valores (`{ $palette-open }`): o teste de
    traduções confere que cada idioma cita os mesmos, e `get_with_markers` troca pelo atalho do usuário.
  - Revisão: no Git saiu "Push e pull: use o terminal" (o painel já tem os botões); "Ctrl+P" fixo virou "abrir
    pelo nome"; novo parágrafo sobre o idioma em Configuração. Nenhum resto de português em en/es; os textos
    iguais nas duas línguas (Cancelar, Terminal...) foram conferidos.
  - DESENVOLVIMENTO: cuidados com `fl!` antes do `init`, textos comparados no código, ids e o guia.
  - [x] Teste manual (aprovado): Idioma = English; reiniciar; F1 → Guia: textos em inglês, atalhos em negrito no lugar
    certo (ex.: Ctrl+Shift+P em "Getting started"); repetir com Español; voltar para Automático e ver o guia
    em português.

### IDE básica (antes dos plugins)

O que mais falta no uso diário. Cada item é uma entrega com teste manual; a ordem é a sugerida.

- [x] **Sessão:** ao abrir um projeto, voltam as abas (com a posição do cursor), a aba ativa, os painéis abertos
  e o tamanho da janela.
  - Um arquivo por projeto em `$XDG_STATE_HOME/idetea/sessions/` (fora do projeto: nada para o git). O tamanho
    da janela fica no `state.toml`, que vale para todos.
  - Grava ao sair e sempre que as abas ou os painéis mudam (um travamento perde no máximo o cursor).
  - `idetea arquivo...` abre só os arquivos pedidos, sem a sessão (uso rápido no terminal). `idetea pasta`
    restaura a sessão daquela pasta.
  - Abas sem nome e arquivos que sumiram ficam de fora. `[ui] restore_session = false` desliga (também em
    Configurações → Aparência).
  - Painéis restaurados não recebem "painel apareceu" (o terminal e a busca pegariam o teclado): o terminal
    restaurado mostra o botão "Iniciar terminal"; o git se atualiza sozinho ao achar o repositório.
  - Custo (release): cerca de 12 ms por aba na abertura (lê e colore o arquivo); com 2 abas, 22 → 50 ms.
    Se pesar com muitas abas, as de fundo podem carregar só quando forem clicadas.
  - [x] Teste manual (aprovado): abrir 3 arquivos, mover o cursor, deixar outro ativo, abrir Git e Estrutura, mudar o
    tamanho da janela; sair e abrir de novo: tudo volta. `idetea Cargo.toml` abre só ele. Desligar em
    Configurações e reabrir: abre vazio.
- [x] **Recentes:** "Arquivo: abrir recente…" (`Ctrl+R`, como no VS Code) abre a paleta com os projetos e os
  arquivos abertos por último.
  - Uma lista para todas as janelas em `$XDG_STATE_HOME/idetea/recent.toml` (20 projetos, 50 arquivos). Cada
    mudança relê o arquivo antes de gravar: duas janelas não apagam o que a outra acrescentou.
  - Sem busca: projetos primeiro, depois arquivos, do mais novo para o mais velho. Com busca: pelo nome (o lugar
    vale um pouco). Fora da lista: o projeto atual, o arquivo ativo e o que não existe mais.
  - Projeto abre numa nova janela (como "abrir pasta"); arquivo abre numa aba (`Alt+←` volta).
  - [x] Teste manual (aprovado): abrir a IDE em duas pastas diferentes e alguns arquivos; `Ctrl+R` lista a outra pasta e os
    arquivos; digitar parte de um nome filtra; Enter num projeto abre outra janela, num arquivo abre a aba;
    `Ctrl+R` de novo fecha a paleta. Ajuda → Guia cita o atalho.
- [x] **Recuperar não salvos:** a cada 3 s, as abas com alterações que mudaram são copiadas; depois de um
  travamento, a IDE oferece recuperar.
  - Cópias em `$XDG_STATE_HOME/idetea/backups/<projeto>/`: `<pid>-<chave>.txt` (o texto) e `.toml` (de que
    arquivo é). Abas sem nome também. A cópia sai quando a aba é salva, fica sem alterações ou fecha; sair
    normalmente apaga todas as da janela.
  - Ao abrir: cópias de uma janela que não está mais rodando (no Linux, `/proc/<pid>` com o mesmo executável)
    geram o diálogo "A IDE fechou sem salvar N arquivos" com Recuperar, Descartar e Depois (pergunta de novo
    na próxima vez). Cópia igual ao disco sai calada; duas do mesmo arquivo: fica a mais nova.
  - Recuperar abre cada arquivo com o texto da cópia como alteração não salva (`Editor::replace_text`: troca
    só o trecho que mudou, e desfazer volta ao do disco).
  - O tique de 3 s só roda enquanto há alterações (é uma thread; o iced daqui não tem `time::every`).
  - [x] Teste manual (aprovado): alterar dois arquivos e uma aba sem nome, esperar uns 5 s e matar a IDE
    (`pkill -9 idetea`); abrir de novo: o diálogo lista os três; Recuperar: as abas voltam com ● e o texto
    alterado, `Ctrl+Z` volta ao do disco. Repetir e escolher Descartar: abre limpo e não pergunta mais. Sair
    normalmente com alterações descartadas: não pergunta na próxima.
- [x] **Renomear símbolo (F2):** "LSP: renomear símbolo…" abre a paleta com o nome no cursor; Enter pede ao
  servidor (`textDocument/rename`) e uma prévia mostra "N mudanças em M arquivos" com a lista, antes de mudar
  qualquer coisa.
  - Confirmando, os arquivos que faltam abrem em abas e todos ficam alterados, sem salvar (uma edição por arquivo:
    `Ctrl+Z` desfaz). Novo "Arquivo: salvar todos" (`Ctrl+Alt+S`) grava tudo.
  - Aceita as duas formas da resposta (`changes` e `documentChanges`); criar, renomear ou apagar arquivos (ex.:
    renomear um módulo no Rust) fica de fora por enquanto. Servidor que recusa: a mensagem dele aparece.
  - O teste com o clangd confere as três mudanças de `soma` e que o disco não muda antes de aplicar.
  - [x] Teste manual (aprovado, Rust): `F2` numa função usada em outro arquivo, novo nome, Enter: a prévia lista os dois
    arquivos; Renomear: as abas abrem com ●, o código compila depois de `Ctrl+Alt+S`; `Ctrl+Z` numa aba volta o
    nome. `F2` num espaço avisa; Esc na paleta ou Cancelar na prévia não muda nada.
- [x] **Substituir no projeto:** "Busca: substituir no projeto" (`Ctrl+Shift+H`, ou ▸ ao lado da busca) abre
  "Substituir por" no painel Busca. Cada resultado mostra a linha como fica (antes em vermelho, depois em verde);
  "Substituir" troca a linha ou o arquivo, "Substituir tudo" (ou Enter no campo) pede confirmação. No modo regex
  `$1`/`${nome}` valem; fora dele o texto vai como está. A troca é linha a linha, como a busca.
  A troca é do núcleo (`tea_core::replace`, pedido `Request::Replace`, resposta `CoreEvent::Replaced`): arquivo
  aberto numa aba muda na memória (um passo de desfazer, fica para salvar; "Arquivo: salvar todos" grava); os
  outros são gravados direto no disco (sem desfazer — o diálogo avisa). O que foi trocado sai da lista, sem buscar
  de novo (a busca lê o disco, e as abas ainda não foram salvas). Com a busca rodando ou a regex inválida, os
  botões ficam desligados.
  Limites: em aba com alterações, a linha é a do disco (se mudou de lugar, a troca daquela linha pode não achar
  nada); com a busca cortada em 2000 resultados, "Substituir tudo" só muda os arquivos listados (o diálogo avisa).
  - [x] Teste manual (aprovado): `Ctrl+Shift+H`, buscar um nome usado em vários arquivos (um deles aberto numa aba),
    escrever a troca: a prévia aparece em cada linha. "Substituir" numa linha de arquivo fechado: o arquivo
    muda no disco (Git mostra) e a linha sai da lista. "Substituir tudo": confirma, o aberto fica com ● e
    `Ctrl+Z` volta; regex `(\w+)_(\w+)` → `$2_$1` troca a ordem.
- [x] **Editor e salvar:** seção `[editor]` no config e aba "Editor" nas Configurações: quebrar linhas
  longas, mostrar espaços, guias de recuo, tamanho do recuo, recuar com espaços ou tabs; ao salvar, tirar
  espaços no fim das linhas (nunca em Markdown, a não ser que o `.editorconfig` mande), garantir a quebra de
  linha final, e formatar com o LSP. A limpeza é uma edição (um passo de desfazer) e vale para todo salvar
  (Ctrl+S, salvar todos, fechar salvando); formatar só no Ctrl+S, esperando até 2 s pelo servidor (depois
  grava sem formatar, com aviso).
  `.editorconfig` (`indent_style`, `indent_size`/`tab_width`, `trim_trailing_whitespace`,
  `insert_final_newline`): lido da pasta do arquivo para cima até `root = true`, o mais perto vence; vale
  mais que o config. Sem ele, arquivo que já recua com tabs (maioria das linhas recuadas) continua com tabs.
  As opções são recalculadas ao abrir, ao recarregar o config, ao salvar um `.editorconfig` na IDE e ao
  renomear.
  Limites: o caractere tab sempre ocupa 4 colunas na tela (fixo no widget; `tab_size` só muda quantos
  espaços o Tab insere); faixas `{1..3}` nos padrões do `.editorconfig` não são aceitas (a seção é
  ignorada); `.editorconfig` mudado fora da IDE só vale ao reabrir o arquivo ou recarregar o config.
  - [x] Teste manual (aprovado): Configurações → Editor: desligar "Quebrar linhas longas" e "Mostrar espaços" muda na
    hora. Ligar "Tirar espaços" e "Quebra de linha no fim": salvar um `.rs` com espaços no fim limpa, e
    `Ctrl+Z` volta. Um `.md` mantém os dois espaços. Criar `.editorconfig` com `[*.rs]` e `indent_size = 2`,
    salvar: Tab num `.rs` insere 2 espaços. Num Makefile, Tab insere o caractere tab. Com "Formatar ao
    salvar", `Ctrl+S` num `.rs` desarrumado formata e grava.
- [x] **Dica de parâmetros:** ao digitar um caractere que o servidor anuncia (`triggerCharacters` e
  `retriggerCharacters` do `signatureHelpProvider`; sem lista, `(` e `,`), o editor pede
  `textDocument/signatureHelp`. A caixa aparece acima do cursor (abaixo, sem espaço em cima), com a assinatura e
  o parâmetro atual em negrito e sublinhado, a documentação do parâmetro (primeiro parágrafo) e `1/2` quando há
  sobrecargas. Aberta, cada movimento do cursor pede de novo: o parâmetro acompanha, e sai da chamada (ou fecha
  o parêntese) = o servidor responde vazio e a caixa some. `Esc` fecha (com o autocomplete aberto, o `Esc` é
  dele). Erro do servidor só esconde. Parâmetro em texto é achado no rótulo em ordem, depois do `(`; em
  posições, conta UTF-16 (como diz o protocolo).
  - [x] Teste manual (aprovado, Rust): digitar `soma(` numa chamada de uma função com dois parâmetros: a caixa aparece
    acima com o primeiro destacado; `1, ` passa para o segundo; `)` fecha. `Esc` no meio fecha. Com o menu de
    autocomplete aberto, a dica continua em cima e as setas são do menu.
- [x] **Git:** marcas na margem nas linhas alteradas (clique mostra o diff), trocar e criar branch, blame da
  linha atual. Em três entregas:
  - [x] **Marcas na margem:** o git manda a versão do último commit (`git show HEAD:<arquivo>`) de cada
    arquivo aberto (`Request::DiffBase`; de novo quando o HEAD muda: commit, checkout, pull). O editor compara
    com o texto da aba (crate `similar`, linha a linha, até 50 ms; 300 ms depois de parar de digitar, sem
    precisar salvar) e desenha uma faixa à esquerda dos números: verde = nova, azul = mudada, cunha vermelha =
    apagadas. Clicar na faixa (8 px) abre uma caixa com as linhas antigas e "Desfazer mudança" (volta o
    trecho na aba, um passo de desfazer); Esc ou clicar no texto fecha. O resto dos números continua sendo
    dos breakpoints. Sem marcas: arquivo novo (fora do commit), binário, maior que 4 MB, fora do repositório.
    Compara com o HEAD (não com o stage): pôr no stage não apaga as marcas.
    - [x] Teste manual (aprovado): abrir um arquivo do repositório, mudar uma linha, acrescentar outra e apagar uma:
      aparecem azul, verde e a cunha vermelha sem salvar. Clicar na azul: mostra a linha antiga; "Desfazer
      mudança" volta e `Ctrl+Z` refaz. Commitar pelo painel: as marcas somem.
  - [x] **Trocar e criar branch:** "Git: trocar de branch" (`git.checkout`; também clicando no nome da branch
    na barra de status ou no painel) abre a lista na paleta: locais da mais recente para a mais antiga
    (`for-each-ref --sort=-committerdate`), com a data e "atual"; depois as remotas sem local de mesmo nome
    (escolher = `git switch --track`). Texto que não é uma branch vira "Criar branch “nome”" (`git switch -c`, a
    partir de onde está; vem primeiro se nada combina, senão por último, para o Enter não criar sem querer).
    Alterações não commitadas vão junto; se colidirem, o git recusa e a mensagem explica (commit ou descarte).
    Depois de trocar: aviso, abas sem alterações recarregam, a árvore e as marcas atualizam.
    A lista é genérica: `tea_core::pick` (`Request::Pick`) deixa qualquer extensão mostrar itens na paleta,
    com a linha opcional de "criar" para o texto digitado; o escolhido volta como mensagem da extensão.
    - [x] Teste manual (aprovado): clicar na branch na barra de status: a lista aparece com a atual marcada. Digitar um
      nome novo + Enter: cria e troca (a barra mostra). Voltar para a main pela lista. Com um arquivo mudado
      que existe diferente na outra branch, trocar: o aviso explica que precisa commitar ou descartar.
  - [x] **Blame da linha atual:** na barra de status, ao lado da branch: "autor, há 3 dias · mensagem" da linha do
    cursor (até 70 caracteres); linha mexida e salva mas não commitada: "Você · ainda não commitado". Clique abre
    o commit (mensagem, autor, quando, hash) com "Copiar hash". `git blame --porcelain` do arquivo inteiro numa
    thread, guardado por arquivo; de novo ao salvar, quando o HEAD muda ou arquivos mudam por fora. É do arquivo
    no disco: com a aba alterada e não salva, some (as linhas podem não bater). Arquivo fora do commit: nada.
    - [x] Teste manual (aprovado): andar com o cursor por um arquivo antigo do repositório: autor e mensagem mudam por linha.
      Clicar: aparece o commit; "Copiar hash" copia. Digitar algo: some; salvar: a linha mostra "ainda não
      commitado".
- [x] **Tarefas:** rodar `cargo run`/`cargo test` ou um comando do config, com a saída no terminal; no Rust,
  "rodar este teste".
  `Ctrl+Shift+B` (`tasks.run`) abre a lista na paleta: primeiro a última rodada, depois as de `[tasks]` no config
  (`nome = "comando"`, global ou do projeto; vencem uma achada de mesmo nome), cargo build/run/test/clippy (com
  `Cargo.toml` na raiz), scripts do `package.json` (npm, ou pnpm/yarn pelo lockfile) e até 15 alvos do `Makefile`.
  A tarefa roda na raiz do projeto, num terminal só dela ao lado do shell (abas "Terminal" / "Tarefa: nome" no
  topo do painel, mais "Fechar tarefa"): `/bin/sh -c` com `eval`, então `&&`, pipes e variáveis funcionam.
  `Ctrl+C` para só o comando; no fim, "[nome terminou: código N · Enter fecha]". Rodar outra substitui a
  anterior. `Ctrl+Shift+R` (`tasks.rerun`) roda a última de novo (sem nenhuma, abre a lista).
  "Rust: rodar o teste no cursor" (`tasks.test_here`, sem atalho): o `fn` mais perto acima do cursor com
  `#[test]`/`#[x::test]`/`#[rstest]`; pacote = o `Cargo.toml` com `[package]` mais perto; em `tests/x.rs` junta
  `--test x`. Roda `cargo test -p pacote [--test x] nome`. O evento é genérico (`CoreEvent::RunTask`), para
  plugins poderem rodar comandos também.
  - [x] Teste manual (aprovado): `Ctrl+Shift+B` neste repo: aparecem as tarefas do cargo; escolher "cargo build": a aba
    "Tarefa" abre com a saída colorida e o código no fim; Enter fecha e volta ao shell. `Ctrl+Shift+R` roda de
    novo. Rodar "cargo run" e `Ctrl+C`: para e mostra o código. Criar `[tasks] oi = "echo oi && exit 2"` no config
    do projeto: aparece primeiro e termina com código 2. Num `#[test] fn` (ex. `tasks.rs`), "Rust: rodar o teste no
    cursor" roda só ele.
- [x] **Editor dividido:** dois arquivos lado a lado.
  Cada aba fica num lado (esquerdo ou direito), e cada lado tem a sua barra de abas e mostra uma aba; um mesmo
  arquivo fica num lado só (abrir de novo leva o foco até ele). `Ctrl+\` (`editor.split`, ou "Mandar para a
  direita/esquerda" no menu da aba) manda a aba para o outro lado: sem divisão, divide (com uma aba só, avisa que
  precisa de outra); era a última do lado, junta tudo de novo. Abrir arquivo vai para o lado com foco. O foco
  troca clicando no editor ou numa aba do outro lado, ou com `Ctrl+1`/`Ctrl+2` (`editor.focus_left/right`). A
  aba ativa do lado com foco fica em destaque; a do outro lado, mais apagada. `Ctrl+PageDown/PageUp` e "fechar
  outras/à direita" valem dentro do lado. A divisa arrasta (duplo clique volta ao meio) e a largura fica no
  `state.toml`; a sessão lembra o lado de cada aba e a aba mostrada no outro lado.
  - [x] Teste manual (aprovado): com dois arquivos abertos, `Ctrl+\`: o atual vai para a direita e o outro fica à
    esquerda. Abrir um arquivo pela árvore: vai para o lado com foco. Clicar no texto do outro lado e digitar:
    digita lá (a barra de status acompanha). `Ctrl+1`/`Ctrl+2` trocam. Arrastar a divisa; duplo clique centraliza.
    Fechar todas as abas da direita: volta a um editor só. Fechar a IDE dividida e abrir: volta dividida.
- [x] **Distribuição:** `.desktop` com ícone, `cargo install`, AppImage ou Flatpak; CI no GitHub (testes,
  clippy e o executável a cada versão).
  O executável só depende da libc (Wayland, X11, Vulkan e xkbcommon são abertos na hora), então um `.tar.gz` e um
  AppImage sem bibliotecas dentro rodam em qualquer distro com glibc igual ou mais nova que a do build (Ubuntu
  22.04). Flatpak fica de fora: a sandbox atrapalha uma IDE (git, LSP, cargo e o shell do terminal são os do
  sistema).
  - `idetea --install-desktop` grava `~/.local/share/applications/idetea.desktop` (no menu, `Exec` = este
    executável, ou `$APPIMAGE`) e o ícone em `icons/hicolor/scalable/apps`; `--uninstall-desktop` tira. O
    `.desktop` e o SVG vão dentro do executável (`include_str!`), então funciona depois de `cargo install`.
  - Aberta pelo menu (`--menu %F`): sem pasta nos argumentos, a raiz é o repositório do arquivo aberto (pasta com
    `.git` acima dele, senão a do arquivo) ou o projeto mais recente que ainda existe (senão, a home).
  - `cargo install --locked --git https://github.com/fabiocmazzo/idetea tea-app`.
  - `packaging/dist.sh` (`just dist`): perfil `dist` (release + `strip`), `idetea-<v>-<arch>-linux.tar.gz` (com
    `install.sh`: `~/.local/bin` + `--install-desktop`) e `IDE_TEA-<v>-<arch>.AppImage` (`appimagetool` baixado
    em `target/`; `--no-appimage` pula).
  - CI: `ci.yml` (push na main e PRs: clippy `-D warnings` e testes) e `release.yml` (tag `v*`: confere a tag com a
    versão do `Cargo.toml`, testa, gera os pacotes no Ubuntu 22.04 e publica a versão no GitHub).
  - `just desktop` (desenvolvimento) não sobrescreve um atalho já instalado.
  - [x] Teste manual (aprovado): `cargo install --path crates/tea-app --locked` e `idetea --install-desktop`: a IDE aparece no
    menu com o ícone e abre o último projeto. Abrir um arquivo de um repositório pelo gerenciador de arquivos
    ("Abrir com IDE TEA"): a raiz é o repositório. `just dist`: gera o `.tar.gz` e o AppImage; o AppImage abre com
    `./IDE_TEA-*.AppImage .`. Depois do push: o CI passa no GitHub.
- [x] **Arquivos grandes:** medir com um arquivo de 50 MB e com uma linha gigante (objetivo "não trava").
  Medido com `cargo run --release -p tea-editor --example large_render`: quadros de verdade (layout, desenho e texto
  no wgpu, sem janela), 1200×800, guardando a rolagem entre um quadro e outro, como no app.
  - **50 MB de Rust (1,46 milhão de linhas): não trava.** Abrir ~80 ms; quadro normal 5–30 ms; o pior quadro
    (saltar para o meio, o destaque alcança aos poucos) ~350 ms; digitar ~160 ms por tecla no pior caso; texto
    inteiro (salvar) ~14 ms.
  - **Uma linha gigante travava.** Sem quebra de linha, o desenho do texto molda a linha inteira e cresce com o
    quadrado do tamanho: 100 KB = 0,7 s por quadro, 1 MB = 56 s (10 MB não terminou). Com quebra, o destaque refaz a
    linha a cada tecla: ~0,65 s por MB (10 MB = vários segundos por tecla).
  - **Modo leve** (`tea_editor::LONG_LINE`, linha com mais de 20 mil bytes, o mesmo limite do VS Code para parar o
    destaque): quebra de linha sempre ligada (a opção é ignorada nessa aba), texto puro, sem espaços visíveis, guias,
    cores de parênteses, amostras de cor nem servidor de linguagem. A barra de status mostra "JSON (modo leve)" e um
    aviso explica ao abrir. Com ele, um JSON de 10 MB numa linha só: abrir ~5 ms, quadro 20–75 ms, rolar ~140 ms no
    pior caso, digitar ~170 ms por tecla.
  - Ficou como está (medido, não trava): a cópia de recuperação de uma aba de 50 MB com alterações custa ~75 ms a
    cada 3 s (copiar o texto e o hash); as marcas do git, ~100 ms depois de parar de digitar. Números de linha com 7
    dígitos (mais de um milhão de linhas) perdem o primeiro dígito: a margem do widget tem largura fixa.
  - [x] Teste manual (aprovado): abrir um arquivo de 50 MB (`cargo run --release -p tea-editor --example large_file --
    1500000 --write /tmp/grande.rs`) e um JSON de uma linha só de alguns MB: rolar com a barra até o fim, `Ctrl+End`,
    buscar e digitar sem travar; o JSON abre com o aviso do modo leve e "(modo leve)" na barra de status.
- [x] **Soltar do terminal:** `idetea ./` devolve o prompt na hora. Aberta de um terminal (stdout ou stderr é um
  terminal), a IDE roda de novo a si mesma numa sessão própria (`setsid`, sem terminal: nem o Ctrl+C nem fechar o
  terminal a derrubam) e sai. A saída da cópia vai para `~/.local/state/idetea/idetea.log` (recomeça acima de 1 MB).
  Não solta com `-f`/`--foreground`, em build de debug (`cargo run` mostra os logs) nem fora de um terminal (menu,
  "Abrir pasta"). No AppImage, roda o `$APPIMAGE` de novo (a montagem some quando o primeiro processo sai). Linux e
  macOS; no Windows nada muda. Medido: o prompt volta em ~20 ms.
  - [x] Teste manual (aprovado): `cargo install --path crates/tea-app --locked` e `idetea .` num terminal: o prompt volta e a
    IDE abre; fechar o terminal: a IDE continua. `idetea -f .`: fica presa (Ctrl+C fecha). `cargo run -p tea-app`
    continua preso, com os logs. O AppImage (`just dist`) também solta e abre.

### Interfaces declarativas (antes dos plugins)

Agentes e processos externos gravam um documento JSON em `.idetea/ui/<nome>.json`; a IDE desenha um painel nativo
e grava a resposta do usuário em `.idetea/ui/responses/<nome>/`. Estudo, decisões e riscos:
[RFC 0001](docs/rfc/0001-interfaces-declarativas.md). Desligada por padrão:
`[extensions] interfaces = { enabled = true }` liga (precisa reiniciar).

- [x] **Formato v1 e instruções para agentes:** `docs/interfaces/formato.md` (documento, resposta, regras de escrita
  atômica e de confirmação), `docs/interfaces/idetea-ui.schema.json` (referência para quem escreve; a IDE valida com
  tipos `serde`), exemplos e um texto curto para colar no `AGENTS.md`/`CLAUDE.md`.
- [x] **`tea-declarative-ui`, modelo:** tipos do documento e da resposta, validação com limites (tamanho, número de
  componentes, nomes), ponteiros JSON (`/a/b`), estado local com a regra de atualização (o agente vence só onde
  mudou o valor) e montagem da resposta com os caminhos do `context`. Só testes, sem iced.
- [x] **`tea-declarative-ui`, desenho:** `text`, `button`, `checkbox`, `select` e `input` em coluna; componente
  desconhecido vira aviso no lugar; botões desligados depois de responder (`submit: "once"`). Testes com `iced_test`.
- [x] **`tea-ext-ui` (extensão `interfaces`):** observa `.idetea/ui/` sozinha (o observador do app pula pastas do
  `.gitignore`); lê com proteção (symlink recusado, tamanho máximo, nome validado); ignora arquivos versionados no
  git; cria `.idetea/ui/.gitignore` com `*`; grava respostas com `create_new` + `rename`; mostra "aguardando o
  agente" até a resposta ser apagada; guarda o que foi digitado em `$XDG_STATE_HOME/idetea/ui/`; painel
  "Interfaces" no slot direito, que aparece sozinho sem tirar o foco do editor.
  - [x] Teste manual: com `examples/interfaces/pergunta.sh`, a interface aparece sozinha; escolher, marcar, digitar
    e clicar gera a resposta, que o script lê e apaga ("recebido" no painel); o script grava uma revisão nova e a
    interface muda sem perder o que estava digitado; fechar e abrir a IDE mantém interface e valores; um JSON
    inválido mostra o erro no painel. (aprovado)
- [x] **Teste com um agente de verdade** (Claude Code com o texto de instruções): anotar o que confundiu o agente e
  ajustar o formato antes de considerar estável. (aprovado) O formato bastou; o texto para agentes ganhou os cinco
  componentes, o que volta na resposta, como esperar em bash e o que apagar no fim. A IDE passou a mostrar o painel
  ao abrir quando há pergunta sem resposta e a apagar a pasta de respostas vazia de interfaces fechadas.
- [x] **Ensinar os agentes:** comando `interfaces.teach` (e botão no painel vazio) grava as instruções no
  `AGENTS.md`/`CLAUDE.md` do projeto, no idioma da interface, entre marcadores (rodar de novo só troca o bloco).
  Pergunta antes, dizendo quais arquivos; pula o `CLAUDE.md` que já importa `@AGENTS.md`; symlink entre os dois
  conta uma vez e continua symlink; abre o arquivo depois.
  - [x] Teste manual: num projeto sem `AGENTS.md`, o botão do painel vazio cria o arquivo e o abre; rodar de novo
    avisa "já atualizadas"; mudar o idioma da IDE e rodar troca o bloco sem mexer no resto do arquivo; com
    `CLAUDE.md` existente, o diálogo lista os dois. (aprovado)
- [x] **Arquivos nas interfaces:** componente `file` (`path` relativo à raiz, sem `..`; `line`, `column`, `label`,
  `open`) para o que não cabe num campo: mostrar um trecho de código, pedir um texto longo, ajustar um arquivo. O
  link abre o arquivo na posição; `open: true` abre sozinho quando a revisão chega, **menos** se o usuário digitou
  no editor nos últimos 2 s (aí só um aviso: o cursor não é roubado no meio de uma linha). Ao clicar num botão, a
  IDE salva antes os arquivos dos componentes `file` com alterações (`Request::SaveFiles`) e só então grava a
  resposta; se um não salvar, não há resposta. A IDE não cria arquivos: o agente cria (rascunhos em
  `.idetea/ui/rascunhos/`, fora do git). Formato, schema, exemplo, instruções dos agentes (3 idiomas) e o painel
  dos plugins também (o clique abre o arquivo).
  - [x] Teste manual: `examples/interfaces/arquivos.sh` (cria um projeto em `/tmp`, abre a IDE e guia os
    passos): o rascunho abre sozinho na linha 3; editar sem salvar e clicar "Commitar": o arquivo é salvo e a
    resposta aparece. A revisão 2 chega enquanto digita noutro arquivo: não abre, aparece o aviso; os links do
    painel abrem. Um `path` com `..` mostra o erro no painel. (aprovado)
- [ ] **Desenvolver a IDE TEA com a IDE TEA (dogfooding):** o agente de código deste repositório pergunta e entrega os
  testes manuais pelo painel Interfaces (instruções no `CLAUDE.md`). Cada lacuna encontrada entra na lista abaixo,
  com o que aconteceu; a lista é a base da [RFC 0003](docs/rfc/0003-interfaces-pelo-uso.md) (evolução das interfaces: decisões, artefatos para revisão,
  simulações), em vez de planejar tudo antes do uso.
  - Lacunas encontradas:
    1. "Ensinar os agentes" num projeto sem `AGENTS.md` nem `CLAUDE.md` cria só o `AGENTS.md`; o Claude Code lê o
       `CLAUDE.md` (a confirmar se ele também lê o `AGENTS.md`). Opção: criar o `AGENTS.md` e um `CLAUDE.md` com
       `@AGENTS.md`. (resolvida: item abaixo)
    2. Não há texto formatado (Markdown) no painel: uma explicação mais longa do agente fica no terminal, e o painel
       só aponta para lá. (resolvida: componente `markdown`)
    3. O usuário quis fazer perguntas de volta ao agente e usou o campo "Observação" para isso. Não há um jeito
       padrão de comentar uma resposta; avaliar um comentário opcional em toda resposta. (resolvida: campo de
       comentário da IDE)
    4. Na entrega de um teste, o passo "edite o config global" não teve link: o `file` só aceita caminhos do
       projeto (por segurança). Opção: alguns destinos conhecidos fora do projeto (`config:global`), sem caminho
       livre. Também faltou lista numerada: cada passo virou um `text` (lacuna 2). (resolvida: links
       `idetea:config` e `idetea:config-project` no `markdown`)
    5. O botão "Aprovado" de um teste não dizia se podia commitar: precisou de uma segunda pergunta (revisão 2).
       Nas entregas, oferecer "Aprovado, pode commitar" junto.
    6. Instruções para agentes: o usuário prefere o texto sempre em inglês (agentes leem melhor), com uma linha
       pedindo para conversar no idioma do usuário, em vez de um texto por idioma da IDE. (resolvida: item abaixo)
    7. Com o fluxo no painel, uma pergunta que o agente fez só no chat passou despercebida: o usuário foi responder
       no painel e não havia nada lá. Toda pergunta que espera resposta vai para o painel. (as instruções dos
       agentes agora dizem isso)
    8. Um teste foi aprovado sem ter rodado na versão nova: a IDE aberta era a antiga, e o resultado parecia certo. O
       agente conferiu o arquivo e pediu de novo. Nas entregas, dizer como reconhecer a versão nova (ou o agente
       conferir o efeito no disco antes de commitar, como aqui). Aconteceu de novo no teste da lacuna 9. (resolvida:
       item abaixo; o agente continua conferindo no disco)
    10. Aprovado antes de terminar todos os passos: "Ensinar os agentes" ficou de fora duas vezes e "Reiniciar a
       IDE" uma; o agente pegou conferindo o disco e o processo. (RFC 0003, etapa B)
    11. O agente não sabe qual IDE respondeu: cruzou `pgrep`, `/proc` e o PID dentro do `event`, que não é
       contrato. (RFC 0003, etapa A)
    12. Um aviso fixo do agente sobre a IDE ("esta é a antiga") continuou na tela depois que deixou de ser verdade,
       e o teste falhou por isso. (RFC 0003, etapas A e C)
    13. No segundo projeto (o jogo), o agente novo foi ler o código da IDE para saber se o painel mostra imagem e
       áudio: nem o formato nem as instruções diziam o que o `file` faz com mídia. (resolvida: item abaixo)
    14. Do `lacunas.md` do jogo (15 itens, conferidos no código): a aba de imagem não recarregava quando o PNG
       mudava no disco; um `context` com caminho de objeto perdia o que o usuário mudou dentro dele; o link do
       formato aponta para um repositório privado; o tocador de áudio sugerido era só o `paplay`; não dizia o que
       `open` faz com vários arquivos; o agente não sabia que as instruções mudaram. (resolvidas: item abaixo)
       Ficam para a [RFC 0004](docs/rfc/0004-midia-no-painel.md) de mídia: imagem dentro do painel e lado a lado, pixel art sem borrar e fundo
       quadriculado, animar spritesheets, tocar áudio no painel (duração, loop), saber o que foi visto ou ouvido,
       abrir no navegador.
    15. Reusar o nome de uma interface: o agente leu uma resposta velha que tinha esquecido de apagar como se fosse
       nova, e o documento recriado com a mesma `revision` já respondida veio com os botões travados. Opção: a IDE
       esquece o estado de uma interface quando o documento some, e as instruções pedem `revision` sempre maior.
       (resolvida: item abaixo)
    16. Do jogo, depois de M1: a aba de imagem nem sempre recarregava (um azul→vermelho ficou azul por 8 min); o
       observador de arquivos vê os três jeitos de regravar (no lugar, apagar e criar, renomear), e a causa não
       apareceu. E o `FORMAT.md` citava o JSON Schema por um link que não existe no projeto. (resolvidas: item
       abaixo)
    17. Do jogo, depois da RFC 0004 (instruções `f6058fda`): 8 lacunas resolvidas pela mídia nova; abertas: (1) um
       botão não pode esperar uma escolha (`gallery`, `select`) nem outro campo; (2) sem status entre o clique e a
       revisão; (3) ajuste por item vai num comentário só; (4) o `FORMAT.md` copiado tem links que não existem no
       projeto; (5) validar contra o schema pede ferramenta de fora; (6) as instruções mudam no meio de uma
       revisão; (7) uma IDE mais velha regravou instruções mais novas (sem a linha de versão); (8) `idetea:open`
       fora do `media`; (9) o agente não vê duração, pico e aviso de 0 dBFS; (10) "não completou" sem o porquê;
       (11) imagem sem clique não deixa rastro. (4, 8, 9 e 10: item abaixo; 1, 5 e 7: itens seguintes; 2 e 3 pedem
       RFC; 6 já é coberto por `ide.instructions`; 11 não tem como: o iced não diz o que está visível.)
    9. Os comandos que o agente sugere no painel (`cargo run -p tea-app -- .`) não são fáceis de copiar: o texto
       do painel não seleciona. Opção: botão de copiar nos blocos de código do `markdown`. (resolvida: item
       abaixo)
- [x] **Instruções para agentes só em inglês** (lacuna 6): um texto só (`agents/instructions.md`), qualquer que seja
  o idioma da IDE, com o pedido de conversar e escrever as interfaces no idioma do usuário, e de pôr no painel toda
  pergunta que espera resposta depois que o painel estiver em uso (lacuna 7). Saem os textos em português e
  espanhol; quem já tem o bloco num deles recebe o inglês ao rodar o comando de novo.
  - [x] Teste manual: nesta pasta, com a IDE nova, rodar "Interfaces: ensinar os agentes": o diálogo lista o
    `CLAUDE.md`, e o bloco ganha o parágrafo sobre idioma; rodar de novo avisa "já atualizadas". Trocar o idioma da
    IDE e rodar de novo: continua "já atualizadas". (aprovado, pelo painel)
- [x] **Markdown nas interfaces** (lacunas 2 e 4): componente `markdown` (`text`) desenhado pelo `markdown` do
  iced, com as cores do tema da IDE; imagens não são carregadas. Links: web (`http`, `https`, `mailto`, no
  navegador), arquivos do projeto (`src/main.rs:42:5`, na posição; sem `..`) e o config da IDE
  (`idetea:config`, `idetea:config-project`, o único jeito de apontar para fora do projeto); qualquer outro só
  avisa. O documento guarda só o texto; cada painel guarda o Markdown lido (as peças do iced não vão entre
  threads). Vale no painel Interfaces e nos painéis dos plugins. Formato, schema, exemplo e instruções dos agentes.
  - [x] Teste manual: a entrega deste item, no painel, já usa o `markdown`: os passos numerados aparecem
    formatados, com `código`; o link do config abre o config global; o link de arquivo abre na linha; o link da
    web abre o navegador; o link `../fora` só mostra o aviso. Rodar "Ensinar os agentes": o `CLAUDE.md` ganha as
    linhas sobre `markdown`. (aprovado, pelo painel)
- [x] **Comentário em toda resposta** (lacuna 3): a IDE põe um campo "comentário para o agente" logo antes dos
  botões do fim (ou no fim, se o documento não termina em botões), em toda interface com botões; o texto vai na
  resposta como `comment` (sem o campo quando vazio) e o campo esvazia depois de gravar. O documento tira com
  `"comment": false`; os painéis dos plugins não mostram. O que foi escrito sobrevive a reiniciar a IDE, como os
  outros campos. Formato, schema e instruções dos agentes (não criar um campo "observação" próprio).
  - [x] Teste manual: a entrega deste item não tem campo "Observação": escrever no campo de comentário e
    clicar num botão; o agente mostra o comentário recebido. Rodar "Ensinar os agentes": o `CLAUDE.md` ganha a
    linha sobre `comment`. (aprovado, pelo painel)
- [x] **Ensinar cria também o `CLAUDE.md`** (lacuna 1): num projeto sem `AGENTS.md` nem `CLAUDE.md`, o comando cria
  o `AGENTS.md` com o bloco e um `CLAUDE.md` só com `@AGENTS.md` (o texto fica num lugar só). Projetos que já têm um
  dos dois continuam como antes.
  - [x] Teste manual: numa pasta vazia, o botão do painel vazio lista `AGENTS.md (novo)` e `CLAUDE.md (novo)`; os
    dois aparecem, o `CLAUDE.md` com `@AGENTS.md`; rodar de novo avisa "já atualizadas". (aprovado, pelo painel)
- [x] **Copiar comandos do painel** (lacuna 9): os blocos de código do `markdown` ganham um botão "Copiar" no canto
  (o bloco é o do iced, com o botão por cima); o texto vai para a área de transferência, com o aviso "Copiado". Vale
  no painel Interfaces e nos plugins. As instruções dos agentes pedem os comandos em blocos de código.
  - [x] Teste manual: na entrega deste item, "Copiar" no bloco do comando e colar num terminal: vem o comando
    inteiro, sem o "```"; rodar "Ensinar os agentes": o `CLAUDE.md` ganha a frase sobre blocos de código.
    (aprovado, pelo painel)
- [x] **A IDE percebe que foi recompilada** (lacuna 8): na abertura guarda a data do próprio executável (no
  AppImage, a do `.AppImage`); uma thread olha a cada 3 s e, quando a data muda e para de mudar (o linker
  terminou), a IDE avisa uma vez e o título ganha "[versão nova: reinicie]". O comando `app.restart` ("Reiniciar a
  IDE") passa pelo mesmo diálogo de arquivos não salvos do "Sair", salva a sessão, roda o executável novo com os
  mesmos argumentos (solto, numa sessão própria, saída no log) e sai. Se não der para rodar, avisa e fica aberta.
  - [x] Teste manual: com a IDE aberta, `touch target/debug/idetea`: em alguns segundos vem o aviso e o título muda;
    "Reiniciar a IDE" com um arquivo alterado pergunta antes; a IDE volta com as mesmas abas e o título normal.
    (aprovado, pelo painel)
- [x] **Quem respondeu** ([RFC 0003](docs/rfc/0003-interfaces-pelo-uso.md), etapa A; lacunas 8, 11 e 12): a
  resposta ganha `ide` (`pid`, `version`, `outdated`); o topo do painel avisa quando a IDE está desatualizada. O
  app manda `CoreEvent::BinaryChanged` quando percebe o executável novo (lacuna 8). Formato e instruções dos
  agentes (não escrever avisos sobre qual IDE está aberta).
  - [x] Teste manual: responder a entrega deste item: o agente mostra o `ide` recebido, com o PID desta IDE e
    `outdated: false`; `touch target/debug/idetea`: o topo do painel mostra o aviso; responder de novo: chega
    `outdated: true`. Rodar "Ensinar os agentes": o `CLAUDE.md` ganha o parágrafo sobre `ide`. (aprovado, pelo
    painel)
- [x] **Passos que se marcam** (RFC 0003, etapa B; lacuna 10): componente `steps` (cada passo um `checkbox`
  numerado, texto em Markdown de uma linha, `optional`); o valor é a lista dos ids marcados, na ordem dos passos;
  `requires` no `button` deixa o botão desabilitado até os passos obrigatórios estarem marcados, com "Faltam N
  passos" embaixo (o estado também recusa o clique). Um `requires` que não aponta um `steps` vira aviso. Vale nos
  plugins. Formato, schema e instruções dos agentes.
  - [x] Teste manual: a entrega deste item usa `steps`: "Aprovado" fica desligado até marcar os passos
    obrigatórios (o opcional não conta), "Falhou" funciona sempre; a resposta traz a lista marcada. Rodar "Ensinar
    os agentes": o `CLAUDE.md` ganha o parágrafo sobre `steps`. (aprovado, pelo painel)
- [x] **Convenções de entrega nas instruções** (RFC 0003, etapa C): entrega de teste com os três botões, conferir
  no disco antes de commitar, não afirmar o estado da IDE, ler o `ide` da resposta. Nas instruções dos agentes e
  numa seção "Entregas de teste" do formato.
  - [x] Teste manual: rodar "Ensinar os agentes": o `CLAUDE.md` ganha o parágrafo "Handing me a manual test".
    (aprovado, pelo painel)
- [ ] **Segundo projeto de dogfooding** (RFC 0003, decisão 3): depois de A, B e C; decide imagem e mídia.
  Escolhido (pelo painel): "Correio de balão", jogo 2D original em JS com canvas, sem framework, em
  `~/NewProjects/tea-jogo`, feito por uma sessão nova do agente que só conhece o que "Ensinar os agentes" ensina.
  Ela anota as lacunas num `lacunas.md` de lá; as que forem da IDE entram aqui.
- [x] **O que o `file` faz com mídia** (lacuna 13): formato e instruções dos agentes dizem como cada tipo abre
  (imagem na aba com zoom, PDF, SVG como texto com prévia; áudio, vídeo e outros binários não abrem; imagem no
  `markdown` só mostra o texto alternativo) e como pedir revisão de áudio (o comando num bloco de código).
  - [x] Teste manual: "Ensinar os agentes" aqui e no `tea-jogo`: o `CLAUDE.md` daqui e o `AGENTS.md` do jogo
    ganham o parágrafo "Media". (aprovado, pelo painel)
- [x] **Lacunas do jogo, as pequenas** (lacuna 14):
  - A aba de imagem (e de PDF) recarrega quando o arquivo muda no disco (antes a checagem lia como texto, falhava e
    achava que nada mudou).
  - Um caminho de objeto no `context` traz o objeto com as mudanças do usuário nos campos de dentro (a resposta é
    montada sobre o `data` com tudo o que o usuário mudou por cima).
  - A IDE grava `.idetea/ui/FORMAT.md` com o formato da versão aberta; as instruções apontam para ele em vez do
    GitHub (o repositório é privado).
  - O bloco do `AGENTS.md` termina com "Instructions version" e as respostas trazem `ide.instructions`: diferentes,
    o agente pede para rodar "Ensinar os agentes".
  - Formato e instruções: tocadores de áudio por sistema, o que `open` faz com vários arquivos, caminho de objeto no
    `context`.
  - [x] Teste manual: no `tea-jogo`, com a IDE nova: `.idetea/ui/FORMAT.md` aparece; abrir um PNG, regerar pelo
    script e a aba mostra a imagem nova; "Ensinar os agentes" põe a linha "Instructions version"; o agente do jogo
    confere que a versão bate com a da resposta e manda uma revisão com caminho de objeto no `context`.
    (aprovado, pelo painel)
- [x] **Mídia M1: `image` e `gallery`** ([RFC 0004](docs/rfc/0004-midia-no-painel.md); lacunas 4, 5 e 12 do
  jogo): imagem dentro do painel (`pixelated`, `background: checker`, `height`); `gallery` lado a lado com `select`
  `none`/`one`/`many` (uma caixa de marcar embaixo de cada imagem). A thread que observa a pasta lê e decodifica as
  imagens (`tea_declarative_ui::Media`), com limites conferidos pelo cabeçalho antes de decodificar (8 MB, 4096 px,
  32 imagens, 64 Mpx), e relê quando um arquivo de imagem muda. Um clique abre a aba de imagem, que usa o vizinho
  mais próximo em imagens de até 256 px. Nos plugins, só o aviso por enquanto. Formato, schema e instruções.
  - [x] Teste manual: o agente do jogo revisa os sprites com `image` e `gallery`, sem gerar a `previa-sprites.png`;
    os sprites aparecem nítidos e com o quadriculado; escolher numa `gallery` volta na resposta; regerar um sprite
    pelo script atualiza o painel sozinho; um clique abre a aba, sem borrar no zoom. (aprovado, pelo painel)
- [x] **Mídia M2: `audio`** (lacunas 9 a 11): Tocar/Parar pelo tocador do sistema (`pw-play`, `paplay`, `aplay`;
  `afplay` no macOS), um de cada vez, num grupo de processos próprio que morre inteiro; `loop` (roda de novo se
  terminou bem); para numa revisão nova, quando o documento some e quando a IDE sai. WAV (PCM 8 a 32 bits, float)
  com leitor próprio: duração, taxa, canais, bits, pico em dBFS com aviso em 0 dBFS, forma de onda; relidos quando
  o arquivo muda. Extensões `wav`, `ogg`, `oga`, `opus`, `flac`, `mp3`. O `[ui] audio_player` fica para quando
  as extensões lerem o config.
  - [x] Teste manual (aprovado, pelo painel): no painel, tocar e parar três sons do jogo, um em `loop`; trocar de som para o anterior; o
    `batida.wav` mostra o aviso de 0 dBFS.
- [x] **Mídia M3: `sprite`** (lacuna 6): quadros de um spritesheet animados no painel, com pausa e quadro a quadro.
  Fila de `frames` quadros ou grade (`frame_width` e `frame_height`), `fps` de 1 a 60; quadros recortados na
  thread de leitura, contando nos limites. Relógio numa thread só enquanto a interface selecionada tem um sprite
  animando (o núcleo não avisa quando o painel fecha); uma revisão nova volta a animar.
  - [x] Teste manual (aprovado, pelo painel): o voo do pássaro a 8 fps como no jogo; pausar, passar quadro a quadro os 3 do balão;
    regerar a folha atualiza os quadros.
- [x] **Mídia M4: o que foi visto e ouvido** (lacuna 14): `media` na resposta, pelo caminho: `size` e `modified`
  do arquivo que estava na tela (lacuna 2 do jogo) e as contagens da revisão (`opened`, `played`, `completed`,
  `stepped`), só quando passam de zero.
  - [x] Teste manual (aprovado, pelo painel): abrir uma imagem na aba, tocar um som até o fim e parar outro, passar quadros de um sprite;
    a resposta traz cada contagem e a data do arquivo na tela.
- [x] **Mídia M5: `idetea:open`** (lacuna 13): link que abre um arquivo do projeto no app do sistema, só com as
  extensões da lista (`html`, `htm`, `pdf`, imagens, áudio, `mp4`, `webm`); symlink, pasta ou extensão fora da
  lista só dão aviso.
  - [x] Teste manual (aprovado, pelo painel): `idetea:open/` de um HTML abre no navegador e de um PDF no leitor; um `.desktop` e um `.sh`
    só avisam.
- [x] **Lacunas pequenas do jogo depois da mídia** (lacuna 17: 4, 8, 9 e 10): o `FORMAT.md` do projeto troca os
  links para o repositório da IDE por texto com o caminho (e `agentes.md` pelo `AGENTS.md` do projeto); o `media`
  ganha `seconds`, `peak_db` e `clips` de um WAV, `stopped` e `interrupted`, e o `opened` de um link
  `idetea:open/` (com a versão do disco na hora do clique).
  - [x] Teste manual (aprovado, pelo painel): num documento com um WAV estourado, tocar e parar, tocar por cima,
    abrir um link `idetea:open/`; a resposta traz cada coisa; o `FORMAT.md` sem links quebrados (conferido neste
    projeto; o do jogo é regravado quando a IDE abrir lá).
- [x] **`idetea ui check`** (lacuna 17, 5): confere documentos pela linha de comando com o mesmo leitor do painel:
  `ok`, `erro` (o que o painel recusaria), `aviso` e `mídia` (imagem, sprite ou som que não dá para ler); saída 1
  com erro. A IDE grava `.idetea/ui/check`, que chama o executável aberto (o do `PATH` pode ser velho e abriria uma
  janela). O temporário `.<nome>.json.tmp` vale pelo nome final.
  - [x] Teste manual (aprovado, pelo painel): `.idetea/ui/check` num documento bom, num quebrado e num com mídia
    faltando (rodado neste projeto; no jogo o script aparece quando a IDE abrir lá).
- [x] **`requires` em qualquer campo** (lacuna 17, 1): um caminho ou uma lista; de um `steps`, os passos
  obrigatórios; de outro campo (`gallery`, `select`, `input`, `checkbox`), um valor preenchido. O painel diz o que
  falta ("Falta preencher: …"). Caminho sem componente: aviso e ignorado.
  - [x] Teste manual (aprovado, pelo painel): um botão que espera uma escolha na `gallery`, um texto e uma
    `checkbox`; liga só com os três.
- [x] **Aba de imagem sempre em dia + schema junto do formato** (lacuna 16): as abas de imagem e PDF conferem
  tamanho e data do arquivo a cada segundo (além do observador) e recarregam quando mudam; o rodapé da aba mostra a
  hora em que o arquivo foi gravado, para conferir qual versão está na tela. A IDE grava também
  `.idetea/ui/idetea-ui.schema.json` ao lado do `FORMAT.md`. Teste do observador com os três jeitos de regravar.
  - [x] Teste manual (aprovado, pelo painel): com um PNG aberto na aba, regravar o arquivo várias vezes (mesmo tamanho, outra cor): a aba
    acompanha em até 1 s e o rodapé mostra a hora nova; `.idetea/ui/idetea-ui.schema.json` aparece no jogo.
- [x] **Reusar o nome de uma interface** (lacuna 15): a IDE guarda o hash do documento; recriado com o mesmo nome
  antes de ela ver o arquivo sumir, outro conteúdo com `revision` menor (ou na mesma, já respondida) é interface
  nova: campos do zero, botões ligados, aviso de pergunta nova; o `seq` das respostas continua subindo. As
  instruções pedem apagar as respostas que sobraram e usar `revision` maior que todas as já escritas no nome.
  - [x] Teste manual (aprovado, pelo painel): responder uma interface, apagar e recriar na mesma `revision` com outro texto num comando só:
    a nova aparece com os botões ligados, e a resposta sai com `seq` maior.
- Depois, só se o uso pedir: `Row`/`Column`, progresso, Markdown sem links, vários painéis (`PanelId` dinâmico no
  núcleo), adaptador A2UI v1.0, e a mesma crate nos painéis dos plugins WASM (Fase 4).

### Fase 4 — Plugins externos (melhoria futura)

Adiado: prévias de arquivos (Markdown, SVG, imagem, PDF) vêm antes.

Estudo, medições e plano: [RFC 0002](docs/rfc/0002-plugins-wasm.md). Protótipo na feature `plugins` (fora do padrão).

- [x] **Protótipo: host WASM (`wasmtime`) + WIT versionado** (`crates/tea-plugins`, `idetea:plugin@0.1.0`): plugin é
  um componente `wasm32-wasip2` com `plugin.toml` em `~/.local/share/idetea/plugins/<id>/`; vira uma extensão
  (comandos com o prefixo do id, eventos de arquivo salvo e arquivo ativo); vê o projeto em `/project` pelo WASI;
  prazo por chamada (2 s no `activate`, 500 ms nas outras) desliga um plugin preso sem travar a IDE.
  - [x] Teste manual: instalar `examples/plugins/contador` (README dele), abrir a IDE com `--features plugins`;
    os comandos aparecem na paleta, o painel conta cliques e arquivos salvos, e "Contador: travar" desliga o
    plugin em meio segundo com o motivo no painel, sem travar a IDE. (aprovado)
- [x] **Protocolo de UI declarativa para painéis de plugins:** o painel é um documento `idetea-ui` (RFC 0001) que o
  plugin manda com `set-panel`; clique volta em `on-action`. (Feito no protótipo; fecha junto com ele.) (aprovado)
- [x] RFC 0002 aprovada (2026-11-02, pelo painel Interfaces): a feature `plugins` fica fora do padrão até existir
  um plugin de verdade; a parte 2 começa por `host.run` e config própria.
- [x] **API 0.2: `host.run` + config do plugin.** `host.config()` devolve `[plugins.<id>]` do config (global +
  projeto) em JSON, e salvar o config manda `config-changed`. `host.run(id, command)` roda um programa sem
  esperar (thread da IDE) e o resultado chega em `run-finished`: 4 por plugin, prazo de 10 s (até 60 s), 1 MB de
  saída, `cwd` dentro do projeto, grupo de processos morto inteiro no prazo. O exemplo `contador` ganha o botão
  "Rodar" com o comando da config (padrão `git status --short`).
  - [x] Teste manual: instalar o contador novo (README dele) e abrir a IDE com `--features plugins` num
    repositório git: "Rodar" mostra o `git status --short`; pôr `[plugins.contador] comando = ["sh", "-c", "sleep
    1; ls"]` no config e salvar: o painel mostra o comando novo, e "Rodar" mostra "Rodando..." por 1 s sem travar
    a IDE; `comando = ["sleep", "30"]`: em 5 s aparece o erro de prazo. (aprovado, pelo painel)
- [ ] Partes seguintes: `/data` e mais eventos (quando um plugin pedir), instalar e gerenciar pelas
  Configurações, texto para autores, e decidir se a feature entra no padrão (com um plugin de verdade).
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
