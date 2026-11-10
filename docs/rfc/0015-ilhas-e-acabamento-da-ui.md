# RFC 0015 — Ilhas e acabamento da UI

- **Status:** aprovada em 2026-11-10 (pelo painel), com as decisões do fim. Base: `main` em `1bd4021` (0.6.0).
- **Origem:** pedido do usuário no chat (2026-11-10): "Precisamos criar uma RFC sobre melhorias da UI. Ver se é
  viável, se vale a pena. Os 2 últimos prints […] mostram alguns detalhes da 'junção' de painéis dos produtos do
  IntelliJ e da barra de rolagem, que são mais arredondados, definem uma fronteira melhor […] e também outras
  melhorias para deixar mais elegante e suave a IDE".

Marcadores: **[Fato]** conferido nesta sessão no código do IDE TEA, do iced 0.14 (`iced_widget` 0.14.2,
`iced_core` 0.14.0) ou do fork do `iced-code-editor` (`375327c`); **[Referência]** visto nos prints do IntelliJ
(o visual novo "Islands" dos produtos JetBrains); **[Hipótese]** provável, não testado; **[Recomendação]** proposta,
ainda não decidida.

## Resumo

Dá para fazer, e vale a pena. O visual "em ilhas" do IntelliJ (cada painel num cartão de cantos arredondados,
separados por um vão de cor mais escura) e as barras de rolagem em pílula usam só o que o iced 0.14 já desenha:
retângulos com cantos arredondados, bordas, sombras e estilos de barra de rolagem por estado. Não precisa de fork do
iced nem de renderizador próprio.

Há uma limitação: o iced recorta o conteúdo num retângulo, não num retângulo arredondado. O conteúdo que pinta o
próprio fundo (editor, terminal) passaria por cima dos cantos. Um respiro interno de cerca de 0,3× o raio (3 px para
raio 10) resolve, sem custo.

Proposta em três etapas, cada uma utilizável sozinha:

1. **Ilhas e barras de rolagem** (o que os prints mostram), atrás de `[ui] style = "islands" | "classic"`.
   Estimativa: 1 a 2 dias.
2. **Acabamento**: abas mais discretas, painel com foco destacado, cabeçalhos de painel uniformes, raios e
   espaçamentos de um lugar só. Estimativa: 1 a 2 dias.
3. **Suavidade**: transições curtas (hover, divisor, abrir/fechar sobreposições) com a `Animation` do iced.
   Opcional, mais cara pelo que rende.

## 1. O que os prints mostram [Referência]

`~/Pictures/Screenshots/Screenshot_2026-11-10_12-47-48.png` e `…12-48-04.png` (DataGrip/IntelliJ, tema escuro):

- **Ilhas.** Cada área (árvore, grade de resultados, console) é um cartão com cantos arredondados (raio de
  ~8–10 px), sobre um fundo mais escuro que o cartão. Entre dois cartões fica um vão de ~6 px dessa cor escura. É o
  vão, não uma linha de 1 px, que marca a fronteira; por isso ela "lê" melhor.
- **Barra de rolagem.** Uma pílula fina (~6–8 px), totalmente arredondada, cinza médio, **sem trilho visível** (o
  fundo atrás dela é o do próprio painel). Fica a alguns pixels da borda do cartão, não colada nela.
- **Barra de ferramentas flutuante** ("2 rows ⌄ ⋮"): uma cápsula com borda fina e raio de ~6 px, solta sobre o
  conteúdo.
- **Divisões horizontais** entre a área de cima e o console são faixas do fundo escuro, não linhas.

## 2. Como a IDE desenha hoje [Fato]

- **Layout** (`tea-app/src/main.rs`, `workspace_view`): uma `column` com `padding(6)`. Barra de atividades,
  `rule::vertical(1)`, painel esquerdo, divisor, centro (editores, divisor, painel de baixo), divisor, painel
  direito. Abaixo de tudo, `rule::horizontal(1)` e a barra de status. Todos os painéis ficam no mesmo fundo
  (`background.base` do tema); a fronteira é a linha de 1 px do divisor.
