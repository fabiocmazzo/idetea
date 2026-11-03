# RFC 0005 — Skills, projeto novo, vários agentes e conserto do projeto

- **Status:** aprovada em 2026-11-03 (pelo painel), com as decisões do fim. Base: `main` em `f6fb9f0`.
- **Depende de:** [RFC 0001](0001-interfaces-declarativas.md) (painel Interfaces), [RFC 0002](0002-plugins-wasm.md)
  (plugins, `host.run`) e [RFC 0003](0003-interfaces-pelo-uso.md) (evoluir pelo uso).
- **Origem:** comentário do usuário no painel, ao escolher as lacunas 6 e 11: "pensar em um instalador de skills
  para o projeto […]; um painel New Project que o usuário já escolhe as skills; agentes executados pelo agente
  principal […] num terminal adicional dentro do IDETEA, futuramente Firecracker ou OpenShell; como vai ter mais de
  um agente, um esperar uma demanda ser respondida […]; e uma espécie de idetea-fix que deixa um projeto utilizável
  de novo, mesmo que se percam algumas comunicações pela interface".

Marcadores: **[Fato]** conferido no código, no uso ou na fonte citada; **[Hipótese]** provável, não testado;
**[Recomendação]** proposta, ainda não decidida.

## Resumo

São cinco ideias de tamanhos muito diferentes. Duas são pequenas e se apoiam no que já existe (conserto do projeto,
instalar skills). Duas pedem peças novas na IDE (terminais pedidos por um agente, projeto novo). Uma é pesquisa
(vários agentes coordenados, sandbox).

**[Decisão]** Ordem: (1) `idetea-fix`, (2) instalar skills, (3) "Novo projeto", que reusa (2), (4) terminais de
agente, (5) vários agentes esperando uns aos outros, só depois de (4) ser usado, e (6) sandbox por último. Cada
etapa vale sozinha; nenhuma exige a seguinte.

O fio comum: hoje **o agente principal é invisível para a IDE** **[Fato]**. A IDE só sabe que um agente existe
quando ele escreve em `.idetea/ui/`. Ela não sabe se ele ainda está esperando uma resposta, quem ele é nem se ele
acabou. Foi isso que aconteceu em 2026-11-03: o agente desistiu de esperar o "Próximo passo" depois de 25 minutos;
o usuário respondeu 50 minutos depois, a resposta ficou parada no disco, e quando ele quis marcar o push o botão já
estava desligado. Várias etapas abaixo resolvem pedaços disso.

## 1. `idetea-fix`: deixar o projeto utilizável de novo

### O que pode quebrar hoje [Fato]

O que a IDE e os agentes deixam num projeto:

| Onde | O que | Como estraga |
|---|---|---|
| `.idetea/ui/<nome>.json` | documentos | o agente morreu ou desistiu; o documento fica no painel para sempre, com botões que ninguém lê |
| `.idetea/ui/responses/<nome>/` | respostas | ninguém apaga; o painel mostra "aguardando o agente ler" |
| `.idetea/ui/.<nome>.json.tmp` | temporários | o agente caiu entre o `cat` e o `mv` |
| `.idetea/ui/check` | script | aponta para um executável que não existe mais (outra pasta, `cargo clean`) |
| `.idetea/ui/FORMAT.md`, schema | formato | de uma IDE de outra versão (só regrava quando a IDE abre o projeto) |
| `AGENTS.md`, `CLAUDE.md` | bloco de instruções | versão velha, sem a linha de versão ou de uma IDE mais nova (o `check` já avisa) |
| `.idetea/config.toml` | config do projeto | `interfaces = { enabled = false }` desliga o painel |
| `~/.local/state/idetea/ui/<projeto>.json` | campos e revisões respondidas | já é podado quando o documento some (`saved.retain`) |

### Proposta [Recomendação]

Um comando "Interfaces: consertar este projeto" e o mesmo pela linha de comando (`idetea ui fix`, ao lado do
`idetea ui check`). Primeiro mostra no painel o que acharia e o que faria, e só faz depois do "Consertar":

