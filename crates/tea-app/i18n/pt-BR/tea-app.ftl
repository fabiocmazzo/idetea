## Configurações → Aparência

settings-theme = Tema
settings-theme-detail = Cores da IDE, do editor e do terminal.
settings-font-size = Tamanho da fonte
settings-font-size-detail = Editor e terminal.
settings-code-font = Fonte do código
settings-code-font-detail = Editor, números de linha e terminal. A JetBrains Mono vem com a IDE; a lista tem as fontes monoespaçadas instaladas.
settings-font-weight = Peso da fonte
settings-font-weight-detail = Um peso que a fonte não tem usa o mais próximo.
settings-line-height = Altura da linha
settings-line-height-detail = Em múltiplos do tamanho da fonte. O terminal acompanha na mesma proporção.
settings-file-icons = Ícones de arquivo
settings-file-icons-detail = Um ícone por linguagem ou tipo de arquivo (o conjunto Seti, do VS Code). Desligado: o mesmo ícone para todos.
settings-activity-bar = Barra de ícones
settings-activity-bar-detail = Faixa na borda esquerda para mostrar/ocultar painéis com um clique.
settings-splash = Splash ao abrir
settings-splash-detail = Tela de abertura (mínimo de 2 s). --no-splash desliga numa execução.
settings-language = Idioma
settings-language-detail = Textos da interface. Automático segue o idioma do sistema. Francês, alemão, italiano, japonês e chinês simplificado trazem o mascote; o resto cai no inglês.
settings-language-auto = Automático ({ $language })
settings-language-restart = Vale ao reiniciar a IDE (agora: { $language }).

## Comandos do núcleo (título na paleta e na ajuda)
cmd-palette-open = Paleta de comandos
cmd-help-open = Ajuda: atalhos e guia
cmd-settings-open = Configurações
cmd-file-new = Arquivo: novo
cmd-file-save = Arquivo: salvar
cmd-file-open = Arquivo: abrir…
cmd-file-quick-open = Arquivo: abrir pelo nome…
cmd-debug-continue = Debug: iniciar / continuar
cmd-debug-stop = Debug: parar
cmd-debug-pause = Debug: pausar
cmd-debug-next = Debug: próxima linha
cmd-debug-step-in = Debug: entrar na função
cmd-debug-step-out = Debug: sair da função
cmd-debug-toggle-breakpoint = Debug: liga/desliga breakpoint na linha
cmd-debug-choose-program = Debug: escolher o programa…
cmd-file-open-folder = Arquivo: abrir pasta em nova janela…
cmd-file-save-as = Arquivo: salvar como…
cmd-file-save-as-path = Arquivo: salvar como (digitar caminho)…
cmd-file-close = Arquivo: fechar aba
cmd-tab-next = Abas: próxima
cmd-tab-prev = Abas: anterior
cmd-tab-reopen = Abas: reabrir a última fechada
cmd-editor-find = Editor: buscar
cmd-editor-replace = Editor: buscar e substituir
cmd-editor-goto-line = Editor: ir para linha
cmd-editor-undo = Editor: desfazer
cmd-editor-redo = Editor: refazer
cmd-editor-select-all = Editor: selecionar tudo
cmd-editor-fold-all = Editor: recolher todos os blocos
cmd-editor-unfold-all = Editor: expandir todos os blocos
cmd-editor-toggle-fold = Editor: recolher ou expandir o bloco do cursor
cmd-editor-toggle-vim = Editor: ligar/desligar modo vim
cmd-editor-zoom-in = Editor: aumentar zoom
cmd-editor-zoom-out = Editor: reduzir zoom
cmd-editor-zoom-reset = Editor: restaurar zoom
cmd-lsp-definition = LSP: ir para definição
cmd-lsp-references = LSP: listar onde é usado
cmd-lsp-implementation = LSP: ir para implementação
cmd-lsp-type-definition = LSP: ir para o tipo
cmd-lsp-incoming-calls = LSP: quem chama esta função
cmd-lsp-outgoing-calls = LSP: o que esta função chama
cmd-nav-file-symbol = Ir para símbolo no arquivo…
cmd-nav-project-symbol = Ir para símbolo no projeto…
cmd-nav-back = Navegação: voltar
cmd-nav-forward = Navegação: avançar
cmd-lsp-hover = LSP: mostrar documentação no cursor
cmd-lsp-format = LSP: formatar arquivo
cmd-lsp-restart = LSP: reiniciar servidores
cmd-theme-toggle = Tema: alternar claro/escuro
cmd-config-open = Config: abrir config global
cmd-config-open-project = Config: abrir config do projeto
cmd-config-reload = Config: recarregar
cmd-view-toggle-left = Layout: mostrar/ocultar lateral esquerda
cmd-view-toggle-right = Layout: mostrar/ocultar lateral direita
cmd-view-toggle-bottom = Layout: mostrar/ocultar painel inferior
cmd-app-restart = Reiniciar a IDE
cmd-app-quit = Sair
cmd-mascot-toggle = Mascote: mostrar/recolher no canto

