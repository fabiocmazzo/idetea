<p align="center">
  <img src="assets/idetea.svg" width="112" alt="Ícone da IDE TEA: um copo de chá gelado">
</p>

<h1 align="center">IDE TEA</h1>

<p align="center"><b>Just code. Add ice.</b></p>

<p align="center">
  Uma IDE rápida e sem firulas, feita em Rust com <a href="https://github.com/iced-rs/iced">iced</a>.<br>
  Livre e gratuita, sempre. Licença <a href="LICENSE">Apache 2.0</a>.
</p>

---

O nome é um trocadilho com *Iced Tea*: a IDE é feita com o iced.

## A ideia

- **Rápida.** Abre em dezenas de milissegundos (a meta é ficar abaixo de 200 ms) e edita arquivos de 200 mil linhas sem travar.
- **Sem firulas.** Editor, árvore de arquivos, busca, terminal, git, LSP e debug. Sem conta, sem telemetria, sem marketplace.
  A IDE só usa a rede quando você pede (push, pull e fetch).
- **Pequena por dentro.** O núcleo só sabe editar arquivos; o resto (árvore, terminal, git...) são extensões, e cada uma
  pode ser desligada.
- **Do seu jeito.** Tema, fonte, atalhos e posição dos painéis ficam num arquivo TOML simples, global ou por projeto,
  e também numa tela de configurações.

> **Status:** em desenvolvimento ativo, antes da versão 1.0. Feita e testada no Linux (Wayland/COSMIC);
> macOS e Windows ainda não foram testados.

## Funcionalidades

### Editor
- Abrir arquivo pelo nome (`Ctrl+P`) com busca aproximada; `main.rs:42` abre direto na linha.
- Ir para um símbolo (função, tipo, método…) do arquivo (`Ctrl+Shift+O`) ou do projeto (`Ctrl+T`).
- Várias abas, indicador de alteração, abrir (`Ctrl+O`), salvar e salvar como, com a janela de arquivos do sistema.
- Syntax highlight para dezenas de linguagens, temas claro e escuro (e os temas do iced: Nord, Dracula...).
- Buscar e substituir, ir para linha, desfazer/refazer, multi-cursor, dobrar código e modo vim opcional.
- Prévia de Markdown e SVG ao lado do editor (`Ctrl+Shift+V` ou o botão na barra de abas), atualizada enquanto você digita.
- Imagens (png, jpg, gif, webp, bmp, ico...) abrem numa aba, com zoom pela roda do mouse e arrastar.
- PDF abre numa aba (desenhado pelo `hayro`, puro Rust), com zoom e ajuste à largura.
- Preserva CRLF. Arquivo que não é UTF-8 não abre, com aviso, em vez de ser corrompido.

### Projeto
- **Árvore de arquivos:** criar, renomear e apagar (vai para a lixeira), revelar o arquivo atual e cores do git
  (modificado, novo, conflito, ignorado).
- **Busca no projeto** enquanto você digita, respeitando o `.gitignore`, com regex, palavra inteira e maiúsculas/minúsculas.
- **Menu do botão direito** na árvore, nas abas e nos arquivos do git.

### Linguagens (LSP)
- Erros e avisos (painel de problemas e contagem na barra de status), hover, autocomplete, ir para definição e formatar.
- Ir para implementação (`Ctrl+F12`) e para o tipo; com mais de um resultado, a lista vai para o painel.
- Onde é usado (`Shift+F12`, ou `Ctrl+clique` na declaração) num painel agrupado por arquivo, e voltar/avançar
  depois de cada salto (`Alt+←` / `Alt+→`, ou os botões laterais do mouse).
- Quem chama uma função (`Alt+Shift+H`), em árvore que abre aos poucos, e o que ela chama.
- Painel Estrutura (`Alt+7`): funções, structs, métodos e campos do arquivo em árvore, mostrando o que é público,
  privado ou restrito (`pub(crate)`, `protected`...), com o símbolo do cursor destacado.
- Já configurados: rust-analyzer, clangd, pyright, typescript-language-server, gopls e lua-language-server.
  Basta ter o servidor instalado; dá para trocar o comando ou adicionar outros.

### Terminal
- Terminal integrado (alacritty) no painel de baixo, com as cores e a fonte do tema.
- Os atalhos do shell (`Ctrl+C`, `Ctrl+W`...) vão para o shell; os atalhos da IDE com `Ctrl+Shift` continuam valendo.

### Git
- Branch, commits à frente/atrás e número de mudanças na barra de status.
- Painel com stage/unstage por arquivo ou tudo, diff colorido, commit, descartar alterações (com confirmação).
- Push, pull e fetch. A IDE nunca fica esperando uma senha: usa o agente ssh ou o credential helper do seu git.
- Usa o `git` instalado, então respeita sua config, seus hooks e suas credenciais.

### Debug
- Breakpoints com um clique no número da linha (`F9`), e eles andam junto com o texto quando você edita.
- `F5` compila e roda: em projetos Rust usa o `cargo build` e acha o executável sozinho; nos outros, um `[debug]`
  no config diz o programa, os argumentos e o comando de build.
