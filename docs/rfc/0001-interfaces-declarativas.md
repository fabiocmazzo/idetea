# RFC: Interfaces declarativas e reativas na IDE TEA

Status: **aceita para o MVP**, com as recomendações abaixo (01/11/2026). Base: código da `main` em `047da35`,
SPEC.md e A2UI em novembro de 2026. As tarefas estão no SPEC.md, em "Interfaces declarativas".

Marcadores usados no texto:

- **[Fato]**: conferido no código ou na documentação citada.
- **[Hipótese]**: provável, mas não testado.
- **[Recomendação]**: escolha sugerida; ainda não é decisão.

---

## 1. Resumo executivo e recomendação

**Veredito:** viável e adequado à arquitetura, se a funcionalidade entrar como **extensão** e o formato for
**pequeno, próprio e compatível com o A2UI** (dá para traduzir um no outro). O núcleo quase não muda. O risco
maior não é técnico. É de produto e de segurança.

Pontos que pesam a favor:

1. **Encaixa num plano que já existe.** A SPEC prevê, na Fase 4, um "protocolo de UI declarativa para painéis de
   plugins" (WASM). Um renderizador declarativo serviria a agentes agora e a plugins WASM depois. É o argumento mais
   forte: o trabalho não fica isolado.
2. **A API de extensões já dá conta do MVP.** Uma extensão pode registrar um painel, observar uma pasta por conta
   própria (o git já faz isso), desenhar widgets iced e gravar arquivos. Nada disso exige mudar o `tea-core`.
3. **Arquivo é o transporte mais neutro.** Qualquer agente (Claude Code, Codex, um script em shell) sabe escrever e
   ler arquivos. Não precisa de ACP, MCP, socket nem SDK.

Pontos de atenção:

1. **Repositório malicioso.** Um `.idetea/ui/*.json` versionado num repositório clonado viraria uma interface "da
   IDE" pedindo, por exemplo, um token. É o risco nº 1; a mitigação está em 8.1.
2. **O observador atual não serve.** Ele ignora pastas do `.gitignore`, e `.idetea/` costuma ser ignorada. A
   extensão precisa do próprio observador (fato em 2.3).
3. **Um painel por interface não cabe na API atual.** `PanelId` é `&'static str` e não há como remover painel. O MVP
   usa **um painel só ("Interfaces")** com a lista das interfaces dentro.
4. **Adoção.** Se nenhum agente souber do formato, ele não é usado. O MVP precisa vir com um texto curto de
   instruções para o agente (algo para colar no `CLAUDE.md`/`AGENTS.md`).

**Recomendações:**

- **Arquitetura:** alternativa **C**. Uma crate `tea-declarative-ui` (modelo, validação e renderização iced, sem
  saber de arquivos nem da IDE) e uma extensão `tea-ext-ui` (pasta, observador, respostas, painel). A mesma crate
  serve depois ao host WASM.
- **Pasta:** `.idetea/ui/`, e não `.tea/ui/`. O projeto já usa `.idetea/` para o config.
- **Atrás de uma feature e de opção de config:** desligada até o MVP provar valor.

### Decisões para o MVP

As questões da seção 12 ficaram assim (dá para rever depois do teste com um agente de verdade):

| Questão | Decisão |
|---|---|
| Interfaces em arquivos rastreados pelo git | Ignoradas sempre, com um aviso na lista ("arquivo versionado: ignorado") |
| Onde aparece | Um painel "Interfaces", no slot direito (`[extensions] interfaces = { slot = ... }` muda) |
| `submit` padrão | `"once"` |
| Pasta | `.idetea/ui/`, com um `.gitignore` próprio (`*`) criado pela IDE |
| Texto ligado aos dados | Fora do MVP |
| Rodar um comando a cada resposta | Fora: a IDE não executa processos por causa de uma interface |
| A2UI | Só um adaptador, quando a v1.0 sair de candidata |
| Ordem no roadmap | Antes da Fase 4 (WASM), que reaproveita a crate |
| Ligada por padrão | Não: `[extensions] interfaces = { enabled = true }` liga (precisa reiniciar) |
| Id da extensão | `interfaces` (comandos `interfaces.*`); as crates são `tea-declarative-ui` e `tea-ext-ui` |

---

## 2. Arquitetura atual (o que existe de verdade)

### 2.1 Extensões, eventos, comandos e painéis (`crates/tea-core/src/lib.rs`)

- **[Fato] O contrato.** `trait Extension` tem `Message` próprio e os métodos `activate`, `on_event`, `on_command`,
  `update`, `view(PanelId)` e `subscription`. O app guarda as extensões como `Box<dyn AnyExtension>` e apaga o tipo da
  mensagem com `ExtMessage(Arc<dyn Any + Send + Sync>)`. Uma mensagem enviada à extensão errada é ignorada
  (`AnyExtension::update`, `downcast_ref`).
