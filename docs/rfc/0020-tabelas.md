# RFC 0020 — Tabelas

- **Status:** aprovada em 2026-11-17 (pelo painel): opção B. **Implementada** (degrau B); a opção C entrou em
  2026-11-26 (sem mover linhas e colunas).
- **Origem:** comentário no painel (2026-11-17) sobre um cliente de banco de dados. Depois da primeira versão
  desta RFC (sobre banco), no painel: "a IDE deveria mostrar as tabelas. Acho que devemos mudar o CSV para tabelas,
  e sem nada relacionado a banco, mas conseguir exibir csv, tsv, jsonl e inclusive editar". Banco de dados ficou
  fora; a análise de filosofia daquela versão está no fim, em [Banco de dados](#banco-de-dados-fora).
- **Relaciona:** [MANIFESTO](../../MANIFESTO.pt-BR.md), [RFC 0001](0001-interfaces-declarativas.md) (painel
  Interfaces), [RFC 0004](0004-midia-no-painel.md) (mídia no painel), [RFC 0006](0006-ide-como-cli-para-agentes.md)
  (`idetea ask`/`open`).

Marcadores: **[Fato]** conferido nesta sessão no código ou nos documentos; **[Referência]** dito por documentação
externa, não conferido; **[Hipótese]** provável, não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

Um `.csv`, `.tsv` ou `.jsonl` aberto na IDE pode ser visto como **tabela**, e editado nela. A tabela não é um
documento à parte: **é outra vista do texto da aba**, como a prévia do Markdown. Editar uma célula vira uma edição
no texto, só daquele campo. Por isso desfazer, salvar, o aviso de alterações não salvas e a percepção de mudanças
feitas fora da IDE continuam como são hoje, sem regra nova.

Para os agentes, um componente **`table`** no painel Interfaces: o agente grava um CSV e te mostra como tabela, e
você pode **responder escolhendo linhas**.

**[Recomendação]** Ver e editar células primeiro; acrescentar e apagar linhas e colunas depois. Ordenar e filtrar
mudam só a vista, nunca o arquivo.

## O que existe hoje

- **[Fato]** Um `.csv` abre como texto puro. A IDE não tem nenhuma vista de tabela.
- **[Fato]** A prévia (`tea-ext-preview`) é um painel que segue o texto da aba ativa enquanto você digita
  (`watch_text`, `CoreEvent::ActiveText`) e desenha Markdown ou SVG. O texto continua no editor. É o modelo de "vista
  de um texto" que a tabela segue.
- **[Fato]** Uma extensão não tem como editar o texto de uma aba: o núcleo só oferece `replace` (a troca por regex da
  busca). O app já aplica edições nas abas como alterações não salvas, uma por arquivo no desfazer
  (`apply_file_edits`, usado ao renomear e nas code actions); falta um pedido que uma extensão possa fazer.
- **[Fato]** O painel Interfaces não tem tabela; no `markdown`, "tabelas e HTML aparecem como texto"
  (`.idetea/ui/FORMAT.md`). Limite de 256 KB por documento.
- **[Fato]** O manifesto pede que arquivos grandes abram sem travar (princípio 1) e que nada mude sem você ver
  (princípio 3).

## A vista de tabela

**Onde:** na própria aba, que alterna entre **Texto** e **Tabela** (um botão no canto, como o da prévia, e o
comando `table.toggle`). O painel lateral é estreito demais para uma tabela. Cada aba lembra a vista em que estava.

**Formatos:**

| Formato | Como vira tabela |
|---|---|
| CSV | RFC 4180 **[Referência]**: aspas, aspas dobradas, quebras de linha dentro de aspas. Separador detectado nas primeiras linhas (`,`, `;`, `\|`); a primeira linha é o cabeçalho (dá para desligar). |
| TSV | Tabulação, sem aspas; `\t`, `\n` e `\\` como escapes. |
| JSONL | Um objeto por linha. As colunas são as chaves, na ordem em que aparecem. Um valor aninhado aparece como JSON (`{"a": 1}`). Uma linha que não é objeto, ou não é JSON válido, aparece marcada, com o texto inteiro. |

**Na tela:**

- cabeçalho fixo, número da linha do arquivo à esquerda, rolagem virtual (só as linhas visíveis são desenhadas);
- largura das colunas pela amostra das primeiras linhas, ajustável arrastando;
- números à direita; no JSONL, `null` diferente de `""` e um número diferente de um texto com dígitos;
- **ordenar** por coluna e **filtrar** por texto: só a vista; o arquivo não muda, e a barra diz que há ordem ou
  filtro ativo;
- copiar uma célula, linhas ou a seleção, como TSV (cola numa planilha) ou CSV;
- "ir para o texto" leva à linha no modo Texto, e o cursor do texto leva à célula na volta.

**Arquivos grandes:** a leitura acha só o começo de cada linha (o índice) e interpreta as visíveis. **[Hipótese]**
Até uns 100 MB, a vista abre sem travar; acima de um limite (`[table] max_mb`, padrão 50), a aba fica só em Texto e
diz por quê, como o editor faz com arquivos binários.

## A edição

O princípio: **a tabela edita o texto, e só o que você mudou.**

- **Uma célula:** Enter, F2 ou dois cliques editam; Enter confirma, Esc desiste. A IDE troca só os bytes daquele
  campo no texto. O resto do arquivo (aspas das outras células, espaços, fim de linha `\n` ou `\r\n`, a última
  linha sem quebra) fica igual: o `git diff` mostra só a célula.
- **Aspas no CSV:** um campo que estava entre aspas continua entre aspas; um que não estava ganha aspas só se o
  novo valor precisar (separador, aspas, quebra de linha).
- **JSONL:** a célula mostra e recebe o valor como JSON (`"ana"`, `42`, `true`, `null`, `{"a": 1}`). Assim não há
  dúvida se `42` é número ou texto; um valor inválido não é aceito e diz o erro. A IDE troca só aquele valor na
  linha, sem reformatar o objeto.
- **Desfazer:** cada célula confirmada é um passo no desfazer da aba, o mesmo do modo Texto (`Ctrl+Z` funciona nos
  dois).
- **Salvar:** como qualquer aba. Nada é salvo sozinho; fechar com alterações pergunta.
- **Mudou fora da IDE** (um agente reescreveu o CSV): a aba recarrega como hoje, e a tabela acompanha.
- **Depois (degrau 2):** acrescentar e apagar linhas; acrescentar, apagar e renomear colunas (no CSV muda toda
  linha; no JSONL, a chave em todo objeto que a tem). Cada uma é um passo só no desfazer.

**No núcleo:** um pedido novo, `ctx.edit_text(path, edits)` (intervalos e textos, como as edições do LSP), que o
app aplica com o `apply_file_edits` de hoje. Serve a outras extensões também.

## Para os agentes

### Componente `table` no painel

```json
{"id": "r", "component": "table", "path": ".idetea/ui/rascunhos/falhas.csv",
 "label": "Testes que falharam (37)", "select": "many", "value": "/escolhidos", "key": "nome"}
```

- **Fonte:** `path` para um CSV, TSV ou JSONL do projeto (como a `image`: relativo à raiz, sem `..`, sem symlink),
  ou `columns` e `rows` inline para tabelas pequenas.
- **Na tela:** a mesma tabela da aba, só para ler, com ordenar, filtrar e copiar, e uma altura máxima no painel.
- **Como resposta:** `select: one|many` devolve as chaves (`key`, ou o número da linha) das linhas escolhidas;
  `requires` funciona com ela. O agente pergunta com dados: "quais destes 37 testes eu conserto primeiro?".
- **Releitura:** como a imagem, o painel relê quando o arquivo muda.
- **Na resposta:** em `media`, se a tabela foi vista (`shown`) e quantas linhas tinha.
- **Abrir na aba:** um link `file` (ou `idetea open dados.csv`) abre o arquivo na aba, na vista de tabela, para
  você editar.

### Na linha de comando

`idetea ask --table <arquivo> --pick many [--key coluna]` mostra a tabela e imprime as chaves escolhidas, uma por
linha. Para só mostrar, `idetea open <arquivo.csv>` já basta.

## Cabe na filosofia?

| Princípio | Como fica |
|---|---|
| 1. Rapidez | Rolagem virtual e índice de linhas; um limite acima do qual fica só o texto. |
| 2. Silêncio | Nada de rede; nenhum aviso a não ser o do limite. |
| 3. Previsível | Ordenar e filtrar não tocam no arquivo; editar troca só a célula; desfazer e salvar como sempre. |
| 4. Núcleo pequeno | Uma extensão (`tea-ext-table`) que dá para desligar; no núcleo, só o pedido de edição. |
| 5. Agentes | Mostrar dados como dados, e responder escolhendo linhas. |
| 7. Mentes diferentes | Dados em colunas alinhadas em vez de vírgulas; a tabela aparece quando você pede. |

## As opções

### A. Só ver

Vista de tabela (sem editar) e o componente `table`. O mais barato; editar continua no texto.

### B. Ver e editar células — recomendada

A, mais a edição de células com a troca mínima no texto e o pedido de edição no núcleo.

### C. Edição completa

B, mais linhas e colunas (acrescentar, apagar, renomear, mover). Recomendo como degrau seguinte, não junto.

**[Hipótese]** Tamanho: A, de 1200 a 1800 linhas (leitores de CSV/TSV/JSONL com índice, a vista, o componente no
painel, o `check`, o `FORMAT.md`, testes); B, mais 500 a 800 (troca mínima por formato, pedido de edição, desfazer);
C, mais 400 a 700.

## Recomendação

**[Recomendação]** Opção **B**, nesta ordem:

1. leitores de CSV, TSV e JSONL com índice de linhas e a troca mínima de um campo, com testes de ida e volta (ler,
   trocar uma célula, conferir que o resto do texto ficou igual byte a byte);
2. a vista na aba (alternar Texto/Tabela, rolagem virtual, ordenar, filtrar, copiar) e a edição de células;
3. o componente `table` no painel, `idetea ask --table`, o `check` e o `FORMAT.md`;
4. teste manual e documentação; depois, a opção C.

## Perguntas abertas

1. **Opção?** (A, B ou C)
2. **Como abre um `.csv`:** direto na vista de tabela, ou em texto com o botão para a tabela?
3. **Componente `table` no painel com escolha de linhas:** junto, ou depois da vista na aba?

## Decisões

1. **Opção B** (2026-11-17, pelo painel): ver e editar células; linhas e colunas depois (opção C).
2. **Um `.csv`, `.tsv` ou `.jsonl` abre direto na vista de tabela**; o botão volta para o texto.
3. **O componente `table` do painel vem junto** com a vista na aba.

## Andamento

**2026-11-17: opção B feita**, aprovada no teste manual pelo painel.

- `crates/tea-table`: leitura de CSV (aspas RFC 4180, separador detectado entre `,` `;` `|`), TSV (escapes) e JSONL
  (colunas pelas chaves, na ordem em que aparecem), e a troca mínima de uma célula, com testes de ida e volta.
- Vista na aba (`tea-editor`, `table_view.rs`): abre direto como tabela até 50 MB; rolagem virtual, ordenar,
  filtrar, setas, Enter/F2/dois cliques para editar, `Ctrl+C` copia a célula, `Ctrl+Z`/`Ctrl+Y` pelo desfazer do
  texto; botão na barra de abas e comando `table.toggle`.
- Painel: componente `table` (arquivo ou `columns`/`rows`, `select`, `key`, `height`), relido quando o arquivo muda;
  na resposta, `rows` e `shown` em `media`. `idetea ask --table … --pick … --key …`. `formato.md`, schema e as
  instruções para agentes atualizados.

**Desvio do desenho:** a vista ficou num módulo do `tea-editor`, não numa extensão `tea-ext-table`. A edição usa o
próprio widget do editor (`apply_lsp_text_edits`), que já põe cada célula no desfazer da aba; por isso o pedido
`ctx.edit_text` no núcleo não foi necessário. Uma aba de tabela sem servidor LSP liga um cliente mudo que só conta
as edições, para a tabela saber quando o texto mudou.

**2026-11-18: o resto da opção B**, aprovado no teste manual pelo painel: largura das colunas arrastando a borda
do cabeçalho (dois cliques voltam à medida); linhas escolhidas pelo número (Shift estende, Ctrl+A ou o `#` escolhem
todas) e copiadas como TSV (`Ctrl+C`) ou CSV (`Ctrl+Shift+C`); a caixa "Primeira linha é o cabeçalho" no CSV e no
TSV; do texto para a tabela, a célula do cursor fica escolhida; `[table] max_mb` na configuração (0 desliga).

**2026-11-26: a opção C**, escolhida no painel: na barra, **+ Linha** (depois da escolhida, ou no fim), **Apagar
linhas** (as escolhidas pelo número, ou a da célula; também `Delete`), **+ Coluna** (depois da escolhida, com um nome
livre que já fica escolhido para o Enter renomear) e **Apagar coluna**. Dois cliques num nome de coluna renomeiam:
no CSV/TSV, o campo do cabeçalho; no JSONL, a chave em todo objeto (uma chave repetida é recusada). Cada operação é
uma lista de trocas em `tea-table` (`insert_row`, `delete_rows`, `insert_column`, `delete_column`,
`rename_column`), que o editor aplica como um passo só no desfazer; o resto do texto fica igual, com os mesmos
cuidados da edição de célula (aspas, `\r\n`, a última linha sem quebra). Um registro curto do CSV continua curto.
No JSONL, a coluna nova entra no fim de cada objeto, com `null`.

**Falta:** mover linhas e colunas.

## Banco de dados (fora)

A primeira versão desta RFC tratava de um cliente de banco de dados. A conclusão, guardada aqui: um cliente
completo (árvore de schema, edição de células no banco, conexões abertas) vai contra os princípios 3 (editar e
salvar sem perceber), 4 (um segundo produto dentro da IDE) e 7 (o "shopping" do manifesto). Consultas de agentes com
aprovação (a senha fora do alcance do agente, ensaio de escritas) cabiam no princípio 5, mas o que faltava de fato
era a tabela. Se voltar, será outra RFC, em cima desta.
