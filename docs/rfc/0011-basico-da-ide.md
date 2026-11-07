# RFC 0011 — O básico da IDE em volta do editor: abas, árvore, busca, terminal, git e janela

- **Status:** aprovada em 2026-11-07 (pelo painel), com as decisões do fim. Base: `main` em
  `3de4003` (0.4.0 com as etapas 1–3 da RFC 0010).
- **Origem:** pedido do usuário no chat (2026-11-07), no dogfooding do desenvolvimento pela própria
  IDE: "identificar melhorias no básico de uma IDE e criar uma RFC".
- **Complementa:** [RFC 0009](0009-ergonomia-do-editor.md), que cobriu o básico *dentro* do editor
  (cursor, seleção, busca no arquivo, atalhos em conflito). Esta RFC é o restante do básico: a IDE
  em volta dele. As pendências da 0009 estão reunidas na seção 5 sem serem rediscutidas.

Marcadores: **[Fato]** conferido no código nesta sessão; **[Referência]** comportamento padrão
conhecido das outras IDEs (não conferido nesta sessão); **[Hipótese]** provável, não testado;
**[Recomendação]** proposta, ainda não decidida.

## Resumo

O IDE TEA já tem um "esqueleto" básico completo e em parte acima da média: menu de contexto das
abas com cinco formas de fechar, git com blame e troca de branch, busca no projeto com
substituição e prévia, terminal com abas e tarefas, barra de status com encoding e fim de linha,
ajuda gerada do mapa de teclas. Quem vem de outra IDE não sente falta de *painéis*; sente falta de
**movimento**:

1. **Sem marcação de erros no editor** [Fato]: os diagnósticos chegam do LSP e vão para o painel e
   para o contador da barra de status, mas o texto no editor não mostra sublinhado nenhum.
2. **Sem zoom do editor** [Fato]: o tamanho da fonte só muda no Settings (global, gravado no
   config); não há Ctrl+= / Ctrl+- / Ctrl+0 (pendente da RFC 0009).
3. **Sem autosave** [Fato]: nada salva ao perder o foco ou ao fechar a aba.
4. **Árvore só de mouse** [Fato]: sem setas, Enter e "digitar para achar".
5. **Ctrl+Tab não existe** [Fato]: só Ctrl+PageUp/Down em ordem fixa; sem alternância por ordem de
   uso e sem arrastar aba para reordenar ou fixar.
6. **Git sem stash nem histórico** [Fato]; **terminal sem dividir, renomear ou buscar** [Fato];
   **busca do projeto sem incluir/excluir** [Fato]; **nada de soltar arquivo na janela** [Fato].

**[Recomendação]**: duas versões. Na **0.5.0** (junto com o que falta da 0009): marcação de erros,
zoom, autosave, árvore pelo teclado e Ctrl+Tab. Na **0.6.0**: arrastar/fixar abas, soltar arquivo,
globs na busca, stash do git e divisão do terminal. Minimap, aba de pré-visualização, stage por
hunk e acordes de teclas ficam para depois, cada um com seu motivo (seção 6).

## 1. Como foi feita a pesquisa

- **IDE TEA:** `docs/features.md`, `SPEC.md`, o keymap padrão (`tea-core/src/config.rs:117-169`), a
  árvore (`crates/ext/tea-ext-filetree/src/lib.rs`), a busca (`tea-ext-search`), o terminal
  (`tea-ext-terminal`), o git (`tea-ext-git`), a barra de status (`tea-ext-statusbar`), as abas e o
  split (`tea-app/src/main.rs`) e a fachada do editor (`tea-editor`).
- **Outras IDEs:** VS Code, IntelliJ/RustRover (JetBrains), Zed e Sublime Text, pelos
  comportamentos padrão no Linux, na mesma linha da RFC 0009.

## 2. O que já está bem [Fato]

- **Abas:** fechar/fechar outras/fechar à direita/fechar salvas/fechar todas (menu de contexto,
  `main.rs:2901-2935`), mover para o outro lado do split, botão do meio fecha, reabrir fechada
  (Ctrl+Shift+T), divisão do editor em dois lados (Ctrl+\), Ctrl+PageUp/PageDown.