- **[Fato] Pedidos ao app.** Extensões não chamam o app: acumulam `Request` num `Context` e o app executa depois.
  Pedidos úteis aqui:
  - `RegisterPanel { id, title, slot, visible }`, `PanelVisibility`, `PanelIcon`
  - `Notify`, `Dialog`, `Pick` (lista na paleta), `OpenFile`, `RegisterCommand`
- **[Fato] Eventos.** `CoreEvent` só traz eventos da própria IDE: arquivos, status do editor, config, diagnósticos,
  `FilesChanged` etc. Não existe evento genérico do tipo "pasta X mudou".
- **[Fato] Painéis.**
  - `PanelId(pub &'static str)`, ou seja, uma constante por painel (ex.: `PanelId("git")` em
    `tea-ext-git/src/lib.rs`).
  - O app trata `RegisterPanel` a qualquer momento (`main.rs`, `Request::RegisterPanel`), não só na ativação, e
    guarda `Panel { ext, id, title, slot, default_slot, visible, icon }`.
  - **Não existe pedido para remover um painel.**
  - O slot vem do config **por extensão** (`slot_for` usa `config.extension(id).slot`), não por painel.
- **[Fato] Extensões embutidas.** São ligadas na compilação, uma feature do cargo por extensão
  (`builtin_extensions()` em `main.rs`: `ext-filetree`, `ext-git`...).
- **[Fato] Diálogos.** `Dialog` aceita botões com `ExtMessage` que volta para a extensão dona: é um precedente de
  "interface descrita por dados" já no núcleo. `pick::Pick` é outro (uma lista escolhida na paleta).

**Reutilizável:** registrar um painel, mostrar e esconder, notificar, ícone, comandos, subscription própria.
**Limita:** painéis dinâmicos (ids estáticos, sem remoção, slot por extensão).

### 2.2 Coordenação no `tea-app` (`crates/tea-app/src/main.rs`)

- **[Fato] `App::subscription`.** Junta teclado e mouse, fechamento e tamanho da janela, quadros (só com animação),
  as subscriptions de cada extensão (`e.subscription().with(i)`, que viram `Message::ExtSub(i, m)`), o observador do
  disco (`Subscription::run_with(self.root.clone(), watcher::watch)`) e o tique das cópias de recuperação (só
  enquanto há alterações).
- **[Fato] Foco.** O doc de `Extension::subscription` diz que mensagens vindas dela não tiram o foco do editor. Isso
  importa: uma interface nova não pode roubar o teclado de quem está digitando.
- **[Fato] Trabalho pesado.** Roda em thread própria e volta por `Task::future` com um canal (ex.: `tea-ext-git`,
  `std::thread::spawn` e `Task::future(async move { rx.await.ok() })`). Não há tokio.

### 2.3 Observador do disco (`crates/tea-app/src/watcher.rs`)

- **[Fato]** Usa `notify` com um observador **não recursivo por pasta**. A lista de pastas vem de
  `ignore::WalkBuilder`, que **respeita o `.gitignore`** e nunca entra no `.git`. Agrupa rajadas: 150 ms de silêncio,
  e no máximo 1 s.
- **[Fato]** Entrega `Change::Paths(Vec<PathBuf>)` só ao app (`Message::Disk`, depois `disk_changed`). As extensões
  recebem apenas `CoreEvent::FilesChanged`, **sem os caminhos**.
- **[Fato] Consequência:** se `.idetea/` (ou `.idetea/ui/`) estiver no `.gitignore`, o observador do app não vê nada
  ali. Mesmo vendo, a extensão não saberia quais arquivos mudaram.
- **[Fato] Precedente:** o git observa o `.git` por conta própria, numa subscription com `Subscription::run_with`
  e uma thread que compara datas (`stat`) a cada intervalo (`watch_git_dir`). A extensão de interfaces faria o mesmo
  com `.idetea/ui/`, com `notify` numa pasta só ou com polling barato.

### 2.4 Iced e estado dos widgets

- **[Fato]** A IDE usa iced 0.14 (`Cargo.toml` do workspace). O preview já desenha Markdown com
  `iced::widget::markdown` (`tea-ext-preview`). Os widgets do MVP existem no iced: `text`, `button`, `checkbox`,
  `pick_list` e `text_input`.
- **[Fato] Estado no modelo.** O valor de um campo é guardado pela própria extensão: o iced não guarda o texto de um
  `text_input`. Foco e cursor ficam na árvore de widgets do iced, que casa por posição. Por isso a estrutura precisa
  ser estável entre atualizações.
