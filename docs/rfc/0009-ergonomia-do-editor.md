# RFC 0009 — O básico do editor: seleção, navegação e ergonomia

- **Status:** aprovada em 2026-11-06 (pelo painel), com as decisões do fim. Base: `main` em `0d8953d` (0.4.0).
- **Origem:** pedido do usuário no chat (2026-11-06): "pesquisa de quais funcionalidades mais básicas de uma IDE (na
  parte de usabilidade, ergonomia, seleção de código, etc) ainda podem ser melhoradas […] pegar requisitos de outras
  IDEs para comparar".

Marcadores: **[Fato]** conferido no código do IDE TEA ou do `iced-code-editor` 0.5.2 nesta sessão;
**[Referência]** comportamento padrão conhecido das outras IDEs (não conferido nesta sessão); **[Hipótese]**
provável, não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

O IDE TEA já tem o que costuma faltar em editores novos (LSP completo, multi-cursor, dobrar código, depurador,
painéis para agentes). O que falta é **o básico de "memória muscular"**, que quem vem do VS Code, JetBrains, Zed ou
Sublime usa sem pensar e estranha na hora:

1. **Ctrl+←/→ anda uma letra, não uma palavra**, e Ctrl+Shift+←/→ não seleciona por palavra.
2. **Ctrl+Backspace apaga uma letra**, não a palavra.
3. **Shift+Tab tira o foco do editor** em vez de desfazer o recuo.
4. **Home vai para a coluna 0**, não para o começo do código da linha.
5. **A busca no arquivo só tem "diferenciar maiúsculas"**: sem regex, palavra inteira ou "só na seleção".
6. **Sem correções rápidas do LSP** (Ctrl+. / Alt+Enter), sem zoom do editor, sem reabrir a aba fechada, sem colar
   com o botão do meio (Linux).

Os quatro primeiros são do widget de edição (`iced-code-editor`). O autor aceita PRs (o #36, da seleção pública, já
está aberto), então a recomendação é corrigir lá o que é do widget, e na IDE o resto. Proposta: os itens da
prioridade 1 numa **0.5.0**.

## 1. Como foi feita a pesquisa

- **IDE TEA:** `docs/features.md`, o keymap padrão (`tea-core/src/config.rs`) e o código.
- **Widget de edição:** o código do `iced-code-editor` 0.5.2: as mensagens (`Message`), os atalhos
  (`input/shortcuts.rs`, `input/events.rs`) e a navegação (`input/update/navigation.rs`).
- **Outras IDEs:** VS Code, IntelliJ/RustRover (JetBrains), Zed e Sublime Text, pelos atalhos padrão no Linux.
  Helix e Lapce ficaram de fora da tabela: o Helix é modal (outro modelo), e o Lapce segue o VS Code.

## 2. O que já está bem [Fato]

Desfazer/refazer, recortar/copiar/colar, multi-cursor (Alt+clique, Ctrl+Alt+↑/↓, Ctrl+D na próxima ocorrência),
mover e duplicar linha (Alt+↑/↓, Shift+Alt+↑/↓), comentar (Ctrl+/), dobrar código, ir para a linha (Ctrl+G), Tab
com seleção recua o bloco, recuo automático, fechar parênteses e aspas sozinho, realce do par de parênteses, cores
por nível de parênteses, sticky scroll, clique duplo e triplo, Shift+clique, Ctrl+clique (definição), modo vim, IME,
breadcrumbs, estrutura, divisão do editor, Ctrl+PageUp/PageDown entre abas, botão do meio fecha a aba, recuperação
de arquivos não salvos depois de um travamento.

## 3. As lacunas, por área

Em cada linha: o que as outras fazem e o que o IDE TEA faz hoje.

### 3.1 Mover e selecionar com o teclado

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Andar por palavra | Ctrl+←/→ | Ctrl+←/→ | Ctrl+←/→ | Ctrl+←/→ | **uma letra** [Fato] |
| Selecionar por palavra | Ctrl+Shift+←/→ | igual | igual | igual | **uma letra** [Fato] |
| Home "esperto" (1º caractere, depois coluna 0) | sim | sim | sim | sim | **coluna 0** [Fato] |
| Selecionar a linha | Ctrl+L | Ctrl+W (amplia) | Ctrl+L | Ctrl+L | não |
| Ampliar/reduzir seleção pela sintaxe | Shift+Alt+→/← | Ctrl+W / Ctrl+Shift+W | Alt+↑/↓ | Ctrl+Shift+Space | não |
| Ir ao par do parêntese | Ctrl+Shift+\ | Ctrl+Shift+M | Ctrl+M | Ctrl+M | não |
| Seleção em coluna (retângulo) | Shift+Alt+arrastar | Alt+arrastar | Shift+Alt+arrastar | Shift+botão direito | não |
| Todas as ocorrências | Ctrl+Shift+L | Ctrl+Alt+Shift+J | Ctrl+Shift+L | Alt+F3 | não (só Ctrl+D, uma por vez) |
| Desfazer o último cursor | Ctrl+U | — | Ctrl+U | Ctrl+U | não |

