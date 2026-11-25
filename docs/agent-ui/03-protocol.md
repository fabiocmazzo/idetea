# 03 — Evolução do protocolo

Tudo continua `"format": "idetea-ui", "version": 1`. Um leitor antigo ignora o que não conhece: componente novo
vira `Kind::Unknown` (aviso no lugar), campo novo vira aviso. Um agente que precisa de um recurso novo confere
`ide.version` numa resposta antes ou usa o `idetea ui check`, que já avisa.

Marcados com a etapa do [roadmap](05-roadmap.md) em que entram.

## Anexos na resposta (etapa 1)

Nada muda no documento. O painel ganha, ao lado do comentário, **"Anexar captura da IDE"** e **"Anexar arquivo…"**.
A resposta ganha:

```json
"attachments": [
  { "path": ".idetea/ui/responses/teste-links/anexos/3-1.png", "kind": "screenshot", "size": 184311, "width": 1920, "height": 1080 },
  { "path": ".idetea/ui/responses/teste-links/anexos/3-2.png", "kind": "file", "from": "~/Pictures/erro.png", "size": 51022 }
]
```

- A IDE copia o arquivo para dentro de `responses/<nome>/anexos/` (fora do git, como o resto de `.idetea/ui/`),
  então o agente lê um caminho do projeto, sem permissão para `~/Pictures`.
- `from` só existe para arquivo escolhido, e mostra o caminho com `~` no lugar da pasta pessoal.
- O documento pode pedir: `"attach": "screenshot"` num `button` faz o botão esperar um anexo (como `requires`).
- O agente apaga a pasta `anexos/` junto com a resposta; o `idetea ui fix` limpa anexos órfãos.

## Números e sliders (etapa 2)

```json
{"id": "u", "component": "number", "label": "Usuários simultâneos", "value": "/usuarios", "min": 1, "max": 100000, "step": 1, "unit": "usuários"}
{"id": "r", "component": "slider", "label": "Réplicas", "value": "/replicas", "min": 1, "max": 12, "step": 1}
```

O valor gravado é número JSON. Fora de `min`/`max`, o campo avisa e o botão com `requires` espera.

## Mostrar conforme outro campo (etapa 2)

Qualquer componente aceita `show_if`, uma comparação simples, nunca uma expressão:

```json
{"id": "nome", "component": "input", "label": "Nome do branch", "value": "/branch",
 "show_if": {"path": "/modo", "equals": "branch"}}
```

Operadores: `equals`, `not_equals`, `in` (lista), `truthy`. Um componente escondido não entra no `requires`; o
valor dele vai na resposta se estiver no `context` (o agente decide se usa).

## Lado a lado (etapa 2)

```json
{"id": "cols", "component": "columns", "children": ["opA", "opB"]}
```

`children` são ids de componentes do mesmo documento, que deixam de aparecer na coluna e aparecem lado a lado (até
4). Na largura estreita, voltam a ficar um embaixo do outro. Sem aninhar `columns`.

## Recomendação nas opções (etapa 2)

```json
{"value": "sqlite", "label": "SQLite", "recommended": true, "note": "um arquivo, sem servidor"}
```

O painel marca a opção e mostra a nota embaixo. Hoje o agente escreve "(recomendado)" no rótulo; isso continua
valendo.

## Comparar imagens (etapa 3)

```json
{"id": "c", "component": "compare", "before": "art/ship-v1.png", "after": "art/ship-v2.png",
 "mode": "slider", "pixelated": true, "background": "checker", "height": 240}
```

`mode`: `slider` (uma linha que o usuário arrasta), `toggle` (clique troca) ou `side` (lado a lado). A roda do
mouse dá zoom e arrastar move, nos dois ao mesmo tempo. Na resposta, `media` conta `compared` (vezes que o
usuário mexeu).

## Valores calculados (etapa 5)

Um `data` pode ter valores que começam com `=`:

```json
"data": {
  "usuarios": 500, "mb_por_usuario": 12, "replicas": 3,
  "memoria_mb": "=usuarios * mb_por_usuario / replicas",
  "cabe": "=memoria_mb <= 2048"
}
```

- Variáveis são as chaves de `data` no mesmo nível ou por ponteiro (`=/carga/pico * 2`). Operadores aritméticos,
  comparação, `and`/`or`/`not`, `if(c, a, b)`, `min`, `max`, `round`, `sum(lista)`.
- O avaliador monta o grafo de dependências ao ler o documento: **ciclo, referência inválida e tipo errado
  recusam o documento** com o caminho (`data.memoria_mb: ciclo com data.cabe`). Divisão por zero e resultado não
  finito viram `null` e o painel mostra "—" com o motivo.
- `text` e `markdown` mostram valores com `{/memoria_mb}`. Um `chart` lê listas de `data`:
  `{"component": "chart", "kind": "bar", "series": "/curva", "x": "carga", "y": "ms"}` (`bar` ou `line`, desenhado
  em `canvas`, sem biblioteca).
- Na resposta: `"computed": {"/memoria_mb": 2000, "/cabe": true}`, separado de `values`.
- `requires` num botão aceita um valor calculado booleano (`"requires": "/cabe"`): o botão espera ele ficar
  `true`. Hoje o `check` avisa e ignora.

## Módulo do agente (etapa 6)

```json
"module": {"path": ".idetea/ui/modulos/estimativa.wasm", "inputs": ["/usuarios", "/replicas"], "outputs": "/estimativa"}
```

Um componente WASM com uma função `calcular(entrada: string) -> string` (JSON nos dois lados), sem WASI de arquivo
ou rede. A IDE pede confirmação uma vez por hash do arquivo, roda a cada mudança dos `inputs` com prazo, e grava a
saída em `outputs` (um valor calculado, como acima).

## O que não entra

- Eventos além do clique (o agente não recebe cada tecla).
- Expiração automática: o agente apaga o documento; o `idetea ui fix` já acha órfãos.
- Persistência além do histórico, que já guarda cada revisão respondida.