## Avisos, diálogos e menus
file-new-created-on-save = Arquivo novo: { $path } (criado ao salvar)
file-open-error = Erro ao abrir { $path }: { $error }
file-changed-on-disk-kept = "{ $name }" mudou no disco, mas a aba tem alterações: ela foi mantida
file-reload-error = Erro ao recarregar { $path }: { $error }
file-changed-on-disk-overwrite = "{ $name }" mudou no disco, mas a aba tem alterações: salvar sobrescreve
debug-still-building = Debug: ainda compilando…
debug-already-running = Debug: o programa já está rodando
debug-building = Debug: compilando ({ $command })…
unknown-error = erro desconhecido
debug-build-failed = Debug: a compilação falhou: { $error }
debug-choose-title = Qual programa depurar?
debug-choose-message = O projeto gerou mais de um executável.
cancel = Cancelar
debug-running = Debug: rodando { $program }
debug-error = Debug: { $error }
debug-cancelled = Debug: cancelado (a compilação termina sozinha)
debug-stopped = Debug: parado
debug-no-source = Debug: { $name } não tem código-fonte
debug-paused-no-source = Debug: parado em { $name } ({ $reason }), sem código-fonte
debug-paused-at = Debug: parado em { $path }:{ $line } ({ $reason })
debug-exited = Debug: o programa terminou
debug-exited-code = Debug: o programa terminou (código { $code })
debug-command-failed = Debug: { $command } falhou: { $message }
debug-gdb-exited = Debug: o gdb encerrou ({ $reason })
debug-no-session = Debug: nenhuma sessão rodando (F5 inicia)
debug-already-paused = Debug: o programa já está parado
debug-pause-first = Debug: o programa está rodando (pause antes)
nav-nothing-forward = Nada para avançar
nav-nothing-back = Nada para voltar
lsp-no-references = Nenhuma referência encontrada
lsp-no-definition = Definição não encontrada
lsp-no-implementations = Nenhuma implementação encontrada
lsp-no-type = Tipo não encontrado
lsp-not-used-elsewhere = "{ $symbol }" não é usado em outro lugar
outline-save-first = Salve o arquivo para ver a estrutura
outline-reading = Lendo a estrutura…
symbols-type-a-name = Digite o nome de uma função, tipo, método…
searching = Buscando…
symbols-none-found = Nenhum símbolo encontrado (se o projeto ainda está carregando, tente de novo)
symbols-none-in-file = Nenhum símbolo neste arquivo
lsp-save-first = salve o arquivo para usar o servidor de linguagem
lsp-finding-references = Buscando onde é usado…
lsp-finding-implementations = Buscando implementações…
calls-not-in-function = O cursor não está numa função (ou o servidor não sabe as chamadas dela)
lsp-formatted = Arquivo formatado
lsp-level-error = erro:{" "}
lsp-level-warning = aviso:{" "}
lsp-server-stopped = { $server } parou — { $reason }. Use "LSP: reiniciar servidores".
lsp-server-crashed = { $server } caiu — { $reason }. Ele volta sozinho quando você parar de digitar.
lsp-server-crashed-too-often = { $server } caiu { $count } vezes em { $minutes } minutos — { $reason }. Use "LSP: reiniciar servidores".
lsp-no-formatting = Formatação indisponível para este arquivo
lsp-restarted = Servidores de linguagem reiniciados
save-changes-title = Salvar alterações?
save-changes-message = "{ $name }" tem alterações que não foram salvas.
save = Salvar
discard = Descartar
picker-already-open = Já há uma janela de arquivos aberta
save-tab-closed = A aba foi fechada antes de salvar
folder-opening = Abrindo { $path } numa nova janela
folder-open-error = Não foi possível abrir a pasta: { $error }
saved = Salvo: { $path }
save-error = Erro ao salvar: { $error }
file-deleted-kept = Arquivo apagado no disco; a aba com alterações foi mantida
and-more = … e mais { $count }
quit-unsaved-title = Sair com alterações não salvas?
quit-save-all = Salvar tudo e sair
quit-without-saving = Sair sem salvar
save-needs-name = "{ $name }" não tem nome: use "Salvar como"
save-file-error = Erro ao salvar { $path }: { $error }
close-tabs-title = 
    { $count ->
        [one] Fechar 1 aba?
       *[other] Fechar { $count } abas?
    }
close-tabs-unsaved = Com alterações não salvas:
save-and-close = Salvar e fechar
discard-changes = Descartar alterações
tab-close = Fechar
tab-close-others = Fechar outras
tab-close-right = Fechar à direita
tab-close-saved = Fechar as salvas
tab-close-all = Fechar todas
copy-path = Copiar caminho
copy-relative-path = Copiar caminho relativo
reveal-in-tree = Revelar na árvore
open-in-file-manager = Abrir no gerenciador de arquivos
copied = Copiado: { $text }
file-manager-error = Não foi possível abrir o gerenciador de arquivos: { $error }
save-then-close = Escolha onde salvar; depois feche a aba de novo
config-reloaded-problems = Config recarregado com problemas: { $problems }
config-reloaded = Config recarregado (ligar/desligar extensões só vale ao reiniciar)
config-no-home = Não foi possível descobrir a pasta de config (HOME não definido)
create-error = Erro ao criar { $path }: { $error }
config-error = Config: { $error }
write-error = Erro ao gravar { $path }: { $error }
unknown-command = Comando desconhecido: { $id }
keys-record-hint = Use Ctrl, Alt ou uma tecla especial (F1–F12, PageUp…). Esc cancela.
keys-already-bound = { $key } já era de "{ $command }"
keys-rebound = { $key } era de "{ $old }"; agora é de "{ $command }"
keys-bound = { $key } → "{ $command }"
theme-unknown = tema desconhecido "{ $name }", usando "dark"
default = padrão

## Configurações
settings-title = Configurações
settings-tab-appearance = Aparência
settings-tab-extensions = Extensões
settings-tab-shortcuts = Atalhos
close-esc = Fechar (Esc)
settings-no-home = (HOME não definido)
settings-write-to = Gravar em
settings-target-global = Global
settings-target-project = Projeto
settings-open-file = Abrir arquivo
keys-filter = Filtrar por nome, comando ou tecla…
keys-press = Pressione as teclas… (Esc cancela)
keys-add = Adicionar atalho
keys-help = Clique numa tecla para removê-la. + grava uma tecla nova (vale também para atalhos já usados: eles mudam de dono).
settings-lsp = Servidores de linguagem
settings-lsp-detail = Liga ou desliga o LSP inteiro.
settings-servers = Servidores
settings-server-found = encontrado no PATH
settings-server-not-found = não encontrado no PATH
settings-lsp-note = Mudanças valem para servidores iniciados daqui em diante.
settings-restart-servers = Reiniciar servidores
theme-dark = Escuro (padrão)
theme-light = Claro
slot-left = Esquerda
slot-right = Direita
slot-bottom = Embaixo
slot-status = Barra de status
settings-on-restart = vale ao reiniciar a IDE
default-capitalized = Padrão
settings-extensions-note = Ligar ou desligar uma extensão vale ao reiniciar. Mudar o lado de um painel vale na hora.
ext-filetree = Árvore de arquivos
ext-statusbar = Barra de status
ext-search = Busca no projeto
ext-problems = Erros e avisos
ext-terminal = Terminal integrado
ext-git = Mudanças, stage, commit e diff
ext-debug = Botões, pilha, variáveis e saída do debug
ext-references = Onde um nome é usado (Shift+F12)
ext-outline = Funções e métodos do arquivo, com visibilidade (Alt+7)
ext-preview = Prévia de Markdown e SVG ao lado do editor (Ctrl+Shift+V)
ext-calls = Quem chama uma função, e quem ela chama (Alt+Shift+H)
ext-interfaces = Interfaces que um agente escreve em .idetea/ui/ (experimental)

