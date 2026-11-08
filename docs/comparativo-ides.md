# Comparativo Honesto: IDE TEA vs. Outras IDEs do Mercado

Este documento faz uma avaliação técnica, direta e sem maquiagem da **IDE TEA** (estado atual, `v0.4.0`) frente às principais IDEs e editores do mercado:

- **VS Code / VSCodium** (o padrão universal baseado em Electron)
- **Cursor / Windsurf** (forks de VS Code focados em IA)
- **Zed** (IDE nativa em Rust focada em velocidade, colaboração e IA integrada)
- **JetBrains (RustRover, IntelliJ, PyCharm, CLion)** (IDEs semânticas pesadas)
- **Neovim / Helix / Lapce** (editores modais/nativos enxutos)

> **Escopo desta análise:** o foco é estritamente técnico — **estabilidade, desempenho, funcionalidades de edição, LSP, depuração, Git, ecossistema e integração com agentes**. Não leva em conta o tamanho da equipe ou quem mantém o projeto.

---

## 1. Visão Geral Executiva (Matriz Direta)

| Dimensão | IDE TEA | VS Code | Cursor / Windsurf | Zed | JetBrains | Helix / Neovim |
|---|---|---|---|---|---|---|
| **Tempo de abertura (cold start)** | **Excelente (< 200 ms)** | Lento (1–3 s) | Lento (1,5–4 s) | **Excelente (< 150 ms)** | Muito lento (5–20 s) | **Instantâneo (< 50 ms)** |
| **Consumo de RAM em repouso** | **Baixo (~60–150 MB)** | Alto (400 MB–1,5 GB) | Muito alto (600 MB–2 GB) | Baixo (~120–300 MB) | Muito alto (1,5–4 GB+) | **Mínimo (15–60 MB)** |
| **Arquivos gigantes (50 MB+ / minificados)** | **Excelente (modo leve automático)** | Médio (trava em linhas longas) | Médio | Bom | Sofre acima de 20 MB | Depende de plugins (Neovim) |
| **Privacidade / Offline / Zero conta** | **Total (zero rede salvo git push/pull)** | Telemetria por padrão (salvo VSCodium) | Exige conta e nuvem | Pede conta p/ IA e colaboração | Exige conta / licença | **Total** |
| **Protocolo estruturado p/ agentes (UI declarativa, aprovação de comandos, mascote)** | **Única na categoria (nativo em arquivos)** | Não tem (depende de extensões de chat) | Focado em chat/diff proprietário | Focado em chat/inline assist | Focado em chat/inline | Não tem (só terminal) |
| **Autocomplete de IA inline ("Ghost text" / Tab)** | **Não tem (por filosofia)** | Via Copilot | **Excelente (líder)** | Muito bom (Zeta) | Bom | Via plugins |
| **Maturidade do Editor de texto** | Boa no básico, faltam recursos avançados | **Referência** | **Referência** | Muito alta | **Referência** | **Referência (modal)** |
| **Profundidade de LSP e Refatoração** | Média (sem Code Actions e Inlay Hints) | **Completa** | **Completa** | Alta | **Insuperável (AST próprio)** | Alta |
| **Depuração (Debug)** | Básica (só GDB 14+ para Rust/C/C++/Zig) | **Completa (qualquer linguagem)** | **Completa** | Básica/Média (DAP recente) | **Excelente** | Básica (nvim-dap) |
| **Controle de Versão (Git)** | Básica/Média (sem gutter diff nem merge 3-vias) | **Completa** | **Completa** | Média | **Excelente (melhor merge)** | Via LazyGit / Fugitive |
| **Suporte Multiplataforma** | **Só Linux (Wayland) testado hoje** | **Linux, macOS, Windows, Web** | **Linux, macOS, Windows** | **macOS, Linux, Windows** | **Linux, macOS, Windows** | **Qualquer SO / SSH** |

---

## 2. Estabilidade e Confiabilidade