- **Divisor** (`tea-app/src/divider.rs`): widget próprio, área de 5 px para o mouse, linha de 1 px
  (`background.strong`). Com o mouse em cima vira 3 px na cor de destaque.
- **Barras de rolagem:** 34 chamadas de `scrollable(…)` no app e nas extensões, todas no estilo padrão do iced
  (`scrollable::default`). Esse estilo é uma trilha com fundo (`background.weak`), raio 2, 10 px, com o polegar
  `background.strongest`. É o "retangular e pesado" que se vê na IDE. As abas e os breadcrumbs usam uma barra de
  2 px.
- **Barra do editor:** do fork do `iced-code-editor` (`canvas_editor/render/view.rs`, `canvas_scrollbar_style`).
  Raio 4 fixo, largura padrão (10 px), cores de `Style.scrollbar_background`/`scroller_color`, e **o mesmo estilo
  em qualquer estado** (não muda no hover nem ao arrastar).
- **Terminal** (`iced_term`): não tem barra de rolagem.
- **Raios espalhados:** 4 (ajuda, configurações, dicas), 5 (diálogo, `main.rs`), 6 (menu de contexto, caixas do
  editor), 8 (paleta, mascote), 10 (diálogo, sobreposições). Sombras com `blur_radius` 6, 14, 16, 24 e 30. Cada
  tela escolheu o seu.
- **Abas do editor:** botões, a ativa com `button::primary` (fundo inteiro na cor de destaque, ciano no Tokyo
  Night). Chama mais atenção que o código.
- **Temas:** os do próprio iced (`Theme::TokyoNight` escuro e `Theme::Light` claro como padrão, e qualquer um de
  `Theme::ALL`). Não há tema da IDE; as cores saem de `theme.extended_palette()`.

## 3. O que o iced 0.14 permite [Fato]

| Precisa de | Tem? | Onde |
|---|---|---|
| Cartão com cantos arredondados, borda e fundo | sim | `container::Style { background, border: Border { radius, .. } }`; raio por canto (`border::Radius`) |
| Sombra sob o cartão | sim | `container::Style.shadow` (já usado no diálogo e na paleta) |
| Barra em pílula, trilho transparente | sim | `scrollable::Style { vertical_rail: Rail { background: None, scroller: Scroller { border: radius } } }` |
| Barra que muda no hover / ao arrastar | sim | o estilo recebe `Status::{Active, Hovered, Dragged}`, com `is_vertical_scrollbar_hovered` etc. |
| Barra mais fina e afastada da borda | sim | `Scrollbar::new().width(..).scroller_width(..).margin(..)` |
| Barra que aparece só com o mouse sobre a área | quase | `Status::Hovered` vale com o cursor **sobre a área rolável**: o polegar pode ficar quase transparente fora dela. Sem timer de "some depois de 1 s", que exigiria widget próprio |
| Recortar o conteúdo no canto arredondado | **não** | o recorte (`with_layer`) é retangular: ver 4.2 |
| Transições | sim, manuais | `iced::Animation` (`iced_core/src/animation.rs`: `go`, `interpolate`, `is_animating`), com redesenho pedido pelo app |

O estilo padrão das barras não muda por tema: o iced 0.14 não deixa trocar o padrão de um widget para o app todo
sem um tipo de tema próprio. Cada `scrollable` precisa de `.style(…)`, o que é trocar 34 chamadas por um ajudante
comum. Ver 4.3.

## 4. Proposta

### 4.1 Ilhas [Recomendação]

- **Fundo da janela ("canvas")** um pouco mais escuro que o dos painéis. No tema escuro, `background.base`
  misturado com ~25 % de preto; no claro, ~6 % de cinza. Calculado do tema, para servir a qualquer tema do iced.
- **Cada área vira uma ilha:** árvore/painel esquerdo, cada lado do editor (abas + breadcrumbs + editor), painel
  de baixo, painel direito. Fundo `background.base`, raio 10, sem borda (ou borda de 1 px quase invisível nos
  temas claros, onde o contraste do vão é menor).
