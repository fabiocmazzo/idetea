# 07 — Exemplos

Os exemplos marcados **funciona hoje** passam no `idetea ui check` da 0.19.0. Os marcados **proposto** usam campos
de [03](03-protocol.md) e hoje mostram avisos (campo desconhecido) ou um componente desconhecido no lugar.

Para cada cenário: como o agente pede, como a IDE valida e mostra, como volta a resposta e o que acontece se der
errado.

## 1. Decisão arquitetural (cenário A) — proposto

O agente compara três formas de guardar dados, com estimativas, e pede a escolha.

```json
{"format": "idetea-ui", "version": 1, "revision": 1, "title": "Onde guardar o histórico", "source": "Claude",
 "data": {"opcao": "sqlite", "migrar": true, "usuarios": 500},
 "components": [
  {"id": "ctx", "component": "markdown", "text": "Hoje o histórico é um JSON por entrada. Com 500 usuários, a pasta passa de 100 mil arquivos em um ano."},
  {"id": "a", "component": "markdown", "text": "**SQLite**\n\n- um arquivo\n- busca por índice\n- ~4 MB/ano"},
  {"id": "b", "component": "markdown", "text": "**JSONL por mês**\n\n- texto puro, diff legível\n- busca linear\n- ~6 MB/ano"},
  {"id": "c", "component": "markdown", "text": "**Como está**\n\n- nada a fazer\n- 100 mil arquivos/ano"},
  {"id": "lado", "component": "columns", "children": ["a", "b", "c"]},
  {"id": "u", "component": "number", "label": "Usuários previstos", "value": "/usuarios", "min": 1, "max": 100000},
  {"id": "op", "component": "select", "label": "Escolha", "value": "/opcao", "options": [
    {"value": "sqlite", "label": "SQLite", "recommended": true, "note": "o menor e o único com índice"},
    {"value": "jsonl", "label": "JSONL por mês"},
    {"value": "nada", "label": "Deixar como está"}]},
  {"id": "m", "component": "checkbox", "label": "Migrar o histórico existente", "value": "/migrar", "show_if": {"path": "/opcao", "not_equals": "nada"}},
  {"id": "ok", "component": "button", "label": "Decidir", "action": {"name": "decidir", "context": ["/opcao", "/migrar", "/usuarios"]}}
 ]}
```

- **Validação:** `columns` com três ids que existem; `number` com intervalo; `show_if` com operador conhecido.
- **Resposta:** `{"action": "decidir", "values": {"/opcao": "sqlite", "/migrar": true, "/usuarios": 500}}`. Com
  "Registrar como decisão", vira ADR.
- **Erro ou cancelamento:** sem clique em 10 minutos, o `idetea ask` sai com 4 e a pergunta fica; o agente pergunta
  no chat. IDE fechada: o documento fica e reaparece ao abrir.

## 2. Comparar sprites (cenário B) — funciona hoje (galeria); proposto (comparação)

Hoje, quatro alternativas numa `gallery` com escolha:

```json
{"format": "idetea-ui", "version": 1, "revision": 1, "title": "Qual nave?", "source": "Claude",
 "components": [
  {"id": "g", "component": "gallery", "select": "one", "value": "/nave", "pixelated": true, "background": "checker", "height": 96,
   "items": [{"id": "a", "path": "art/nave-a.png", "label": "A"}, {"id": "b", "path": "art/nave-b.png", "label": "B"},
             {"id": "c", "path": "art/nave-c.png", "label": "C"}, {"id": "d", "path": "art/nave-d.png", "label": "D"}]},
  {"id": "ok", "component": "button", "label": "Usar esta", "requires": "/nave", "action": {"name": "ok", "context": ["/nave"]}}
 ]}
```

Proposto, a versão nova contra a anterior:

```json
{"id": "c", "component": "compare", "before": "art/nave-v1.png", "after": "art/nave-v2.png", "mode": "slider", "pixelated": true, "background": "checker", "height": 240}
```

- **Validação e mídia:** cada imagem passa pelos limites antes de decodificar; uma que falta mostra o erro no lugar.
- **Resposta:** `values["/nave"] = "b"`, e `media` diz quais imagens ficaram à vista (`shown`) e quais foram abertas
  na aba (`opened`), com o `size` e o `modified` do arquivo que estava na tela. O agente confere se o usuário viu a
  versão atual.
- **Retomada:** o agente usa `art/nave-b.png` e segue.

## 3. Escolher um som (cenário C) — funciona hoje (Linux e macOS)