- Passo a passo (`F10`, `F11`, `Shift+F11`), pausar e parar, com a linha atual marcada no editor.
- Painel com a pilha de chamadas (clique para ir ao quadro), variáveis que abrem como árvore e a saída do programa.
- Usa o `gdb` 14 ou mais novo (Rust, C e C++).

### Interface
- Paleta de comandos com todos os comandos e seus atalhos (`Ctrl+Shift+P`).
- Ajuda (`F1`) com todos os atalhos atuais e um guia curto.
- Configurações (`Ctrl+,`): aparência, extensões, atalhos (grava a tecla que você apertar) e servidores LSP.
- Barra de ícones para mostrar e esconder painéis, painéis redimensionáveis com o mouse.

### Sem perder trabalho
- Fechar uma aba ou sair com alterações sempre pergunta antes.
- Apagar arquivo vai para a lixeira do sistema.
- Percebe sozinha arquivos mudados por fora (terminal, `cargo fmt`, outro editor, `git checkout`): a árvore e o git
  atualizam, abas sem alterações recarregam e abas com alterações são mantidas, com um aviso.

## Instalando

Linux x86_64. Escolha um:

- **AppImage:** baixe o `IDE_TEA-<versão>-x86_64.AppImage` em [Releases](https://github.com/fabiocmazzo/idetea/releases),
  `chmod +x` e rode. Para pôr no menu de aplicativos: `./IDE_TEA-*.AppImage --install-desktop`.
- **.tar.gz:** baixe o `idetea-<versão>-x86_64-linux.tar.gz`, extraia e rode `./install.sh` (põe o `idetea` em
  `~/.local/bin` e a IDE no menu de aplicativos).
- **Com o Rust** ([rustup](https://rustup.rs), estável):
  ```sh
  cargo install --locked --git https://github.com/fabiocmazzo/idetea tea-app
  idetea --install-desktop   # menu de aplicativos e ícone
  ```

Para tirar do menu: `idetea --uninstall-desktop`.

## Começando

```sh
idetea                # abre a pasta atual como projeto
idetea src/main.rs    # pasta atual + este arquivo
idetea ~/projeto      # outra pasta
```

Aberta pelo menu de aplicativos, abre o último projeto (ou, ao abrir um arquivo pelo gerenciador de arquivos, o
repositório dele).

Para compilar daqui: `cargo build --release` (executável em `target/release/idetea`). Com o
[`just`](https://github.com/casey/just), `just run` também instala o ícone para o painel do seu desktop.

Opcionais: `git` (para a integração com git) e os servidores LSP das linguagens que você usa.

### Atalhos principais

| Atalho | O quê |
|---|---|
| `Ctrl+P` | Abrir arquivo pelo nome |
| `Ctrl+Shift+P` | Paleta de comandos |
| `F1` | Ajuda: todos os atalhos e o guia |
| `Ctrl+,` | Configurações |
| `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S` | Abrir / salvar / salvar como |
| `Ctrl+B` | Mostrar/esconder a árvore de arquivos |
| `Ctrl+Shift+F` | Buscar no projeto |
| `Ctrl+Shift+T` | Terminal |
| `Ctrl+Shift+G` | Git |
| `Ctrl+Shift+M` / `F8` | Problemas / próximo problema |
| `F12` | Ir para a definição |

Todos podem ser trocados nas configurações ou em `~/.config/idetea/config.toml`.

## Roadmap

- [x] **Fase 0 — Spike:** validar o editor com arquivos grandes e o modelo de extensões.
- [x] **Fase 1 — MVP:** abas, salvar, temas, paleta, config em TOML, árvore de arquivos, barra de status.
- [x] **Fase 2 — IDE:** busca no projeto, LSP, painel de problemas, terminal integrado.
- [x] **Fase 3 — Git e polimento:** git completo (incluindo push/pull), menus de contexto, ajuda, configurações,
      ícones, painéis redimensionáveis, janelas nativas de arquivo.
- [ ] **Próximo — mais polimento:**
  - outras melhorias de uso do dia a dia.
- [ ] **Fase 4 — Plugins:** plugins de terceiros em WASM, sem recompilar a IDE.
- [x] **Fase 5 — Debug:** breakpoints, passo a passo, pilha e variáveis com o gdb.
- [ ] **Depois do debug:** outros adaptadores (codelldb, debugpy, delve...).

**Mais para frente:**
- sublinhado de erros e marcas do git dentro do editor (dependem do widget de edição);
- formatar ao salvar;
- outros encodings além de UTF-8.

## Documentação

- [SPEC.md](SPEC.md): especificação, arquitetura, decisões e o roadmap detalhado, fase a fase.
- [DESENVOLVIMENTO.md](DESENVOLVIMENTO.md): como compilar, testar e rodar, a estrutura das crates e como escrever uma extensão.

## Licença

IDE TEA é software livre sob a [Licença Apache 2.0](LICENSE): pode usar, estudar, modificar e distribuir, inclusive comercialmente.
O projeto é e vai continuar sendo gratuito.

Os ícones da interface são da fonte [Codicons](https://github.com/microsoft/vscode-codicons) © Microsoft,
sob [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) (ver [`crates/tea-core/assets/NOTICE.md`](crates/tea-core/assets/NOTICE.md)).