- **O vão é o divisor:** 6 px de canvas entre ilhas. O `Divider` passa a ocupar o vão inteiro (área de arraste
  de 6 px em vez de 5) e deixa de desenhar a linha de 1 px. No hover/arraste desenha uma barrinha arredondada de
  2–3 px na cor de destaque, centrada no vão. Duplo clique continua voltando ao padrão.
- **Barra de atividades e barra de status** ficam no canvas, fora das ilhas, sem as `rule` de 1 px: o contraste
  com as ilhas já separa.
- **Ilha com foco:** borda de 1 px em `primary.weak` (ou só um pouco mais clara) na ilha que tem o foco do teclado.
  Hoje não há indicação de qual painel recebe as teclas. Ver 4.4.
- **Opção:** `[ui] style = "islands"` (padrão) ou `"classic"` (o layout atual, para quem quer cada pixel). O vão
  custa ~12–18 px na largura e ~6 px na altura. Em telas pequenas, o modo clássico continua disponível.

### 4.2 Os cantos e o recorte retangular [Fato + Recomendação]

O iced recorta cada widget num retângulo. O editor (canvas com fundo e calha de números de linha de cor própria)
e o terminal pintam o fundo até a borda. Dentro de uma ilha arredondada, eles cobririam os cantos e o canto voltaria
a ser reto.

A conta: com raio `r`, o arco do canto passa a `r·(1 − 1/√2) ≈ 0,29·r` da quina do retângulo, na diagonal.
Um **respiro interno de `⌈0,3·r⌉`** (3 px para raio 10) põe o conteúdo inteiro dentro da curva. Com o fundo da ilha
igual ao do editor, esse respiro não aparece como borda, só como margem. Sem máscara, sem shader.

O que precisa casar: o fundo da ilha do editor = o fundo do editor (`set_theme` já passa o tema para o fork). A
calha de números de linha, de outra cor, fica 3 px para dentro, o que também é o que o IntelliJ faz.

[Hipótese] O renderizador por software (`iced_tiny_skia`, usado quando não há GPU) desenha cantos arredondados e
sombras; conferir no teste manual com `ICED_BACKEND=tiny-skia`.

### 4.3 Barras de rolagem [Recomendação]

Um ajudante em `tea-core` (por exemplo `tea_core::style::scrollbar`) com o estilo, e as 34 chamadas passam a usá-lo:

- Trilho **transparente** (sem fundo, sem borda).
- Polegar em pílula (raio = metade da largura), 6 px, a 2 px da borda (`margin`).
- Cor por estado: fora da área, `background.strong` com ~35 % de opacidade (quase some); com o cursor na área,
  ~60 %; sobre o polegar ou arrastando, 100 % (ou a cor de destaque fraca). É o efeito "aparece quando importa" sem
  timer.
- Abas e breadcrumbs mantêm a barra de 2 px, com o mesmo trilho transparente.

