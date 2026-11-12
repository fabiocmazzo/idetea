# Skills für Agenten

<p><a href="skills.md">Português</a> · <a href="skills.fr.md">Français</a> · <b>Deutsch</b> ·
<a href="skills.it.md">Italiano</a> · <a href="skills.ja.md">日本語</a> · <a href="skills.zh-CN.md">中文（简体）</a></p>

Ein Skill ist ein Ordner mit einer `SKILL.md` (ein Kopf mit `name` und `description` und die Anweisungen) und Dateien zur
Unterstützung: Referenzen, Vorlagen, manchmal Skripte. Der Agent liest die Beschreibung aller und öffnet den ganzen Skill, wenn er
zu dem passt, was der Agent gerade tut. Studie und Entscheidungen: [RFC 0005](rfc/0005-agentes-e-projetos.md) (auf Portugiesisch), Stufe 2.

`idetea skills` installiert Skills aus einem git-Repository in das Projekt des aktuellen Ordners:

```sh
idetea skills                               # die in diesem Projekt installierten
idetea skills list                          # die von anthropics/skills (die Standardquelle)
idetea skills list dono/repo                # die einer anderen Quelle: owner/repo auf GitHub, eine git-URL oder ein Ordner
idetea skills add anthropics/skills pdf frontend-design
idetea skills update                        # was sich in der Quelle geändert hat, mit dem Diff; noch ändert sich nichts
idetea skills update --apply                # installiert, was sich geändert hat
idetea skills remove pdf
```

- **Wo sie liegen:** eine Kopie in `.agents/skills/<Name>/` (Codex, OpenCode und andere) und eine weitere in
  `.claude/skills/<Name>/` (Claude Code). Beide gehen ins git, zusammen mit `.idetea/skills.toml`, das festhält, aus welcher
  Quelle und von welchem Commit jede kam.
- **Auf einen Commit festgehalten:** der Skill bleibt, wie er installiert wurde, bis zu einem `update --apply`. `update` ohne `--apply` zeigt
  den Diff, damit Sie lesen, was sich geändert hat, bevor etwas passiert.
- **Es läuft nichts:** die IDE kopiert nur Dateien. Die Quelle liegt in einem Cache (`~/.cache/idetea/skills/` oder
  `$XDG_CACHE_HOME/idetea/skills/`) und wird aus der git-Historie gelesen. Symlinks und Submodule eines Skills werden nicht
  kopiert. In der Liste markiert „enthält Skripte“ den Skill, der Dateien mitbringt, die der Agent ausführen kann: lesen Sie sie vorher.
- **Von Hand geänderte Kopie:** `add`, `update --apply` und `remove` lehnen ab; `--force` ersetzt (oder löscht) trotzdem. Ein
  Ordner mit demselben Namen, der nicht von der IDE kam, wird ebenfalls nur mit `--force` ersetzt.

Ausgabe: 0 (erledigt), 1 (Fehler, oder eine Aktualisierung ist noch anzuwenden), 2 (falsche Verwendung).

## Neues Projekt

„Projekt: neu…“ (Palette) zeigt ein Formular im Bereich Interfaces: wo erstellt wird, der Name des Ordners, die Vorlage, ein
Satz über das Projekt, ob git angelegt wird und welcher Agent geöffnet wird. Auf der Kommandozeile:

```sh
idetea new meu-app --modelo rust --descricao "um conversor de unidades" --agente claude
```

Die IDE legt den Ordner an, git, die Skills der Vorlage, die `AGENTS.md` mit der **Anfangsanfrage** und den Anweisungen für den Bereich, und
eine `CLAUDE.md`, die die `AGENTS.md` importiert. Das Projekt selbst erstellt der Agent.

**Der Agent in einem Terminal der IDE.** Das Formular bietet die Agenten an, die im `PATH` liegen (`claude`, `codex`, `gemini`,
`opencode`; der erste gefundene ist vorausgewählt) und „Keiner“. Mit einem Agenten schreibt die IDE im neuen Projekt eine
[Terminalanfrage](terminais-de-agente.md) (vorerst auf Portugiesisch) `.idetea/agents/agente.json` mit dem Befehl
`<agente> 'Start with the initial request in AGENTS.md.'`. Das neue Fenster zeigt die Anfrage in der Registerkarte „Terminalanfrage“,
wie jede andere: der Agent öffnet erst nach „Zulassen und ausführen“. Mit „Keiner“ öffnen Sie ihn selbst im neuen Ordner
und bitten ihn, anzufangen.

| Vorlage | Skills |
|---|---|
| `vazio` | `novo-projeto` (der Agent fragt, was das Projekt ist) |
| `rust` | `novo-projeto`, `novo-projeto-rust` |
| `site` | `novo-projeto`, `novo-projeto-site`, `frontend-design` (aus `anthropics/skills`) |
| `python` | `novo-projeto`, `novo-projeto-python` |
| `fastapi` | `novo-projeto`, `novo-projeto-python`, `novo-projeto-fastapi` |

Die Skills liegen in [`skills/`](../skills/) in diesem Repository, die Vorlagen in
[`skills/modelos.toml`](../skills/modelos.toml): eine neue Vorlage ist nur ein Eintrag dort.