### Onde a IDE TEA é forte de verdade
1. **Arquitetura sem WebView / Chromium:** diferente de VS Code, Cursor e Windsurf, não há processos de renderização Chromium nem dezenas de *extension hosts* Node.js vazando memória ao longo do dia. O binário é único, compilado em Rust seguro.
2. **Isolamento de falhas de extensões e processos externos:**
   - Servidores de linguagem (`tea-lsp`), depurador (`tea-dap` / `gdb`), chamadas `git` e agentes rodam em processos separados. Se o `rust-analyzer` ou o `gdb` travar ou morrer, a janela da IDE não congela.
   - As extensões internas comunicam-se com o núcleo apenas por mensagens e eventos (`Request` / `CoreEvent`), sem compartilhar referências mutáveis do estado da aplicação.
3. **Proteção contra perda de trabalho:**
   - Fechar aba ou sair da IDE com alterações não salvas sempre pede confirmação.
   - Apagar arquivos na árvore envia para a **lixeira do sistema** (não dá `unlink` irreversível).
   - Mudanças externas no disco (`git checkout`, `cargo fmt`, agentes editando arquivos) são detectadas pelo watcher: abas sem edição local recarregam sozinhas; abas com edição local não salva **nunca** são sobrescritas silenciosamente.
   - Arquivos binários ou não-UTF-8 são recusados com aviso claro em vez de corromperem o buffer ao salvar.
   - O protocolo `.idetea/` usa escrita atômica (`.tmp` + `rename`) e possui comando de autorrecuperação (`idetea fix` / *"Interfaces: consertar este projeto"*).

### Onde a estabilidade da IDE TEA ainda perde para as grandes
1. **Testada apenas em Linux (Wayland):** hoje a IDE TEA é desenvolvida e validada no Linux. Em macOS e Windows há diferenças reais de empacotamento, fontes, atalhos (`Cmd` vs `Ctrl`), PTY do terminal (`conpty` no Windows) e caminhos que ainda não passaram por fogo real. VS Code, JetBrains e Zed rodam de forma madura nos três sistemas.
2. **Limitações do ecossistema `iced` 0.14 para aplicações complexas:**
   - A IDE opera em janela única (`iced::application`). Não é possível destacar um terminal, uma prévia ou um editor para um segundo monitor em outra janela nativa do SO.
   - Não há *crash recovery* com *hot-stash* automático do texto não salvo (como o *Hot Exit* do VS Code ou o *Local History* do JetBrains): se o processo tomar um `SIGKILL` ou faltar energia antes de você dar `Ctrl+S`, o texto digitado em memória na aba se perde.
3. **Destaque de sintaxe baseado em regex (`syntect`) vs. `tree-sitter`:**
   - Embora a IDE TEA tenha um modo leve excelente para arquivos de 50 MB ou linhas minificadas gigantes, em arquivos de código normais com milhares de linhas o `syntect` (baseado em gramáticas `.sublime-syntax` com regex) é estruturalmente menos resiliente a código incompleto do que o `tree-sitter` usado por Zed, Helix e Neovim.

---

## 3. Fluxo com Agentes de IA (Onde a IDE TEA é única — e onde ela não compete)

Esta é a diferença mais profunda de filosofia de produto no mercado hoje.

### Como Cursor, Windsurf e Zed tratam IA
- São pensados para **o modelo dentro do editor**:
  - *Autocomplete preditivo (Tab / Next Edit Prediction)* que adivinha as próximas linhas enquanto você digita.
  - *Inline Edit (`Ctrl+K`)* e *Composer/Agent Chat* que mostram diffs coloridos (verde/vermelho) diretamente entre as linhas do editor para aceitar/rejeitar hunk por hunk.
  - Em troca, acoplam você a contas na nuvem, assinaturas mensais ou clientes HTTP de provedores dentro da IDE.

### Como a IDE TEA trata IA (e onde ela ganha)
A IDE TEA assume que **os melhores agentes já rodam no terminal** (Claude Code, Codex, Gemini CLI, Aider, OpenCode) e resolve o problema que nenhuma outra IDE resolveu de forma nativa e aberta: **como esses agentes conversam com o humano sem virar uma parede de texto no terminal**.

