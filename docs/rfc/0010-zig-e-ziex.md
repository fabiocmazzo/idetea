# RFC 0010 — Zig e Ziex

- **Status:** aprovada em 2026-11-06 (pelo painel), com as decisões do fim. Base: `main` em `c09451b`.
- **Origem:** pedido do usuário no chat (2026-11-06): "Suporte a Zig e ao Ziex (que aparece como extensão do Zed
  aqui https://github.com/ziex-dev/ziex/tree/main/ide/zed) qual dificuldade? Se for possível fazer até um PR no
  ziex para suporte ao idetea".

Marcadores: **[Fato]** conferido no código ou na fonte; **[Hipótese]** provável, não testado;
**[Recomendação]** proposta, ainda não decidida.

## Resumo

- **Zig: fácil.** As cores e o ícone já existem. Falta o servidor de linguagem (`zls`) entre os padrões, e tarefas
  e depuração que entendam o `build.zig`, como já entendem o `Cargo.toml`.
- **Ziex, servidor de linguagem: fácil.** Todos os editores rodam o mesmo `zx lsp`, e o config da IDE já aceita
  um servidor com comando, argumentos e extensões.
- **Ziex, cores dos `.zx`: média, e é o único trabalho de verdade.** A gramática do Ziex existe em tree-sitter
  (Zed, Helix, Neovim) e em TextMate (VS Code). A IDE usa o syntect, que lê `.sublime-syntax`. A gramática do VS
  Code é pequena e dá para converter. Mas o widget não deixa acrescentar sintaxes: isso pede um PR no
  `iced-code-editor`.
- **PR no Ziex: fácil**, se for só documentação: `ide/idetea/README.md`, como os guias de Helix e Emacs. Vale
  depois de uma versão da IDE com o suporte, para o guia apontar para ela.

## 1. O que a IDE já tem para Zig [Fato]

- **Cores:** a sintaxe Zig vem no `two-face` (a lista do `bat`), escolhida pela extensão `.zig`.
- **Ícone:** o Seti tem `zig` (`_zig` e `_zig_light`).
- **Servidor de linguagem:** o `zls` aparece só como exemplo comentado no config padrão
  (`# servers.zls = { command = "zls", languages = { zig = "zig" } }`). Os padrões de verdade
  (`tea-lsp::default_servers`) são rust-analyzer, clangd, pyright, typescript, gopls e lua.
- **Tarefas** (`tasks.rs`): Cargo, `package.json` e `Makefile`. Nada de `build.zig`.
- **Depuração** (`debug_build.rs`): compila sozinho só com `Cargo.toml`. Nos outros projetos, pergunta o
  executável. O depurador é o gdb, que lê o DWARF dos executáveis do Zig.
- Nesta máquina: `zig 0.17.0` e `zls 0.15.1`. **[Hipótese]** Um zls de outra versão do Zig funciona mal (o zls
  acompanha a versão do Zig). Isso é do ambiente, não da IDE.

## 2. Como o Ziex se integra aos editores [Fato]

Fonte: `ide/` no repositório `ziex-dev/ziex` (MIT), commit `f8387ea` (2026-11-06).

- **Linguagens:** `.zx` (Zig com marcação parecida com JSX) e `.mdzx` (Markdown com ZX).
- **Servidor:** `zx lsp`, o mesmo em todos os editores. Ele cuida da marcação (completar, hover, formatar,
  diagnósticos) e repassa o resto a um ZLS embutido. Sem o CLI `zx` no PATH, os editores compilam o servidor do
  próprio projeto: `zig build zx -Dziex-lsp=true --release=fast -- lsp`, na pasta do `build.zig`.
- **Extensão do Zed** (`ide/zed`): a gramática tree-sitter (`pkg/tree-sitter-zx`, `pkg/tree-sitter-mdzx`), as
  queries e um código Rust de cerca de 100 linhas que escolhe o comando: o binário do config, senão `zx lsp`,
  senão `zig build ...` quando o `build.zig.zon` da raiz depende de `ziex`. As configurações do ZLS vão na seção
  `zls`.
- **VS Code:** gramáticas TextMate (`zx.tmLanguage.json`, 12 KB, 48 expressões, 15 regras; `mdzx`;
  `zx-injection`). A `zx` inclui `source.zig`. A injeção põe a marcação dentro do Zig.
- **Helix e Emacs:** só um guia (`README.md`) com o config a copiar. É o modelo para a IDE.
- **Versões:** Ziex 0.1.0 pede Zig 0.17.x.

## 3. O que falta e quanto custa

### 3.1 Zig

| Item | Onde | Tamanho |
|---|---|---|
| `zls` entre os servidores padrão | `tea-lsp::default_servers` (uma linha) | pequeno |
| Tarefas do `build.zig`: `zig build`, `zig build test`, `zig build run` e os passos de `zig build -l` | `tasks.rs` | pequeno |
| Depurar sem configurar: `zig build` e o executável em `zig-out/bin/` (se for um só; senão pergunta) | `debug_build.rs` | médio |
| Problemas da compilação na aba Problemas (formato `arquivo:linha:coluna: error:`) | **[Hipótese]** o leitor do gcc/clang já serve | a conferir |

### 3.2 Ziex: servidor de linguagem

Hoje já funciona sem mudar nada, só com o config do projeto (`.idetea/config.toml`):

