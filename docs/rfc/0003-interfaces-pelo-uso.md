# RFC 0003 — Evolução das interfaces pelo uso (dogfooding)

- **Status:** aprovada em 2026-11-02 (pelo painel), com as decisões do fim. Base: `main` em `18b5f33`.
- **Depende de:** [RFC 0001](0001-interfaces-declarativas.md) (formato `idetea-ui`, painel Interfaces) e
  [RFC 0002](0002-plugins-wasm.md) (plugins, `host.run`).
- **Origem:** `ideia.md` (fora do git) pedia um plano amplo: decisões, simulações, execução de código, revisão de
  imagem, áudio, sprites, vídeo e 3D. A decisão (pelo painel) foi outra: desenvolver a IDE usando o próprio painel e
  deixar o uso dizer o que falta, em vez de planejar tudo antes.

Marcadores: **[Fato]** conferido no código ou no que aconteceu; **[Hipótese]** provável, não testado;
**[Recomendação]** proposta, ainda não decidida.

## Resumo

Durante o desenvolvimento de cerca de 8 itens do SPEC, todas as perguntas e entregas de teste passaram pelo painel. Saíram 9
lacunas no SPEC, todas resolvidas, e mais 3 que esta RFC registra.

A principal conclusão é que **os componentes não foram o gargalo**. Com `text`, `markdown`, `select`, `checkbox` e
`button`, deu para fazer tudo o que foi preciso. O que falhou, e quase gerou commits errados, foi a **confirmação**:

- um teste aprovado numa IDE antiga (duas vezes);
- um passo pulado antes do "Aprovado" (três vezes);
- um aviso fixo que o próprio agente escreveu e que ficou falso entre revisões.

Em todos esses casos, o agente só pegou o erro conferindo por fora: o arquivo no disco, o processo e o log.

**[Recomendação]** As próximas etapas devem fazer a IDE dizer ao agente **o que de fato aconteceu**, em vez de
acrescentar componentes. Imagem, áudio, sprites, simulações e execução de código ficam de fora até um projeto
dogfooded precisar deles.

## Como foi o uso

Interfaces usadas nesta fase, todas escritas pelo agente de código deste repositório (contagem aproximada):

| Tipo | Uso | Forma |
|---|---|---|
| Entrega de teste manual | uma por item; várias com revisão 2 ou 3 | `markdown` com os passos + "Aprovado, pode commitar" / "Aprovado, sem commitar" / "Falhou" |
| Próximo passo | depois de cada merge | `select` com as opções (a recomendada primeiro) + `checkbox` "Fazer o push" |
| Decisão de projeto | 2 (`ideia.md`, RFC 0002) | vários `select` e um botão |

- O campo de comentário (lacuna 3) foi usado em quase toda resposta que trouxe novidade: perguntas de volta,
  "falhou porque…" e ideias. A lacuna 9 nasceu de um desses comentários.
- Esperar por arquivo (`until ls …; do sleep 2; done`, em segundo plano) funcionou sem problemas. Nenhuma resposta
  se perdeu e nenhuma foi lida duas vezes.

**[Recomendação]** Não trocar a pasta por IPC agora: não houve nenhum problema que IPC resolvesse.

## O que o uso mostrou

### Lacunas resolvidas (1–9)

Estão no SPEC, no item "Desenvolver a IDE TEA com a IDE TEA". Em resumo:

| Lacuna | O que faltou | O que entrou |
|---|---|---|
| 1 | "Ensinar" não criava o `CLAUDE.md` | `AGENTS.md` + `CLAUDE.md` com `@AGENTS.md` |
| 2, 4 | Texto formatado, passos numerados, link para o config | componente `markdown`, links `idetea:config*` |
| 3 | Responder com uma pergunta | comentário em toda resposta |
| 5 | "Aprovado" sem dizer se pode commitar | convenção dos três botões (só nas instruções) |
| 6 | Instruções em três idiomas | um texto só, em inglês, que pede o idioma do usuário |
| 7 | Pergunta feita só no chat passou despercebida | instruções: toda pergunta vai para o painel |
| 8 | Teste rodado na IDE antiga | a IDE percebe que foi recompilada e oferece reiniciar |
| 9 | Comando do painel difícil de copiar | botão "Copiar" nos blocos de código |

Seis das nove foram de **forma** (texto, links, copiar, idioma). Só a 8 foi de confirmação, e ela se repetiu.

### Lacunas novas (10–12)

10. **Aprovado antes de terminar todos os passos.** "Ensinar os agentes" ficou de fora duas vezes, e "Reiniciar a
    IDE" uma vez. O botão vale pela entrega inteira, e o usuário clica quando o passo principal deu certo.
    **[Fato]** O agente só percebeu olhando o `CLAUDE.md` no disco e o PID que vem no `event` da resposta.
11. **O agente não sabe qual IDE respondeu.** Para descobrir, ele cruzou `pgrep`, `/proc/<pid>/exe` e o horário
    de abertura com o PID que está no `event`. **[Fato]** Isso funcionou, mas depende do formato do `event`, que
    não é contrato.