- **Árvore:** criar/renomear/apagar (lixeira), recolher tudo, revelar o arquivo ativo (automático,
  `filetree/src/lib.rs:365-371`), cores do git, ícones por linguagem, menu de contexto com copiar
  caminho e abrir no gerenciador de arquivos.
- **Busca no projeto:** regex, palavra inteira, maiúsculas, substituição com prévia por linha,
  arquivo e tudo, respeitando `.gitignore` (`tea-ext-search`).
- **Git:** stage/unstage por arquivo ou tudo, diff colorido, commit, descartar com confirmação,
  push/pull/fetch, troca e criação de branch, **blame** (`tea-ext-git/src/lib.rs:203`), abrir o
  arquivo do diff.
- **Terminal:** abas múltiplas, aba da tarefa, abas de agente, herda tema e fonte do IDE,
  comandos `terminal.new/close/restart`.
- **Barra de status:** notificação, atividade em segundo plano, contagem de problemas (clicável),
  posição do cursor, linguagem, encoding, fim de linha (`tea-ext-statusbar`).
- **Janela:** recentes (Ctrl+R) com projetos e arquivos, abrir pasta em nova janela, restaurar
  sessão (abas e painéis), ajuda (F1) gerada do keymap com busca e guia, settings com UI.
- **LSP:** hover, completar, dica de parâmetros, definição, referências, implementação, tipo,
  hierarquia de chamadas, renomear com prévia, formatar, outline, breadcrumbs.

## 3. As lacunas, por área

Em cada linha: o que as outras fazem e o que o IDE TEA faz hoje.

### 3.1 Abas

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Arrastar aba para reordenar | sim | sim | sim | sim | **não** [Fato] |
| Ctrl+Tab por ordem de uso (MRU) | sim | sim (Switcher) | sim | sim | **não** — só PageUp/Down em ordem fixa [Fato: `config.rs:130-131`] |
| Fixar aba (pin) | sim | sim | sim | — | **não** [Fato] |
| Aba de pré-visualização (clique único na árvore abre "temporária") | sim | sim | sim | — | **não** — clique abre aba comum [Fato: `filetree/src/lib.rs:443-446`] |
| Mais de dois grupos de editor | sim | sim | sim | sim | só dois lados [Fato: `Tab.right` + `other_tab`, `main.rs:435-478`] |

### 3.2 Árvore de arquivos

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Navegar com setas, Enter abre | sim | sim | sim | sim | **não** [Fato: só `Clicked`, sem `on_key` em `filetree/src/lib.rs`] |
| Digitar para achar na árvore | sim | sim | — | — | não [Fato] |
| Copiar/colar arquivos | sim | sim | sim | sim | não — só copiar caminho [Fato: `filetree/src/lib.rs:166-168`] |
| Arrastar arquivo para mover | sim | sim | sim | sim | não [Fato] |
| Seleção múltipla | sim | sim | — | — | não [Fato: `selected: Option<PathBuf>`] |

### 3.3 Busca no projeto

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Incluir/excluir arquivos (globs) | sim | sim | sim | sim | **não** [Fato: sem include/exclude em `tea-ext-search`] |
| Histórico de buscas | sim | sim | — | — | não [Hipótese] |
| Substituir com prévia | sim | sim | sim | sim | **sim** ✅ |

### 3.4 Terminal

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Dividir o terminal (split) | sim | sim | sim | — | **não** — só abas [Fato] |
| Renomear aba | sim | — | sim | — | **não** — "Terminal 1, 2…" automático [Fato] |
| Buscar no histórico (find) | sim | sim | sim | — | não [Hipótese: depende do `iced_term`] |
| Copiar seleção (Ctrl+Shift+C) | sim | sim | sim | sim | não confirmado [Hipótese: verificar o `iced_term`] |
| Enviar seleção do editor para o terminal | sim | sim | sim | — | não |