```toml
[lsp.servers.zxls]
command = "zx"
args = ["lsp"]
languages = { zx = "zx", mdzx = "mdzx" }
```

**[Recomendação]** Pôr o `zxls` entre os padrões, com a mesma escolha do Zed: `zx lsp` se o `zx` estiver no PATH;
senão, se o `build.zig.zon` da raiz depender de `ziex`, `zig build zx -Dziex-lsp=true --release=fast -- lsp`.
Assim, quem abre um projeto Ziex não precisa configurar nada.

### 3.3 Ziex: cores dos `.zx` e `.mdzx`

O caminho:

1. **Converter** `zx.tmLanguage.json` e `mdzx.tmLanguage.json` para `.sublime-syntax`. São pequenas. **[Fato]** Não
   usam `while`, que o syntect não tem. Também não usam `\G`, `\h`, `\p{}` nem grupos atômicos; usam lookbehind, que
   o `fancy-regex` (o motor do widget) aceita. A licença é MIT.
2. **A injeção:** a marcação aparece dentro de expressões Zig (`return (<main>...)`). O VS Code resolve isso
   injetando a gramática no Zig, e o syntect não tem injeção. **[Hipótese]** O `with_prototype` de um
   `embed: scope:source.zig` põe as regras do ZX em todos os contextos do Zig, que é como as sintaxes de ERB e PHP
   fazem, e o syntect suporta isso. Precisa de um teste com os exemplos do Ziex.
3. **Carregar a sintaxe:** **[Fato]** O widget monta o conjunto de sintaxes uma vez, num `OnceLock`, só com o
   `two-face`, e não tem como acrescentar outras. Precisa de um PR no `iced-code-editor`, por exemplo uma função
   para registrar sintaxes extras antes do primeiro uso. Com isso, a IDE pode também ler sintaxes do usuário
   (`~/.config/idetea/syntaxes/*.sublime-syntax`), o que serve para qualquer linguagem que o `bat` não tenha.

Sem o passo 3, dá para fazer o `.zx` usar as cores do Zig (uma linha em `syntax_for`). O Zig fica colorido, e a
marcação fica com cores erradas ou sem cor. Serve de quebra-galho até o PR.

**Ícone:** o Seti não conhece `.zx`. O mais próximo é usar o ícone do Zig para `zx` e `mdzx`, na tabela própria
da IDE.

## 4. PR no Ziex

**[Recomendação]** Um PR só de documentação, no mesmo formato dos guias de Helix e Emacs:

- `ide/idetea/README.md`: o que a IDE já faz sozinha (servidor e cores, depois deste RFC), o config para quem não
  tem o `zx` no PATH, e como passar as opções do ZLS. **[Hipótese]** Isso pede que o config da IDE aceite
  `initializationOptions` ou `settings` por servidor, que hoje não aceita. Sem isso, o guia só cita os padrões.
- Uma linha em `ide/README.md` com o link.

O momento certo é depois de a IDE publicar uma versão com o suporte, para o guia não descrever algo que ainda não
saiu. A sintaxe convertida fica na IDE (embutida, MIT, com aviso no `NOTICE.md`), não no Ziex: é mais fácil de
aceitarem só um guia do que uma terceira gramática para eles manterem.

## 5. Etapas [Recomendação]

1. **Zig:** `zls` padrão, tarefas do `build.zig` e depuração com `zig-out/bin`.
2. **Ziex sem cores próprias:** `zxls` padrão, `.zx`/`.mdzx` com as cores e o ícone do Zig.
3. **Cores do ZX:** o PR no widget para sintaxes extras, a conversão das gramáticas e o teste com os exemplos do
   Ziex (`site/pages/examples`). Junto, a pasta de sintaxes do usuário.
4. **PR no Ziex**, depois da versão da IDE com as etapas 1 a 3.

## 6. Para decidir

1. Fazer **Zig e Ziex** juntos, ou **só Zig** primeiro?
2. Cores do ZX: **converter e embutir** (com o PR no widget), ou ficar com as **cores do Zig** por enquanto?
3. Junto com o item 2, a **pasta de sintaxes do usuário**?
4. **PR no Ziex** só com o guia, depois da versão da IDE?

## Decisões (2026-11-06, pelo painel)

1. **Zig e Ziex juntos.**
2. Cores do ZX: **converter a gramática e embutir**, com o PR no `iced-code-editor` para sintaxes extras.
3. **Com** a pasta de sintaxes do usuário.
4. **PR no Ziex só com o guia**, depois da versão da IDE.

## Andamento

- **Etapas 1 e 2:** feitas (`zls` e `zxls` padrão, com o servidor do projeto quando falta o `zx`; tarefas do
  `build.zig`; F5 com `zig build` e `zig-out/bin`; `.zx`/`.mdzx` com o ícone e, por enquanto, as cores do Zig).
- **Etapa 3:** feita. **[Fato]** Juntar sintaxes ao conjunto do `two-face` religa todas (~400 ms em release, contra
  ~20 ms do conjunto sozinho): as registradas ficam num conjunto à parte, e por isso a do Zig que o ZX inclui vem
  junto. **[Fato]** O `with_prototype` resolveu a injeção, como previsto; faltou só tirar a marcação de dentro das
  strings do Zig, que o VS Code exclui no `injectionSelector`.
- **Etapa 4** (PR no Ziex): depois da versão da IDE.
