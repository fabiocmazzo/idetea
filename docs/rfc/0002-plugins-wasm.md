# RFC 0002 — Plugins WASM

- **Status:** aprovada em 2026-11-02. API 0.2 (`host.run` e `host.config`). A feature `plugins` continua fora
  do padrão até existir um plugin de verdade (ver "Decisões tomadas").
- **Fase do SPEC:** 4 — Plugins externos.
- **Depende de:** [RFC 0001](0001-interfaces-declarativas.md) (o painel de um plugin é um documento `idetea-ui`).

## Resumo

Um plugin é um **componente WASM** (`cargo build --target wasm32-wasip2`) com um `plugin.toml`, instalado em
`~/.local/share/idetea/plugins/<id>/`. A IDE carrega o plugin com o **wasmtime** e conversa com ele por um contrato
**WIT** versionado (`crates/tea-plugins/wit/plugin.wit`). Para a IDE, cada plugin é uma extensão como as de dentro:
registra comandos, recebe eventos, pode ter um painel. O painel é um documento `idetea-ui`, desenhado pelo mesmo
`tea-declarative-ui` do painel Interfaces.

O protótipo cobre o caminho inteiro com um plugin de exemplo (`examples/plugins/contador`): comandos na paleta,
painel com botões, eventos de arquivo salvo, leitura do projeto pelo WASI, e um plugin preso num laço infinito
sendo interrompido sem travar a IDE.

## O que o protótipo tem

| Peça | Onde |
|---|---|
| Contrato WIT `idetea:plugin@0.1.0` | `crates/tea-plugins/wit/plugin.wit` |
| Host: carregar, WASI, prazo, pedidos | `crates/tea-plugins/src/runtime.rs` |
| Manifesto `plugin.toml` | `crates/tea-plugins/src/manifest.rs` |
| Plugin como `Extension` (painel, comandos, eventos) | `crates/tea-plugins/src/extension.rs` |
| Descoberta dos plugins instalados | `crates/tea-plugins/src/lib.rs` |
| Ligação no app (feature `plugins`) | `crates/tea-app/src/main.rs` (`installed_plugins`) |
| Plugin de exemplo | `examples/plugins/contador/` |
| Medição | `cargo run --release -p tea-plugins --example medir -- <plugin.wasm>` |

### A API 0.2

O plugin **exporta** o que a IDE chama:

- `activate() -> result<_, string>`: uma vez, ao carregar. Erro: o plugin fica desligado, com a mensagem.
- `on-command(id)`: um comando do plugin foi escolhido.
- `on-event(event)`: `file-saved(caminho)` ou `active-file(option<caminho>)` (relativos à raiz),
  `run-finished(run-output)` (terminou um `host.run`) ou `config-changed`.
- `on-action(ui-action)`: um botão do painel foi clicado (`name`, `source` e `values` em JSON).

E **importa** `host`, o que ele pode pedir:

- `notify`, `register-command` (o id tem que começar com `<id do plugin>.`), `open-file`, `set-panel` (documento
  `idetea-ui`), `show-panel`, `log`.
- `config() -> string`: a tabela `[plugins.<id>]` do config (global + projeto, como o resto do config) em JSON;
  `{}` sem nada. Salvar o config manda `config-changed`.
- `run(id, command)`: roda um programa **sem esperar** (`program`, `args`, `cwd` relativo à raiz, `stdin`,
  `timeout-ms`); o resultado (`status`, `stdout`, `stderr`, `error`) chega em `run-finished` com o mesmo `id`.
  Ver a decisão 10.

Os pedidos são juntados durante a chamada e aplicados quando ela termina. Não há chamada da IDE de volta para o
plugin no meio de outra (sem reentrada), o que deixa o host simples e previsível.

### Manifesto

```toml
id = "contador"          # = nome da pasta; prefixo dos comandos
name = "Contador"
version = "0.1.0"
api = "0.2"              # versão do WIT; outra versão não carrega
wasm = "contador.wasm"   # na mesma pasta

[panel]                  # opcional
title = "Contador"
slot = "right"           # left, right, bottom
```