```json
{"format": "idetea-ui", "version": 1, "revision": 1, "title": "Som do pulo", "source": "Claude",
 "data": {"som": "b"},
 "components": [
  {"id": "a", "component": "audio", "path": "sfx/pulo-a.wav", "label": "A: curto"},
  {"id": "b", "component": "audio", "path": "sfx/pulo-b.wav", "label": "B: com eco"},
  {"id": "c", "component": "audio", "path": "sfx/pulo-c.wav", "label": "C: grave"},
  {"id": "s", "component": "select", "label": "Qual fica?", "value": "/som", "options": ["a", "b", "c"]},
  {"id": "ok", "component": "button", "label": "Usar", "action": {"name": "ok", "context": ["/som"]}}
 ]}
```

- **O que o agente confere:** `media["sfx/pulo-b.wav"].completed` (ouviu até o fim?) e `clips` (estourou a 0 dBFS).
- **No Windows hoje:** o botão Tocar fica desligado com o motivo (sem tocador); a etapa 4 resolve.

## 4. Teste manual com captura (cenário D) — proposto (anexo)

```json
{"format": "idetea-ui", "version": 1, "revision": 3, "title": "Teste: abas que não cabem", "source": "Claude",
 "data": {"done": []},
 "components": [
  {"id": "md", "component": "markdown", "text": "**O que mudou:** as abas que não cabem rolam.\n\n```sh\n./target/debug/idetea .\n```"},
  {"id": "f", "component": "file", "path": "crates/ext/tea-ext-terminal/src/lib.rs", "line": 1093, "label": "O código"},
  {"id": "s", "component": "steps", "value": "/done", "items": [
    {"id": "a", "text": "Abra 12 abas de terminal"},
    {"id": "b", "text": "Role a barra de abas com a roda do mouse"}]},
  {"id": "ok", "component": "button", "label": "Aprovado, pode commitar", "requires": "/done", "action": {"name": "ok-commit", "context": ["/done"]}},
  {"id": "ok2", "component": "button", "label": "Aprovado, não commite", "style": "secondary", "requires": "/done", "action": {"name": "ok-no-commit", "context": ["/done"]}},
  {"id": "no", "component": "button", "label": "Falhou", "style": "danger", "attach": "screenshot", "action": {"name": "fail", "context": ["/done"]}}
 ]}
```

- **Novo:** "Falhou" espera uma captura (`attach`). A resposta leva
  `"attachments": [{"path": ".idetea/ui/responses/teste-abas/anexos/1-1.png", "kind": "screenshot"}]`, e o agente
  abre o PNG em vez de perguntar onde está o print.
- **Antes de commitar:** o agente confere o que o teste deixou (arquivos no disco, `ide.outdated: false`), como já
  faz.

## 5. Simulação de capacidade (cenário E) — proposto (valores calculados)

```json
{"format": "idetea-ui", "version": 1, "revision": 1, "title": "Quantas réplicas?", "source": "Claude",
 "data": {
   "usuarios": 2000, "mb_por_usuario": 3, "replicas": 2, "limite_mb": 4096,
   "memoria_mb": "=usuarios * mb_por_usuario / replicas",
   "cabe": "=memoria_mb <= limite_mb",
   "custo_mes": "=round(replicas * 18.5)"
 },
 "components": [
  {"id": "u", "component": "number", "label": "Usuários simultâneos", "value": "/usuarios", "min": 1, "max": 1000000},
  {"id": "r", "component": "slider", "label": "Réplicas", "value": "/replicas", "min": 1, "max": 12, "step": 1},
  {"id": "res", "component": "markdown", "text": "Memória por réplica: **{/memoria_mb} MB** de {/limite_mb}. Custo: **US$ {/custo_mes}/mês**."},
  {"id": "w", "component": "text", "style": "warning", "text": "Não cabe: aumente as réplicas.", "show_if": {"path": "/cabe", "equals": false}},
  {"id": "ok", "component": "button", "label": "Usar esta configuração", "requires": "/cabe", "action": {"name": "ok", "context": ["/usuarios", "/replicas"]}}
 ]}
```

- **Validação:** o grafo `memoria_mb → cabe` não tem ciclo; todas as variáveis existem.
- **Eventos:** cada mudança do slider refaz `memoria_mb`, `cabe` e `custo_mes` na IDE, sem falar com o agente.
- **Resposta:** `values` com o que o usuário escolheu e `computed` com o que a IDE calculou. Com 3 réplicas:
  `{"/memoria_mb": 2000, "/cabe": true, "/custo_mes": 56}`.
- **`requires` num valor calculado:** hoje o `check` avisa que `/cabe` não é o `value` de nenhum componente e
  ignora. A etapa 5 passa a aceitar um valor calculado booleano em `requires`.
- **Quando não cabe numa expressão:** o mesmo documento com `"module"` (etapa 6) no lugar das contas.
