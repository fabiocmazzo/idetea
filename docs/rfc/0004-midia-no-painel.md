# RFC 0004 — Mídia no painel (imagem, animação, áudio)

- **Status:** implementada (M1 a M5, aprovadas pelo painel em 2026-11-02). Aprovada em 2026-11-02, com as
  decisões do fim. Base: `main` em `ec25f8e`. Fica de fora o `[ui] audio_player` (as extensões ainda não leem o
  config).
- **Depende de:** [RFC 0001](0001-interfaces-declarativas.md) (formato `idetea-ui`) e
  [RFC 0003](0003-interfaces-pelo-uso.md) (decisão 3: o segundo projeto de dogfooding decide sobre mídia).
- **Origem:** o `lacunas.md` do "Correio de balão" (`~/NewProjects/tea-jogo`), escrito pelo agente do jogo
  enquanto revisava 14 assets com o usuário. Das 15 lacunas, as 6 pequenas já foram resolvidas (`ec25f8e`); esta
  RFC trata das que sobraram.

Marcadores: **[Fato]** conferido no código ou no uso; **[Hipótese]** provável, não testado; **[Recomendação]**
proposta, ainda não decidida.

## Resumo

O que a RFC 0003 deixou para depois apareceu no primeiro projeto com mídia. Para revisar sprites e sons, o painel
manda o usuário para fora dele: a imagem abre numa aba separada, sem comparação; a animação só se vê rodando o jogo;
e o som se ouve copiando um comando para o terminal.

**[Recomendação]** São cinco etapas pequenas, nesta ordem:

1. componentes `image` e `gallery`;
2. áudio com o tocador do sistema;
3. animação de spritesheet;
4. a resposta diz o que foi visto e ouvido;
5. abrir no app do sistema.

Nenhuma dessas etapas pede uma biblioteca nova que dependa do sistema. Vídeo e 3D ficam de fora.

## O que o uso mostrou

**[Fato]** Do `lacunas.md`, o que sobrou:

| Lacuna | O que faltou |
|---|---|
| 4 | Imagem dentro do painel, ao lado da pergunta; comparar antes e depois, ou duas opções lado a lado |
| 5 | Pixel art sem borrar (vizinho mais próximo) e fundo quadriculado para ver a transparência |
| 6 | Ver os quadros de um spritesheet animados (o balão tem 3, o pássaro 2) sem rodar o jogo |
| 9, 10, 11 | Tocar o som no painel, com duração, taxa e pico; ouvir o loop sem o intervalo do `paplay` |
| 12 (o resto) | Um veredito por item: hoje é um `checkbox` por asset (agora com caminho de objeto no `context`, `ec25f8e`) |
| 13 | Abrir no navegador (o `index.html` do jogo) |
| 14 | Saber se o usuário de fato abriu a imagem ou tocou o som |

**[Fato]** O agente contornou a falta de comparação gerando uma `previa-sprites.png` (todos os sprites ampliados
6×, sobre fundo quadriculado) e abrindo essa imagem com o `file`. Funcionou, mas o agente teve que escrever um
renderizador só para isso.

## O que já existe

**[Fato]**

- O iced 0.14 com a feature `image` (já ligada no `tea-editor` e no `tea-ext-preview`) desenha imagens com
  `filter_method(FilterMethod::Nearest)`, `content_fit`, `scale` e `opacity`. O `image::viewer` (o da aba de
  imagem) também aceita `filter_method`. Hoje a aba não usa esse método, e por isso a pixel art borra no zoom.
- A crate `image` 0.25 já está no `Cargo.lock`, trazida pelo iced. Com ela dá para recortar os quadros de um
  spritesheet e montar um `Handle::from_rgba` por quadro.
- Não existe nenhuma crate de áudio. O AppImage é feito **sem bibliotecas dentro** (SPEC, Distribuição).
  `rodio`/`cpal` ligam com a `libasound` no Linux: compilar pede os headers, e o executável passa a precisar da
  biblioteca.