- **[Hipótese] Ids de foco.** Dar ao `text_input` um `widget::Id` derivado do id do componente mantém o foco mesmo se
  componentes forem inseridos acima. Falta testar no iced 0.14.

### 2.5 Persistência e sessões (`crates/tea-app/src/session.rs`)

- **[Fato]** A sessão de cada projeto fica fora do projeto, em `$XDG_STATE_HOME/idetea/sessions/<pasta>-<hash>.toml`.
  Guarda abas, a aba ativa, o painel de baixo e `panels: BTreeMap<String, bool>`. A gravação é atômica (temporário
  e depois `rename`).
- **[Fato] Colisão de nomes.** A sessão grava o painel só pelo `id.0`, sem o nome da extensão, então duas extensões
  com o mesmo id de painel colidiriam. Hoje cada extensão usa um id diferente.
- **[Fato]** Extensões não têm API de persistência própria; cada uma resolve sozinha. As cópias de recuperação, por
  exemplo, ficam em `backup.rs` no app.

### 2.6 Painéis dinâmicos: dá?

- **[Fato]** Dá para registrar painéis depois da ativação, mas com id `&'static str`. Ids vindos de arquivo exigiriam
  `Box::leak`, ou seja, memória que nunca volta, e uma sessão poluída por ids que somem.
- **[Fato]** Não dá para remover um painel registrado.
- **[Recomendação]** Para o MVP, **um** painel `PanelId("ui")` mostrando a lista de interfaces (abas ou lista +
  conteúdo). Ids dinâmicos e remoção só se o uso pedir. Essa mudança no núcleo pode ser feita depois sem quebrar
  nada.

---

## 3. Alternativas de arquitetura

| Critério | **A.** Extensão `tea-ext-ui` com tudo | **B.** Subsistema no `tea-core` | **C.** Crate `tea-declarative-ui` + extensão `tea-ext-ui` |
|---|---|---|---|
| Acoplamento | Baixo com o núcleo, mas renderizador preso à extensão | Alto: núcleo passa a saber de UI declarativa, contra "o núcleo só sabe editar arquivos" | Baixo: a crate não sabe de IDE nem de arquivo; a extensão só sabe de arquivo |
| Complexidade inicial | A menor | Maior: API nova no núcleo, versionada | Pequena: uma crate a mais |
| Testabilidade | Boa, mas parse e renderização misturados com I/O | Boa | A melhor: modelo e validação em testes puros; renderização com `iced_test` (já usado em `tea-editor`) |
| Manutenção | Ok enquanto só agentes usarem | Toda mudança mexe no núcleo | Formato e renderização evoluem num lugar só |
| WASM no futuro | Fraca: o host WASM teria de copiar ou importar a extensão | Natural, mas cedo demais | Natural: o host WASM (no app) usa a mesma crate para os painéis dos plugins |

**[Recomendação]** C. Ela segue a "regra de ouro" da SPEC (extensões usam a mesma API que um plugin usaria),
mantém o núcleo intacto e prepara a Fase 4. A crate dependeria só de `iced`, `serde` e `serde_json`, todas já no
workspace.

---

## 4. A2UI ou formato próprio