1. **Interfaces Declarativas Nativas (`.idetea/ui/` + `idetea ask`):**
   - Em vez de o agente perguntar no meio de 200 linhas de log no terminal, ele publica um JSON validado e a IDE renderiza formulários nativos em `iced`: seletores, checkboxes, listas de passos de teste manual (onde o botão *Aprovar* só habilita após cumprir os passos), prévias de imagem/galeria, spritesheets animados e áudio WAV com forma de onda.
   - **Nenhuma outra IDE do mercado (VS Code, Cursor, Zed, JetBrains) possui um protocolo declarativo baseado em arquivos para agentes de terminal desenharem UI nativa com histórico auditável (`.idetea/history/`).**
2. **Governança e Segurança de Execução (`idetea run`):**
   - Quando um agente quer abrir um terminal interativo ou delegar tarefa, a IDE mostra o comando exato numa aba de aprovação. A lista de comandos permitidos (`[agents] allow`) só é lida do config global do usuário (`~/.config/idetea/config.toml`) e **jamais** do repositório — impedindo que um repositório malicioso se autoconceda permissão de execução.
3. **Delegação em Git Worktrees Isolados (`--worktree` + `$IDETEA_ANSWER`):**
   - A IDE gerencia nativamente a criação e limpeza de worktrees para subagentes trabalharem em paralelo sem sujar a árvore principal.
4. **Mascote Reativo Nativo (Tea) + `idetea mascot`:**
   - Enquanto ferramentas como o *Masko.ai* exigem macOS + servidor MCP na nuvem para ter um mascote que reage ao Claude Code, a IDE TEA tem o **Tea** embutido nativamente observando `.idetea/ui/`, `.idetea/agents/`, tarefas e LSP em tempo real, além de reagir ao mouse (`curious` / `annoyed`) e aceitar pacotes `mascot.toml`.

### Onde a IDE TEA perde no fluxo de IA
- **Zero Autocomplete Preditivo ("Ghost Text" / Tab):** para quem depende do `Tab` do Cursor/Copilot para completar boilerplate enquanto digita manualmente, a IDE TEA não tem equivalente.
- **Sem visualização de Diff Inline de IA no editor:** quando um agente no terminal edita um arquivo, o arquivo recarrega na aba da IDE TEA. Para ver o que mudou, você precisa ir ao painel **Git**; não há botões *"Accept / Reject hunk"* flutuando dentro do editor de código como no Cursor ou Windsurf.
- **Passagem de contexto do editor para o agente ainda é manual:** embora o agente possa rodar `idetea status` para descobrir o arquivo ativo e a linha do cursor, ainda não há um atalho na IDE (ex.: `Ctrl+L`) que envie um trecho selecionado ou um erro do LSP direto para o prompt do agente aberto no terminal.

---

## 4. Editor de Código: O que tem vs. O que falta

### O que já está sólido na IDE TEA
- Abertura rápida de arquivos (`Ctrl+P` com busca fuzzy e salto `arquivo:linha`), símbolos no arquivo (`Ctrl+Shift+O`) e no projeto (`Ctrl+T`).
- Multi-cursor, seleção palavra por palavra (`Ctrl+Setas`), `Shift+Clique`, dobramento de código (folding), desfazer/refazer, busca e substituição.
- Zoom temporário do editor (`Ctrl+=`, `Ctrl+-`, `Ctrl+0`, `Ctrl+Scroll`) sincronizado entre abas e splits sem afetar o resto da UI.
- Divisão do editor lado a lado (split), breadcrumbs clicáveis (`projeto › pasta › arquivo › símbolo`), ícones Seti por linguagem e fontes customizáveis com ligaduras (`JetBrains Mono` embutida).
- Modo Vim opcional.
- Abas nativas para imagens (com zoom e arrasto), PDF (renderizado em Rust puro via `hayro`) e prévia lado a lado de Markdown e SVG em tempo real.