- `tea_core::os::open_url` abre com `xdg-open`/`open`/`explorer`, e é ele que os links da web usam.
- Os plugins já rodam processos com prazo e limites (`host.run`, RFC 0002). O mesmo jeito de rodar serve para um
  tocador.

## Proposta

### M1. `image` e `gallery` (lacunas 4, 5 e 12). Complexidade baixa a média

```json
{"id": "balao", "component": "image", "path": "assets/sprites/balao.png", "label": "Balão (3 quadros)",
 "pixelated": true, "background": "checker", "height": 96}

{"id": "escolha", "component": "gallery", "value": "/escolha", "select": "one",
 "pixelated": true, "background": "checker", "height": 64, "items": [
   {"id": "a", "path": "assets/sprites/passaro-a.png", "label": "A"},
   {"id": "b", "path": "assets/sprites/passaro-b.png", "label": "B"}]}
```

- `path` segue as regras do `file`: relativo, sem `..`.
- A imagem é lida e decodificada na thread que já lê a pasta, nunca na thread da interface. Ela é relida quando o
  arquivo muda. Limites: 8 MB por arquivo, 4096×4096 px e 32 imagens por documento. Acima disso, aparece um aviso
  no lugar da imagem.
- `pixelated`: usa o vizinho mais próximo.
- `background`: `checker` (quadriculado) ou `none`.
- `height`: altura na tela; a largura acompanha a proporção.
- Um clique abre a imagem na aba de imagem.
- `gallery` põe as imagens lado a lado, quebrando a linha quando não cabem, com o rótulo embaixo de cada uma.
  `select` pode ser `none`, `one` ou `many`. O valor é o id escolhido ou a lista de ids. Isso cobre "escolha entre
  4 sprites" e "antes/depois" (dois itens com `select: none`).
- A aba de imagem passa a usar `pixelated` quando o zoom passa de 100%.
- Toca em: `tea-declarative-ui` (`document.rs`, `view.rs`), `tea-ext-ui` (ler as imagens no `store`),
  `tea-editor` (`image_file.rs`), formato, schema e instruções.
- Pronto quando: o agente do jogo revisa os 14 sprites sem gerar `previa-sprites.png`.

### M2. Áudio pelo tocador do sistema (lacunas 9, 10 e 11). Complexidade média

```json
{"id": "vento", "component": "audio", "path": "assets/sons/vento.wav", "label": "Vento", "loop": true}
```

