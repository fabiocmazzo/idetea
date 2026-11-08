<p align="center">
  <img src="assets/idetea.svg" width="112" alt="Ícone da IDE TEA: um copo de chá gelado">
</p>

<h1 align="center">IDE TEA</h1>

<p align="center"><b>Just code. Add ice.</b></p>

<p align="center">
  Uma IDE rápida e silenciosa, em Rust puro com <a href="https://github.com/iced-rs/iced">iced</a>,<br>
  feita para trabalhar lado a lado com agentes de código. Gratuita, sempre: <a href="LICENSE">Apache 2.0</a>.
</p>

<p align="center">
  <a href="README.md">English</a> · <b>Português</b> · <a href="README.es.md">Español</a><br>
  <a href="https://idetea.belabs.cloud/pt-br/"><b>Site e guias</b></a>
</p>

<p align="center"><img src="assets/demo.gif" width="886" alt="Um agente faz uma pergunta no painel Interfaces e depois pede um terminal para rodar os testes; o usuário responde e permite."></p>

---

A IDE TEA abre num piscar de olhos, não atrapalha e diz o que está fazendo. Sem conta, sem telemetria, sem
marketplace, sem pop-ups pedindo a sua atenção. Ela só usa a rede quando você pede (push, pull, fetch).

Também é uma IDE pensada para o jeito como muita gente programa hoje: **com um agente no terminal ao lado**. Em vez
de uma caixa de chat pendurada no canto, a IDE TEA dá aos agentes um protocolo pequeno, feito de arquivos, para
falar com você pela própria IDE, e deixa você no comando de tudo o que eles rodam.

> **Status:** em desenvolvimento ativo, antes da 1.0. Compilada e testada em Linux, macOS (Apple Silicon) e Windows (x86_64).
> Espere arestas, e por favor avise quando achar uma.

## Por que a IDE TEA

**Rápida.** Abre bem abaixo de 200 ms. Um arquivo de 50 MB abre em um décimo de segundo, e um arquivo minificado
com uma linha gigante abre num modo leve em vez de travar a janela.

**Silenciosa.** Uma janela, nenhum aviso que você não causou, nenhuma atualização surpresa. Aparência, atalhos e
painéis ficam num arquivo TOML simples (global ou por projeto), com uma tela de configurações para as mesmas coisas.

**Completa onde importa.** Editor com multi-cursor, zoom temporário e modo vim opcional, árvore de arquivos, busca no projeto,
terminal integrado com abas, git (stage, diff, commit, push e pull), LSP para as linguagens de sempre, debug com
gdb, e prévias de Markdown, SVG, imagens e PDF.

**Pequena por dentro.** O núcleo só sabe editar arquivos. O resto (árvore, terminal, git, debug…) é extensão que
você pode desligar.

**Feita para agentes, com você no comando.**

- **Perguntas num painel de verdade, não no chat.** O agente escreve um arquivo JSON pequeno e a IDE mostra como
  formulário: opções, caixas de marcar, campos de texto, imagens, sons, testes manuais passo a passo com botões de
  "aprovado" e "falhou". A sua resposta volta num arquivo que o agente lê.
- **Terminais que o agente pede e você aprova.** O agente pode pedir à IDE um terminal rodando um comando: um
  servidor de desenvolvimento, outro agente, uma ferramenta interativa. Você vê o comando inteiro antes de qualquer
  coisa rodar. Uma lista curta no *seu* config (nunca no do projeto) pode pular a pergunta para os comandos em que
  você confia.
- **Agentes que delegam para agentes.** Um agente ajudante pode rodar no próprio worktree do git, no próprio
  branch, para dois agentes nunca mexerem nos mesmos arquivos. A resposta do ajudante volta num arquivo; o código
  volta pelo branch e pela sua revisão de sempre.
- **Projetos novos que já começam com as skills certas.** "Projeto: novo…" cria a pasta, o git, as skills de agente
  da pilha (Rust, Python, FastAPI, site…) e um pedido inicial, e abre o agente num terminal da IDE para você aprovar.
- **Um comando para ensinar qualquer agente.** "Interfaces: ensinar os agentes deste projeto" grava o protocolo no
  `AGENTS.md` e no `CLAUDE.md`. Funciona com o Claude Code, o Codex e qualquer agente que leia esses arquivos.
