# RFC 0008 — Ícones de arquivo por linguagem e tipo

- **Status:** aprovada em 2026-11-06 (pelo painel), com as decisões do fim. Base: `main` em `60a02d2` (0.3.1).
- **Origem:** pergunta do usuário no chat (2026-11-06): "quais opções temos para colocar no lugar do símbolo
  genérico de arquivo um ícone que lembre a linguagem ou o tipo, como é vscode e similares".

Marcadores: **[Fato]** conferido no código ou na fonte; **[Hipótese]** provável, não testado;
**[Recomendação]** proposta, ainda não decidida.

## Resumo

Dá para fazer sem dependência nova e sem mexer no resto da IDE: **um conjunto de ícones é só uma tabela "nome ou
extensão → desenho e cor"**, e o VS Code já tem um formato público para essa tabela (os *file icon themes*). A
recomendação é embutir o **Seti**, o tema padrão do VS Code (MIT, uma fonte de 37 KB, 383 ícones com cores e
variante para tema claro), lido pelo mesmo formato JSON do VS Code. Assim, depois, outros temas no mesmo formato
(Material Icon Theme, vscode-icons) entram sem reescrever nada. Recurso novo sem quebra: versão menor (**0.4.0**).

## 1. Como é hoje [Fato]

- Todo arquivo usa o mesmo ícone, `Icon::File` (Codicons `0xEA7B`), em dois lugares: a árvore
  (`tea-ext-filetree`, `lib.rs:567`) e a lista do Ctrl+P (`tea-app/src/palette.rs:372` e `:401`).
- As abas, os breadcrumbs, a busca e o painel do git mostram arquivos **sem** ícone.
- Os ícones da IDE vêm de uma fonte só, a **Codicons** (`tea-core/assets/codicon.ttf`, 80 KB, CC BY 4.0), desenhada
  como texto: segue a cor do tema e o tamanho da fonte de graça.
- O widget `svg` do iced já está compilado no executável (a prévia usa, `tea-ext-preview` com a feature `svg`):
  ícones em SVG não trazem dependência nova.
- A IDE já sabe a linguagem de cada arquivo (o editor escolhe as cores da sintaxe e o servidor LSP por ela).

## 2. O que se espera (como no VS Code)

- Um ícone por **nome exato** (`Cargo.toml`, `Dockerfile`, `.gitignore`, `README.md`), senão pela **extensão** (a
  mais longa primeiro: `d.ts` antes de `ts`, `test.rs` se o tema tiver), senão pela **linguagem**, senão o genérico.
- Cor por tipo, legível no tema escuro e no claro.
- O mesmo ícone em todo lugar que mostra um arquivo.

## 3. As opções

### A. Só Codicons, com cor por tipo

A Codicons tem poucos ícones de arquivo genéricos (código, JSON, Markdown, mídia, PDF, zip, binário,
configuração). Mudaria a forma em uns 8 casos e daria uma cor por linguagem.

- **A favor:** zero arquivo novo; nada a licenciar além do que já há.
- **Contra:** não "lembra a linguagem": Rust, Python e TypeScript ficam com o mesmo desenho, só a cor muda.

### B. Seti, embutido como fonte [Recomendação]

