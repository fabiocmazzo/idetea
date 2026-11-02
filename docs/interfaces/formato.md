# Interfaces declarativas: o formato `idetea-ui` v1

Um agente (ou qualquer programa) escreve um arquivo JSON e a IDE TEA mostra uma interface pequena no painel
**Interfaces**. Quando o usuário clica num botão, a IDE grava a resposta num arquivo que o agente lê. Nada roda
código: o documento só descreve textos, botões, caixas de marcar, listas de escolha e campos de texto.

As decisões e o porquê estão em [`docs/rfc/0001-interfaces-declarativas.md`](../rfc/0001-interfaces-declarativas.md).
O JSON Schema está em [`idetea-ui.schema.json`](idetea-ui.schema.json); a IDE grava uma cópia em
`.idetea/ui/idetea-ui.schema.json`, ao lado do `FORMAT.md`, para validar o documento antes de publicar (ex.:
`check-jsonschema --schemafile .idetea/ui/idetea-ui.schema.json .idetea/ui/<nome>.json`, se houver um validador).
Os outros links deste texto (RFCs, exemplos) só funcionam no repositório da IDE.

## Ligar

A extensão vem desligada. Em `~/.config/idetea/config.toml` (ou `.idetea/config.toml` do projeto):

```toml
[extensions]
interfaces = { enabled = true }
```

Ou em Configurações → Extensões. Reinicie a IDE depois de mudar.

Para o agente saber usar o painel, rode **Interfaces: ensinar os agentes deste projeto**: grava as instruções de
[`agentes.md`](agentes.md) no `AGENTS.md` do projeto.

## Pastas

```
.idetea/ui/
├── .gitignore                       # criado pela IDE, com "*": nada daqui vai para o git
├── FORMAT.md                        # este texto, da versão da IDE aberta (a IDE regrava)
├── idetea-ui.schema.json            # o JSON Schema do documento, da mesma versão
├── <nome>.json                      # o documento (escrito pelo agente)
├── rascunhos/                       # opcional: arquivos que o agente cria para o usuário editar
└── responses/<nome>/
    └── 000001-<event>.json          # as respostas (escritas pela IDE)
```

- `<nome>`: minúsculas, números, `_` e `-`, começando por letra ou número, até 64 caracteres. Outros nomes são
  ignorados.
- **Um escritor por arquivo.** O agente escreve e apaga `<nome>.json`; a IDE escreve as respostas; o agente
  apaga cada resposta depois de ler — é assim que a IDE sabe que ele recebeu.
- Escreva o documento num temporário e renomeie (`mv`): a IDE nunca vê metade do arquivo. Temporários que
  terminam em `.tmp` são ignorados (e apagados depois de uma hora).
- Para fechar a interface, apague `<nome>.json`. O resto (`.gitignore`, `FORMAT.md`, `idetea-ui.schema.json`,
  `responses/`) é da IDE: a pasta de
  respostas de uma interface fechada sai sozinha quando fica vazia.
- Um documento só com `text`, sem botões, serve para mostrar algo sem esperar resposta.
- Arquivos versionados no git (que vieram de um repositório clonado) são ignorados por segurança. Symlinks
  também.

## O documento

```json
{
  "format": "idetea-ui",
  "version": 1,
  "surface": "revisar-migracao",
  "revision": 1,
  "title": "Revisar a migração",
  "source": "agente de banco de dados",
  "data": { "backup": false, "ambiente": "staging" },
  "components": [
    { "id": "aviso", "component": "text", "text": "A migração apaga a coluna antiga.", "style": "warning" },
    { "id": "amb", "component": "select", "label": "Ambiente", "value": "/ambiente",
      "options": [ "staging", { "value": "prod", "label": "Produção" } ] },
    { "id": "backup", "component": "checkbox", "label": "Fiz backup", "value": "/backup" },
    { "id": "nota", "component": "input", "label": "Observação", "placeholder": "opcional", "value": "/nota" },
    { "id": "aplicar", "component": "button", "label": "Aplicar",
      "action": { "name": "aplicar", "context": ["/ambiente", "/backup", "/nota"] } },
    { "id": "cancelar", "component": "button", "label": "Cancelar", "style": "secondary",
      "action": { "name": "cancelar" } }
  ]
}
```

| Campo | Obrigatório | O que é |
|---|---|---|
| `format` | sim | Sempre `"idetea-ui"`. |
| `version` | sim | `1`. Uma versão maior que a IDE não conhece é recusada. |
| `components` | sim | A lista de componentes, de cima para baixo (até 200). |
| `surface` | não | Se vier, tem que ser igual ao nome do arquivo. |
| `revision` | não | Número que cresce a cada mudança do documento (padrão `0`). Ver *Revisões*. |
| `title` | não | Título no painel (padrão: o nome do arquivo). |
| `source` | não | Quem criou, mostrado ao lado do caminho. |
| `data` | não | Objeto com os valores iniciais dos campos (até 64 KB, 16 níveis). |
| `submit` | não | `"once"` (padrão): depois de um clique, os botões ficam desligados até a próxima revisão. `"many"`: aceita vários cliques. |
| `comment` | não | `true` (padrão): a IDE põe um campo "comentário para o agente" logo antes dos botões do fim; o texto vai na resposta. `false` tira o campo. |
| `$schema` | não | Ignorado. |