## Decisões propostas

1. **Component Model + WIT, não um ABI próprio.** O WIT é um contrato legível, versionado no nome do pacote, e o
   `wit-bindgen` gera os dois lados (Rust hoje; JS, Go, C e Python também compilam para componentes). Um ABI
   próprio sobre módulos WASM puros obrigaria a escrever serialização dos dois lados. Processos externos com
   JSON-RPC (como o LSP) eram a outra opção séria: isolam melhor e não pesam no binário, mas cada plugin seria um
   executável por plataforma. O WASM é um arquivo só, para todas as plataformas.
2. **WASI p2 com o projeto em `/project`, leitura e escrita.** Segue a decisão do SPEC: plugins têm confiança
   total (como no VS Code), sem sandbox de permissões. Mesmo assim, o protótipo só abre o que foi preciso: a pasta
   do projeto. Rede e processos ficam para quando um plugin pedir (processos não existem no WASI p2; seria uma
   função `host.run`).
3. **Prazo por chamada (interrupção por época).** 2 s para `activate`, 500 ms para as outras. Estourou ou deu
   trap: o plugin é desligado, com aviso e o motivo no painel; a IDE segue. Não religa sozinho (um plugin que
   trava uma vez tende a travar de novo). Medido: o laço infinito do exemplo é interrompido em ~500 ms.
4. **Chamadas na thread da interface, por enquanto.** Medido em release: carregar ~12–16 ms, `activate` ~2–3 ms,
   uma chamada típica abaixo de 1 ms (o clique do exemplo dá ~2 ms porque ele lista a pasta a cada redesenho).
   Com o prazo de 500 ms, o pior caso é um engasgo de meio segundo e o plugin desligado. Uma thread por plugin fica
   para quando um plugin real precisar de trabalho longo; a API (pedidos em lote) já permite a troca sem mudar o
   WIT.
5. **O painel é um documento `idetea-ui`.** Fecha o item "Protocolo de UI declarativa para painéis de plugins" do
   SPEC sem inventar um segundo formato: o plugin chama `set-panel(json)`, a IDE valida com os mesmos limites,
   guarda o que o usuário digitou e devolve o clique em `on-action`.
6. **Plugins só da pasta do usuário, nunca do projeto.** `~/.local/share/idetea/plugins/<id>/`
   (`IDETEA_PLUGINS_DIR` troca). Um repositório clonado não instala código na IDE; mesma regra das interfaces
   versionadas no git (RFC 0001).
7. **Ids.** O id é o nome da pasta, segue as regras dos nomes de interface e não pode ser o id de uma extensão de
   dentro nem o prefixo de um comando do núcleo (`file`, `editor`, `git`...): o comando iria para o lugar errado.
   Um plugin com problema no manifesto aparece desligado, com o motivo, em vez de sumir.
8. **Versão da API.** Na fase 0.x, o `api` do manifesto tem que ser igual (`0.1`). A partir da 1.0: semver; a IDE
   mantém o mundo da versão anterior junto (o `bindgen!` gera os dois) por pelo menos uma versão.
9. **Feature `plugins` fora do padrão até existir um plugin de verdade.** Ver custos e "Decisões tomadas".
10. **`host.run` é assíncrono.** A chamada só registra o pedido; o processo roda numa thread da IDE e o resultado
    volta como evento. Esperar dentro da chamada travaria a interface pelo tempo do programa (um formatador leva
    centenas de ms) e o prazo da chamada mataria o plugin. Limites: 4 processos por plugin ao mesmo tempo, prazo
    de 10 s (até 60 s), 1 MB de saída e de erro (o resto é lido e descartado, para o programa não travar no pipe
    cheio), `cwd` dentro do projeto. O processo roda num grupo próprio: estourado o prazo, morre o grupo inteiro
    (um `sh -c` leva os filhos). Sem shell no meio: `program` e `args` vão direto ao sistema. Como no resto da
    RFC, é confiança total (o plugin roda o que quiser), não sandbox; os limites protegem a IDE de um plugin com
    defeito.