O tema padrão do VS Code. Vem do projeto [seti-ui](https://github.com/jesseweed/seti-ui) (MIT); o VS Code o mantém
em `extensions/theme-seti` (MIT). **[Fato]** O arquivo `vs-seti-icon-theme.json` tem 383 ícones, 238 extensões,
101 nomes de arquivo e 83 linguagens mapeados, cada um com o caractere na fonte e a cor (ex.: `rs` → `\E082`,
`#6d8086`; `md` → `\E060`, `#519aba`), e uma seção `light` com as cores para tema claro. A fonte `seti.woff` tem
37 KB.

- **A favor:** pequeno; é texto, como a Codicons (mesmo jeito de desenhar, nítido em qualquer tamanho); cores
  pensadas para escuro e claro; é o que a maioria já conhece do VS Code; o mapeamento já está pronto.
- **Contra:** o estilo é simples, de uma cor, menos "colorido" que o Material. Não tem ícones de pasta (as pastas
  continuam com os da Codicons).
- **Trabalho:** converter `seti.woff` para `.ttf` uma vez (**[Hipótese]** o leitor de fontes do iced, `fontdb`/
  `ttf-parser`, não lê WOFF; a conversão é com o `fontTools`, fora do build) e embutir a fonte e o JSON.

### C. Material Icon Theme, em SVG

O tema de ícones mais instalado no VS Code ([material-extensions/vscode-material-icon-theme](https://github.com/material-extensions/vscode-material-icon-theme),
MIT, ativo). **[Fato]** 910 SVGs coloridos, 714 KB no total.

- **A favor:** o visual mais moderno e reconhecível; ícones de pasta por nome (`src`, `docs`, `.github`...).
- **Contra:** 714 KB a mais (ou um recorte só das linguagens mais comuns); as cores são fixas no desenho, e alguns
  ícones perdem contraste no tema claro (**[Hipótese]**: o tema tem variantes `_light` para parte deles); o
  mapeamento fica num código TypeScript do projeto, não num JSON pronto (**[Hipótese]**: o pacote publicado traz um
  JSON gerado no formato do VS Code). Desenhar SVG por linha é mais caro que texto (**[Hipótese]**: o iced guarda a
  imagem já rasterizada por tamanho, então o custo é na primeira vez).

### D. vscode-icons, em SVG

[vscode-icons/vscode-icons](https://github.com/vscode-icons/vscode-icons) (MIT). **[Fato]** 1723 SVGs, 3,1 MB.
Cobre quase tudo, mas é pesado demais para embutir; só faria sentido como tema externo (opção G).

### E. Nerd Fonts (Symbols)

Uma fonte com milhares de símbolos, incluindo os do Seti e os Devicons. **[Fato]** A licença do repositório é mista
(`NOASSERTION` no GitHub: cada conjunto de símbolos tem a sua). **[Hipótese]** A fonte só de símbolos passa de 2 MB.
Não traz mapeamento de extensão para ícone: teríamos de montar a tabela. Sem vantagem sobre o B.

### F. Logotipos das linguagens (Devicon, Simple Icons)

[Devicon](https://github.com/devicons/devicon) (MIT) tem os logotipos oficiais. São **marcas registradas** (o uso
como ícone de arquivo é comum, mas cada marca tem regras); não cobrem configs e tipos genéricos (`.env`, `.lock`,
`.toml`); e logotipos coloridos destoam do resto da interface. Não recomendo como base.

### G. Temas externos no formato do VS Code (depois)

Ler um *file icon theme* do VS Code de uma pasta: o JSON (`iconDefinitions`, `fileExtensions`, `fileNames`,
`languageIds`, `light`) com ícones de fonte (`fontCharacter`, como o Seti) ou de imagem (`iconPath`, como o Material).
Com o leitor do B pronto, aceitar `iconPath` em SVG é o passo que falta. Quem quiser o Material instala o tema e
aponta a IDE para ele, sem a IDE carregar 714 KB para todo mundo:

```toml
[ui]
file_icons = "seti"        # "seti" (embutido), "none" (o genérico de hoje) ou o caminho de um tema do VS Code
```

## 4. Comparação

| | A Codicons | B Seti | C Material | D vscode-icons | E Nerd Fonts |
|---|---|---|---|---|---|
| Tamanho a mais | 0 | ~40 KB + JSON | 714 KB (ou recorte) | 3,1 MB | > 2 MB |
| Lembra a linguagem | não | sim | sim, mais | sim | sim |
| Cores no tema claro | sim | sim (seção `light`) | parcial | parcial | a montar |
| Mapeamento pronto | a montar | sim (JSON VS Code) | a gerar | sim (JSON) | a montar |
| Licença | CC BY 4.0 (já temos) | MIT | MIT | MIT | mista |
| Desenho | texto | texto | SVG | SVG | texto |

## 5. Onde mostrar

1. **Árvore de arquivos** e **Ctrl+P** (trocam o genérico de hoje).
2. **Abas** (hoje sem ícone): um ícone pequeno antes do nome ajuda a achar a aba certa.
3. **Breadcrumbs** (o último pedaço, o arquivo), **busca** (cabeçalho de cada arquivo) e **painel do git**.

**[Recomendação]** 1 e 2 na primeira etapa; 3 logo depois, porque é só chamar a mesma função.

## 6. Como fica no código [Recomendação]

- Em `tea-core`, um módulo `file_icons` com uma função só: `file_icon(path, language) -> FileIcon` (caractere,
  fonte, cor no escuro e no claro). Lê o JSON do tema uma vez, na primeira chamada.
- Ordem de busca do VS Code: nome exato (minúsculo), extensões compostas da mais longa para a mais curta, linguagem,
  o padrão do tema; nada achado → `Icon::File`, como hoje.
- Pastas continuam com os ícones da Codicons (o Seti não tem pasta).
- A árvore, o Ctrl+P e as abas chamam essa função; a cor vem do tema escuro ou claro conforme o tema da IDE.
- Testes: a tabela (nome exato vence extensão, `d.ts` vence `ts`, linguagem quando não há extensão, desconhecido
  vira o genérico) e o JSON embutido lido sem erro.
- Licenças: a fonte e o JSON do Seti no `NOTICE.md` e a licença MIT junto nos pacotes, como foi feito com a
  JetBrains Mono.

## 7. Etapas

1. **B na árvore, no Ctrl+P e nas abas**, com `[ui] file_icons = "seti" | "none"`.
2. Breadcrumbs, busca e git.
3. **G**: temas externos do VS Code, primeiro os de fonte, depois os de SVG (o Material entra aqui).

## 8. Para decidir

1. Qual conjunto embutido: **B (Seti, recomendado)**, A (só Codicons) ou C (Material, mais pesado)?
2. Ícones nas abas, que hoje não têm?
3. Onde mais na primeira etapa (breadcrumbs, busca, git) ou deixar para a etapa 2?
4. Temas externos (G) entram no plano ou ficam fora por enquanto?

## Decisões (2026-11-06, pelo painel)

1. **Seti**, embutido.
2. **Abas com ícone**, ao lado do nome ("dá para incluir abas aí também, ao lado do nome do arquivo?").
3. Breadcrumbs, busca e git **junto**, na primeira etapa: as etapas 1 e 2 viraram uma só.
4. Temas externos do VS Code (G) **no plano**, como a próxima etapa.

## Como ficou

- `tea-core::file_icons`: a tabela do Seti lida do JSON do VS Code (uma vez), `lookup(nome)` e `icon(nome, tamanho,
  colorido)`. A fonte convertida de WOFF para TrueType por um script de 30 linhas (tabelas zlib → sfnt), sem o
  `fontTools`; carregada antes da janela, com a JetBrains Mono (`fonts::load_embedded`).
- **[Fato]** O tema do Seti mapeia várias linguagens só por `languageId` (`.py`, `.go`, `.c` não estão em
  `fileExtensions`): a IDE tem a sua tabela extensão → `languageId` do VS Code, e nomes como `Dockerfile` e `Makefile`.
- **[Fato]** O Seti não tem ícone separado para `.d.ts`; extensões compostas que ele tem: `spec.js`, `test.tsx`,
  `js.map`...
- Na aba ativa e na linha escolhida do Ctrl+P (fundo na cor de destaque), o ícone fica na cor do texto.
- `[ui] file_icons = "seti" | "none"`, também em Configurações → Aparência; outro valor vira aviso no config.