### 3.5 Git

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Stash (list/apply/pop) | sim | sim | sim | — | **não** [Fato: nada de stash em `tea-ext-git/src/git.rs`] |
| Histórico de commits (log) | sim | sim | sim | — | **não** [Fato: `git log -1` só para a linha do HEAD, `git.rs:337-338`] |
| Stage por hunk | sim | sim | — | — | não — por arquivo [Fato: `Stage(path)`] |
| Blame | sim | sim | sim | — | **sim** ✅ |
| Resolver conflitos (merge) | editor 3 vias | editor 3 vias | — | — | só a cor de conflito na árvore |

### 3.6 Janela e editor

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Marcação de erros no texto (sublinhado) | sim | sim | sim | sim | **não** [Fato: `marks.rs` desenha breakpoints, exec e git, nada de diagnósticos] |
| Zoom do editor (Ctrl+= / Ctrl+- / Ctrl+0) | sim | opt. | sim | sim | **não** — só o slider global do Settings [Fato] |
| Autosave (perder foco / atraso) | opt. | padrão | opt. | opt. | **não** [Fato: sem handler de foco] |
| Soltar arquivo na janela abre | sim | sim | sim | sim | **não** [Fato: sem `FileDropped`] |
| Minimap | sim | — | — | sim | **não** [Fato] |
| Alternar quebra de linha (comando) | paleta | paleta | — | — | não — só no config [Fato] |
| Zen mode | sim | — | sim | — | não |

### 3.7 Mapa de teclas

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Acordes (Ctrl+K Ctrl+C…) | sim | sim | sim | sim | **não** — acorde único [Fato: `HashMap<Chord, String>`, `keymap.rs:143`] |
| Atalhos por tipo de arquivo | sim | sim | — | — | não [Hipótese] |

## 4. Onde cada coisa é feita

| Item | Widget / cliente | Fachada (`tea-editor`) | IDE (`tea-app`) | Extensão |
|---|---|---|---|---|
| Marcação de erros no texto | desenho no widget (ou camada como `marks.rs`) | passar os diagnósticos | recebe do `tea-lsp` | — |
| Zoom do editor | — | `set_font_size` já existe (`tea-editor/src/lib.rs:567`) | comando + atalhos | — |
| Autosave | — | salvar sem intervenção | evento de foco/janela | — |
| Árvore pelo teclado e digitar para achar | — | — | — | `tea-ext-filetree` |
| Ctrl+Tab MRU, arrastar/fixar aba | — | — | `main.rs` (abas) | — |
| Aba de pré-visualização | — | — | `main.rs` + clique da árvore | `tea-ext-filetree` |
| Soltar arquivo na janela | — | — | subscription `FileDropped` | — |
| Globs na busca | — | — | — | `tea-ext-search` |
| Dividir/renomear/buscar no terminal | depende do `iced_term` | — | — | `tea-ext-terminal` |
| Stash e log do git | — | — | — | `tea-ext-git` |
| Acordes de teclas | — | — | `tea-core/src/keymap.rs` | — |
| Minimap | widget | — | — | — |

## 5. Pendências da RFC 0009 (não rediscutir, só lembrar)

Aprovadas na 0009 e ainda não feitas (o "Andamento" dela lista só palavra/Home/Shift+Tab e os
atalhos no padrão VS Code):

- Correções rápidas do LSP em Ctrl+. — o atalho já está livre, mas não há `codeAction` no
  `tea-lsp` [Fato: `client.rs` não tem o pedido].
- Busca no arquivo (Ctrl+F) com regex, palavra inteira e "só na seleção" — do widget.
- Colar com o botão do meio (Linux) — `read_primary` no iced.
- Zoom do editor — **entra nesta RFC** (seção 3.6), pois é o fechamento do item 5 da 0009.
- Prioridade 2 da 0009: realce de ocorrências do símbolo, inlay hints, selecionar linha/todas as
  ocorrências, par do parêntese, Ctrl+Tab com popup, arrastar abas, soltar arquivo, árvore pelo
  teclado. Parte entra aqui com prioridade própria; o resto segue na 0009.

## 6. Prioridades [Recomendação]

**Prioridade 1: 0.5.0 — o básico que se sente todo dia**