## Medições

Máquina de desenvolvimento, Rust 1.98, wasmtime 50.0.1.

| O quê | Sem plugins | Com plugins |
|---|---|---|
| Binário `dist` (sem símbolos) | 44,3 MB | 61,9 MB (**+17,6 MB, +40%**) |
| Build `dist` do zero do que muda | 1m27s | 4m32s (wasmtime + cranelift) |
| Plugin de exemplo (`.wasm`, release, `opt-level = "s"`) | — | 85 KB |
| Carregar (compilar com cranelift + instanciar) | — | 12–16 ms |
| Abrir já compilado (`Component::deserialize`) | — | 0,4 ms |
| `activate` do exemplo | — | 2–3 ms |

O tokio entra pelo `wasmtime-wasi` (que roda o WASI síncrono sobre ele), mas não no nosso código, e já estava no
binário pelo terminal (`iced_term`).

## Custos e riscos

- **Tamanho e tempo de build.** +17,6 MB e ~3 min a mais num build do zero, quase tudo cranelift. A CI também fica
  mais lenta (o `tea-plugins` é membro do workspace). Opções, se pesar: (a) host de plugins num executável à parte
  (`idetea-plugins`), iniciado só quando há plugin instalado, falando com a IDE por um canal: o binário principal
  não muda de tamanho, ao custo de um processo e de serialização; (b) Pulley/Winch não ajudam no tamanho, porque
  ainda precisam do cranelift para gerar o código.
- **Ritmo do wasmtime.** Sai uma versão maior por mês. O WIT e o formato dos componentes são estáveis; a API do
  host muda às vezes (o protótipo já pegou uma: `FsPerms` em vez de `DirPerms`/`FilePerms`). Atualizar a cada
  poucas versões, não a cada uma.
- **Cache do compilado.** Abrir do cache é 30x mais rápido, mas `deserialize` é `unsafe`: o arquivo tem que ter
  vindo desta build. Com 12 ms por plugin, não vale ainda; se valer, o cache fica em `~/.cache/idetea/plugins/`
  com o hash do `.wasm` e da versão do wasmtime no nome.
- **Plugin malicioso.** Confiança total é decisão do SPEC. O que o host garante é só que um plugin com defeito não
  trava a IDE nem enche a memória (textos e documentos vindos do plugin têm tamanho máximo).

## Plano em partes (proposta para o SPEC)

1. **Host e WIT 0.1** — o protótipo: carregar, prazo, comandos, eventos, painel. Falta: revisão da API com um
   segundo plugin de verdade, e o texto para autores.
2. **API que um plugin real pede** — feito: config própria (`[plugins.<id>]`, `host.config`) e `host.run` (API
   0.2). Falta, quando um plugin pedir: pasta de dados (`/data`) e mais eventos (arquivo aberto, diagnósticos).
3. **Instalar e gerenciar** — "Plugins: instalar de uma pasta…" (copia e confere o manifesto), plugins listados em
   Configurações → Extensões com liga/desliga e o erro de cada um.
4. **Para autores** — `docs/plugins/`: como começar (o exemplo como modelo), a API, como depurar
   (`IDETEA_PLUGIN_LOG=1`).
5. **Decidir o padrão** — feature `plugins` no padrão, ou o host num executável à parte, conforme o uso.

## Decisões tomadas (2026-11-02)

Respondidas pelo painel Interfaces (o desenvolvimento da IDE usa a própria IDE).

1. **Feature no padrão:** ainda não. Decidir quando houver um plugin de verdade, com uso para medir; até lá, a
   feature `plugins` fica fora do padrão (nada muda para quem não a liga).
2. **Começo da parte 2:** `host.run` + config própria (API 0.2, feito). O resto da parte 2 (`/data`, mais
   eventos) espera um plugin que precise.
