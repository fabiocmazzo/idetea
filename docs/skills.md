# Skills de agentes

Uma skill é uma pasta com um `SKILL.md` (um cabeçalho com `name` e `description`, e as instruções) e arquivos de
apoio: referências, modelos, às vezes scripts. O agente lê a descrição de todas e abre a skill inteira quando ela
serve para o que está fazendo. Estudo e decisões: [RFC 0005](rfc/0005-agentes-e-projetos.md), etapa 2.

`idetea skills` instala skills de um repositório git no projeto da pasta atual:

```sh
idetea skills                               # as instaladas neste projeto
idetea skills list                          # as de anthropics/skills (a fonte padrão)
idetea skills list dono/repo                # as de outra fonte: dono/repo do GitHub, uma URL git ou uma pasta
idetea skills add anthropics/skills pdf frontend-design
idetea skills update                        # o que mudou na fonte, com o diff; nada muda ainda
idetea skills update --apply                # instala o que mudou
idetea skills remove pdf
```

- **Onde ficam:** uma cópia em `.agents/skills/<nome>/` (Codex, OpenCode e outros) e outra em
  `.claude/skills/<nome>/` (Claude Code). As duas vão para o git, junto com `.idetea/skills.toml`, que anota de qual
  fonte e de qual commit veio cada uma.
- **Fixadas num commit:** a skill fica como foi instalada até um `update --apply`. O `update` sem `--apply` mostra
  o diff, para ler o que mudou antes.
- **Nada roda:** a IDE só copia arquivos. A fonte fica num cache (`~/.cache/idetea/skills/`, ou
  `$XDG_CACHE_HOME/idetea/skills/`) e é lida do histórico do git. Symlinks e submódulos de uma skill não são
  copiados. Na lista, "tem scripts" marca a skill que traz arquivos que o agente pode rodar: leia antes.
- **Cópia mexida à mão:** `add`, `update --apply` e `remove` recusam; `--force` troca (ou apaga) mesmo assim. Uma
  pasta com o mesmo nome que não veio da IDE também só é trocada com `--force`.

Saída: 0 (feito), 1 (erro, ou atualização por aplicar), 2 (uso errado).

## Projeto novo

"Projeto: novo…" (paleta) mostra um formulário no painel Interfaces: onde criar, o nome da pasta, o modelo, uma
frase sobre o projeto e se cria o git. Pela linha de comando:

```sh
idetea new meu-app --modelo rust --descricao "um conversor de unidades"
```

A IDE cria a pasta, o git, as skills do modelo, o `AGENTS.md` com o **pedido inicial** e as instruções do painel, e
um `CLAUDE.md` que importa o `AGENTS.md`. Quem cria o projeto é o agente: abra-o na pasta nova e peça para começar.

| Modelo | Skills |
|---|---|
| `vazio` | `novo-projeto` (o agente pergunta o que é o projeto) |
| `rust` | `novo-projeto`, `novo-projeto-rust` |
| `site` | `novo-projeto`, `novo-projeto-site`, `frontend-design` (do `anthropics/skills`) |
| `python` | `novo-projeto`, `novo-projeto-python` |
| `fastapi` | `novo-projeto`, `novo-projeto-python`, `novo-projeto-fastapi` |

As skills ficam em [`skills/`](../skills/) neste repositório, e os modelos em
[`skills/modelos.toml`](../skills/modelos.toml): um modelo novo é só uma entrada lá.
