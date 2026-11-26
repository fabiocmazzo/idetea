# RFC 0021 — Encodings além de UTF-8

- **Status:** implementada em 2026-11-18 (opção B).
- **Origem:** escolhida no painel em 2026-11-18, entre os itens do roadmap (`docs/features.md`: "Encodings other than
  UTF-8"). O `docs/comparativo-ides.md` lista como lacuna 3: "Quem trabalha com arquivos `ISO-8859-1` /
  `Windows-1252` não consegue editá-los na IDE TEA."
- **Relaciona:** [MANIFESTO](../../MANIFESTO.pt-BR.md) (princípios 1 e 3).

Marcadores: **[Fato]** conferido nesta sessão no código; **[Referência]** dito por documentação externa, não
conferido; **[Hipótese]** provável, não testado; **[Recomendação]** proposta, ainda não decidida.

## Resumo

Hoje um arquivo que não é UTF-8 não abre: a IDE avisa "arquivo não é UTF-8 (ainda não suportado)". É a escolha
segura (nada é corrompido ao salvar), mas deixa de fora quem mexe com código legado, legendas, CSV exportado por
Excel no Windows, arquivos `.properties` antigos, fontes em Shift_JIS ou GBK.

**[Recomendação]** Abrir no encoding do arquivo e **salvar no mesmo encoding**, sem converter nada sem você pedir.
A barra de status mostra o encoding (hoje ela sempre diz `UTF-8`), e um clique nele oferece **reabrir com outro**
(o arquivo foi lido errado) e **salvar com outro** (converter, por exemplo para UTF-8). Um caractere que não cabe no
encoding do arquivo não vira `?` em silêncio: o salvar recusa e diz qual e onde.

## O que existe hoje

- **[Fato]** `Editor::open` (`crates/tea-editor/src/lib.rs`) lê os bytes e faz `String::from_utf8`; falhando,
  devolve o erro `not-utf8`. `write_to` grava o texto em UTF-8, restaurando CRLF quando o arquivo tinha.
- **[Fato]** A barra de status recebe `encoding: "UTF-8"` fixo (`EditorStatus` em `tea-core`, preenchido em
  `tea-app/src/main.rs`).
- **[Fato]** Outros lugares leem o arquivo do disco como UTF-8, cada um do seu jeito:
  - `reload_changed_tabs` (depois de um `git pull`) usa `read_to_string`: num arquivo não UTF-8 ele falha e a aba
    **nunca** é recarregada (o erro só conta como mudança quando o arquivo sumiu);
  - trocar no projeto (`replace`) usa `read_to_string` e pula o arquivo que não abre (seguro, mas em silêncio);
  - a busca no projeto (`tea-ext-search/src/engine.rs`) usa `from_utf8_lossy`: um `ç` em Latin-1 vira `�` e a busca
    por `ação` não acha.
- **[Fato]** O `.editorconfig` já é lido (`tea-app/src/editorconfig.rs`), mas não a chave `charset` (`latin1`,
  `utf-8`, `utf-8-bom`, `utf-16be`, `utf-16le`).
- **[Fato]** Um UTF-8 com BOM abre hoje (o BOM é um caractere válido), e o BOM fica como `U+FEFF` no começo do texto.
- **[Fato]** O crate `encoding_rs` (o do Firefox) já está no `Cargo.lock` (vem pelo `serde-saphyr`, via
  `rust-i18n`), então usá-lo não baixa nada novo. **[Referência]** Ele cobre os encodings do padrão WHATWG:
  Windows-1252 e os outros Windows-125x, ISO-8859-*, KOI8, Shift_JIS, EUC-JP, ISO-2022-JP, GBK/GB18030, Big5,
  EUC-KR, UTF-16 LE/BE; decodifica sem perder bytes e diz quando um caractere não cabe ao codificar.

## Como fica

### Ao abrir

Nesta ordem, parando no primeiro que decide:

1. **BOM:** `EF BB BF` → UTF-8 com BOM; `FF FE` → UTF-16 LE; `FE FF` → UTF-16 BE.
2. **`charset` do `.editorconfig`**, se houver.
3. **UTF-8 válido** → UTF-8 (o caso de quase todo arquivo, sem custo novo: é o teste de hoje).
4. **Bytes nulos** nos primeiros 8 KB (e sem BOM de UTF-16) → é binário: continua recusado, como hoje.
5. Senão, o **encoding de reserva**: `[editor] fallback_encoding`, padrão `windows-1252` (que contém o Latin-1
   imprimível e é o que o Windows grava). **[Hipótese]** É o caso de quase todo arquivo legado em português,
   espanhol, francês, alemão e italiano.

A barra de status mostra o encoding lido (`Windows-1252`, `UTF-16 LE`, `UTF-8 BOM`). Se o palpite do passo 5 errou
(um Shift_JIS lido como Windows-1252 aparece como `‚±‚ñ`), **Reabrir com…** relê o arquivo com o que você
escolher. Com alterações não salvas, pergunta antes.

### Ao salvar

- No **mesmo encoding** em que abriu, com o mesmo BOM (ou a falta dele) e o mesmo fim de linha.
- Um caractere que **não cabe** (um emoji num Windows-1252): o salvar **recusa**, diz o caractere e a linha, e
  oferece **Salvar como UTF-8**. Nada vira `?`.
- **Salvar com…** converte: grava no encoding escolhido e a aba passa a ser dele.
- Arquivo novo: UTF-8, como hoje (ou o `charset` do `.editorconfig`).

### O resto da IDE

- Uma função só para ler texto do disco (a do editor), usada também por `reload_changed_tabs`, pelo trocar no
  projeto e pela busca. Assim a aba Latin-1 recarrega depois de um `git pull`, e a busca acha `ação` nela.
- O trocar no projeto grava cada arquivo no encoding dele (e recusa o arquivo onde a troca não cabe, dizendo qual).
- LSP: o servidor sempre recebe o texto já decodificado (o protocolo é UTF-8); nada muda.
- Git: o diff da margem compara textos; a base (`git show`) é decodificada com o mesmo encoding da aba.

## As opções

### A. Só os ocidentais, sem dependência

UTF-8 com BOM, UTF-16 LE/BE e Windows-1252/Latin-1, escritos à mão (umas 150 linhas: tabelas de 32 caracteres para o
Windows-1252). Cobre o comparativo (ISO-8859-1, Windows-1252) e nada mais; um Shift_JIS continua sem abrir.

### B. `encoding_rs`, com encoding de reserva configurável — recomendada

Todos os encodings do WHATWG. Detecção pelos passos acima (BOM, `.editorconfig`, UTF-8, reserva). Reabrir e salvar
com qualquer um deles pela barra de status. **[Hipótese]** O binário cresce algumas centenas de KB (as tabelas de
CJK); vale medir antes de decidir.

### C. B mais detecção automática

B, mais o `chardetng` (o detector do Firefox, crate novo) no passo 5, no lugar do encoding de reserva fixo: ele
adivinha o encoding pelo conteúdo (acerta Shift_JIS, GBK, KOI8). **[Referência]** Precisa de uns KB de texto para
acertar; num arquivo curto erra para Windows-1252 de qualquer jeito.

## Cabe na filosofia?

| Princípio | Como fica |
|---|---|
| 1. Rapidez | O caminho do UTF-8 continua o de hoje; decodificar outro encoding é linear e só acontece com ele. |
| 3. Previsível | Salva no encoding em que abriu; não converte sem você pedir; não troca caractere por `?`; diz o encoding na barra. |
| 4. Núcleo pequeno | Uma dependência que já está no lock (B), ou nenhuma (A). |
| 7. Mentes diferentes | O arquivo da planilha de outra pessoa abre sem um passo de conversão antes. |

## Recomendação

**[Recomendação]** Opção **B**, nesta ordem:

1. ler e gravar com encoding no editor (BOM, UTF-16, reserva, `charset` do `.editorconfig`), com testes de ida e
   volta byte a byte para cada encoding e o teste do caractere que não cabe;
2. a barra de status com o encoding e o menu Reabrir com… / Salvar com…, e `[editor] fallback_encoding`;
3. a leitura única usada por `reload_changed_tabs`, trocar no projeto e busca;
4. teste manual e documentação. A opção C fica para depois, se o encoding de reserva errar muito na prática.

## Perguntas abertas

1. **Opção?** (A, B ou C)
2. **Encoding de reserva padrão:** `windows-1252`, ou perguntar na primeira vez que um arquivo não UTF-8 abre?
3. **UTF-8 com BOM:** manter o BOM ao salvar (recomendado: o arquivo não muda sem você pedir), ou tirar?

## Decisões

1. **Opção B** (2026-11-18, pelo painel): `encoding_rs`, com encoding de reserva configurável.
2. **Reserva padrão `windows-1252`** (`[editor] fallback_encoding`); errou, Reabrir com….
3. **UTF-8 com BOM mantém o BOM** ao salvar.

## Andamento

**2026-11-18: opção B implementada**, aprovada no teste manual pelo painel. O módulo `tea_core::encoding` decodifica
(BOM, dica do `.editorconfig`, UTF-8, binário, reserva) e codifica (o UTF-16 à mão, porque o `encoding_rs` só o
decodifica), e é a leitura única usada pelo editor, por `reload_changed_tabs`, pelo trocar no projeto e pela busca.
A barra de status mostra o encoding; um clique leva ao comando `file.encoding` (Reabrir com o encoding… / Salvar com o
encoding…). Reabrir exige a aba sem alterações e recusa bytes inválidos no encoding escolhido. Um caractere que não
cabe faz o salvar recusar, dizendo qual, a linha e a coluna.

**2026-11-18: o resto**, aprovado no teste manual pelo painel. A margem do git recebe os bytes do último commit
(`head_bytes` em `tea-ext-git`) e o app os decodifica com o encoding de cada aba, então um arquivo Latin-1 sem
alterações não tem marcas. O caractere que não cabe abre um diálogo com **Salvar como UTF-8**, e salvar com outro
encoding avisa o git como um salvar comum.

**2026-11-26: a opção C, só para CJK** ([issue #11](https://github.com/fabiocmazzo/idetea/issues/11)). Depois do
UTF-8 e da checagem de binário, o `chardetng` dá um palpite. Ele só vale quando:

- é um encoding CJK de vários bytes (Shift_JIS, EUC-JP, GBK, GB18030, Big5, EUC-KR);
- decodifica sem nenhum byte inválido;
- o texto tem ao menos dois caracteres CJK, e quase todo o resto que não é ASCII também é CJK.

Fora isso, vale a reserva, como antes. Um palpite de um byte só (Latin-1, cirílico) nunca troca a reserva: em texto
curto o detector erra entre eles, e a reserva é a escolha do usuário. Com uma reserva CJK, não há palpite. ISO-2022-JP
fica de fora, porque os bytes dele são ASCII e o arquivo já abre como UTF-8. Os testes cobrem as seis codificações e
textos latinos curtos que também seriam Shift_JIS válido (`ação` em Windows-1252).

**Falta:** nada desta RFC.