### O que falta no editor comparado a VS Code, Zed e JetBrains
1. **Substituição com Regex e Case-Preserving no editor:** a busca no projeto (`Ctrl+Shift+F`) tem regex, mas a substituição em massa no projeto todo (*Search & Replace across files*) ainda não existe — apenas busca/substituição no arquivo atual.
2. **Snippets:** não há motor de snippets (Tab-stops `$1`, `$2`) para expandir blocos rápidos de código.
3. ** Seleções estruturais por AST (Expand/Shrink Selection):** em Zed, Helix e JetBrains, um atalho expande a seleção da palavra → expressão → bloco `if` → função inteira via `tree-sitter`/AST.
4. **Encodings legados:** só abre UTF-8 (preservando LF/CRLF). Quem trabalha com arquivos `ISO-8859-1` / `Windows-1252` não consegue editá-los na IDE TEA.
5. **Múltiplas janelas / múltiplos projetos na mesma instância:** cada projeto é um processo `idetea` separado; não há "Multi-root Workspaces" na mesma janela.

---

## 5. Inteligência de Linguagem (LSP) e Refatoração

### O que a IDE TEA já entrega muito bem
- Diagnósticos em tempo real com **sublinhado ondulado no texto do editor**, tooltip de erro no hover, contagem na barra de status e painel **Problemas** (`Ctrl+Shift+M` / `F8`).
- Autocompletar, assinatura de parâmetros, ir para definição (`F12`), ir para tipo e ir para implementação (`Ctrl+F12`).
- Buscar referências (`Shift+F12` ou `Ctrl+Clique` na declaração) agrupadas por arquivo, com histórico de navegação Voltar/Avançar (`Alt+←` / `Alt+→` e botões laterais do mouse).
- **Hierarquia de chamadas (`Alt+Shift+H`):** quem chama a função e o que ela chama.
- **Painel de Estrutura / Outline (`Alt+7`):** árvore de símbolos do arquivo indicando visibilidade (`pub`, `pub(crate)`, privado) e destacando o símbolo sob o cursor.
- **Renomear símbolo (`F2`)** em todo o projeto com diálogo de pré-visualização de quantos arquivos e mudanças serão afetados antes de aplicar.
- Formatação ao salvar e respeito a `.editorconfig`.

### Lacunas importantes de LSP frente a VS Code, Zed e JetBrains
1. **Sem Code Actions / Quick Fixes (`Ctrl+.` / lâmpada do LSP):**
   - Esta é hoje a **maior falta prática no dia a dia com Rust/TypeScript/Go**: quando o `rust-analyzer` sugere *"Import `std::collections::HashMap`"*, *"Add missing match arms"* ou *"Fill struct fields"*, na IDE TEA você vê a mensagem de diagnóstico, mas não consegue aplicar o *Quick Fix* (`textDocument/codeAction`) com um atalho.
2. **Sem Inlay Hints:**
   - Não exibe os tipos inferidos inline (`let x: Vec<String> = ...`) nem nomes de parâmetros em chamadas de função, recurso muito usado em Rust e TypeScript no VS Code, Zed e RustRover.
3. **Sem Semantic Tokens do LSP:**
   - As cores de sintaxe vêm exclusivamente da gramática `.sublime-syntax` (`syntect`), sem enriquecimento semântico do LSP (ex.: diferenciar variável mutável de imutável ou macro de função pela cor do servidor de linguagem).

---

## 6. Depuração (Debug)

### O que a IDE TEA faz hoje
- Integração nativa com `gdb` 14+ (via DAP): breakpoints clicando no número da linha (`F9`) que acompanham o texto se você editar o arquivo, `F5` (compila e roda automaticamente em projetos Rust e Zig), Step Over (`F10`), Step Into (`F11`), Step Out (`Shift+F11`), Pausar e Parar.
- Painel com pilha de chamadas (clique leva à linha do frame), árvore de variáveis locais inspecionável e saída do programa.

### Onde perde para VS Code e JetBrains
- **Atrelado ao GDB 14+ (Rust, C, C++, Zig):** ainda não suporta adaptadores DAP para Python (`debugpy`), Go (`delve`), JavaScript/TypeScript (`js-debug`) ou `codelldb` (macOS).
- **Recursos avançados de debug ausentes:** não há breakpoints condicionais (*"parar só quando `i == 42`"*), logpoints, janela de *Watch Expressions* arbitrárias (REPL durante o breakpoint) nem valores de variáveis desenhados inline ao lado do código no editor.

---

## 7. Git e Controle de Versão