Campos desconhecidos viram um aviso no painel, não um erro. Um componente desconhecido aparece como aviso no lugar
dele; o resto da interface continua.

### Componentes

Todo componente tem `id` (`[A-Za-z0-9_-]`, até 64, único no documento) e `component`.

| `component` | Campos | Notas |
|---|---|---|
| `text` | `text`, `style` | `style`: `normal` (padrão), `muted`, `warning`, `error`. |
| `button` | `label`, `action`, `style`, `requires` | `style`: `primary` (padrão), `secondary`, `danger`. Botões seguidos ficam na mesma linha. `requires`: o `value` de um `steps`; o botão fica desligado até os passos obrigatórios estarem marcados. |
| `checkbox` | `label`, `value` | Valor booleano. |
| `select` | `options`, `value`, `label` | `options`: até 500, cada uma `"x"` ou `{"value": "x", "label": "X"}`. Valor: o `value` escolhido. |
| `input` | `value`, `label`, `placeholder` | Texto de uma linha. |
| `file` | `path`, `line`, `column`, `label`, `open` | Um arquivo do projeto. Ver [Arquivos](#arquivos). |
| `markdown` | `text` | Texto formatado: listas, `código`, ênfase, títulos, links. Ver [Markdown](#markdown). |
| `image` | `path`, `label`, `pixelated`, `background`, `height` | Uma imagem do projeto dentro do painel. Ver [Imagens](#imagens). |
| `gallery` | `items`, `select`, `value`, `pixelated`, `background`, `height` | Imagens lado a lado, para comparar ou escolher. Ver [Imagens](#imagens). |
| `steps` | `items`, `value` | Passos numerados que o usuário marca. `items`: até 500, cada um `{"id": "x", "text": "…", "optional": true}` (`text` em Markdown de uma linha; `optional` não conta para o `requires`). Valor: a lista dos ids marcados, na ordem dos passos. |

`value` é o lugar do valor em `data`: um ponteiro JSON (RFC 6901) como `"/usuario/nome"`, ou
`{"path": "/usuario/nome"}`.

`action` é `{"name": "...", "context": ["/caminho", ...]}`: o nome volta na resposta, junto com o valor atual de cada
caminho do `context`.

Uma entrega de teste: os passos se marcam, e "Aprovado" espera todos (o "Falhou" não espera nada):

```json
{"id": "passos", "component": "steps", "value": "/feitos", "items": [
  {"id": "copiar", "text": "Clique em **Copiar** e cole num terminal"},
  {"id": "ensinar", "text": "Paleta: `Interfaces: ensinar os agentes`"},
  {"id": "reiniciar", "text": "Reinicie a IDE", "optional": true}]},
{"id": "ok", "component": "button", "label": "Aprovado", "requires": "/feitos",
 "action": {"name": "aprovado", "context": ["/feitos"]}},
{"id": "falhou", "component": "button", "label": "Falhou", "style": "danger",
 "action": {"name": "falhou", "context": ["/feitos"]}}
```

### Limites

256 KB por arquivo, 200 componentes, 16 KB por texto, 500 opções, 32 imagens, 64 KB e 16 níveis em `data`. Acima disso o
documento é recusado e o erro aparece no painel com o campo (ex.: `components[3].value`).

## Arquivos

O componente `file` mostra um arquivo do projeto no painel. Serve para o que não cabe num campo: um trecho de
código para olhar, um texto longo para escrever (mensagem de commit, descrição de PR, um rascunho) ou um arquivo de
configuração para ajustar.

```json
{"id": "msg", "component": "file", "path": ".idetea/ui/rascunhos/commit.md", "line": 3, "column": 1,
 "label": "Edite a mensagem do commit", "open": true}
```

- `path`: relativo à raiz do projeto, sem `..` nem caminho absoluto (o documento é recusado).
- `line` e `column` começam em 1; sem eles (ou `0`), o arquivo abre no início.
- `label`: o texto do link (o caminho aparece embaixo). Sem ele, o link é o caminho.
- Um clique no link abre o arquivo na posição.
- `open: true`: abre sozinho quando a revisão chega (vários com `open`: uma aba cada, a última na frente; use
  em um só). Se o usuário digitou no editor nos últimos 2 segundos, não
  abre (o cursor dele não é roubado no meio de uma linha): só aparece um aviso, e o link continua no painel.
- **O arquivo é a resposta.** Ao clicar num botão, a IDE primeiro salva os arquivos dos componentes `file` que
  estão abertos com alterações, e só então grava a resposta. Ao ver a resposta, leia o arquivo do disco. Se algum
  não puder ser salvo, a resposta não é gravada e o usuário vê o erro.
- A IDE não cria o arquivo: escreva-o antes do documento (um arquivo que não existe abre como aba nova, e é criado
  quando o usuário salvar). Para rascunhos, use `.idetea/ui/rascunhos/`: fica fora do git (o `.gitignore` da
  pasta) e a IDE não mexe nele. Apague-os ao terminar.

**Mídia.** O que acontece ao abrir cada tipo de arquivo (pelo `file` ou por um link do `markdown`):

| Arquivo | Abre como |
|---|---|
| Imagem (`png`, `jpg`, `gif`, `webp`, `bmp`, `ico`, `tif`, `tga`, `qoi`) | aba de imagem: zoom com a roda do mouse, arrastar para mover, tamanho em pixels; até 256 px de lado, sem borrar (pixel art) |
| `pdf` | aba de PDF |
| `svg` | texto, com prévia |
| Áudio, vídeo e outros binários | não abrem: aparece o erro "arquivo não é UTF-8" |

Para mostrar imagens dentro do painel, use `image` e `gallery` ([Imagens](#imagens)); uma imagem no `markdown`
aparece só como o texto alternativo. Áudio ainda não toca no painel: peça ao usuário para tocar o arquivo fora da
IDE (um bloco de código com o comando tem o botão Copiar). Use um tocador que exista na máquina: `paplay`,
`pw-play` ou `aplay` no Linux, `afplay` no macOS (`command -v` diz qual). Uma aba de imagem aberta recarrega
quando o arquivo muda no disco: dá para regerar e olhar de novo.

## Imagens

```json
{"id": "balao", "component": "image", "path": "assets/sprites/balao.png", "label": "Balão (3 quadros)",
 "pixelated": true, "background": "checker", "height": 96}

{"id": "passaro", "component": "gallery", "select": "one", "value": "/passaro",
 "pixelated": true, "background": "checker", "height": 64, "items": [
   {"id": "a", "path": "assets/sprites/passaro-a.png", "label": "A"},
   {"id": "b", "path": "assets/sprites/passaro-b.png", "label": "B"}]}
```

- `path`: como no `file`, relativo à raiz e sem `..`. Symlinks não são seguidos.
- `pixelated: true`: amplia com o vizinho mais próximo (pixel art sem borrar).
- `background`: `checker` (quadriculado, mostra a transparência) ou `none` (padrão).
- `height`: altura na tela, em pixels (8 a 1024); a largura segue a proporção, até 640. Sem ele, a imagem fica no
  tamanho dela, até 160 de altura.
- Um clique na imagem abre a aba de imagem (zoom e arrastar).
- **A imagem é relida quando o arquivo muda:** regere e o painel mostra a nova, sem revisão nova do documento.
- `gallery`: as imagens lado a lado, com o rótulo (`label`, senão o `id`) embaixo de cada uma, quebrando a linha
  quando não cabem. `select`: `none` (padrão, só para olhar, ex. antes e depois), `one` (o valor em `value` é o
  `id` escolhido, ou `null`) ou `many` (o valor é a lista dos `id` marcados, na ordem dos itens).
- Limites: 8 MB por arquivo, 4096 px de lado, 32 imagens e 64 milhões de pixels somados por documento. Acima disso
  (ou se não der para ler), o motivo aparece no lugar da imagem; o resto do documento continua.
- Formatos: os da aba de imagem (`png`, `jpg`, `gif`, `webp`, `bmp`, `ico`, `tif`, `tga`, `qoi`). Um GIF mostra o
  primeiro quadro.
- Nos painéis de plugins as imagens ainda não aparecem (só o aviso).

## Markdown

O componente `markdown` serve para o que um `text` faz mal: passos numerados, comandos, explicações mais longas.

```json
{"id": "passos", "component": "markdown",
 "text": "1. Abra [o config](idetea:config) e salve.\n2. Rode `cargo test`.\n3. Confira [a linha 42](src/main.rs:42)."}
```

Os links levam a três lugares; qualquer outro só mostra um aviso:

- **Web:** `http://`, `https://` e `mailto:` abrem no navegador.
- **Arquivos do projeto:** `src/main.rs`, `src/main.rs:42` ou `src/main.rs:42:5` abrem na IDE, na posição. Mesma
  regra do `file`: relativo à raiz, sem `..`.
- **Config da IDE:** `idetea:config` (o global) e `idetea:config-project` (o do projeto). É o único jeito de
  apontar para um arquivo fora do projeto, e só para esses dois.

Blocos de código (entre ```` ``` ````) ganham um botão **Copiar** no canto: ponha num bloco os comandos que o
usuário deve rodar. Imagens não são carregadas (aparece o texto alternativo). Tabelas e HTML aparecem como texto.

## A resposta

Um clique grava `.idetea/ui/responses/<nome>/<seq>-<event>.json`:

```json
{
  "format": "idetea-ui-response",
  "version": 1,
  "surface": "revisar-migracao",
  "revision": 1,
  "event": "19a4c2f81b0-3e8f-1",
  "seq": 1,
  "action": "aplicar",
  "source": "aplicar",
  "time": "2026-11-01T20:41:07Z",
  "values": { "/ambiente": "prod", "/backup": true, "/nota": null },
  "comment": "faça o backup antes, mesmo em staging",
  "ide": { "pid": 48213, "version": "0.0.1", "outdated": false, "instructions": "4f2a9c1e" }
}
```

- `seq` cresce a cada resposta da mesma interface; o nome do arquivo começa por ele com zeros, então a ordem
  alfabética é a ordem dos cliques.
- `event` é único: se o agente vir o mesmo duas vezes, ignore a segunda.
- `revision` é a revisão que o usuário via ao clicar. Se for menor que a atual, o usuário respondeu a uma versão
  antiga.
- `values`: um valor nunca preenchido vem `null`. Um caminho de objeto no `context` (`/ajustar`) traz o objeto
  inteiro, com o que o usuário mudou nos campos de dentro (ex.: um `checkbox` por item em `/ajustar/<item>`).
- `time` é só informativo.
- `comment`: o que o usuário escreveu no campo de comentário, se escreveu (sem o campo quando vazio). Pode ser uma
  pergunta de volta ou uma ressalva: responda numa revisão nova ou no chat. Não ponha um campo "observação"
  próprio no documento; o comentário já faz isso.
- `ide`: a IDE que gravou a resposta. `pid` é o processo; `outdated: true` quer dizer que o executável foi
  recompilado (ou atualizado) depois que essa IDE abriu, então o clique veio do código antigo. Numa entrega de
  teste, uma resposta com `outdated: true` não vale: peça para reiniciar a IDE (paleta: "Reiniciar a IDE") e
  repetir. O painel mostra o mesmo aviso no topo; não escreva no documento avisos sobre qual IDE está aberta, eles
  ficam falsos assim que o usuário reinicia. `instructions` é a versão das instruções para agentes desta IDE; o
  bloco do `AGENTS.md` termina com a linha "Instructions version" da versão que foi gravada. Diferentes: a IDE
  tem instruções mais novas, e falta rodar "Interfaces: ensinar os agentes".

Depois de ler, **apague o arquivo da resposta**. Enquanto ele existir, o painel mostra "aguardando o agente ler".

## Revisões

Para mudar a interface, reescreva o arquivo com `revision` maior. A IDE avisa o usuário e:

- mantém o que o usuário digitou nos campos em que **o agente não mudou** o valor em `data`;
- troca pelo valor do agente nos campos em que ele mudou;
- religa os botões (com `submit: "once"`).

O que o usuário digitou sobrevive a reiniciar a IDE (fica em `~/.local/state/idetea/ui/`, fora do projeto).

Apagar o documento encerra a interface, e a IDE esquece o estado dela. Se o agente apagar e recriar com o mesmo
nome antes de a IDE ver o arquivo sumir, a IDE reconhece pelo conteúdo: outro conteúdo com `revision` menor, ou na
mesma `revision` já respondida, é uma interface nova (campos do zero, botões ligados). Para reusar um nome, o
agente apaga antes as respostas que sobraram em `responses/<nome>/` e usa `revision` maior que todas as que já
escreveu nele: assim uma resposta com outra `revision` é velha.

## Entregas de teste

O que o uso mostrou ([RFC 0003](../rfc/0003-interfaces-pelo-uso.md)) sobre pedir um teste manual ao usuário:

- Uma interface só: o que mudou num `markdown`, o que fazer num `steps`, e três botões: "Aprovado, pode
  commitar", "Aprovado, sem commitar" e "Falhou". Os dois primeiros com `requires`; o "Falhou" leva o valor do
  `steps` no `context`, para o agente ver até onde o usuário chegou.
- O "Aprovado" é o julgamento do usuário, não a prova: antes de commitar, o agente confere o que o teste devia
  deixar (arquivos no disco, `ide.outdated: false` na resposta). Faltou algo: uma revisão nova dizendo o que não
  encontrou.
- Nada de avisos sobre qual IDE está aberta: o painel mostra isso (e a resposta traz `ide`).

## Um ciclo completo em shell

Veja [`examples/interfaces/pergunta.sh`](../../examples/interfaces/pergunta.sh): escreve uma pergunta, espera a
resposta, mostra e fecha a interface.