- Um botão **Tocar**/**Parar**. A IDE roda o tocador que existir: `pw-play`, `paplay` ou `aplay` no Linux,
  `afplay` no macOS, ou `[ui] audio_player = ["prog", "{path}"]` no config.
- Um som de cada vez. Trocar de revisão, fechar o painel ou sair da IDE mata o processo, do mesmo jeito que o
  `host.run` faz.
- `loop`: toca de novo quando termina. **[Fato]** Fica um intervalo pequeno entre uma vez e outra, o mesmo que o
  agente notou.
- Metadados de WAV (duração, canais, taxa, pico) e uma forma de onda pequena. A IDE lê o cabeçalho e as amostras
  com um leitor próprio (umas 60 linhas, PCM 8/16/24/32 e float), sem crate nova. A forma de onda é desenhada como
  uma imagem RGBA, sem precisar da feature `canvas`. OGG e MP3 tocam, mas aparecem sem metadados.
- **[Recomendação]** O tocador do sistema, e não `rodio`, por dois motivos: não pede biblioteca de sistema nova (o
  AppImage continua sem dependências) e não leva áudio para dentro do processo da IDE. A desvantagem é que não há
  seek nem loop sem intervalo. Se o uso pedir isso, vale uma etapa própria com `rodio`, atrás de uma feature.
- Pronto quando: o agente do jogo revisa os sons sem pedir para copiar comandos.

### M3. Animação de spritesheet (lacuna 6). Complexidade média

```json
{"id": "voo", "component": "sprite", "path": "assets/sprites/passaro.png", "frames": 2, "fps": 6,
 "pixelated": true, "background": "checker", "height": 96}
```

- `frames`: quadros na horizontal, todos da mesma largura. Opcionalmente `frame_width` e `frame_height`, para uma
  grade.
- Os quadros são recortados uma vez, com a crate `image` (vira dependência direta, mas já está na árvore).
- A animação anda por `window::frames()` só enquanto o painel está visível. Um botão pausa e passa quadro a quadro.
- Depende de M1 (leitura, limites, `pixelated`, `background`).
- Pronto quando: o voo do pássaro e a chama do balão aparecem no painel na velocidade do jogo.

### M4. O que foi visto e ouvido (lacuna 14). Complexidade baixa

- A resposta ganha `media`, com o que o usuário fez com cada componente de mídia:

  ```json
  "media": {"vento": {"played": 2}, "balao": {"opened": true}}
  ```

- Isso é informação, não prova: o painel não sabe se o som estava baixo. As convenções de entrega (RFC 0003, C)
  continuam valendo.
- Depende de M1 e M2.

### M5. Abrir no app do sistema (lacuna 13). Complexidade baixa

- Link `idetea:open/<caminho>` no `markdown`, por exemplo `[jogar](idetea:open/index.html)`. Abre um arquivo do
  projeto com o app padrão do sistema, pelo `open_url`.
- **[Recomendação]** Só extensões de uma lista: `html`, `htm`, `pdf`, imagens, `wav`, `ogg`, `mp3`, `mp4` e
  `webm`. O motivo é que o `xdg-open` executa um `.desktop`, e um agente não deve conseguir rodar programas por um
  link. Fora da lista, aparece só um aviso.

## Fora desta RFC

- **Vídeo:** decodificar vídeo no iced pede GStreamer ou FFmpeg, que são pesados e ligados ao sistema.
  `idetea:open` (M5) cobre o caso de hoje.
- **3D (glTF):** nenhum uso até agora.
- **Anotar regiões de uma imagem** e **comentários em trechos de áudio:** o comentário em toda resposta
  (`comment`) cobre isso por enquanto. **[Hipótese]** Precisaria de `canvas`; só vale fazer se um projeto pedir.

## Riscos

- **Imagem malformada.** A `image` é Rust seguro, mas um PNG feito para isso pode pedir dimensões enormes. Os
  limites de M1 valem antes de decodificar, lendo só o cabeçalho.
- **Memória.** São 32 imagens de até 4096×4096 em RGBA, ou seja, até 2 GB no pior caso. **[Recomendação]** Também
  um limite de pixels somados por documento: 64 Mpx, cerca de 256 MB.
- **Tocador preso ou ausente.** Se não houver tocador, o botão fica desligado com o motivo. O processo roda num
  grupo próprio e morre inteiro, como no `host.run`.
- **Um agente que toca som sem o usuário pedir:** não acontece, só um clique toca.

## Decisões para aprovar

1. A ordem M1 → M2 → M3 → M4 → M5?
2. Áudio pelo tocador do sistema (sem crate nova), deixando `rodio` para se o uso pedir seek ou loop sem
   intervalo?
3. `gallery` como componente próprio (escolha entre imagens), em vez de vários `image` com um `select` embaixo?
4. `idetea:open` só com a lista de extensões?

## Decisões tomadas (2026-11-02, pelo painel)

1. **Ordem M1 → M2 → M3 → M4 → M5.**
2. **Áudio pelo tocador do sistema**, sem crate nova. O `rodio` entra só se o uso pedir seek ou loop sem
   intervalo.
3. **`gallery` como componente próprio** para escolher entre imagens.
4. **`idetea:open` só com a lista de extensões.**