- documentos sem resposta há mais de N horas, ou já respondidos e lidos: oferece apagar;
- respostas que ninguém leu: mostra o conteúdo (pode ser a única cópia de uma decisão) e oferece apagar;
- temporários `.tmp` antigos: apaga;
- `check`, `FORMAT.md` e schema: regrava com o executável atual;
- bloco de instruções: o mesmo "ensinar", com o aviso de versão mais nova;
- painel desligado na config do projeto: avisa (não muda a config sozinho).

Pela linha de comando, sem a IDE aberta, faz só o que não depende dela e diz o resto. O "se perder algumas
comunicações" do pedido vira regra: **nada é apagado sem mostrar antes**, e uma resposta não lida é mostrada inteira.

**Custo:** pequeno, tudo é código que já existe (`store`, `teach`, `check`). **Risco:** baixo.

## 2. Instalar skills no projeto

### O que é uma skill [Fato]

Uma skill é uma pasta com um `SKILL.md` que tem `name` e `description` no cabeçalho, mais arquivos de apoio. O
Claude Code lê as skills do projeto em `.claude/skills/<nome>/` e as do usuário em `~/.claude/skills/`. O repositório
mais conhecido é o `anthropics/skills`. Já existem instaladores de terceiros (por exemplo `openskills`), que clonam
um repositório, acham os `SKILL.md` e copiam as pastas escolhidas.

### Onde cada agente lê (conferido em 2026-11-03)