- **Mascote reativo (Tea) que acompanha os agentes.** Uma xícara de chá em pixel art no canto da IDE (recolhível na
  barra de status) reage em tempo real ao que está acontecendo: avisa quando há pergunta esperando ou pedido de
  terminal (um clique abre o painel certo), ferve enquanto um agente trabalha, comemora quando termina, reage com
  curiosidade ou irritação se você mexer com ela no mouse, e aceita estados pelo terminal (`idetea mascot`) ou
  pacotes customizados (`mascot.toml`).

## Instalando

**Linux, macOS e Windows, pronta para rodar:** baixe o pacote do seu sistema (`.tar.gz` ou `.AppImage` para Linux x86_64/aarch64, `.tar.gz` com `IDE TEA.app` para macOS aarch64, ou `.zip` com `idetea.exe` para Windows x86_64) em
[Releases](https://github.com/fabiocmazzo/idetea/releases). No Linux, o `./install.sh` põe o `idetea` em
`~/.local/bin` e no menu de aplicativos; o AppImage roda direto (`chmod +x` antes) e `--install-desktop` põe no
menu. No macOS, `--install-desktop` cria `~/Applications/IDE TEA.app`. O `SHA256SUMS` tem as somas de verificação.

**Pelo código-fonte:** você precisa do [Rust](https://rustup.rs) (estável). Depois:

```sh
cargo install --locked --git https://github.com/fabiocmazzo/idetea tea-app
idetea --install-desktop   # opcional: entrada no menu de aplicativos e ícone
```

Opcionais: `git` para a integração com git, `gdb` 14+ (ou `lldb-dap` no macOS) para o debug, e os servidores de linguagem que você usa
(rust-analyzer, clangd, pyright, typescript-language-server, gopls e lua-language-server já vêm configurados).

## Assinatura de Código e Privacidade

A assinatura gratuita de código no Windows é fornecida por [SignPath.io](https://signpath.io), com certificado da
[SignPath Foundation](https://signpath.org). Este programa não transfere nenhuma informação para sistemas externos sem
solicitação explícita do usuário. Consulte [`CODE_SIGNING_POLICY.md`](CODE_SIGNING_POLICY.md).

## Começando

```sh
idetea                # abre a pasta atual como projeto
idetea src/main.rs    # pasta atual + este arquivo
idetea ~/projeto      # outra pasta
idetea new meu-app --modelo rust --agente claude   # um projeto novo, com um agente pronto para começar
```

Dentro da IDE: `Ctrl+Shift+P` abre a paleta de comandos, `Ctrl+P` abre um arquivo pelo nome, `F1` mostra todos os
atalhos.

## Sobre o nome

Começou como brincadeira. A IDE é feita com o **iced**, então ela tinha que ser **Iced Tea**.

Aí apareceu o outro sentido. Em português, **TEA** é a sigla de *Transtorno do Espectro Autista*. O autor é
autista, o duplo sentido pareceu certo e o nome ficou. Talvez isso explique algumas coisas da IDE: ela gosta de
previsibilidade, de texto simples que dá para ler e do mínimo de barulho possível.

Mais no [manifesto](MANIFESTO.pt-BR.md).

## Documentação

- [Manifesto](MANIFESTO.pt-BR.md): no que a IDE TEA acredita, e o que ela não vai fazer.
- [Funcionalidades e atalhos](docs/features.md): a lista completa, para quem quer o detalhe técnico (em inglês).
- [Interfaces para agentes](docs/interfaces/formato.md) e [terminais de agente](docs/terminais-de-agente.md).
- [SPEC.md](SPEC.md) e [DESENVOLVIMENTO.md](DESENVOLVIMENTO.md): especificação, arquitetura e como compilar,
  testar e escrever uma extensão.

## Licença

A IDE TEA é software livre sob a [Licença Apache 2.0](LICENSE): use, estude, modifique e distribua, inclusive
comercialmente. É gratuita e vai continuar sendo.

Os ícones da interface são da fonte [Codicons](https://github.com/microsoft/vscode-codicons) © Microsoft, sob
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) (ver
[`crates/tea-core/assets/NOTICE.md`](crates/tea-core/assets/NOTICE.md)).