**[Fato] O que é o A2UI hoje** ([a2ui.org](https://a2ui.org/concepts/overview/)):

- **Versões:** v0.8 é legada, v0.9 estável, v0.9.1 é a atual e v1.0 é candidata. Da v0.8 para a v0.9 as mensagens
  mudaram de nome (`beginRendering` virou `createSurface`, `surfaceUpdate` virou `updateComponents`,
  `dataModelUpdate` virou `updateDataModel`, `userAction` virou `action`), segundo a
  [CopilotKit](https://www.copilotkit.ai/blog/a2ui-whats-new-in-google-generative-ui-spec).
- **É um protocolo de mensagens**, não um documento. O servidor manda
  [`createSurface`, `updateComponents`, `updateDataModel` e `deleteSurface`](https://a2ui.org/specification/v0.9-a2ui/),
  e o cliente devolve `action` com `name`, `surfaceId`, `sourceComponentId`, `timestamp` e `context`.
- **Componentes numa lista plana.** Cada um tem `id` e `component`, e um deles é o `root`. Filhos são referenciados
  por id (`children`/`child`).
- **Dados via JSON Pointer.** Ligações como `{"path": "/user/name"}` são de mão dupla, mas locais: nada vai ao
  servidor até uma ação.
- **Catálogo básico:** Text, Image, Icon, Video, AudioPlayer, Row, Column, List, Card, Tabs, Divider, Modal, Button,
  CheckBox, TextField, DateTimeInput, ChoicePicker, Slider.
- **Transporte:** não depende de nenhum, mas exige entrega confiável e em ordem.
- **Implementação em Rust:** existe um renderizador comunitário (crate [`a2ui`](https://docs.rs/crate/a2ui/0.3.0),
  com backend iced opcional), de um autor só e mirando a v1.0 candidata.

**Comparação**

| | A2UI (v0.9) direto | Formato próprio compatível |
|---|---|---|
| Modelo | Fluxo de mensagens em ordem | Documento inteiro, estado atual (cabe num arquivo) |
| Estabilidade | Antes da 1.0; já mudou os nomes uma vez | Sob nosso controle, versionado |
| Tamanho | 18 componentes, catálogos, funções de validação, temas | 5 componentes no MVP |
| Arquivos | Mensagens em ordem não combinam com "sobrescrever o arquivo"; precisaríamos aplicar um log | Sobrescrever o arquivo é a atualização |
| Ecossistema | Agentes já treinados nele; renderizadores existem | Agente precisa de instruções (um parágrafo) |
| Dependência | Usar a crate `a2ui` traria uma dependência nova, jovem e de um autor | Nenhuma nova |

**[Recomendação]** Formato próprio, mas **com as ideias do A2UI v0.9** para que um adaptador seja só tradução:
lista plana com `id` e `component`, dados separados com ponteiros JSON (`{"path": "/x"}`), e ação com `name`,
`context` e `sourceComponentId`. O arquivo é um **retrato** (documento completo): equivale a `createSurface` +
`updateComponents` + `updateDataModel` aplicados juntos. Quando o A2UI chegar à 1.0, o adaptador
"mensagens A2UI → documento" fica no transporte (ver seção 5), sem mudar o renderizador.

| MVP (TEA) | A2UI v0.9 |
|---|---|
| `text` | `Text` |
| `button` | `Button` (com `action`) |
| `checkbox` | `CheckBox` |
| `select` | `ChoicePicker` (uma escolha) |
| `input` | `TextField` |

---

## 5. Schema declarativo (ilustrativo, não definitivo)

### 5.1 Documento: `.idetea/ui/<surface>.json`

```json
{
  "format": "idetea-ui",
  "version": 1,
  "surface": "revisar-migracao",
  "revision": 3,
  "title": "Migração do banco: confirmar",
  "source": "agente de migração",
  "data": {
    "ambiente": "staging",
    "backup": true,
    "nota": ""
  },
  "components": [
    { "id": "intro", "component": "text",
      "text": "Vou aplicar 3 migrações (0012 a 0014). Revise antes de continuar." },
    { "id": "aviso", "component": "text", "style": "warning",
      "text": "A 0013 apaga a coluna `users.legacy_id`." },
    { "id": "amb", "component": "select", "label": "Ambiente",
      "options": [
        { "value": "staging", "label": "Staging" },
        { "value": "producao", "label": "Produção" }
      ],
      "value": { "path": "/ambiente" } },
    { "id": "bkp", "component": "checkbox", "label": "Fazer backup antes",
      "value": { "path": "/backup" } },
    { "id": "obs", "component": "input", "label": "Observação para o agente",
      "placeholder": "opcional", "value": { "path": "/nota" } },
    { "id": "aprovar", "component": "button", "label": "Aplicar", "style": "primary",
      "action": { "name": "aprovar", "context": ["/ambiente", "/backup", "/nota"] } },
    { "id": "recusar", "component": "button", "label": "Não aplicar", "style": "secondary",
      "action": { "name": "recusar", "context": ["/nota"] } }
  ],
  "submit": "once"
}
```

Decisões ilustradas, todas **[Recomendação]**:

- **Componentes:** `components` é uma lista em ordem, desenhada em coluna. O MVP não tem contêineres. `root`,
  `children` e `Row`/`Column` podem entrar depois sem quebrar, porque um documento sem `root` continua valendo como
  "coluna na ordem".
- **Dados:** valores iniciais em `data`; os campos se ligam por `{"path": ...}`; `text` aceita texto fixo (ligação a
  um caminho fica para depois).
- **Ação:** `action.context` lista os caminhos enviados na resposta. Ficam de fora os que não foram pedidos, para não
  vazar o que não precisa.
- **Respostas:** `submit: "once"` desliga os botões depois da primeira resposta até chegar uma nova revisão;
  `"many"` deixa responder várias vezes (ex.: um painel de controle).
- **Proibido de propósito:** links que abrem programas, comandos, caminhos de arquivo para abrir, HTML, scripts,
  campo de senha. O renderizador só conhece estes componentes e propriedades. O que ele não conhece vira um aviso no
  lugar do componente; a interface continua.

### 5.2 Resposta: `.idetea/ui/responses/<surface>/<seq>-<event>.json`

```json
{
  "format": "idetea-ui-response",
  "version": 1,
  "surface": "revisar-migracao",
  "revision": 3,
  "event": "1730493667123-4f2a-0007",
  "seq": 7,
  "action": "aprovar",
  "source": "aprovar",
  "time": "2026-11-01T20:41:07Z",
  "values": {
    "/ambiente": "producao",
    "/backup": true,
    "/nota": "pode aplicar depois das 22h"
  }
}
```

- `revision` diz a qual versão do documento o usuário respondeu, e evita aplicar uma resposta a uma pergunta que já
  mudou.
- `seq` cresce por interface (ordem); `event` é único (deduplicação). O nome do arquivo começa com `seq` com zeros à
  esquerda, então listar em ordem alfabética já é a ordem dos eventos.

### 5.3 Validação e limites (sem biblioteca de JSON Schema)

**[Recomendação]** Validar com tipos `serde` (`#[serde(deny_unknown_fields)]` nos lugares certos) e checagens à mão.
Um arquivo `idetea-ui.schema.json` serve de documentação e autocomplete para quem escreve, mas não é carregado em
tempo de execução, então não entra dependência nova.

Limites iniciais (chute, ajustar medindo):

| Item | Limite |
|---|---|
| Documento | 256 KB |
| Componentes | 200 |
| Texto | 16 KB |
| Opções de um `select` | 500 |
| `data` | 64 KB e profundidade 16 |
| Interfaces ao mesmo tempo | 32 |
| Nome da interface | `^[a-z0-9][a-z0-9_-]{0,63}$` |

Documento fora dos limites ou inválido: a interface aparece com o erro (linha e coluna do `serde_json`) em vez de
sumir. Isso ajuda o agente a corrigir.

**Versão:** `format` mais `version` inteiro. A IDE aceita as versões que conhece e mostra "versão N não suportada"
nas outras. Mudança compatível (componente ou propriedade nova e opcional) não sobe `version`. A IDE ignora
propriedade desconhecida com aviso, em vez de recusar o documento.

---

## 6. Fluxo: criação, atualização e resposta

```text
Agente                         Disco (.idetea/ui/)                    IDE (tea-ext-ui)
  │ grava x.json.tmp, rename ──▶ x.json                                 │
  │                              │── notify/poll ──────────────────────▶│ lê, valida, monta o modelo
  │                              │                                      │ painel "Interfaces" + aviso (sem roubar foco)
  │                              │                     usuário clica ──▶│ junta os valores do `context`
  │                              │ responses/x/0007-….json ◀── tmp + rename
  │◀── lê / observa ─────────────│                                      │ mostra "enviado, aguardando o agente"
  │ apaga a resposta (= recebi)──▶ (arquivo some)                       │
  │                              │── notify/poll ──────────────────────▶│ mostra "recebido"
  │ grava x.json (revision 4) ──▶ x.json                                │ atualiza, mantendo o que o usuário digitou
  │ apaga x.json ───────────────▶ (sumiu)                               │ some do painel (respostas pendentes ficam)
```

### 6.1 Detecção

- **[Recomendação]** A extensão observa só `.idetea/ui/` e `.idetea/ui/responses/*`, em subscription própria
  (`Subscription::run_with(raiz, observar_ui)`), como o git faz com o `.git`. O `.gitignore` não atrapalha.
- **[Hipótese]** `notify` numa pasta só, com fallback de polling (`stat` a cada 500 ms) quando o inotify falhar ou
  em sistemas de arquivos onde ele não funciona bem (pastas de rede, virtiofs: ver a seção 11).

- Escrita pela metade: o agente grava `x.json.tmp` e faz `rename` (atômico no mesmo sistema de arquivos). A IDE
  ignora `*.tmp` e arquivos começados por `.`. Se mesmo assim ler JSON quebrado (agente que não segue a regra), ela
  espera a próxima mudança e tenta de novo uma vez depois de 200 ms antes de mostrar o erro.

### 6.2 Concorrência: um dono por arquivo

**[Recomendação]** A regra que evita quase todos os problemas: **cada arquivo tem um só escritor**.

| Arquivo | Quem escreve | Quem apaga |
|---|---|---|
| `ui/<surface>.json` | o agente | o agente |
| `ui/responses/<surface>/*.json` | a IDE | o agente (é a confirmação de leitura) |

- Dois agentes não usam o mesmo nome de interface. **[Recomendação]** Prefixo do agente no nome
  (`claude-revisar-migracao`); a IDE não força isso.
- Duas janelas da IDE no mesmo projeto: as duas mostrariam a interface e as duas poderiam responder. **[Hipótese]**
  Aceitável no MVP: `seq` com o pid no `event` evita sobrescrever, e o agente vê duas respostas. Depois, um
  `responses/<surface>/.lock` com o pid da janela que responde.

### 6.3 Ordem, deduplicação e confirmação

- **Ordem:** `seq` por interface, guardado no estado da extensão (6.5) para continuar depois de reiniciar.
- **Deduplicação:** `event` único. A IDE nunca regrava uma resposta já gravada; o agente ignora `event` repetido.
- **Confirmação:** o agente apaga o arquivo de resposta. Enquanto ele existe, a IDE mostra "aguardando o agente".
  Não precisa de um arquivo de confirmação a mais.

### 6.4 Persistência e queda

- Interfaces e respostas são arquivos, então sobrevivem ao fechamento da IDE e do agente. Ao abrir, a IDE lê o que
  houver.
- Resposta gravada e não confirmada continua lá. O agente a encontra quando voltar (ver a seção 11).
- Queda no meio da escrita: sobra no máximo um `*.tmp`, que a IDE ignora e apaga se tiver mais de 1 h.

### 6.5 Isolamento e proteção

- **Por projeto:** tudo fica dentro da raiz do projeto. Os valores que o usuário está digitando ficam fora, em
  `$XDG_STATE_HOME/idetea/ui/<pasta>-<hash>/`, como as sessões (mesma função `session::path_for`/`fnv1a`).
- **Symlinks:**
  - Usar `symlink_metadata` em `.idetea`, em `ui/`, em `responses/` e em cada arquivo. Symlink é recusado.
  - Antes de gravar, confirmar que o destino canônico está dentro da raiz canônica.
  - A resposta é criada com `create_new` (`O_EXCL`) num nome novo e depois renomeada: nunca segue um link plantado.
- **Path traversal:** o nome da interface vem do nome do arquivo e é validado pela regex de 5.3. Nenhum caminho
  vindo do JSON é usado para abrir ou gravar arquivo.
- **Tamanho:** checar `metadata.len()` antes de ler (256 KB) e ler com limite, por segurança contra arquivo que
  cresce.

---

## 7. Estado

Há três fontes de verdade e cada uma tem um dono:

| Estado | Dono | Onde |
|---|---|---|
| Estrutura e valores iniciais (`components`, `data`) | agente | o documento |
| Valores mudados pelo usuário (ainda não enviados) | IDE | memória da extensão, cópia em `$XDG_STATE_HOME` |
| Foco, cursor do campo, rolagem | iced | árvore de widgets (casada por posição ou `widget::Id`) |

**Atualização do documento (revisão nova)** **[Recomendação]**:

- Para cada caminho em `data`, se o valor **do agente** mudou entre a revisão anterior e a nova, o agente vence e a
  mudança local naquele caminho é descartada.
- Se o agente não mexeu, a mudança do usuário fica.
- Isso exige guardar a revisão anterior de `data` (barato: é pequena).

Também:

- Componentes casam pelo `id`. Um componente que sumiu leva junto o estado dele; um que entrou começa do `data`.
- `submit: "once"` respondido é guardado por `(surface, revision)`. Uma nova revisão reabre a interface.
- Ao reiniciar a IDE, as mudanças locais voltam do arquivo de estado se a `revision` for a mesma. Se mudou, vale a
  regra acima.
- Nada disso passa pelo `CoreEvent` nem pela sessão do app. É estado privado da extensão, como o dos outros painéis.

---

## 8. Riscos técnicos e pontos de atenção

### 8.1 Segurança (o principal)

1. **Interface vinda de repositório clonado.** Qualquer arquivo versionado em `.idetea/ui/` apareceria como se fosse
   da IDE. **[Recomendação]**, em camadas:
   - Ignorar documentos rastreados pelo git (a extensão git já calcula o status; ou checar `git ls-files` na
     primeira leitura).
   - A interface sempre mostra a origem ("arquivo `.idetea/ui/x.json`, escrito por um processo local"), num visual
     diferente dos diálogos da IDE.
   - Opção `[ui_files] enabled = false` por padrão até o MVP amadurecer.
   - Sem campo de senha e sem nada que abra links ou rode comandos.
2. **Respostas em texto puro no disco.** O que o usuário digitar fica em `.idetea/ui/responses/`. Pode vazar num
   commit se a pasta não estiver no `.gitignore`. **[Recomendação]**:
   - Ao criar `.idetea/ui/`, a extensão grava `.idetea/ui/.gitignore` com `*`, que funciona mesmo sem mexer no
     `.gitignore` do projeto.
   - Avisar no texto do campo que a resposta fica num arquivo.
3. **Engenharia social por agente.** O agente é confiável por definição (roda com as permissões do usuário), mas o
   texto que ele mostra pode vir de conteúdo que ele leu (prompt injection). A interface não amplia o poder do agente:
   só coleta respostas. Tratar como texto puro; Markdown só depois do MVP e sem links clicáveis (ou com confirmação).
4. **DoS:** cobertos pelos limites de 5.3 e pelo observador restrito.
   - Arquivo gigante: limite de tamanho.
   - Milhares de arquivos: limite de interfaces e de respostas listadas.
   - Reescrita contínua: agrupar mudanças em rajadas, como o `watcher.rs` faz.

### 8.2 Técnicos

- **Painéis dinâmicos:** limitados pela API (2.6). O MVP contorna com um painel só.
- **Ids de foco no iced:** hipótese a testar. Se não funcionar, o foco pode pular ao atualizar a interface enquanto
  o usuário digita. Mitigação: adiar a revisão nova até o campo perder o foco, com um aviso "o agente atualizou".
- **inotify em virtiofs e pastas de rede:** pouco confiável **[Hipótese]**. Por isso o polling de fallback.
- **Relógio:** `time` é informativo. A ordem vale por `seq`, não pela hora.
- **Windows e macOS:** `rename` sobre arquivo existente é atômico no Linux e no macOS. No Windows precisa de
  `MoveFileEx` com substituição; o `std::fs::rename` do Rust já faz isso, mas pode falhar se o outro lado estiver com
  o arquivo aberto **[Hipótese]**. A IDE hoje só é testada no Linux.

### 8.3 Produto

- **Adoção:** sem instruções para o agente, ninguém escreve esses arquivos. O MVP precisa de um texto pronto para o
  `AGENTS.md`/`CLAUDE.md` e de um script de exemplo.
- **Escopo:** a vontade natural é ir crescendo até virar um framework de UI. A SPEC diz "sem firulas". O catálogo
  deve crescer só com casos reais (perguntas, aprovações, escolhas, progresso).

---

## 9. MVP recomendado

**Objetivo:** provar os 7 passos do pedido com o mínimo de peças.

| Passo | Como |
|---|---|
| 1. Processo externo cria o JSON | Script de exemplo `examples/ui/pergunta.sh` (shell + `jq` opcional) |
| 2. A TEA detecta | `tea-ext-ui`: subscription própria observando `.idetea/ui/` |
| 3. Painel aparece | Um painel "Interfaces" (slot direito), `show_panel` + aviso, sem tirar o foco do editor |
| 4. Usuário interage | `text`, `button`, `checkbox`, `select` (`pick_list`), `input` (`text_input`), em coluna |
| 5. Resposta gravada | `responses/<surface>/<seq>-<event>.json`, com `create_new` + `rename` |
| 6. Processo consome | O script espera o arquivo (polling simples), lê e apaga |
| 7. Atualização ao vivo | Agente regrava o documento com `revision` maior; a regra da seção 7 preserva o que foi digitado |

**Fora do MVP:**

- contêineres, Markdown e tabelas;
- progresso;
- vários painéis;
- formato A2UI;
- ACP, Firecracker;
- validação por funções;
- tema.

**Critérios de aceite:**

- Uma interface aparece em menos de 300 ms depois do `rename`.
- Digitar num campo enquanto o agente atualiza o documento não perde o texto.
- Reiniciar a IDE no meio mantém interface, valores digitados e respostas pendentes.
- Documento inválido mostra o erro e a IDE continua estável.
- Documento rastreado pelo git não aparece.
- Testes: modelo e merge em testes puros; renderização e cliques com `iced_test`; observador com pasta temporária
  (mesmo padrão de `watcher.rs`).

---

## 10. Plano incremental

1. **Formato (sem código de UI).** Escrever a seção do formato na SPEC, o `idetea-ui.schema.json` de referência e
   três exemplos (pergunta, aprovação, escolha). Validar com um ou dois agentes reais se eles escrevem certo só com
   as instruções.
2. **`tea-declarative-ui`: modelo.** Tipos `serde`, validação, limites e merge de estado (seção 7). Só testes, sem
   iced.
3. **`tea-declarative-ui`: renderização.** `view(&Surface, &State) -> Element<UiMessage>` com os 5 componentes;
   testes com `iced_test` (clique gera `Action` com os valores certos).
4. **`tea-ext-ui`.** Observador próprio, leitura com proteção (symlink, tamanho, nome), painel único, respostas
   atômicas, estado em `$XDG_STATE_HOME`, filtro de arquivos rastreados, `.gitignore` interno. Atrás de
   `ext-ui` e de `[ui_files] enabled`.
5. **Teste manual com um agente de verdade** (Claude Code com o texto de instruções), registrando o que confundiu o
   agente.
6. **Depois, só se o uso pedir:**
   - `Row`/`Column`, `progress`, Markdown sem links;
   - painéis dinâmicos no núcleo (`PanelId` com `String`, `UnregisterPanel`);
   - adaptador A2UI v1.0 quando ela sair de candidata;
   - uso da crate pelo host WASM (Fase 4).

---

## 11. Evolução: ACP e Firecracker (avaliação, nada disso no MVP)

**Separação de papéis:**

| Camada | Responsabilidade | Exemplos |
|---|---|---|
| Protocolo | O que é uma interface e uma resposta (documento, revisão, ação, `seq`/`event`) | `idetea-ui` v1, ou A2UI |
| Transporte | Como documentos e respostas chegam e voltam | pasta, ACP, vsock, pasta compartilhada |
| Runtime do agente | Quando perguntar, como esperar, como retomar | Claude Code, agente ACP, VM |
| IDE | Validar, desenhar, guardar o estado local, gravar respostas | `tea-ext-ui` + `tea-declarative-ui` |

**[Recomendação]** Dentro da extensão, isolar o transporte atrás de algo como
`trait SurfaceSource { fn changes(...) ; fn respond(...) }`, com o `FileSource` como primeira implementação.
Com isso:

- **Agentes ACP:** a IDE seria cliente ACP e as interfaces viriam como mensagens do agente (A2UI dentro do ACP, ou uma
  extensão própria). Um `AcpSource` montaria o mesmo documento. O renderizador não muda. **[Hipótese]** O ACP ainda
  não define UI declarativa padrão; seria uma extensão do protocolo.
- **Firecracker:**
  - O agente roda numa microVM e não vê o disco do host. Ou o projeto é compartilhado (virtiofs), e o `FileSource`
    funciona com polling, porque o inotify no virtiofs não é confiável; ou um processo ponte no guest repassa por
    vsock para um `VsockSource` no host.
  - Em ambos, quem valida e desenha é sempre o host: a VM nunca manda widgets, só dados.
- **Respostas com o agente desligado:** já atendido pelo design de arquivos (respostas ficam até serem apagadas). Num
  transporte em rede, a extensão guardaria a fila no estado local e reenviaria ao reconectar.
- **Retomar depois de reiniciar a IDE:** atendido por documento + estado local + `revision`.
- **Vários agentes:** cada um com o próprio prefixo de nome; o painel agrupa por `source`. Um limite de interfaces
  por agente evita que um só encha o painel.

---

## 12. Questões em aberto

1. **Arquivos rastreados:** ignorar sempre os rastreados pelo git, ou perguntar uma vez por projeto ("confiar nas
   interfaces deste repositório")?
2. **Onde o painel aparece:** slot direito fixo, painel de baixo, ou um diálogo para perguntas curtas e bloqueantes
   (o `Dialog` atual serviria)?
3. **`submit: once`:** é o padrão certo? Aprovações querem "uma vez"; painéis de controle querem "muitas".
4. **Nome da pasta:** `.idetea/ui/` (coerente com o config) ou algo mais genérico para outros editores adotarem?
5. **Texto ligado ao `data` no MVP** (ex.: "Ambiente: {/ambiente}")? Ajuda, mas é um passo para virar template.
6. **Notificar o agente:** além de ele observar a pasta, vale a IDE rodar um comando configurado quando há resposta
   (ex.: `[ui_files] on_response = "..."`)? Isso faria a IDE executar processos, o que a seção 5 evita de propósito.
7. **Momento do A2UI:** adotar quando sair a 1.0, ou só traduzir se surgir um agente que fale A2UI?
8. **Prioridade:** isso entra antes ou depois da Fase 4 (WASM), já que as duas dividem o renderizador?

---

## Fontes

- Código: `crates/tea-core/src/lib.rs`, `crates/tea-app/src/{main.rs,watcher.rs,session.rs,backup.rs}`,
  `crates/ext/tea-ext-git/src/lib.rs` (`watch_git_dir`), `crates/ext/tea-ext-preview/src/lib.rs`, SPEC.md (modelo de
  extensões e Fase 4).
- [A2UI, visão geral e versões](https://a2ui.org/concepts/overview/)
- [A2UI v0.9, especificação](https://a2ui.org/specification/v0.9-a2ui/)
- [A2UI v0.8 (legado)](https://github.com/a2ui-project/a2ui/blob/main/specification/v0_8/docs/a2ui_protocol.md)
- [CopilotKit: o que mudou na v0.9](https://www.copilotkit.ai/blog/a2ui-whats-new-in-google-generative-ui-spec)
- [Crate `a2ui` (renderizadores Rust comunitários)](https://docs.rs/crate/a2ui/0.3.0)