| Agente | Pasta do projeto | Fonte |
|---|---|---|
| Claude Code | `.claude/skills/` | docs da Anthropic **[Fato]** |
| Codex | `.agents/skills/`, da pasta atual até a raiz do repositório | [docs oficiais](https://developers.openai.com/codex/skills) **[Fato]** |
| OpenCode | `.opencode/skills/`; também lê `.claude/skills/` e `.agents/skills/` | [docs oficiais](https://opencode.ai/docs/skills) **[Fato]** |
| Gemini CLI | `.gemini/skills/` ou `.agents/skills/` | [fontes de terceiros](https://github.com/vercel-labs/skills/wiki), que discordam **[Hipótese]** |
| Copilot | `.github/skills/` ou `.agents/skills/` | idem **[Hipótese]** |

`.agents/skills/` está virando o lugar comum; o Claude Code precisa de `.claude/skills/`.

### Proposta [Recomendação]

- Fonte: um repositório git (padrão `anthropics/skills`, outros na config `[skills] sources = [...]`), clonado com
  o `git` do sistema numa pasta de cache, **fixado num commit**.
- O painel lista as skills achadas (nome, descrição, tamanho, se tem scripts) e o usuário marca quais quer.
- Instalar = copiar a pasta para `.claude/skills/<nome>/` e anotar a origem e o commit num `.idetea/skills.toml`.
  Atualizar mostra o diff entre o commit anotado e o novo antes de copiar.
- A IDE **não roda nada** da skill. Ela só copia arquivos.

**Riscos [Fato]:**

- Uma skill é instrução para o agente seguir e pode trazer scripts que o agente vai rodar. Instalar de um
  repositório é confiar nele como se fosse código. Por isso: commit fixo, diff antes de atualizar e scripts
  destacados na lista.
- Skills grandes ocupam contexto do agente. A descrição é o que ele lê sempre, então ela entra na lista.

**Custo:** médio. Clonar e copiar é simples; o trabalho é a tela de escolha e o diff de atualização.

### Decisões e o que foi feito (pelo painel, 2026-11-03)

- **Uma cópia em `.agents/skills/` e outra em `.claude/skills/`** (funciona em todo sistema; symlink no Windows é
  ruim). Os agentes que leem as duas (OpenCode) veem a mesma skill duas vezes, com o mesmo conteúdo.
- **Linha de comando primeiro:** `idetea skills [list|add|update|remove]` (crate `tea-skills`, ver
  [`docs/skills.md`](../skills.md)). O agente pergunta pelo painel quais skills o usuário quer e roda o comando. A
  tela vem com o "Novo projeto".
- A fonte vai para um cache (`~/.cache/idetea/skills/`) como repositório *bare*; tudo é lido do histórico
  (`ls-tree`, `cat-file`), então a cópia de trabalho não existe e nada do repositório roda. Symlinks e submódulos
  não são copiados. Uma cópia mexida à mão não é sobrescrita nem apagada sem `--force`.
- Ainda não: fontes na config (`[skills] sources`), skills do usuário (`~/.agents/skills/`), escolher a ref
  (`dono/repo@tag`).

## 3. Painel "Novo projeto"

**[Fato]** Hoje a IDE não cria projetos: abre uma pasta que já existe.

**[Decisão]** Uma tela "Novo projeto" com: pasta, nome, um modelo, `git init` e "ensinar os agentes" já marcado.
No fim, abre o projeto. **Um modelo é um conjunto de skills, não uma pasta de arquivos:** quem cria o projeto é o
agente, seguindo as skills instaladas. Modelos da primeira versão: vazio, Rust, Site, Python e FastAPI. Um modelo
novo é só uma lista de skills (de qual fonte e qual commit); não exige código.

Depende da etapa 2.

**Decisões e o que foi feito (pelo painel, 2026-11-03):**

- As skills dos modelos são **escritas por nós, neste repositório** (`skills/`), que fica público para servir de fonte
  (`fabiocmazzo/idetea`). O `anthropics/skills` não tem skills de Rust, Python nem FastAPI. Uma skill comum,
  `novo-projeto` (perguntar pelo painel, primeiro commit, teste manual), e uma por pilha: `novo-projeto-rust`,
  `-python`, `-fastapi` e `-site`. O Site também leva a `frontend-design` do `anthropics/skills`.
- Os modelos ficam em `skills/modelos.toml` (embutido no binário): as skills de cada um e o pedido inicial.
- **A tela é o próprio painel Interfaces:** "Projeto: novo…" (paleta) faz a IDE escrever o formulário
  `idetea-novo-projeto` e tratar o clique ela mesma (sem arquivo de resposta). Pela linha de comando:
  `idetea new <pasta> [--modelo id] [--descricao texto] [--sem-git]`.
- **Depois de criar,** a IDE abre o projeto numa janela nova. O pedido inicial fica no `AGENTS.md` (e o `CLAUDE.md`
  importa o `AGENTS.md`); o usuário abre o agente. Abrir o agente num terminal da IDE fica para a etapa 4.
- Se as skills não vierem (rede, fonte), nada fica pela metade: a pasta criada é apagada.

**Custo:** médio. É uma tela nativa (não um documento `idetea-ui`, porque ainda não há projeto onde gravar o
documento).

## 4. Agentes rodando em terminais da IDE

### O que existe [Fato]

- O terminal tem um shell e um terminal de tarefa (`CoreEvent::RunTask`), não abas.
- As tarefas vêm da config `[tasks]`. Nenhum processo de fora consegue pedir um terminal.
- O agente principal já roda processos sozinho, inclusive outros agentes. O que falta não é o poder de rodar: é
  **ver** e **poder intervir** (digitar, parar) dentro da IDE.

### Proposta [Recomendação]

- Primeiro, várias abas no painel do terminal (útil sozinho, sem agentes).
- Depois, um pedido por arquivo, no mesmo estilo das interfaces: `.idetea/agents/<nome>.json` com `command`, `cwd`
  e um título. A IDE abre uma aba com o comando, mostra quem pediu e grava em `.idetea/agents/<nome>.status.json`
  se está rodando e, no fim, o código de saída.
- **Cada pedido é confirmado pelo usuário** (no painel, mostrando o comando inteiro), ou por uma lista de comandos
  permitidos na config do usuário. Sem isso, qualquer processo que escreve na pasta do projeto ganharia um jeito de
  rodar comandos com a IDE aparecendo como dona.

**Custo:** abas, médio; pedido por arquivo, médio; os dois juntos são uma RFC própria.

**O que foi feito (2026-11-03):** abas no terminal (aprovado pelo painel); pedido por arquivo como proposto, com a
confirmação numa aba do próprio painel do terminal (não num diálogo: um Enter digitado em outro lugar não permite
nada) e o status também com `refused`, `closed`, `invalid` e `lost`. Apagar o pedido encerra o terminal. Detalhes em
`docs/terminais-de-agente.md`.

## 5. Vários agentes, um esperando o outro

### O que já funciona [Fato]

- Várias interfaces ao mesmo tempo, cada uma com nome próprio. O `source` diz quem escreveu.
- A resposta é um arquivo. Qualquer processo pode esperar por ela, inclusive outro agente.

### O que falta

- **Quem está esperando.** Hoje o painel não sabe se o agente ainda espera (o caso do "Próximo passo").
  **[Recomendação]** O agente mantém um arquivo de batimento enquanto espera (por exemplo `.idetea/ui/<nome>.wait`,
  com a data atualizada a cada poucos segundos). O painel mostra "o agente está esperando" ou "ninguém esperando há
  X min". Isso é pequeno, vale já com um agente só, e é a base do resto.
- **Pedidos entre agentes.** Um agente pede algo ao outro (revisar, gerar uma imagem) e espera. **[Hipótese]** O
  mesmo protocolo serve: o pedido é um documento numa pasta do agente de destino e a resposta volta pelo mesmo
  caminho. O painel mostraria a fila de cada agente.
- **Conflitos.** Dois agentes mexendo nos mesmos arquivos. **[Recomendação]** Um `git worktree` por agente, criado
  pela IDE, com o merge pelo fluxo de sempre (teste manual e aprovação no painel).

**[Recomendação]** Não desenhar a coordenação antes de existir a etapa 4 em uso. O batimento pode entrar já.

## 6. Sandbox: OpenShell ou Firecracker

**[Fato]**

- **OpenShell** (NVIDIA, Apache 2.0) põe agentes como Claude Code e Codex em contêineres Docker com políticas em
  YAML (arquivos, rede, credenciais), aplicadas fora do processo do agente. Pede Docker rodando. A criação é do tipo
  `openshell sandbox create -- claude`. É recente; as fontes encontradas são principalmente blogs de terceiros.
- **Firecracker** são microVMs. Pedem `/dev/kvm` (Linux; não roda em macOS), uma imagem do sistema e mais trabalho
  de montagem: isolamento mais forte, custo bem maior.

**[Recomendação]** A IDE não implementa sandbox. Na etapa 4, o `command` pode ser `openshell sandbox create -- …`,
e a IDE só precisa saber montar a pasta do projeto (ou do worktree) lá dentro e ler as respostas. Firecracker fica
para quando um uso pedir um isolamento que o contêiner não dá.

## Decisões (pelo painel, 2026-11-03)

1. **Começar pelo `idetea-fix`, com o batimento da etapa 5 junto.**
2. **Skills: conferir os outros agentes antes** (Codex, OpenCode e outros: onde cada um lê skills), em vez de
   começar só por `.claude/skills/`.
3. **"Novo projeto": modelos vazio, Rust, Site, Python e FastAPI**, cada um um conjunto de skills; o agente cria
   tudo.
4. **Terminais de agente: confirmar cada pedido** (sem lista de permitidos por enquanto).

## Fontes

- [NVIDIA OpenShell (resumo de terceiros)](https://brenner-axiom.codeberg.page/research/nvidia-openshell-2026-03-17)
- [OpenShell em jimmysong.io](https://jimmysong.io/ai/openshell/)
- [OpenShell explicado (Activepieces)](https://www.activepieces.com/blog/openshell-by-nvidia-sandbox-for-ai-agents-explained.md)
- [Instalar skills de um repositório (install-from-github)](https://github.com/Camusugar/install-from-github)
- [openskills: primeira skill](https://lzw.me/docs/opencodedocs/es/numman-ali/openskills/start/first-skill/)
