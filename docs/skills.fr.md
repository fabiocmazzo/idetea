# Compétences des agents

[Português](skills.md) · **Français** · [Deutsch](skills.de.md) · [Italiano](skills.it.md) · [日本語](skills.ja.md) · [中文（简体）](skills.zh-CN.md)

Une compétence (skill) est un dossier avec un `SKILL.md` (un en-tête avec `name` et `description`, et les
instructions) et des fichiers d'appui : références, modèles, parfois des scripts. L'agent lit la description de
toutes et ouvre la compétence entière quand elle sert à ce qu'il fait. Étude et décisions :
[RFC 0005](rfc/0005-agentes-e-projetos.md), étape 2 (en portugais).

`idetea skills` installe dans le projet du dossier courant des compétences venues d'un dépôt git :

```sh
idetea skills                               # celles installées dans ce projet
idetea skills list                          # celles d'anthropics/skills (la source par défaut)
idetea skills list dono/repo                # celles d'une autre source : owner/repo GitHub, une URL git ou un dossier
idetea skills add anthropics/skills pdf frontend-design
idetea skills update                        # ce qui a changé dans la source, avec le diff ; rien ne change encore
idetea skills update --apply                # installe ce qui a changé
idetea skills remove pdf
```

- **Où elles sont :** une copie dans `.agents/skills/<nom>/` (Codex, OpenCode et d'autres) et une autre dans
  `.claude/skills/<nom>/` (Claude Code). Les deux vont dans git, avec `.idetea/skills.toml`, qui note de quelle
  source et de quel commit chacune vient.
- **Figées sur un commit :** la compétence reste telle qu'elle a été installée jusqu'à un `update --apply`. Le
  `update` sans `--apply` montre le diff, pour lire ce qui a changé avant.
- **Rien ne s'exécute :** l'IDE ne fait que copier des fichiers. La source reste dans un cache
  (`~/.cache/idetea/skills/`, ou `$XDG_CACHE_HOME/idetea/skills/`) et est lue depuis l'historique git. Les liens
  symboliques et les sous-modules d'une compétence ne sont pas copiés. Dans la liste, « contient des scripts » marque
  la compétence qui apporte des fichiers que l'agent peut exécuter : lisez-les avant.
- **Copie modifiée à la main :** `add`, `update --apply` et `remove` refusent ; `--force` remplace (ou efface) quand
  même. Un dossier du même nom qui ne vient pas de l'IDE n'est lui aussi remplacé qu'avec `--force`.

Sortie : 0 (fait), 1 (erreur, ou mise à jour à appliquer), 2 (usage incorrect).

## Nouveau projet

« Projet : nouveau… » (palette) montre un formulaire dans le panneau Interfaces : où créer, le nom du dossier, le
modèle, une phrase sur le projet, s'il faut créer le git et quel agent ouvrir. En ligne de commande :

```sh
idetea new meu-app --modelo rust --descricao "um conversor de unidades" --agente claude
```

L'IDE crée le dossier, le git, les compétences du modèle, le `AGENTS.md` avec la **demande initiale** et les
instructions du panneau, et un `CLAUDE.md` qui importe le `AGENTS.md`. C'est l'agent qui crée le projet.

**L'agent dans un terminal de l'IDE.** Le formulaire propose les agents qui sont dans le `PATH` (`claude`, `codex`,
`gemini`, `opencode` ; le premier trouvé vient sélectionné) et « Aucun (je l'ouvrirai) ». Avec un agent, l'IDE écrit
dans le nouveau projet une [demande de terminal](terminais-de-agente.md) (en portugais pour l'instant)
`.idetea/agents/agente.json` avec la commande
`<agente> 'Start with the initial request in AGENTS.md.'`. La nouvelle fenêtre montre la demande dans l'onglet
« demande de terminal », comme n'importe quelle autre : l'agent ne s'ouvre qu'après « Autoriser et exécuter ». Avec
« Aucun », ouvrez-le vous-même dans le nouveau dossier et demandez-lui de commencer.

| Modèle | Compétences |
|---|---|
| `vazio` | `novo-projeto` (l'agent demande de quoi il s'agit) |
| `rust` | `novo-projeto`, `novo-projeto-rust` |
| `site` | `novo-projeto`, `novo-projeto-site`, `frontend-design` (d'`anthropics/skills`) |
| `python` | `novo-projeto`, `novo-projeto-python` |
| `fastapi` | `novo-projeto`, `novo-projeto-python`, `novo-projeto-fastapi` |

Les compétences sont dans [`skills/`](../skills/) de ce dépôt, et les modèles dans
[`skills/modelos.toml`](../skills/modelos.toml) : un nouveau modèle n'est qu'une entrée là.