[Fato] O widget recebe `ArrowKey(direção, shift)`: Ctrl não chega. `handle_home` põe a coluna em 0.
[Fato] O `tea-lsp` já declara `selectionRange` no protocolo: a seleção pela sintaxe tem de onde vir.

### 3.2 Editar

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Apagar a palavra | Ctrl+Backspace/Delete | igual | igual | igual | **uma letra** [Fato] |
| Desfazer recuo | Shift+Tab | igual | igual | igual | **tira o foco do editor** [Fato] |
| Apagar a linha | Ctrl+Shift+K | Ctrl+Y | Ctrl+Shift+K | Ctrl+Shift+K | não (Ctrl+Shift+K é o hover aqui) |
| Juntar linhas | Ctrl+J (sem atalho no Linux) | Ctrl+Shift+J | Ctrl+J | Ctrl+J | não (Ctrl+J é "desdobrar tudo" no widget) |
| Maiúsculas/minúsculas, ordenar linhas | paleta | paleta | paleta | menu | não |
| Colar com o botão do meio (Linux) | sim | sim | sim | sim | **não** [Fato: o iced tem `clipboard::read_primary`] |

[Fato] Shift+Tab vira `FocusNavigationShiftTab`, que só tira o foco do editor (`handle_focus_navigation`): quem
aperta para desfazer um recuo perde o cursor.

### 3.3 Buscar no arquivo

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Regex | Alt+R | sim | sim | Alt+R | **não** [Fato] |
| Palavra inteira | Alt+W | sim | sim | Alt+W | **não** [Fato] |
| Só na seleção | Alt+L | sim | sim | Alt+S | **não** [Fato] |
| Busca já com a palavra do cursor | sim | sim | sim | sim | [Hipótese] não |

[Fato] O `SearchState` do widget tem `query`, `replace_with` e `case_sensitive`, e mais nada. A busca no
**projeto** (Ctrl+Shift+F) já tem regex e palavra inteira: dá para reaproveitar a mesma lógica.

### 3.4 Linguagem (LSP)

| | VS Code | JetBrains | Zed | Sublime (LSP) | IDE TEA |
|---|---|---|---|---|---|
| Correção rápida / ações de código | Ctrl+. | Alt+Enter | Ctrl+. | sim | **não** [Fato] |
| Realçar as outras ocorrências do símbolo | sim | sim | sim | sim | **não** [Fato] |
| Dicas no meio do código (tipos do Rust) | sim | sim | sim | sim | **não** [Fato] |

[Fato] Nada de `codeAction`, `documentHighlight` ou `inlayHint` no código. Renomear (F2), hover, completar,
definição, referências, implementação, formatar e a dica de parâmetros já existem. Para quem usa rust-analyzer, as
correções rápidas (importar, preencher os campos do `match`) e as dicas de tipo são o que mais faz falta.

### 3.5 Abas, janela e arquivos

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Zoom do editor | Ctrl+= / Ctrl+- / Ctrl+0 | Ctrl+roda (opção) | Ctrl+= / Ctrl+- | Ctrl+= / Ctrl+- | **não** [Fato] (só imagem e PDF) |
| Reabrir a aba fechada | Ctrl+Shift+T | Ctrl+Shift+T (recentes) | Ctrl+Shift+T | Ctrl+Shift+T | **não** [Fato] |
| Alternar entre as abas recentes | Ctrl+Tab | Ctrl+Tab | Ctrl+Tab | Ctrl+Tab | não (só em ordem, Ctrl+PageUp/Down) |
| Arrastar abas para reordenar | sim | sim | sim | sim | **não** [Fato] |
| Soltar um arquivo na janela para abrir | sim | sim | sim | sim | [Hipótese] não (o iced tem `FileDropped`) |
| Salvar sozinho (ao perder o foco ou depois de um tempo) | opção | padrão | opção | opção | **não** [Fato] |
| Régua de coluna (80, 100) | `editor.rulers` | sim | `wrap_guides` | `rulers` | não |

### 3.6 Árvore de arquivos

| | VS Code | JetBrains | Zed | Sublime | IDE TEA |
|---|---|---|---|---|---|
| Andar com as setas, Enter abre | sim | sim | sim | sim | **não** [Fato] |
| Arrastar para mover arquivos | sim | sim | sim | sim | não |
| Digitar para achar na árvore | sim | sim | — | — | não |

## 4. Conflitos de atalho [Fato]

Alguns atalhos do widget ocupam lugares que, nas outras IDEs, são de outra coisa:

| Atalho | No IDE TEA hoje | Nas outras |
|---|---|---|
| Ctrl+. | dobrar no cursor (widget) | correção rápida |
| Ctrl+K | dobrar tudo (widget) | começo de acordes no VS Code (Ctrl+K Ctrl+C…) |
| Ctrl+J | desdobrar tudo (widget) | juntar linhas (Zed, Sublime) |
| Ctrl+Shift+K | hover (IDE) | apagar a linha |
| Ctrl+Shift+T | terminal (IDE) | reabrir a aba fechada |