1. **Marcação de erros no editor**: sublinhado ondulado (erro/avisos) nos ranges dos diagnósticos
   que já chegam; clique ou hover vai ao detalhe. Widget ou camada como `marks.rs`.
2. **Zoom do editor**: Ctrl+= / Ctrl+- / Ctrl+0 e Ctrl+roda, em cima do `set_font_size` que já
   existe na fachada. Fecha o item da 0009.
3. **Autosave**: `[editor] autosave = "off" | "on_focus_lost" | "after_delay"` (padrão `off`),
   com o aviso de aba fechando com alterações ajustado para não atrapalhar quando ligado.
4. **Árvore pelo teclado**: setas movem o selecionado, Enter abre/expande, `filetree.focus` com
   atalho (Ctrl+Shift+E já revela; acrescenta foco), e digitar filtra.
5. **Ctrl+Tab por ordem de uso**: alterna entre a aba ativa e a anterior, como o Zed/Sublime;
   sem popup (o popup fica para a prioridade 3, junto com a 0009).

**Prioridade 2: 0.6.0**

6. Arrastar abas para reordenar e fixar aba (pin).
7. Soltar arquivo na janela abre; arrastar arquivo na árvore move.
8. Busca no projeto com incluir/excluir (globs).
9. Terminal: copiar seleção e dividir em painéis (se o `iced_term` permitir), renomear aba.
10. Git: stash (list/apply/pop/drop) — os quatro comandos de CLI resolvem o dia a dia.

**Prioridade 3: depois, cada um com seu motivo**

11. Minimap — é o item mais pesado (renderização extra) e o Zed vive bem sem ele; só entra se
    sobrar apetite depois das duas versões acima.
12. Aba de pré-visualização e histórico de busca — conforto, não bloqueio.
13. Stage por hunk — exige visão de diff lado a lado no editor; o diff do painel hoje é
    read-only.
14. Log de commits e gráfico de branches — o terminal já cobre com `git log --graph`; um painel
    só se o uso pedir.
15. Acordes de teclas (Ctrl+K …) — mexe no `keymap.rs` e no F1; esperar a poeira do resto baixar.
16. Zen mode, enviar seleção ao terminal, atalhos por tipo de arquivo.

## 7. Como testar

- Cada item com um teste do `update`/`view` da parte afetada (as extensões já têm esse padrão:
  mensagens simuladas num `Context` de teste, como `filetree/src/lib.rs:731` e
  `problems/src/lib.rs:213`).
- Zoom, autosave e Ctrl+Tab: testes no `tea-app` simulando os eventos (foco, tecla).
- Marcação de erros: teste com diagnósticos fixos sobre um texto pequeno, conferindo os ranges
  desenhados; teste manual com rust-analyzer num arquivo com erro.
- Atalhos novos: teste do keymap padrão (nenhum atalho da IDE e do widget no mesmo lugar sem ser
  de propósito), como na 0009.
- Entrega de cada item pelo painel, com teste manual no dogfooding.

## 8. Para decidir

1. As prioridades da seção 6 estão boas, ou algum item sobe ou desce?
2. Autosave: padrão `off`, com as três opções? Ou `on_focus_lost` por padrão (como o JetBrains)?
3. Ctrl+Tab: MRU simples (só alterna entre duas, como Zed/Sublime) ou já com janela de
   alternância (como VS Code/JetBrains)?
4. Minimap: entra no roadmap da 0.6.0 ou fica fora por ora?
5. Por onde começar?

## Decisões (2026-11-07, pelo painel)

1. **Prioridades:** ficam como estão na seção 6 (0.5.0: erros, zoom, autosave, árvore pelo
   teclado, Ctrl+Tab; 0.6.0: abas, soltar arquivo, globs, terminal, stash).
2. **Autosave:** `off` por padrão, com as opções `on_focus_lost` e `after_delay`.
3. **Ctrl+Tab:** MRU simples, alternando entre as abas recentes (padrão Zed/Sublime); a janela de
   alternância fica para depois.
4. **Minimap:** fora do roadmap por ora.
5. **Começo:** pela marcação de erros no editor.

## Andamento

- Nada ainda; aguardando a primeira etapa (marcação de erros).