### O que a IDE TEA faz hoje
- Barra de status com branch atual, commits à frente/atrás (`ahead/behind`) e total de arquivos modificados.
- Painel Git com stage/unstage por arquivo ou geral, visualização de diff colorido, caixa de mensagem de commit, descarte de mudanças com confirmação, e botões de `push`, `pull` e `fetch` usando o `git` e as credenciais/SSH do sistema.
- Cores de status Git na árvore de arquivos.

### Onde perde para VS Code, Zed e JetBrains
1. **Sem marcações Git na margem do editor (Gutter Diff):** enquanto você edita um arquivo, não há aquelas barrinhas verdes/azuis/vermelhas ao lado dos números de linha mostrando quais linhas foram adicionadas ou alteradas desde o último commit.
2. **Sem Git Blame inline:** não mostra quem alterou a linha atual e em qual commit.
3. **Sem resolvedor visual de conflitos de merge (3-way merge):** em caso de conflito, você resolve editando os marcadores `<<<<<<<` no texto ou pelo terminal.
4. **Sem troca/criação de branches ou Stash pela UI:** para trocar de branch, criar branch ou dar `git stash`, você usa o terminal integrado.

---

## 8. Resumo Honesto: Quando escolher a IDE TEA hoje (e quando não escolher)

### A IDE TEA já é melhor ou única quando você:
1. **Programa em Linux ao lado de agentes de terminal (Claude Code, Codex, Gemini CLI, etc.)** e quer que o agente faça perguntas estruturadas, mostre planos/testes manuais, peça permissão explícita para rodar comandos, use worktrees isolados e tenha um mascote reativo (**Tea**) sem depender de nuvem ou assinatura.
2. **Prioriza velocidade, previsibilidade e baixo ruído sensorial:** abre em menos de 200 ms, consome pouca memória, não tem conta, não tem telemetria, não muda a UI sozinha e respeita arquivos TOML simples.
3. **Lida com projetos em Rust, Zig, C/C++, Python, Go ou Web** onde o fluxo principal é editar rápido, navegar por símbolos/referências do LSP, rodar tasks/testes no terminal integrado e revisar commits.

### Outra IDE ainda é melhor quando você:
1. **Quer autocomplete preditivo de IA a cada tecla (`Tab` / Ghost Text) ou edição inline por diff:** **Cursor**, **Windsurf** ou **Zed** atendem melhor esse estilo.
2. **Depende fortemente de refatorações assistidas por *Code Actions* (`Ctrl+.`), *Inlay Hints* ou depuradores de Python/JS/Go na GUI:** **VS Code** ou **JetBrains** são muito mais completos hoje.
3. **Trabalha em macOS ou Windows, ou depende de SSH Remote Development / DevContainers:** **VS Code** e **Zed** já têm suporte maduro para esses cenários.

---

## 9. Os 5 Próximos Passos de Maior Impacto (Para encostar nos líderes sem perder a alma da IDE TEA)

Se o objetivo for reduzir a distância funcional para VS Code e Zed mantendo o binário leve e a filosofia do manifesto:

1. **Code Actions / Quick Fixes do LSP (`Ctrl+.`):** permitir aplicar os *quick fixes* e *imports* automáticos que o servidor LSP já sabe calcular. É o item nº 1 que mais faz falta ao programar em Rust/TS.
2. **Marcações de Git no Editor (Gutter Diff) + Troca de Branch na Status Bar:** mostrar linhas adicionadas/modificadas na margem do editor e permitir clicar no nome da branch na barra de status para trocar de branch na paleta.
3. **Atalho "Enviar Seleção / Diagnóstico para o Agente" (`Ctrl+L`):** fechar o ciclo entre o editor/LSP e o agente rodando no terminal integrado.
4. **Substituição em Massa no Projeto (`Ctrl+Shift+H`):** aproveitar a busca rápida que já existe em `tea-ext-search` para permitir substituir texto/regex em múltiplos arquivos com prévia.
5. **Auto-save / Backup de Recuperação (Crash Recovery):** gravar um snapshot leve em `.idetea/` das abas com modificações não salvas a cada poucos segundos, garantindo perda zero mesmo em queda de energia.