12. **Texto fixo do agente envelhece.** Um aviso "esta IDE é anterior a este item" continuou na tela depois que o
    usuário abriu a IDE nova, e o teste falhou por causa dele. O agente afirmou algo sobre o estado da IDE que só
    a IDE sabe.

## Proposta

Etapas pequenas, cada uma com teste pelo painel. Elas não dependem umas das outras, salvo onde está indicado.

### A. Quem respondeu (lacunas 8, 11 e 12). Complexidade baixa

A resposta ganha `ide` com o que só a IDE sabe:

```json
{"action": "aprovado-commitar", "ide": {"pid": 579522, "version": "0.0.1", "outdated": false}, "...": "..."}
```

- `outdated`: o executável mudou depois que esta IDE abriu (a lacuna 8 já mede isso).
- O painel mostra o mesmo estado no cabeçalho ("versão nova disponível"). Assim o agente não precisa escrever avisos
  sobre a IDE (lacuna 12), e as instruções dizem para não escrever.
- Toca em: `tea-core` (um evento com o estado do executável), `tea-ext-ui` (resposta e cabeçalho),
  `response.rs`, formato e schema.
- Pronto quando: o agente confere a versão pela resposta, sem `pgrep`; uma resposta dada numa IDE desatualizada
  chega com `outdated: true`.

### B. Passos que se marcam (lacuna 10). Complexidade baixa a média

Um componente `steps`: lista numerada em que cada passo é um `checkbox`, com texto em Markdown de uma linha.

```json
{"id": "passos", "component": "steps", "value": "/feitos", "items": [
  {"id": "copiar", "text": "Clique em **Copiar** e cole num terminal"},
  {"id": "ensinar", "text": "Paleta: `Interfaces: ensinar os agentes`"}]}
```

- O valor é a lista dos ids marcados e vai na resposta como os outros campos.
- O botão pode exigir todos marcados: `"requires": "/feitos"` fica desabilitado até todos os passos estarem
  marcados.
- **[Hipótese]** Isso basta para que um passo não seja esquecido. Se o usuário marcar tudo sem fazer, só a
  conferência no disco resolve, e por isso o agente continua conferindo.
- Alternativa rejeitada: um botão por passo. Fica mais lento e vira um formulário.

### C. Convenções nas instruções (lacunas 5, 7 e 12). Só documentação

O que foi aprendido e que não precisa de código:

- a entrega de teste com os três botões;
- conferir no disco antes de commitar;
- não afirmar o estado da IDE;
- ler o `ide` da resposta (depois da etapa A).

Entra em `agents/instructions.md` e no `formato.md` (seção "Entregas de teste").

### D. Fora por enquanto (o resto do `ideia.md`)

**[Fato]**

- Não existem expressões nem simulações no código. O `ideia.md` diz que existem, mas o
  `tea-declarative-ui` só tem `text`, `markdown`, `select`, `checkbox`, `input`, `file` e `button`.
- Nenhuma entrega desta fase precisou de imagem, áudio, vídeo ou cálculo.

**[Recomendação]**

- **Imagem** (`image` com caminho do projeto, sem zoom) é a candidata mais barata, porque o iced já desenha
  imagens. Entra quando um projeto dogfooded tiver imagens para revisar, por exemplo um jogo 2D aberto na IDE.
- **Cálculo e simulação:** os plugins já rodam código isolado (RFC 0002, `host.run`, prazo e limites). Uma
  simulação pode ser um plugin com painel, sem uma linguagem de expressões dentro do JSON. Só vale reavaliar se
  aparecer um caso que um plugin não resolva bem.
- **Áudio, vídeo e 3D:** são dependências pesadas e não têm uso hoje. Ficam para uma RFC própria, quando houver um
  projeto que precise.

**[Recomendação]** Um segundo alvo de dogfooding, um projeto que não seja a própria IDE, revelaria as lacunas
de mídia. Ver a decisão 3.

## Riscos

- **O próprio agente enviesa as lacunas.** É o mesmo agente que escreve as interfaces e registra o que faltou.
  Comentários do usuário pesam mais. As lacunas 3 e 9 vieram deles.
- **A amostra é pequena.** São um projeto, um agente e um usuário. O que vale para eles pode não valer para outro
  agente: Codex, Gemini e outros nunca leram estas instruções. **[Hipótese]** As instruções em inglês ajudam.
- **`steps` com `requires` pode irritar** quando um passo não se aplica. Um passo opcional
  (`"optional": true`) resolve sem ter que tirar o `requires`.

## Decisões para aprovar

1. Fazer A e B (e C junto) antes de qualquer componente de mídia?
2. `requires` em `button` (o botão espera os passos) ou só mostrar quantos passos faltam, sem bloquear?
3. Escolher um segundo projeto para dogfooding (qual?) antes de decidir sobre imagem e mídia?
4. Simulações como plugins (RFC 0002) em vez de expressões no JSON?

## Decisões tomadas (2026-11-02, pelo painel)

1. **A, B e C antes de mídia.** Nenhum componente de mídia agora.
2. **`requires` no botão:** o botão espera os passos marcados; um passo pode ser `optional`.
3. **Segundo projeto de dogfooding depois de A, B e C.** É ele que decide sobre imagem e mídia.
4. **Simulações como plugins** (RFC 0002), sem expressões no JSON.
