# Interfaces declarativas: o formato `idetea-ui` v1

Um agente (ou qualquer programa) escreve um arquivo JSON e a IDE TEA mostra uma interface pequena no painel
**Interfaces**. Quando o usuário clica num botão, a IDE grava a resposta num arquivo que o agente lê. Nada roda
código: o documento só descreve textos, botões, caixas de marcar, listas de escolha e campos de texto.

As decisões e o porquê estão em [`docs/rfc/0001-interfaces-declarativas.md`](../rfc/0001-interfaces-declarativas.md).
O JSON Schema está em [`idetea-ui.schema.json`](idetea-ui.schema.json).

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
- Para fechar a interface, apague `<nome>.json`. O resto (`.gitignore`, `responses/`) é da IDE: a pasta de
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
| `$schema` | não | Ignorado. |

Campos desconhecidos viram um aviso no painel, não um erro. Um componente desconhecido aparece como aviso no lugar
dele; o resto da interface continua.

### Componentes

Todo componente tem `id` (`[A-Za-z0-9_-]`, até 64, único no documento) e `component`.

| `component` | Campos | Notas |
|---|---|---|
| `text` | `text`, `style` | `style`: `normal` (padrão), `muted`, `warning`, `error`. |
| `button` | `label`, `action`, `style` | `style`: `primary` (padrão), `secondary`, `danger`. Botões seguidos ficam na mesma linha. |
| `checkbox` | `label`, `value` | Valor booleano. |
| `select` | `options`, `value`, `label` | `options`: até 500, cada uma `"x"` ou `{"value": "x", "label": "X"}`. Valor: o `value` escolhido. |
| `input` | `value`, `label`, `placeholder` | Texto de uma linha. |
| `file` | `path`, `line`, `column`, `label`, `open` | Um arquivo do projeto. Ver [Arquivos](#arquivos). |

`value` é o lugar do valor em `data`: um ponteiro JSON (RFC 6901) como `"/usuario/nome"`, ou
`{"path": "/usuario/nome"}`.

`action` é `{"name": "...", "context": ["/caminho", ...]}`: o nome volta na resposta, junto com o valor atual de cada
caminho do `context`.

### Limites

256 KB por arquivo, 200 componentes, 16 KB por texto, 500 opções, 64 KB e 16 níveis em `data`. Acima disso o
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
- `open: true`: abre sozinho quando a revisão chega. Se o usuário digitou no editor nos últimos 2 segundos, não
  abre (o cursor dele não é roubado no meio de uma linha): só aparece um aviso, e o link continua no painel.
- **O arquivo é a resposta.** Ao clicar num botão, a IDE primeiro salva os arquivos dos componentes `file` que
  estão abertos com alterações, e só então grava a resposta. Ao ver a resposta, leia o arquivo do disco. Se algum
  não puder ser salvo, a resposta não é gravada e o usuário vê o erro.
- A IDE não cria o arquivo: escreva-o antes do documento (um arquivo que não existe abre como aba nova, e é criado
  quando o usuário salvar). Para rascunhos, use `.idetea/ui/rascunhos/`: fica fora do git (o `.gitignore` da
  pasta) e a IDE não mexe nele. Apague-os ao terminar.

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
  "values": { "/ambiente": "prod", "/backup": true, "/nota": null }
}
```

- `seq` cresce a cada resposta da mesma interface; o nome do arquivo começa por ele com zeros, então a ordem
  alfabética é a ordem dos cliques.
- `event` é único: se o agente vir o mesmo duas vezes, ignore a segunda.
- `revision` é a revisão que o usuário via ao clicar. Se for menor que a atual, o usuário respondeu a uma versão
  antiga.
- `values`: um valor nunca preenchido vem `null`.
- `time` é só informativo.

Depois de ler, **apague o arquivo da resposta**. Enquanto ele existir, o painel mostra "aguardando o agente ler".

## Revisões

Para mudar a interface, reescreva o arquivo com `revision` maior. A IDE avisa o usuário e:

- mantém o que o usuário digitou nos campos em que **o agente não mudou** o valor em `data`;
- troca pelo valor do agente nos campos em que ele mudou;
- religa os botões (com `submit: "once"`).

O que o usuário digitou sobrevive a reiniciar a IDE (fica em `~/.local/state/idetea/ui/`, fora do projeto).

## Um ciclo completo em shell

Veja [`examples/interfaces/pergunta.sh`](../../examples/interfaces/pergunta.sh): escreve uma pergunta, espera a
resposta, mostra e fecha a interface.