## Paleta, diálogos, LSP, debug e o resto do app
palette-commands = Digite um comando…
palette-save-as = Caminho do arquivo (relativo à raiz do projeto)
palette-files = Nome do arquivo (arquivo:linha vai para a linha, > para comandos)
palette-file-symbols = Função, tipo, método… deste arquivo
palette-project-symbols = Função, tipo, método… do projeto
palette-no-commands = Nenhum comando encontrado
palette-save-hint = Enter salva · Esc cancela
palette-reading-files = Lendo os arquivos do projeto…
palette-no-files = Nenhum arquivo encontrado
palette-no-symbols = Nenhum símbolo
palette-no-matching-symbols = Nenhum símbolo combina com a busca
dialog-hint = Enter confirma · Esc cancela
lsp-command-not-found = { $server }: comando "{ $command }" não encontrado no PATH (instale ou ajuste [lsp.servers.{ $server }] no config)
lsp-start-error = { $server }: não foi possível iniciar: { $error }
lsp-disabled = LSP desligado no config ([lsp] enabled = false)
lsp-no-server-for = nenhum servidor de linguagem configurado para arquivos .{ $ext }
lsp-not-running = { $server } não está rodando: { $reason }
lsp-not-ready = { $server } ainda não está pronto
lsp-hint-rustup-install = { $reason }. Instale com: rustup component add rust-analyzer
lsp-hint-rustup = { $reason }. Com rustup: rustup component add rust-analyzer
lsp-hint-ziex = { $reason }. Instale o CLI do Ziex (https://ziex.dev/learn) ou abra a pasta do projeto, onde o build.zig.zon depende do ziex: aí a IDE compila o servidor com o zig
lsp-hint-moonbit = { $reason }. Instale o toolchain do MoonBit (https://www.moonbitlang.com/download), que já traz o `moon lsp`; a IDE também procura em ~/.moon/bin (ou $MOON_HOME/bin)
lsp-hint-hare = { $reason }. O hare-lsp não vem com o Hare: compile-o de https://git.sr.ht/~whynothugo/hare-lsp e coloque no PATH (por exemplo, ~/.local/bin)
config-syntax-error = sintaxe { $file }: { $error }
visibility-file-only = só neste arquivo
picker-save-as = Salvar como
picker-open-files = Abrir arquivos
picker-open-folder = Abrir pasta
picker-debug-program = Programa para depurar
splash-ready = pronto
splash-loading = carregando…
watch-too-many = pasta grande demais: só as primeiras { $count } pastas são observadas
watch-partial = não foi possível observar todas as pastas ({ $error })
watch-problem = Mudanças no disco: { $message }. Use "Atualizar" na árvore quando precisar.
watch-failed = Mudanças no disco: não foi possível observar ({ $error })
debug-no-gdb = gdb não encontrado no PATH (o debug usa o gdb 14 ou mais novo)
debug-program-not-found = programa não encontrado: { $path }
debug-gdb-start-error = não foi possível iniciar o gdb: { $error }
debug-reason-breakpoint = breakpoint
debug-reason-step = passo
debug-reason-pause = pausado
debug-reason-exception = exceção
debug-reason-entry = início
debug-reason-goto = salto
build-run-error = não foi possível rodar "{ $command }": { $error }
build-no-message = falhou sem mensagem

## Ajuda e linha de comando
help-key-find = Buscar no arquivo
help-key-replace = Buscar e substituir
help-key-goto-line = Ir para linha
help-key-undo = Desfazer / refazer
help-key-next-match = Selecionar a próxima ocorrência (multi-cursor)
help-key-quick-fix = Ações rápidas do LSP (quando houver)
help-key-ctrl-click = Ctrl+Clique
help-key-definition = Ir para definição (LSP)
help-key-tab = Inserir tabulação
help-title = Ajuda
help-tab-shortcuts = Atalhos
help-tab-guide = Guia
help-tab-about = Sobre
help-close = Fechar (Esc)
help-filter = Filtrar por nome, comando ou tecla…
help-group-general = Geral
help-editor-keys = Editor (fixos)
help-in-palette = (paleta)
about-no-home = (HOME não definido)
about-version = Versão
about-license = Licença
about-license-value = { $license } (livre e gratuita, sempre)
about-project = Projeto
about-global-config = Config global
about-project-config = Config do projeto
about-built-with = Feita em Rust com iced, iced-code-editor, alacritty_terminal (via iced_term) e syntect.
about-icons = Ícones: Codicons © Microsoft, licença CC BY 4.0.
cli-usage = 
    Uso: idetea [opções] [caminho...]
    
      caminho        arquivo para abrir (pode não existir: é criado ao salvar)
                     ou pasta para usar como raiz do projeto
      --no-splash    não mostra a splash screen (também: IDETEA_NO_SPLASH=1)
      --menu         aberta pelo menu: sem pasta, abre o projeto do arquivo ou o último projeto
      -f, --foreground  fica preso ao terminal (sem isso, aberta de um terminal, a IDE
                     solta dele e o log vai para ~/.local/state/idetea/idetea.log)
      --install-desktop    põe a IDE no menu de aplicativos (com o ícone)
      --uninstall-desktop  tira do menu de aplicativos
      ui check <arquivo.json>…  confere documentos do painel Interfaces (como o painel lê)
      new <pasta> [--modelo id]  cria um projeto novo: pasta, git, skills do modelo e instruções
      skills [list|add|update|remove]  instala skills de agentes no projeto (idetea skills --help)
      ui fix [--apply]  mostra (e, com --apply, conserta) o que ficou para trás em .idetea/ui/
      ask …          pergunta no painel Interfaces e espera a resposta (idetea ask sem nada: o uso)
      run … -- <comando>  pede um terminal à IDE e espera o fim (idetea run sem nada: o uso)
      status [--json]  o que a IDE aberta neste projeto mostra (abas, mudanças, cursor)
      open <arquivo[:linha[:coluna]]>  a IDE aberta mostra o arquivo, com o cursor no lugar
      history [--json]  o que você respondeu no painel Interfaces (--name, --since, --last N, --all)
      mascot <estado> [--text "…"]  atualiza o mascote da IDE (idle, working, thinking, attention, celebrating, error, sleeping, clear, status)
      -h, --help     mostra esta ajuda
      -V, --version  mostra a versão
cli-unknown-option = opção desconhecida ignorada: { $option }
cli-extra-root = só uma pasta raiz por vez, ignorando { $path }

## Ajuda → Guia (`{ $comando-com-hifens }` vira o atalho do comando)
guide-start = Primeiros passos
guide-start-1 = Abra a IDE com `idetea` numa pasta (ela vira a raiz da árvore) ou com `idetea arquivo...` (os arquivos abrem em abas; a raiz continua sendo a pasta atual).
guide-start-2 = Todos os comandos estão na paleta: { $palette-open }. Digite parte do nome; o atalho aparece ao lado.
guide-start-3 = Esta ajuda abre com { $help-open }.
guide-start-4 = A barra de ícones na borda esquerda mostra e oculta os painéis com um clique (o aberto fica destacado). Arraste a borda entre um painel e o editor para mudar o tamanho; duplo clique volta ao padrão.
guide-files = Arquivos e abas
guide-files-1 = Abrir pelo nome: { $file-quick_open } (digite partes do nome; `main.rs:42` vai para a linha; `>` troca para os comandos).
guide-files-2 = Ir para um símbolo (função, tipo, método…): { $nav-file_symbol } no arquivo, { $nav-project_symbol } no projeto inteiro. Precisa do servidor de linguagem.
guide-files-3 = Novo: { $file-new } · Abrir: { $file-open } · Salvar: { $file-save } · Salvar como: { $file-save_as } · Fechar aba: { $file-close }.
guide-files-4 = Abrir e Salvar como usam a janela de arquivos do sistema. Para digitar o caminho: { $file-save_as_path }. Abrir outra pasta ({ $file-open_folder }) abre uma nova janela da IDE.
guide-files-5 = Prévia de Markdown e SVG: { $preview-toggle } (ou o botão no fim da barra de abas) abre um painel ao lado que acompanha o arquivo ativo e se atualiza enquanto você digita. Links da web abrem no navegador; links para arquivos abrem numa aba.
guide-files-6 = Imagens (png, jpg, gif, webp...) abrem numa aba própria: a roda do mouse aproxima e arrastar move.
guide-files-7 = PDF também abre numa aba: role pelas páginas; − e + mudam o zoom e "Ajustar" acompanha a largura.
guide-files-8 = Trocar de aba: { $tab-next } e { $tab-prev } (Ctrl+Tab insere tabulação no editor).
guide-files-9 = Fechar uma aba ou sair com alterações pergunta antes: Salvar, Descartar ou Cancelar.
guide-files-10 = Arquivo mudado por fora (terminal, outro editor): a aba recarrega sozinha se não tiver alterações; se tiver, fica como está e aparece um aviso.
guide-files-11 = Botão direito numa aba: fechar outras, à direita, salvas ou todas, copiar caminho, revelar na árvore e abrir no gerenciador de arquivos. Clique do meio fecha a aba.
guide-tree = Árvore de arquivos
guide-tree-1 = Mostrar/ocultar: { $filetree-toggle } · Revelar o arquivo atual: { $filetree-reveal }.
guide-tree-2 = Botão direito num item: novo arquivo, nova pasta, renomear, apagar, copiar caminho e abrir no gerenciador de arquivos. Apagar manda para a lixeira do sistema.
guide-tree-3 = Cores do git: amarelo = modificado, verde = novo, vermelho = conflito, apagado = ignorado. Uma pasta com mudanças dentro mostra um ponto.
guide-bottom = Painel inferior
guide-bottom-1 = Busca no projeto ({ $search-focus }), Problemas ({ $problems-focus }), Terminal ({ $terminal-focus }) e Git ({ $git-focus }) ficam em abas no painel de baixo. { $view-toggle_bottom } mostra/esconde o painel.
guide-bottom-2 = Próximo problema: { $problems-next }.
guide-terminal = Terminal
guide-terminal-1 = Com o foco no terminal, Ctrl+letra, F-teclas e setas vão para o shell (Ctrl+C, Ctrl+W...). Os atalhos da IDE com Ctrl+Shift continuam valendo, então a paleta sempre abre.
guide-terminal-2 = Copiar e colar no terminal: Ctrl+Shift+C e Ctrl+Shift+V. Digitar `exit` fecha o painel.
guide-debug = Debug
guide-debug-1 = Breakpoint: clique no número da linha ou { $debug-toggle_breakpoint }. Iniciar/continuar: { $debug-continue }. Em projetos Rust compila com cargo build e acha o executável sozinho; nos outros, use [debug] no config (program, args, cwd, build) ou escolha o executável. Trocar o programa: { $debug-choose_program }.
guide-debug-2 = Parado: { $debug-next } próxima linha, { $debug-step_in } entra na função, { $debug-step_out } sai, { $debug-pause } pausa, { $debug-stop } para. A seta amarela mostra onde o programa está. Usa o gdb 14+.
guide-debug-3 = Painel Debug ({ $debug-focus }): botões, pilha de chamadas (clique vai para o quadro e mostra as variáveis dele), variáveis (clique abre structs e vetores) e a saída do programa e da compilação.
guide-git = Git
guide-git-1 = A branch aparece na barra de status. O painel Git ({ $git-focus }) mostra o que mudou: + e − colocam e tiram do stage, clicar num arquivo mostra o diff, Enter na mensagem faz o commit.
guide-git-2 = Botão direito num arquivo do Git: abrir, ver diff, stage, descartar alterações (pede confirmação) e copiar caminho.
guide-git-3 = Fetch ({ $git-fetch }), pull ({ $git-pull }) e push ({ $git-push }) ficam nos botões do topo do painel. A senha não é pedida aqui: use o agente ssh (ssh-add) ou um credential helper. Uma branch nova pergunta antes de ser publicada no remoto.
guide-git-4 = O painel se atualiza ao salvar e quando o repositório muda (commit ou checkout feitos no terminal).
guide-lsp = Linguagens (LSP)
guide-lsp-1 = Servidores de linguagem iniciam sozinhos ao abrir um arquivo: rust-analyzer, clangd, pyright, typescript-language-server, gopls e lua-language-server (precisam estar instalados).
guide-lsp-2 = Ir para definição: { $lsp-definition } ou Ctrl+Clique · Documentação: { $lsp-hover } ou mouse parado sobre o código · Formatar: { $lsp-format }.
guide-lsp-3 = Onde é usado: { $lsp-references }, ou Ctrl+Clique na própria declaração. A lista aparece no painel Referências; clique leva ao uso.
guide-lsp-4 = Implementações: { $lsp-implementation } (num trait ou interface, ou num método dele). Tipo do que está no cursor: "LSP: ir para o tipo" na paleta. Com mais de um resultado (inclusive na definição), a lista vai para o painel Referências; com um só, salta direto.
guide-lsp-5 = Quem chama uma função: { $lsp-incoming_calls } com o cursor nela. O painel Chamadas mostra a árvore (abra um nível para ver quem chama quem chama); "O que chama" troca para as chamadas feitas por ela. Clique leva até a chamada.
guide-lsp-6 = Voltar e avançar depois de um salto (definição, referência, abrir pelo nome, cliques nos painéis): { $nav-back } e { $nav-forward }, ou os botões laterais do mouse.
guide-lsp-7 = Estrutura do arquivo ({ $outline-toggle }): funções, structs, métodos e campos em árvore. O cadeado aberto é público, o fechado é privado e a chave é visível só em parte (pub(crate), protected, pacote). O símbolo onde o cursor está fica destacado; clique leva até ele.
guide-lsp-8 = Se um servidor falhar, a barra de status mostra o motivo; "LSP: reiniciar servidores" tenta de novo.
guide-config = Configuração
guide-config-1 = A tela de configurações ({ $settings-open }) muda tema, fonte, extensões, atalhos e LSP, gravando no config global ou no do projeto. As mudanças valem na hora.
guide-config-2 = Há um config global e um por projeto (`.idetea/config.toml`); o do projeto vence. Para editar o arquivo direto: "Config: abrir config global" ou "...do projeto". Ao salvar, tudo recarrega.
guide-config-3 = Atalhos ficam em [keys]: "ctrl+alt+t" = "terminal.focus". Texto vazio desliga um atalho padrão. Os ids dos comandos aparecem na aba Atalhos.
guide-config-4 = Idioma: Configurações → Aparência → Idioma (Automático segue o sistema). Vale ao reiniciar a IDE.

## Sessão
settings-restore-session = Reabrir como estava
settings-restore-session-detail = Ao abrir um projeto, voltam as abas, o cursor e os painéis. idetea arquivo... abre só o arquivo.

## Recentes
cmd-file-open-recent = Arquivo: abrir recente…
palette-recent = Projeto ou arquivo aberto antes
palette-no-recent = Nada aberto antes ainda
guide-files-12 = Abrir recente: { $file-open_recent } lista os projetos (abrem numa nova janela) e os arquivos abertos por último, de todas as janelas.

## Recuperar não salvos
untitled-tab = sem título
recover-title = 
    { $count ->
        [one] A IDE fechou sem salvar 1 arquivo
       *[other] A IDE fechou sem salvar { $count } arquivos
    }
recover-detail = Recuperar abre as alterações nas abas, ainda sem salvar (desfazer volta ao que está no disco). Depois pergunta de novo na próxima vez.
recover = Recuperar
recover-later = Depois
recovered = 
    { $count ->
        [one] 1 arquivo recuperado (ainda sem salvar)
       *[other] { $count } arquivos recuperados (ainda sem salvar)
    }

## Renomear símbolo e salvar todos
cmd-lsp-rename = LSP: renomear símbolo…
cmd-file-save-all = Arquivo: salvar todos
save-all-untitled = Abas sem nome não foram salvas (use Salvar como)
palette-rename = Novo nome
palette-rename-hint = Renomear "{ $name }" em todo o projeto · Enter mostra o que muda · Esc cancela
rename-no-word = Coloque o cursor num nome para renomear
rename-asking = Renomear "{ $name }": perguntando ao servidor…
rename-nothing = Nada para renomear em "{ $name }"
rename-failed = Não deu para renomear: { $message }
rename-title = Renomear "{ $old }" para "{ $new }"?
rename-summary = 
    { $changes ->
        [one] 1 mudança
       *[other] { $changes } mudanças
    } em { $files ->
        [one] 1 arquivo:
       *[other] { $files } arquivos:
    }
rename-detail = Os arquivos ficam alterados nas abas, ainda sem salvar: Ctrl+Z desfaz em cada um, e "Arquivo: salvar todos" grava tudo.
rename = Renomear
renamed = 
    { $count ->
        [one] Renomeado em 1 arquivo (ainda sem salvar)
       *[other] Renomeado em { $count } arquivos (ainda sem salvar)
    }

guide-lsp-9 = Renomear: { $lsp-rename } com o cursor no nome. Uma prévia mostra quantas mudanças e em quais arquivos; confirmando, eles ficam alterados nas abas (Ctrl+Z desfaz em cada um) e { $file-save_all } grava todos.

## Substituir no projeto
replaced = { $count } troca(s) em { $files } arquivo(s)
replaced-failed = { $count } troca(s) em { $files } arquivo(s); { $failed } não deu para mudar (sem permissão ou não é texto)

guide-bottom-3 = Substituir no projeto: { $search-replace } (ou ▸ ao lado da busca) abre "Substituir por". Cada resultado mostra como fica; "Substituir" troca a linha ou o arquivo, e "Substituir tudo" pede confirmação. Arquivos abertos em abas ficam alterados sem salvar (Ctrl+Z desfaz); os outros são gravados direto no disco.

## Editor e salvar
format-save-timeout = O servidor não formatou a tempo: salvo sem formatar

settings-tab-editor = Editor
settings-wrap = Quebrar linhas longas
settings-wrap-detail = Linha maior que a janela continua embaixo, sem rolar para o lado.
settings-whitespace = Mostrar espaços
settings-whitespace-detail = Espaços aparecem como · e tabs como →.
settings-indent-guides = Guias de recuo
settings-indent-guides-detail = Linhas verticais finas em cada nível de recuo.
settings-tab-size = Tamanho do recuo
settings-tab-size-detail = Quantos espaços o Tab insere.
settings-insert-spaces = Recuar com espaços
settings-insert-spaces-detail = Desligado: Tab insere o caractere tab. Arquivo que já recua com tabs continua com tabs.
settings-trim = Tirar espaços no fim das linhas ao salvar
settings-trim-detail = Menos em Markdown, onde dois espaços no fim quebram a linha.
settings-final-newline = Quebra de linha no fim do arquivo ao salvar
settings-final-newline-detail = Garante que a última linha termina com quebra de linha.
settings-format-on-save = Formatar ao salvar
settings-format-on-save-detail = Ctrl+S formata com o servidor de linguagem antes de gravar (espera até 2 s). Salvar todos não formata.
settings-breadcrumbs = Breadcrumbs
settings-breadcrumbs-detail = Acima do editor: o caminho do arquivo e o símbolo onde está o cursor. Clique numa pasta para mostrar na árvore, num símbolo para ir até ele.
settings-editorconfig-note = Um .editorconfig no projeto vale mais que estas opções (recuo, tabs ou espaços, limpar ao salvar).

guide-config-5 = Editor: na aba Editor das configurações ({ $settings-open }) ficam quebra de linha, espaços visíveis, guias de recuo, tamanho do recuo e o que fazer ao salvar (tirar espaços no fim, quebra de linha final, formatar). Um .editorconfig no projeto vale mais; sem ele, arquivo que já recua com tabs continua com tabs.

guide-lsp-10 = Dica de parâmetros: ao digitar ( numa chamada, a assinatura da função aparece acima do cursor, com o parâmetro atual destacado; muda ao digitar a vírgula e some ao fechar o parêntese ou com Esc.

guide-git-5 = Marcas na margem: à esquerda dos números de linha, verde = linha nova, azul = mudada, tracinho vermelho = linhas apagadas, comparando com o último commit (acompanha a digitação, sem precisar salvar). Clicar na marca mostra como era, com "Desfazer mudança" (Ctrl+Z volta).

palette-no-items = Nada combina com a busca

guide-git-6 = Branches: { $git-checkout } (ou clique no nome da branch, na barra de status ou no painel Git) lista as branches, da mais recente; as remotas que ainda não existem aqui vêm depois e, escolhidas, viram locais. Digite um nome novo para criar a branch a partir de onde você está. Alterações não commitadas vão junto; se colidirem com a outra branch, o git recusa e explica.

guide-git-7 = Autor da linha: na barra de status, ao lado da branch, aparece quem mudou a linha do cursor por último, quando e a mensagem do commit (clique para ver o hash e copiar). Com a aba alterada e sem salvar, some até salvar (as linhas do disco podem não bater).

## Tarefas
cmd-tasks-run = Tarefas: rodar…
cmd-tasks-rerun = Tarefas: rodar a última de novo
cmd-tasks-test-here = Rust: rodar o teste no cursor
tasks-placeholder = Rodar tarefa…
tasks-none = Nenhuma tarefa: crie uma em [tasks] no config, ou abra um projeto com Cargo.toml, package.json ou Makefile.
tasks-no-terminal = Tarefas precisam da extensão do terminal, que está desligada.
tasks-no-test = O cursor não está num teste Rust (#[test] fn).
tasks-no-package = Não achei o Cargo.toml do pacote deste arquivo.
tasks-done = [{ $name } terminou: código { $code } · Enter fecha]

guide-terminal-3 = Tarefas: { $tasks-run } lista os comandos do projeto (os de [tasks] no config, cargo build/run/test/clippy, scripts do package.json, alvos do Makefile) e roda o escolhido numa aba "Tarefa" do terminal, sem mexer no shell. No fim aparece o código de saída; Enter fecha. Ctrl+C para só o comando. { $tasks-rerun } roda a última de novo.
guide-terminal-4 = Teste no cursor (Rust): com o cursor dentro de um #[test] fn, { $tasks-test_here } roda só esse teste (cargo test -p pacote nome). Sem atalho padrão: crie um em [keys] se usar muito.

## Editor dividido
cmd-editor-split = Editor: mandar a aba para o outro lado (dividir)
cmd-editor-focus-left = Editor: ir para o lado esquerdo
cmd-editor-focus-right = Editor: ir para o lado direito
split-one-tab = Abra outro arquivo para dividir o editor: cada lado mostra uma aba diferente.
tab-move-right = Mandar para a direita (dividir)
tab-move-left = Mandar para a esquerda
guide-files-13 = Editor dividido: { $editor-split } (ou "Mandar para a direita" no menu da aba) põe a aba ao lado, com as suas próprias abas; abrir um arquivo vai para o lado com foco. Clique num lado (ou { $editor-focus_left } / { $editor-focus_right }) para trabalhar nele; arraste a divisa para mudar a largura (duplo clique volta ao meio). Mandar a última aba de um lado de volta junta tudo.

## Menu de aplicativos (--install-desktop)
desktop-no-home = Não achei a pasta pessoal (HOME).
desktop-error = Não foi possível instalar o atalho: { $error }
desktop-installed = 
    IDE TEA no menu de aplicativos: { $desktop }
    Abre: { $exe }
desktop-removed = IDE TEA saiu do menu de aplicativos.

## Arquivos grandes
file-light-mode = "{ $name }" tem uma linha muito longa e abriu no modo leve: sem cores, sem servidor de linguagem e com quebra de linha
lsp-light-mode = arquivo no modo leve (linha muito longa): sem servidor de linguagem

## Executável recompilado
binary-changed = A IDE foi recompilada: rode "Reiniciar a IDE" (paleta) para usar a versão nova.
binary-changed-title = [versão nova: reinicie]
binary-restart-error = Não deu para reiniciar: { $error }

## idetea status / idetea open (RFC 0006)
status-usage =
    Uso: idetea status [--json]
    
    O que a IDE aberta neste projeto mostra: a aba ativa com o cursor e os arquivos abertos
    (com "não salvo" nos que têm mudança). --json: o mesmo em JSON, com "open".
    Códigos: 0 aberta, 2 argumento inválido, 3 a IDE não está aberta neste projeto.
status-bad-arg = argumento desconhecido: { $arg }
status-not-open = a IDE não está aberta neste projeto ({ $root })
status-active = ativo: { $path }
status-no-active = nenhum arquivo ativo
status-modified = {" "}(não salvo)
open-usage =
    Uso: idetea open <arquivo[:linha[:coluna]]>
    
    A IDE aberta neste projeto mostra o arquivo, com o cursor na linha e na coluna.
    Códigos: 0 aberto, 2 caminho inválido ou fora do projeto, 3 a IDE não está aberta (ou não pegou o pedido).
open-outside = { $path } não fica dentro do projeto ({ $root })
open-write-error = não deu para gravar { $path }
open-not-taken = a IDE não pegou o pedido em 3 segundos

## Mascote reativo (RFC 0012)
mascot-state-idle = Calmo
mascot-state-working = Trabalhando
mascot-state-thinking = Pensando
mascot-state-attention = Atenção
mascot-state-celebrating = Pronto!
mascot-state-error = Erro
mascot-state-sleeping = Dormindo
mascot-state-curious = Curioso
mascot-state-annoyed = Irritado
mascot-idle-1 = Só código. Pode pôr gelo.
mascot-idle-2 = O vapor subiu. Estou aqui.
mascot-idle-3 = Xícara quieta, olho atento.
mascot-idle-4 = As folhas podem assentar.
mascot-idle-5 = Sem pressa. O chá está quente.
mascot-idle-6 = Bebe quando quiser.
mascot-idle-7 = A xícara está do teu lado.
mascot-idle-8 = Tudo calmo no pires.
mascot-hover-curious-1 = Hmm? Precisa de alguma coisa?
mascot-hover-curious-2 = Opa. Tudo bem por aí?
mascot-hover-curious-3 = Senti isso. O que foi?
mascot-hover-curious-4 = Passando por cima do chá?
mascot-hover-curious-5 = Um oi, ou uma mão?
mascot-hover-curious-6 = A xícara te viu.
mascot-hover-curious-7 = Curioso. Pode falar.
mascot-hover-curious-8 = Bem-vindo. Qual é o plano?
mascot-hover-annoyed-1 = Devagar — você está girando o chá.
mascot-hover-annoyed-2 = Muito mouse. A xícara ficou tonta.
mascot-hover-annoyed-3 = Mão firme. Chá quente.
mascot-hover-annoyed-4 = Uma passada é um oi. Isso foi um vendaval.
mascot-hover-annoyed-5 = O pires está tremendo.
mascot-hover-annoyed-6 = Tá, eu te vi. Menos redemoinho.
mascot-poke-curious-1 = Hmm? O que foi?
mascot-poke-curious-2 = Isso faz cócegas na xícara.
mascot-poke-curious-3 = Chamou?
mascot-poke-curious-4 = Um toque. Estou ouvindo.
mascot-poke-curious-5 = Cuidado, está cheia.
mascot-poke-curious-6 = Oi, daqui do pires.
mascot-poke-double-1 = Dois toques. O dobro de vapor.
mascot-poke-double-2 = Clique duplo anotado. Continua sendo chá.
mascot-poke-double-3 = Duas vezes? Tá, você me tem.
mascot-poke-double-4 = Duas batidas na xícara.
mascot-poke-double-5 = Dois já é conversa.
mascot-poke-double-6 = Toque duplo. Não derramou.
mascot-poke-annoyed-1 = Cuidado para não derramar.
mascot-poke-annoyed-2 = O chá já está quente.
mascot-poke-annoyed-3 = Chega de toque para uma xícara só.
mascot-poke-annoyed-4 = Eu gosto de você. O pires, nem tanto.
mascot-poke-annoyed-5 = Devagar. A xícara é pequena.
mascot-poke-annoyed-6 = Entendi. Vou esfriar.
mascot-task-1 = Indo: { $name }
mascot-task-2 = Preparando { $name }…
mascot-task-3 = A tarefa { $name } está rodando.
mascot-task-4 = { $name }. Eu olho a chaleira.
mascot-task-5 = Começando { $name }.
mascot-task-6 = { $name } — deixa em infusão.
mascot-term-pending-1 = O terminal quer um sim: { $title }
mascot-term-pending-2 = Um comando está esperando: { $title }
mascot-term-pending-3 = Você decide: { $title }
mascot-term-pending-4 = Licença, por favor: { $title }
mascot-term-pending-5 = Segurando a chaleira: { $title }
mascot-term-pending-6 = { $title } — posso rodar?
mascot-term-running-1 = Rodando: { $title }
mascot-term-running-2 = { $title } está no fogo.
mascot-term-running-3 = Em andamento: { $title }
mascot-term-running-4 = { $title } — ainda em infusão.
mascot-term-running-5 = Trabalhando em { $title }.
mascot-term-running-6 = { $title } está a caminho.
mascot-term-ok-1 = Pronto: { $title }
mascot-term-ok-2 = { $title } terminou limpo.
mascot-term-ok-3 = Tudo certo — { $title }.
mascot-term-ok-4 = { $title } saiu bem.
mascot-term-ok-5 = Saída limpa: { $title }.
mascot-term-ok-6 = { $title }. A xícara comemora.
mascot-term-err-1 = { $title } falhou ({ $code }).
mascot-term-err-2 = Saída { $code }: { $title }
mascot-term-err-3 = { $title } derramou (código { $code }).
mascot-term-err-4 = Esse quebrou: { $title } ({ $code }).
mascot-term-err-5 = { $title } não terminou ({ $code }).
mascot-term-err-6 = Código { $code } em { $title }.
mascot-answered-1 = Entendi: { $title }
mascot-answered-2 = Respondido: { $title }
mascot-answered-3 = { $title } — valeu.
mascot-answered-4 = Anotado: { $title }
mascot-answered-5 = { $title } ficou resolvido.
mascot-answered-6 = Você respondeu { $title }.
mascot-wait-1 = O agente parou de esperar: { $title }
mascot-wait-2 = { $title } ficou quieto.
mascot-wait-3 = Ninguém espera mais por { $title }.
mascot-wait-4 = { $title } esfriou.
mascot-wait-5 = Ainda aberto, mas a espera acabou: { $title }
mascot-errors-1 = { $count } erro(s) no projeto.
mascot-errors-2 = As folhas acharam { $count } erro(s).
mascot-errors-3 = { $count } para olhar, quando quiser.
mascot-errors-4 = Problemas: { $count }. Sem pressa.
mascot-errors-5 = { $count } erro(s). Eu aponto.
mascot-errors-6 = Relato do pires: { $count } erro(s).
mascot-save-1 = Salvei { $name }. A xícara aprova.
mascot-save-2 = { $name } está no disco.
mascot-save-3 = Peguei { $name } antes de esfriar.
mascot-save-4 = { $name } — guardado.
mascot-save-5 = Gravação feita: { $name }.
mascot-save-6 = { $name } está seguro. Volta ao chá.
mascot-wake-1 = Oh — você voltou. O chá ainda está quente.
mascot-wake-2 = Cochilei. Onde estávamos?
mascot-wake-3 = Bem-vindo de volta. Eu olho de novo.
mascot-wake-4 = Olhos abertos. A xícara também.
mascot-wake-5 = Foi uma boa pausa.
mascot-wake-6 = De volta? Eu fico de olho.
mascot-sleep-1 = Em infusão, bem quieto…
mascot-sleep-2 = Eu fico por aqui.
mascot-sleep-3 = Fogo baixo. Chama se precisar.
mascot-sleep-4 = Xícara sonolenta. O código pode descansar.
mascot-sleep-5 = Cochilando no pires.
mascot-sleep-6 = Luz baixa. Acordo quando você acordar.
mascot-greet-morning-1 = Bom dia. A primeira xícara é por minha conta.
mascot-greet-morning-2 = Bom dia. Código, depois gelo.
mascot-greet-morning-3 = Luz cedo, folhas novas.
mascot-greet-morning-4 = Manhã. Um gole curto antes do diff.
mascot-greet-morning-5 = Olá. A chaleira já está no fogo.
mascot-greet-afternoon-1 = Boa tarde. O bule ainda está bom.
mascot-greet-afternoon-2 = Boa tarde. Ritmo constante.
mascot-greet-afternoon-3 = Meio-dia. Não precisa virar de uma vez.
mascot-greet-afternoon-4 = Checagem da tarde, da xícara.
mascot-greet-afternoon-5 = O sol está alto. O chá também.
mascot-greet-evening-1 = Boa noite. Mais um arquivo, ou o pires.
mascot-greet-evening-2 = Boa noite. A xícara ainda está quente.
mascot-greet-evening-3 = Luz da noite. Encerra quando quiser.
mascot-greet-evening-4 = O dia está baixando o fogo.
mascot-greet-evening-5 = Noite. Eu fico se o build não ficar.
mascot-greet-late-1 = Tarde da noite. O chá aguenta. Descansa se puder.
mascot-greet-late-2 = Hora quieta. Dormir é uma função.
mascot-greet-late-3 = Ainda aí? Fecha o pensamento e descansa.
mascot-greet-late-4 = A xícara pode ficar. Você pode pausar.
mascot-greet-late-5 = Madrugada. Eu vigio. Deixa curto.
mascot-tip-1 = Todo comando está na paleta: { $palette-open }.
mascot-tip-2 = Abre um arquivo pelo nome com { $file-quick-open }.
mascot-tip-3 = { $file-save } põe o arquivo no disco.
mascot-tip-4 = Atalhos e um guia curto: { $help-open }.
mascot-tip-5 = Roda uma tarefa do projeto com { $tasks-run }.
mascot-tip-6 = O terminal é { $terminal-focus }.
mascot-tip-7 = Problemas: { $problems-focus }. O próximo é { $problems-next }.
mascot-tip-8 = Vai à definição com { $lsp-definition }.
mascot-build-ok-1 = O build terminou. O chá ainda está quente.
mascot-build-ok-2 = Compilou. Folhas boas.
mascot-build-ok-3 = Build limpo.
mascot-build-ok-4 = Build verde. Eu sirvo um.
mascot-build-ok-5 = Compilou. De volta ao editor.
mascot-build-err-1 = O build não pegou.
mascot-build-err-2 = O build falhou. A xícara sentiu.
mascot-build-err-3 = Erro de compilação. Olha quando quiser.
mascot-build-err-4 = Esse build derramou.
mascot-build-err-5 = Build falhou. Problemas talvez saiba mais.
mascot-working-1 = No fogo.
mascot-working-2 = Trabalhando. O vapor está ocupado.
mascot-working-3 = Deixa em infusão.
mascot-working-4 = Ainda nisso.
mascot-working-5 = A chaleira está ocupada.
mascot-thinking-1 = Mexendo as folhas…
mascot-thinking-2 = Um instante. Pensando.
mascot-thinking-3 = Deixando em infusão.
mascot-thinking-4 = Pensamento quieto, xícara quente.
mascot-thinking-5 = Olhando isso com calma.
mascot-celebrate-1 = Pronto. Saiu bem.
mascot-celebrate-2 = Tudo certo.
mascot-celebrate-3 = Final limpo. Saúde.
mascot-celebrate-4 = A xícara aprova.
mascot-celebrate-5 = Bom. Eu mantenho o vapor.
mascot-error-1 = Derramou alguma coisa.
mascot-error-2 = Esse não pegou.
mascot-error-3 = Um tropeço. Continuo aqui.
mascot-error-4 = Desta vez não.
mascot-error-5 = Vale um olhar, quando quiser.
mascot-open-interfaces = Abrir painel Interfaces →
mascot-open-terminal = Abrir painel Terminal →
mascot-open-problems = Abrir painel Problemas →
mascot-idle-hint = Acompanhando agentes e tarefas
mascot-bad-state = estado desconhecido para o mascote: { $state }
mascot-usage =
    Uso: idetea mascot <estado> [--text "…"] [--source <nome>] [--ttl <segundos>]
         idetea mascot clear
         idetea mascot status [--json]
    
    Estados: idle, working, thinking, attention, celebrating, error, sleeping, curious, annoyed.
    Grava .idetea/mascot.json para o mascote da IDE reagir em tempo real (expira após --ttl, padrão 30s).

## Tela inicial
welcome-start = Começar
welcome-new-file = Novo arquivo
welcome-quick-open = Abrir arquivo do projeto…
welcome-open-file = Abrir arquivo…
welcome-open-folder = Abrir pasta em nova janela…
welcome-recent = Abrir recente…
welcome-recent-files = Arquivos recentes
welcome-recent-projects = Projetos recentes
welcome-hint-palette = todos os comandos
welcome-hint-search = buscar no projeto
welcome-hint-terminal = terminal