**Barra do editor (fork):** `canvas_scrollbar_style` passa a receber o `Status` e a usar trilho transparente, raio
total e largura de 8 px (ou configurável no `Style` do editor). É uma mudança pequena e isolada, que também serve
para propor upstream (como o PR #37). Até lá, já dá para deixar o trilho transparente sem mexer no fork: basta
`scrollbar_background = TRANSPARENT` no `Style` que a IDE passa ao editor.

**Extensões e interfaces declarativas:** as extensões embutidas usam o mesmo ajudante. As interfaces do painel
(`tea-ext-ui`) e os plugins WASM herdam o estilo pelo widget que a IDE monta, sem mudar formato nenhum.

### 4.4 Acabamento [Recomendação]

Pontos baratos que somam no "elegante":

1. **Abas do editor.** A ativa deixa de ser um botão inteiro na cor de destaque: fundo um pouco mais claro que a
   barra de abas + uma barrinha de 2 px na cor de destaque embaixo (ou em cima), cantos de 6 px em cima. O "×" só
   aparece no hover e na ativa; arquivo não salvo mostra um ponto no lugar do "×" (como VS Code e JetBrains).
2. **Cabeçalho de painel** com altura fixa (~28 px), título à esquerda e as ações à direita, igual em todos os
   painéis. Hoje cada extensão monta o seu: `text(title.to_uppercase()).size(11)` e depois o conteúdo.
3. **Painel com foco** (4.1): a borda da ilha diz onde o teclado está. Isso ajuda com terminal e editor lado a lado.
4. **Itens de lista e árvore:** seleção com raio 4–6 e 4 px de recuo nas laterais (a seleção não encosta na borda
   da ilha). Hoje a linha selecionada vai de ponta a ponta, em retângulo.
5. **Barra de atividades:** item ativo como pílula de fundo suave (raio 6), em vez de só a cor do ícone.
6. **Um lugar só para as medidas:** `tea_core::style` com `RADIUS_ISLAND = 10`, `RADIUS_CONTROL = 6`,
   `RADIUS_SMALL = 4`, `GAP = 6`, as sombras (`SHADOW_POPUP`, `SHADOW_DIALOG`) e as cores derivadas do tema
   (`canvas`, `island`, `hover`, `selected`). As ~15 telas com raio/sombra próprios passam a usar essas medidas.
   Isso facilita mexer depois e evita a deriva que já existe (5 raios e 5 sombras diferentes).
7. **Sobreposições** (paleta, menu, diálogo, sugestões do LSP): mesmo raio (10 nas grandes, 6 nos menus), borda de
   1 px e sombra suave iguais. Hoje já estão perto disso; é só unificar.

### 4.5 Suavidade [Recomendação, opcional]

Com `iced::Animation` (curvas de easing prontas; o app pede redesenho enquanto `is_animating`):

- hover dos itens (fundo aparece em ~100 ms em vez de piscar);
- barrinha do divisor surgindo no hover;
- paleta, menu e diálogo entrando com opacidade + 4 px de deslocamento (~120 ms);
- barra de rolagem esmaecendo ao sair da área.

Rende menos que 4.1–4.4 e custa mais: o estado da animação vive no app (o iced não anima estilos sozinho), e cada
animação pede quadros extras. Fica para depois das duas primeiras etapas, e só onde se nota. **Rolagem suave** no
editor é do fork e fica fora desta RFC.

## 5. Vale a pena?

**Sim, para as etapas 1 e 2.**

- **Primeira impressão.** O README, o site e os vídeos mostram prints da IDE. Hoje ela parece um protótipo
  competente: tudo no mesmo fundo, linhas de 1 px, barras de rolagem largas e quadradas, aba ativa ciano. As ilhas
  e as pílulas são exatamente o que faz IntelliJ, Zed e VS Code recentes parecerem "acabados". Para quem chega por
  uma linguagem que as outras IDEs não atendem (RFC 0014), é o que separa "parece sério" de "parece experimento".
- **Custo baixo e risco baixo.** É estilo e layout. Nenhuma mudança de formato, de configuração do usuário (fora
  a opção nova) ou de API de extensão. O modo clássico fica como saída.
- **Arruma uma dívida.** As medidas espalhadas viram um módulo só, o que barateia qualquer ajuste visual futuro.

O que **não** vale agora: tema próprio da IDE (substituir os temas do iced), ícones novos para a UI (os Codicons
estão bons), janela sem decoração do sistema com barra de título desenhada pela IDE (custo alto em três sistemas
operacionais, e quebra comportamentos do gerenciador de janelas).

## 6. Riscos

- **Espaço:** o vão consome pixels; mitigado pelo modo clássico e por `GAP` pequeno (6).
- **Temas claros:** o contraste canvas × ilha é menor. Por isso a borda de 1 px opcional nesses temas; conferir
  com `Theme::Light` e mais um claro do iced.
- **Calha do editor e terminal** com cor própria nos cantos: resolvido pelo respiro de 4.2, mas precisa de
  conferência visual em cada tema.
- **Desempenho:** alguns retângulos a mais por quadro; desprezível. As animações (4.5) são o único ponto que
  aumenta quadros, por isso ficam opcionais e curtas.

## 7. Etapas

1. **Ilhas + barras** (4.1, 4.2, 4.3): módulo `tea_core::style` com medidas e cores, ilhas no `workspace_view`,
   divisor no vão, ajudante de barra nas 34 chamadas, trilho transparente no editor via `Style`, opção
   `[ui] style`. Depois, se aprovado, a mudança do fork para a barra do editor com estado.
2. **Acabamento** (4.4): abas, cabeçalhos de painel, foco, seleção de lista, barra de atividades, sobreposições
   unificadas.
3. **Suavidade** (4.5), se ainda fizer sentido depois de usar as duas primeiras.

## 8. Como testar

- **Automático:** teste de que todo `scrollable` do app e das extensões usa o ajudante (busca no código, como o
  `every_pack_is_documented` faz para os packs). Teste das cores derivadas: canvas mais escuro que a ilha nos
  temas escuros e diferente dela em todos os temas de `Theme::ALL`.
- **Manual, pelo painel**, com prints antes/depois: Tokyo Night e Light; painéis esquerdo, direito e de baixo
  abertos; editor dividido; arrastar e dar duplo clique nos divisores; rolar a árvore, o editor e o terminal;
  `[ui] style = "classic"` volta ao atual; `ICED_BACKEND=tiny-skia` desenha igual.

## 9. Para decidir

1. Fazer a etapa 1? Com spike primeiro (ilhas só no layout principal, para ver no print antes de mexer nas 34
   barras)?
2. Padrão `islands` já na próxima versão, ou começar como opção (`classic` padrão)?
3. Medidas: raio 10 e vão 6 como no IntelliJ, ou mais discreto (raio 8, vão 4)?
4. Etapa 2 inteira, ou só abas e foco primeiro?

## Decisões (2026-11-10, pelo painel)

1. **Etapa 1 com spike primeiro:** só as ilhas no layout principal, para ver no print antes de mexer nas barras.
2. **Ilhas já como padrão** (`[ui] style = "islands"`); `"classic"` fica como opção.
3. **Medidas do IntelliJ:** raio 10, vão 6.
4. **Etapa 2 começa por abas e foco**; o resto do acabamento depois.

## Andamento

- **Spike (2026-11-10):** `tea_core::style` (raio, vão, respiro, cor do canvas derivada do tema), ilhas em volta de
  cada lado do editor e dos painéis esquerdo, direito e de baixo, divisor no vão (só aparece com o mouse),
  barra de status no canvas, `[ui] style`. Nos temas quase pretos (Moonfly, Oxocarbon) o canvas não tem como ficar
  mais escuro: fica um pouco mais claro que as ilhas (teste em todos os temas de `Theme::ALL`).
- **Etapa 1 (2026-11-10, aprovada no teste manual):** `tea_core::style::scrollable` e `scrollbar_style` (pílula de
  6 px sem trilho, a 2 px da borda; quase some fora da área, firme em cima ou arrastando) em todas as áreas
  roláveis do app e das extensões, com teste que falha se um `scrollable(` escapar. Barra do editor sem trilho e
  translúcida pelo `Style` que a IDE passa ao widget. Falta, se quisermos: afinar a largura e o raio da barra do
  editor no fork.
- **Etapa 2, abas e foco (2026-11-10, aprovada no teste manual):** `tea_core::style::tab` (ativa com fundo mais
  claro e borda fina na cor de destaque, a do outro lado só com o fundo, as demais apagadas) nas abas do editor e
  do painel de baixo; "×" só na ativa e sob o mouse, "●" nas não salvas, sem mudar a largura. A ilha com o teclado
  ganha borda na cor de destaque (`island_focused`): o lado do editor com o foco, senão o painel do último clique,
  do último comando de painel ou o aberto pela barra de atividades. Falta da etapa 2: cabeçalhos de painel,
  seleção de lista, barra de atividades e sobreposições unificadas (4.4, itens 2, 4, 5 e 7).