**[Recomendação]** Mover dobrar/desdobrar para os atalhos do VS Code (Ctrl+Shift+[ e Ctrl+Shift+]) e o hover para
Ctrl+K Ctrl+I, ou sem atalho (o mouse já mostra). A IDE intercepta as teclas antes do widget (`KeyGate`), então dá
para fazer sem esperar o widget. Quem já acostumou pode voltar no `[keys]`.

## 5. Onde cada coisa é feita

| Item | Widget (PR no `iced-code-editor`) | Fachada (`tea-editor`) | IDE (`tea-app`) |
|---|---|---|---|
| Palavra (andar, selecionar, apagar), Home esperto, Shift+Tab | **sim** (é a lógica do cursor) | dá, com o mesmo truque do Shift+clique, mas frágil | — |
| Busca: regex, palavra inteira, na seleção | sim | — | — |
| Selecionar a linha, todas as ocorrências, par do parêntese | sim | — | — |
| Seleção em coluna | sim | — | — |
| Ampliar seleção (LSP), correções rápidas, ocorrências, dicas de tipo | desenho no widget (dicas, realce) | pedido ao LSP | menu das correções |
| Colar com o botão do meio | — | sim (`read_primary`) | — |
| Zoom, reabrir aba, Ctrl+Tab, arrastar abas, soltar arquivo, salvar sozinho, régua | régua no widget | — | sim |
| Árvore pelo teclado | — | — | sim (`tea-ext-filetree`) |
| Atalhos em conflito | — | `KeyGate` | keymap padrão |

**[Recomendação]** Para o que é do widget, PRs pequenos, um assunto por PR (como o #36). Enquanto não sai versão
nova do widget, a IDE pode usar um `[patch.crates-io]` apontando para o branch do fork. Para a seleção feita de
fora (o Shift+clique, a seleção pelo LSP), um `set_selection` público, no mesmo espírito do #36.

## 6. Prioridades [Recomendação]

**Prioridade 1: 0.5.0, o básico que todo mundo estranha**

1. Palavra: andar, selecionar e apagar (Ctrl+←/→, Ctrl+Shift+←/→, Ctrl+Backspace/Delete). Widget.
2. Shift+Tab desfaz o recuo; Home esperto. Widget.
3. Busca no arquivo com regex, palavra inteira e "só na seleção". Widget.
4. Correções rápidas do LSP (Ctrl+.) e os atalhos em conflito (seção 4). IDE + `KeyGate`.
5. Zoom do editor (Ctrl+= / Ctrl+- / Ctrl+0, e Ctrl+roda), reabrir aba fechada. IDE.
6. Colar com o botão do meio no Linux. Fachada.

**Prioridade 2: 0.6.0**

7. Realçar as ocorrências do símbolo e dicas de tipo (inlay hints).
8. Selecionar a linha, todas as ocorrências, ir ao par do parêntese, ampliar a seleção (LSP `selectionRange`).
9. Ctrl+Tab pelas abas recentes, arrastar abas, soltar arquivo na janela.
10. Árvore pelo teclado.

**Prioridade 3: depois**

11. Seleção em coluna, apagar e juntar linhas, maiúsculas/minúsculas e ordenar linhas.
12. Salvar sozinho, régua de coluna, mover arquivos arrastando na árvore.

## 7. Como testar

Cada item com um teste do `update` (como o `shift_click_extends_the_selection`: mensagens simuladas, depois o
texto ou a posição do cursor) e o teste manual pelo painel. Para os atalhos, um teste do keymap padrão: nenhum
atalho da IDE e do widget no mesmo lugar sem ser de propósito.

## 8. Para decidir

1. As prioridades da seção 6 estão boas, ou algum item sobe ou desce?
2. O que é do widget: **PRs no `iced-code-editor`** (recomendado, com `[patch]` enquanto não sai versão) ou corrigir
   só na fachada da IDE?
3. Os atalhos em conflito (seção 4): mudar para o padrão do VS Code?
4. Por onde começar?

## Decisões (2026-11-06, pelo painel)

1. As prioridades da seção 6 ficam como estão.
2. O que é do widget vai em **PRs no `iced-code-editor`**, com `[patch]` para o fork enquanto não sai versão.
3. Os atalhos em conflito (seção 4) passam para o **padrão do VS Code**.
4. Começar por **palavra (andar, selecionar, apagar), Shift+Tab e Home**.

O usuário liberou os comandos do `cargo` para o agente compilar e testar o fork (o clone fica em
`~/NewProjects/Belabs/iced-code-editor`).

## Andamento

- **Palavra, Home, Shift+Tab:** feito. PR [#37](https://github.com/LuDog71FR/iced-code-editor/pull/37) no upstream
  (andar/selecionar/apagar por palavra, Home inteligente); o Shift+Tab já estava no `main` dele. A IDE usa o fork
  por `[patch.crates-io]`, fixado no commit `097ce7f`, até sair uma versão.
- **Atalhos no padrão do VS Code:** próximo.
