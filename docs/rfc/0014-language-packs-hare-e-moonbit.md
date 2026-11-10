# RFC 0014 — Language packs: Hare e MoonBit

- **Data:** 2026-11-09
- **Status:** implementado e validado no dogfooding, incluindo o smoke test real de `hare-lsp` (2026-11-09; ver o
  README do pack Hare, que registra uma queda do servidor com certos erros de sintaxe).
- **Base:** `main` em `181948b` (0.5.1).
- **Origem:** pedido do usuário no dogfooding: definir uma arquitetura para apoiar linguagens open-source com
  pouca cobertura em outras IDEs, tendo [Hare](https://harelang.org) e MoonBit como primeiros candidatos.

Marcadores: **[Fato]** conferido no código ou na fonte; **[Hipótese]** provável, ainda não testado;
**[Recomendação]** decisão de projeto, cuja implementação é indicada pelo status e pelo registro abaixo.

## Resumo

**[Fato]** Foram criados **language packs declarativos embutidos**, começando por Hare e MoonBit. Um pacote reúne
a identidade da linguagem, os tipos de arquivo, as gramáticas de cores, a configuração padrão do LSP e, quando houver
um marcador confiável de projeto, tarefas comuns. O compilador, o servidor e o depurador continuam sendo ferramentas
externas instaladas pelo usuário.

A primeira versão não instala pacotes pela rede, não baixa toolchains, não usa WASM e não migra o realce para
tree-sitter. Também não coloca debug no manifesto: a descoberta de executáveis varia por backend e merece validação
com os projetos reais antes de virar uma capacidade declarativa.

Os dois pilotos são úteis porque testam situações diferentes:

- **Hare:** já há gramática `.sublime-syntax` reutilizável e um LSP comunitário usado pelo Helix, mas o servidor ainda
  tem cobertura parcial. O valor principal da IDE é oferecer uma integração pronta e honesta sobre esses limites.
- **MoonBit:** tem LSP oficial no toolchain (`moon lsp`), gramática Sublime sob MIT e comandos uniformes de build e
  teste. Já existem integrações para VS Code, Zed e Helix; portanto não é tão descoberto quanto Hare, mas continua
  fora do conjunto atendido por muitas IDEs e é um bom teste de uma toolchain que evolui depressa.

## 1. O que significa apoiar uma linguagem

Não basta colorir uma extensão. O suporte pode ser dividido em capacidades independentes:

| Capacidade | De onde vem |
|---|---|
| Reconhecer arquivos e mostrar um nome/ícone | Registro de linguagem e tema de ícones |
| Cores, comentários e delimitadores | Gramática e comportamento do editor |
| Diagnósticos, hover, completar, navegação e formatação | LSP, quando o servidor oferece |
| Build, teste, execução e formatação pelo CLI | Tarefas e comandos do projeto |
| Breakpoints, pilha e variáveis | DAP mais build/descoberta do executável |
| Modelo de projeto novo | Skills/templates, não o language pack |

**[Recomendação]** Não prometer todas as linhas em todo pacote. Cada linguagem terá uma matriz de suporte explícita;
as capacidades disponíveis dependem também do que o toolchain upstream implementa. O language pack integra essas
capacidades, não reimplementa compilador, parser ou servidor.

Ficam fora deste RFC: code actions, semantic tokens, inlay hints e outras lacunas gerais de LSP; modelos de projeto;
depuração de JavaScript/WASM; e uma loja de pacotes.

## 2. O que a IDE já tem [Fato]

- **LSP genérico:** `tea_lsp::ServerConfig` descreve comando, argumentos e extensão → `languageId`
  (`crates/tea-lsp/src/lib.rs`). `default_servers()` contém os servidores conhecidos, e
  `[lsp.servers.<nome>]` permite ao usuário substituir ou acrescentar um servidor sem mudar a IDE.
- **Sintaxes:** `tea-core/src/syntaxes.rs` embute as do ZX e lê `~/.config/idetea/syntaxes/*.sublime-syntax`.
  O `tea-editor` registra tudo antes da primeira aba. Isso já é quase um pacote de cores, mas sem identidade comum
  com LSP, tarefas e projeto.
- **Tarefas:** `tea-app/src/tasks.rs` junta `[tasks]` do usuário com detecção de Cargo, Zig, npm e Makefile.
  A descoberta é código Rust por ecossistema.
- **Debug:** `[debug]` já aceita `program`, `args`, `cwd` e `build`. Cargo e Zig têm build e descoberta automáticos;
  os demais podem usar o config do projeto.
- **Ícones:** o Seti é consultado por nome, extensão e `languageId`. Extensões desconhecidas podem usar o genérico;
  não é necessário fingir que Hare é C ou que MoonBit é outra linguagem.
- **Plugins WASM:** o host existe como feature fora do padrão, mas sua API atual serve a extensões de UI/comandos,
  não ao registro declarativo de linguagens. Não há ganho em passar por ele para dados embutidos.

A lacuna arquitetural é a **falta de um registro único**: hoje uma linguagem nova pode exigir alterações separadas
em `tea-lsp`, `tea-core`, `tea-editor`, `tasks.rs`, `debug_build.rs` e na tabela de ícones.

## 3. Arquiteturas consideradas

### A. Language packs declarativos embutidos [Recomendação]

Cada pacote tem um manifesto e recursos estáticos. Um registro comum responde qual linguagem pertence a um arquivo e
entrega as configurações às integrações existentes.

**Prós:** pouco código por linguagem; validação e testes uniformes; documentação e licenças ficam junto do recurso;
não aumenta a API de plugins; abre caminho para ler o mesmo formato do usuário no futuro.

**Contras:** não representa bem comportamento dinâmico. O fallback do Ziex e a localização de certos executáveis
continuam precisando de Rust. O formato deve ser pequeno para não virar uma linguagem de programação em TOML.

### B. Uma extensão Rust por linguagem

Uma trait `LanguagePack` teria métodos para arquivos, LSP, tarefas e debug, com uma crate ou módulo por linguagem.

**Prós:** qualquer comportamento é representável; não há limites de schema.

**Contras:** até `extensão + comando + gramática` exigiria código e wiring. A trait tenderia a reproduzir a API de
extensões internas antes de existir necessidade. Continua sendo o escape correto para exceções, não o padrão.

### C. Apenas configuração do usuário

Já é possível configurar LSP e tarefas e copiar uma gramática para a pasta de sintaxes.

**Prós:** nenhuma infraestrutura nova; útil para experimentar uma linguagem antes de embuti-la.

**Contras:** não é suporte pronto; cada usuário repete a integração; os passos ficam dispersos e não há uma matriz de
capacidades ou testes da IDE. Deve continuar existindo como personalização, não como a experiência padrão dos pilotos.

### Por que não WASM agora

LSP e DAP já isolam os processos de linguagem. Um componente WASM adicionaria serialização, versionamento de API e
o custo do host para entregar principalmente strings e arquivos estáticos. **[Recomendação]** Reavaliar somente se
pacotes externos precisarem executar descoberta dinâmica e essa lógica não puder ficar num hook interno.

## 4. Arquitetura proposta

### 4.1 Layout e registro

```text
crates/tea-core/
├── assets/languages/
│   ├── hare/
│   │   ├── pack.toml
│   │   ├── hare.sublime-syntax
│   │   ├── README.md
│   │   └── LICENSE.txt
│   └── moonbit/
│       ├── pack.toml
│       ├── moonbit.sublime-syntax
│       ├── README.md
│       └── LICENSE.txt
└── src/languages.rs
```

`tea_core::languages` tem:

- o manifesto tipado e a validação;
- `builtins()` com os pacotes embutidos por `include_str!`;
- o mapa de extensões para `languageId`, consumido pelo gerenciador de LSP;
- as fontes de sintaxe, reutilizadas por `tea-core::syntaxes` e registradas pelo `tea-editor`;
- descrições de LSP e tarefas, convertidas para os tipos atuais nas camadas que já os consomem.

O `tea-lsp` continua sem depender de UI ou do registro: `tea-app::lsp::servers_from_config` junta os servidores dos
pacotes aos atuais e aplica as sobrescritas do usuário por último. Não é necessário migrar Rust, C/C++, Python, Go,
Lua ou Zig para pacotes nesta etapa.

### 4.2 Manifesto mínimo

O manifesto MoonBit embutido é:

```toml
format = 1
id = "moonbit"
name = "MoonBit"

[files]
extensions = { mbt = "moonbit", mbti = "moonbit" }
syntaxes = ["moonbit.sublime-syntax"]

[project]
markers = ["moon.mod"]

[lsp]
name = "moonbit"
command = "moon"
args = ["lsp"]

[[tasks]]
name = "moon check"
command = "moon check"

[[tasks]]
name = "moon build"
command = "moon build"

[[tasks]]
name = "moon test"
command = "moon test"

[[tasks]]
name = "moon fmt --check"
command = "moon fmt --check"
```

E o essencial de Hare:

```toml
format = 1
id = "hare"
name = "Hare"

[files]
extensions = { ha = "hare" }
syntaxes = ["hare.sublime-syntax"]

[lsp]
name = "hare-lsp"
command = "hare-lsp"
args = ["-S"]
```

Em `[files]`, o valor de `extensions` é o identificador enviado ao LSP em `textDocument/didOpen`; a gramática e o
nome mostrado pelo editor já são resolvidos separadamente. Não há necessidade demonstrada de um terceiro campo para
“tipo de documento”. Não foi encontrada evidência de que `moonbit-interface` seja reconhecido pelo servidor: o
Moonpad documenta apenas `moonbit`, e a configuração Sublime usa o mesmo seletor para `.mbt` e `.mbti`. Por isso os
dois sufixos usam `moonbit`; o smoke test deve confirmar explicitamente o `didOpen` de `.mbti` no `moon lsp` atual.

As tarefas continuam usando `command`, como `TaskDef` e `[tasks]` atuais. O executor existente interpreta essa string
por shell (`eval` no Unix e `cmd.exe /C` no Windows), o que também permite pipelines e `&&` nas tarefas do usuário.
Os comandos dos packs são literais fixos e revisados, sem entrada do usuário ou concatenação de nomes de arquivo;
logo, não há caso concreto de quoting ou injeção que justifique agora `program`/`args` e uma camada de escaping para
cada SO. Os packs não executam nada ao serem carregados: a tarefa só roda quando escolhida, e pedidos de terminal de
agentes continuam sujeitos à política atual de permissão.

Regras implementadas no formato 1:

1. `format = 1` é obrigatório; formato desconhecido é recusado com erro legível.
2. `id`, extensões e nomes de servidor são únicos. Colisões entre pacotes embutidos são erro de teste/build, não uma
   escolha silenciosa por ordem de carregamento.
3. Campos desconhecidos são erro, para revelar erros de digitação. Uma versão futura do formato pode acrescentar
   capacidades sem mudar o significado das atuais.
4. O LSP é registrado mesmo sem o executável no PATH; a IDE informa que o toolchain falta, e o README do pacote aponta
   instalação, documentação e limitações. Não baixa nem compila servidores automaticamente. O ciclo de vida é o atual:
   configuração registrada, cliente iniciado sob demanda ou falha memorizada. Executável ausente ou falha de
   inicialização não provoca novas tentativas automáticas; `lsp.restart` limpa a falha. A falha é isolada pelo nome do
   servidor e não impede abrir/editar o arquivo nem usar outro servidor. **Revisão (2026-11-09):** um servidor que cai
   *depois* do handshake religa sozinho quando a digitação num arquivo dele pausa por 1 s (não na hora, que reabriria o
   texto que o derrubou). Só a primeira queda de uma janela de 5 minutos é avisada; na 11ª queda dessa janela volta à
   regra acima. Como cada religação exige uma edição do usuário, não há laço automático; o teto só segura um servidor
   quebrado de vez. Motivo: o parser da biblioteca padrão do Hare (`hare::parse`), usado pelo `hare-lsp`, aborta com
   qualquer `;` faltando num comando, e religar à mão a cada erro de digitação tornava o servidor inutilizável. O aviso de saída traz o
   identificador do processo, para que a saída de um cliente já trocado por `lsp.restart` não derrube o novo.
5. Os servidores dos packs entram no merge antes de `[lsp.servers.<nome>]`. A configuração do usuário vence por nome,
   inclusive `enabled = false`, que remove o servidor. Servidores nomeados no config continuam ordenados antes dos
   embutidos e podem assumir uma extensão. Recarregar config mantém a semântica atual: vale para servidores iniciados
   depois, sem reconfigurar clientes existentes.
6. Para tarefas de pacote, o app procura o marcador mais próximo subindo os ancestrais do arquivo ativo, sem passar da
   raiz aberta; se não houver arquivo ativo ou marcador nesse caminho, verifica somente a raiz. O diretório encontrado
   vira o `cwd` já existente em `TaskDef`. Não há varredura recursiva nem agregação de tarefas de módulos irmãos.
   `[tasks]` do usuário continua tendo precedência por nome e mantém a raiz como `cwd`.
7. Sintaxes mantêm a precedência atual: recursos do usuário podem substituir a gramática embutida para a extensão.
8. Um pacote inválido não impede os demais nem a abertura da IDE. Para embutidos, testes devem impedir que esse caso
   chegue a uma release.

A raiz aberta continua sendo a fronteira do workspace e do gerenciador de LSP. “Raiz do projeto” nesta regra é apenas
o diretório do marcador usado para descobrir tarefas; o `cwd` de execução já é representado por `TaskDef`. Não se cria
um modelo novo de workspaces. Em particular, `moon.work` pode agrupar módulos, mas não precisa virar outro marcador no
formato 1: rodar no `moon.mod` mais próximo mantém a tarefa no módulo do arquivo ativo e evita executar acidentalmente
o workspace inteiro. Tarefas explícitas para o workspace continuam possíveis em `[tasks]`.

**[Recomendação]** Não incluir `install`, scripts arbitrários, variáveis, condicionais por SO ou hooks no formato 1.
Quando uma necessidade real não couber, ela usa um hook Rust interno com nome estável, em vez de ganhar uma expressão
nova no TOML.

### 4.3 Ativação e fluxo

- **Ao abrir a IDE:** validar os pacotes e registrar as gramáticas antes da primeira aba, como já ocorre com ZX.
- **Ao abrir um arquivo:** a extensão escolhe a gramática e o `languageId`; o gerenciador de LSP reutiliza o fluxo
  atual. Não há um segundo protocolo nem um processo criado pelo pacote.
- **Ao listar tarefas:** o app resolve o projeto pelo arquivo ativo e seus ancestrais, com o fallback de raiz descrito
  acima. As tarefas existentes de Cargo, Zig, npm e Makefile continuam inalteradas.
- **Ao mudar o config:** as mesmas regras de precedência do LSP atual continuam valendo; recarregar config não muda
  gramáticas já registradas, mantendo a limitação conhecida do widget explícita.
- **Ao faltar o toolchain:** cores e edição continuam funcionando; LSP/tarefas dependentes mostram a ausência sem
  transformar isso em erro da linguagem ou num download automático.

Hare não tem um manifesto de projeto próprio e costuma usar `Makefile`. Como esse arquivo não prova que o projeto é
Hare, o pacote não oferecerá tarefas automáticas no formato 1: as tarefas do Makefile já aparecem, e `hare build`,
`hare run` ou `hare test` podem ficar em `[tasks]`. Isso evita tanto uma varredura cara quanto tarefas erradas em
projetos C/C++.

### 4.4 Hooks internos e debug

Hooks não fazem parte do manifesto nem são chamados por pacotes externos. São implementações Rust nomeadas pela IDE,
para decisões como o fallback do Ziex ou a descoberta de executáveis.

Debug fica fora do formato 1 porque `build = "..."` resolve apenas metade do problema: ainda é preciso escolher o
programa e os argumentos. Em MoonBit isso depende de backend, pacote e teste; em Hare, normalmente o Makefile escolhe
o nome e o destino. O `[debug]` atual atende esses casos enquanto não houver um provedor testado.

**[Recomendação]** Fazer depois dos pacotes básicos um spike de MoonBit nativo/LLVM com `lldb-dap`, `moon build -g` e
o script `$MOON_HOME/share/lldb/moonbit.py`. Só generalizar debug no manifesto se uma segunda linguagem apresentar o
mesmo contrato.

## 5. Avaliação dos pilotos

### 5.1 Hare

**[Fato]**

- Arquivos usam `.ha`. O toolchain oferece `hare build`, `hare run` e `hare test`; projetos convencionais usam
  `Makefile` com alvos como `all` e `check` ([estrutura de projetos](https://harelang.org/documentation/usage/project-structure.html)).
- Existe uma gramática Sublime derivada de TextMate em
  [`ravengraves/hare-sublime-syntax`](https://github.com/ravengraves/hare-sublime-syntax), publicada sob Unlicense;
  a extensão de origem tem licenças MIT/Apache-2.0. A gramática é curta e cobre `.ha`, mas sua geração informada é de
  2022. Os testes atuais confirmam escopos léxicos básicos; uma revisão semântica completa contra a especificação
  atual continua pendente e está registrada como limitação de validação.
- O [`hare-lsp`](https://git.sr.ht/~whynothugo/hare-lsp) usado pelo Helix aceita stdio com `-S`. Implementa definição,
  hover, diagnósticos de parsing e símbolos do documento. Não faz ainda validação semântica, completion, references,
  rename ou formatting. Outros servidores listados pelo projeto Hare, como `hare-ls` e `harepls`, também se descrevem
  como trabalho em progresso.
- O Helix tem realce tree-sitter e configura `hare-lsp`; a cobertura existe, mas é fragmentada e limitada pelo
  servidor ([matriz do Helix](https://docs.helix-editor.com/master/lang-support.html)).

**[Recomendação]** Pacote inicial com a gramática e seus testes de escopo, `hare-lsp -S`, documentação de instalação
e matriz de limitações. Não esconder diagnósticos semânticos ausentes nem criar tarefas apenas por existir um
Makefile. Debug pode usar um executável produzido pelo Makefile e o `[debug]` existente, mas não é promessa da etapa
inicial. Uma revisão mais ampla da gramática deve ser devolvida ao upstream quando fizer sentido.

### 5.2 MoonBit

**[Fato]**

- O CLI documenta `moon build`, `moon check`, `moon run`, `moon test`, `moon fmt` e `moon bench`, com seleção de
  backend e `-g` para informações de debug ([comandos do moon](https://docs.moonbitlang.com/en/stable/toolchain/moon/commands.html)).
- O LSP nativo vem no toolchain e roda com `moon lsp`. A integração do Zed usa exatamente esse comando e informa que
  não exige pacote npm separado ([`zed-moonbit`](https://github.com/quirk-lab/zed-moonbit)).
- A gramática [`hyrious/moonbit-syntax-highlight`](https://github.com/hyrious/moonbit-syntax-highlight) é MIT e cobre
  `.mbt` e `.mbti`, incluindo interpolação de strings. É diretamente consumível pelo `syntect`, sujeita a testes com a
  sintaxe atual.
- `moon.mod` é o manifesto atual de módulo. `moon.mod.json` é legado e **não é mais aceito pelos comandos comuns do
  toolchain estável**; a migração documentada é `moon fmt` ([configuração de módulo](https://docs.moonbitlang.com/en/stable/toolchain/moon/module.html)).
  `moon.work` agrupa módulos e permite comandos no workspace inteiro ([workspaces](https://docs.moonbitlang.com/en/stable/toolchain/moon/workspace.html)).
- O plugin oficial de VS Code oferece também code lens, ações e integração de debug; essas capacidades não vêm de
  graça com LSP ([documentação do plugin](https://docs.moonbitlang.com/en/stable/toolchain/vscode/index.html)).
- O backend LLVM gera DWARF/CodeView e o toolchain fornece um script LLDB para estruturas próprias. A documentação de
  outubro de 2025 situava esse backend no canal nightly; a estabilidade atual deve ser conferida no spike, não
  presumida ([debug com LLDB](https://www.moonbitlang.com/blog/llvm-debug)).

**[Recomendação]** Pacote inicial com `.mbt`/`.mbti`, `moon lsp` e as quatro tarefas declarativas do exemplo,
ativadas pelo `moon.mod` mais próximo do arquivo ativo. `moon.mod.json` não ativa tarefas: a documentação estável
orienta migrá-lo antes de build/check. Usar somente comandos estáveis documentados. Não incluir `.mbt.md` nesta etapa:
ele exige reconhecimento de extensão composta e Markdown com blocos executáveis, não apenas mais uma linha no
manifesto. Debug fica no spike posterior.

### 5.3 `moon ide`: oportunidade separada para agentes

A documentação estável mantém `moon ide peek-def`, `find-references`, `rename`, `hover`, `outline`, `doc`,
`workspace-symbols`, `codelens` e outros comandos. Eles fazem análise semântica, funcionam em módulos e workspaces e
podem incluir contexto de código; `find-references --json` e `codelens` oferecem saídas JSON
([MoonBit Agent IDE](https://docs.moonbitlang.com/en/stable/toolchain/moonide/index.html)).

**[Recomendação]** Não integrar isso ao language pack nem ao RFC 0014. Uma proposta futura deve avaliar especificamente
o uso por agentes fora de uma sessão interativa do editor: consultas semânticas diretas pelo CLI, sem o agente precisar
controlar um cliente LSP, e respostas com contexto suficiente para reduzir buscas textuais e leituras adicionais.
Hover, definição, referências e rename interativos continuam no LSP; trocar esses caminhos pelo CLI apenas duplicaria
a integração e criaria processos repetidos. A proposta futura precisa demonstrar esse ganho com um fluxo real de agente
e definir como comandos e resultados passam pela política de permissões existente.

### 5.4 Critérios para futuras linguagens

Um novo pacote embutido deve ter:

1. linguagem e recursos reutilizados com licenças compatíveis e avisos no `NOTICE.md`;
2. pelo menos um usuário ou projeto real para o dogfooding;
3. gramática e comandos revisados contra uma versão identificada do upstream;
4. preferência por LSP/DAP e CLI oficiais ou mantidos, sem esconder estágio experimental;
5. nenhum download/execução automática de toolchain;
6. custo pequeno de manutenção e um motivo para estar embutido em vez de apenas documentado como configuração.

## 6. Manutenção dos pacotes

Cada pacote terá um `README.md` junto do manifesto, sem adicionar metadados de manutenção ao TOML. Esse documento deve
registrar:

- versão do toolchain e versão ou revisão do servidor usados na validação;
- URL e revisão exata de cada gramática, sua licença e eventuais alterações locais;
- comandos e capacidades LSP efetivamente testados, não apenas disponíveis na documentação;
- limitações conhecidas e diferenças entre backends, quando relevantes;
- data e resultado do último dogfooding.

Ao atualizar toolchain, servidor ou gramática, o mantenedor repete os testes automáticos e o roteiro manual, compara o
resultado com essa linha de base e registra a nova revisão validada ou a incompatibilidade encontrada. Um smoke test
com a versão estável atual ajuda a perceber mudanças upstream; falha de gramática, descoberta, LSP ou tarefa deve ser
atribuída à camada correspondente antes de mudar o manifesto. Isso é documentação de validação, não um gerenciador de
versões nem uma exigência de suportar todas as versões antigas.

## 7. Validação

### Testes automáticos

- Manifestos embutidos válidos; ids, extensões, servidores e nomes de tarefa sem colisão.
- Arquivos de sintaxe presentes, parseáveis pelo `syntect` e com licença correspondente.
- Exemplos pequenos de Hare e MoonBit com escopos esperados para palavras, tipos, números, strings, comentários e
  interpolação; o objetivo é detectar gramática quebrada/desatualizada, não comparar uma imagem inteira.
- `.mbt` e `.mbti` → `languageId` `moonbit`, precedência das sintaxes do usuário e merge de LSP com override/disable
  do config; um servidor configurado pelo usuário pode assumir a extensão de um pack.
- Executável ausente gera uma falha por servidor, sem retry automático até `lsp.restart`; outros servidores e a
  abertura/edição de arquivos continuam funcionando.
- Tarefas MoonBit ativadas por `moon.mod` na raiz ou num ancestral do arquivo ativo; o marcador mais próximo define o
  `cwd`. Um módulo aninhado é encontrado sem varredura, a busca não ultrapassa a raiz e módulos irmãos não misturam
  tarefas. Sem arquivo ativo, vale o fallback da raiz; `moon.mod.json` sozinho não ativa tarefas e `Makefile` sozinho
  não ativa Hare.
- Ausência de `hare-lsp`/`moon` não impede abrir, colorir ou editar arquivos.
- Testes de unidade não exigem os toolchains nem rede; smoke tests reais são separados e pulam quando faltam.

### Dogfooding

- **Hare:** abrir um projeto convencional com `.ha`, conferir cores, outline, hover, definição e diagnóstico de parsing;
  registrar claramente que erro semântico e completion não são suportados pelo servidor. Rodar uma tarefa do Makefile.
- **MoonBit:** abrir um módulo com `moon.mod`, conferir cores em `.mbt`/`.mbti`, diagnósticos, completion, navegação e
  formatação que o LSP efetivamente oferecer; verificar explicitamente `didOpen` e respostas do LSP para `.mbti`; rodar
  `moon check`, `moon test` e `moon fmt --check` pelas tarefas. Repetir com o módulo em `workspace/backend` e outro
  projeto na árvore para confirmar descoberta e `cwd`.
- Repetir o básico em uma máquina sem os toolchains para validar a degradação. Debug, quando chegar, terá teste manual
  próprio com breakpoint, pilha, variáveis e script LLDB.

## 8. Validação da hipótese de adoção

O sucesso técnico não basta para confirmar que os packs atraem usuários. Depois da implementação, o projeto deve fazer
uma avaliação pequena e voluntária, sem telemetria ou infraestrutura de analytics:

1. Publicar exemplos Hare e MoonBit com instalação, matriz real de capacidades e limitações, e apresentá-los nos canais
   das duas comunidades, pedindo relatos concretos e correções upstream quando couber.
2. Recrutar usuários externos que já mantenham um projeto nessas linguagens para uma sessão de dogfooding. Sucesso
   técnico significa abrir o projeto, editar, navegar e rodar as tarefas prometidas sem configuração específica da IDE,
   compreendendo também as limitações do servidor.
3. Perguntar separadamente se a pessoa experimentou ou continuou usando o IDETEA **por causa** do suporte, o que faltou
   para adotá-lo e se voltou a usá-lo depois do primeiro teste. Issues, respostas nas comunidades e entrevistas curtas
   são evidência suficiente nesta etapa; downloads ou testes isolados não provam retenção.
4. Só propor uma terceira linguagem quando houver uso externo útil de pelo menos um piloto, evidência de que esse tipo
   de suporte motivou experimentação ou adoção, custo de manutenção aceitável e um projeto/usuário real para validar a
   nova linguagem. Popularidade ou facilidade de escrever outro manifesto, sozinhas, não bastam.

## 9. Etapas propostas

1. **Registro e Hare:** criar `tea-core::languages`, validação e manifesto 1; integrar gramática e LSP sem alterar o
   protocolo; revisar a gramática e entregar o pacote Hare com documentação das limitações.
2. **MoonBit e tarefas:** acrescentar marcadores e tarefas declarativas ao registro; entregar MoonBit com LSP e as
   tarefas comuns; provar que o mesmo formato atende os dois pilotos.
3. **Consolidação:** publicar a matriz de suporte e um guia para novos pacotes embutidos; avaliar a hipótese de adoção
   e o spike de debug de MoonBit separadamente. Não migrar linguagens antigas nem abrir pacotes externos sem uma
   necessidade observada.

   **Feito (2026-11-10):** matriz pública em `docs/languages.md` (todas as linguagens e uma seção por pack, com
   instalação, capacidades conferidas e limitações), exemplos em `examples/languages/{hare,moonbit}`, guia de novos
   packs em `DESENVOLVIMENTO.md` e o teste `every_pack_is_documented`, que exige seção e exemplo para cada pack.
   Ícones: o Seti não tem Hare nem MoonBit, então os packs ganharam `[icon]` (opcional: `glyph`, `color`, `light`),
   apontando para uma fonte própria da IDE (`assets/languages/icons.ttf`, desenhos gerados por
   `tools/language-icons.py`). A extensão de um pack vem antes do Seti na busca do ícone.
   Ficam para depois: a avaliação de adoção da seção 8 (divulgar nas comunidades, sessões com usuários externos) e
   o spike de debug de MoonBit.

Uma etapa futura poderá ler packs de uma pasta do usuário. Nesse momento serão necessários confiança explícita,
versão de formato, instalação/remoção e regras para comandos. Isso não deve atrasar os pacotes embutidos nem ser
implementado de forma implícita pelo primeiro loader.

## 10. Decisões confirmadas (2026-11-09)

1. **Distribuição:** pacotes **embutidos primeiro**; pacotes externos ficam para depois da validação do formato.
2. **Arquitetura:** **declarativa**, com hooks Rust internos apenas para exceções; não uma extensão Rust ou componente
   WASM por linguagem.
3. **Escopo desta etapa:** sem marketplace, sem instalação automática de toolchains e sem debug declarativo. `moon ide`
   e pacotes externos exigiriam propostas separadas, não entram pela implementação dos dois pilotos.
4. **Pilotos:** Hare e MoonBit permanecem no mesmo RFC e compartilham a infraestrutura do manifesto 1.
5. **Hare:** sem auto-detecção de tarefas; Makefile/config continuam disponíveis. A gramática embutida tem testes de
   escopo, com a revisão completa e eventuais melhorias upstream registradas como trabalho posterior.
6. **MoonBit:** `moon lsp` como padrão e somente `.mbt`/`.mbti` nesta etapa; `.mbt.md` fica para depois. **Feito depois (2026-11-10):**
   `.mbt.md` entrou como extensão composta do pack (`"mbt.md" = "moonbit"`), achada antes de `md` para cores, LSP e
   ícone, com uma gramática Markdown pequena e própria que embute a do MoonBit nos blocos `mbt`/`moonbit`. As tarefas
   seguem o `moon.mod` mais próximo do arquivo ativo e rodam nesse diretório, sem agregar módulos nem criar um sistema
   de workspaces.
