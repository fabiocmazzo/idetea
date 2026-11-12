# Skill degli agenti

<p><a href="skills.md">Português</a> · <a href="skills.fr.md">Français</a> · <a href="skills.de.md">Deutsch</a> · <b>Italiano</b> · <a href="skills.ja.md">日本語</a> · <a href="skills.zh-CN.md">中文（简体）</a></p>

Una skill è una cartella con un `SKILL.md` (un'intestazione con `name` e `description`, e le istruzioni) e file di
appoggio: riferimenti, modelli, a volte script. L'agente legge la descrizione di tutte e apre la skill intera quando
serve a quello che sta facendo. Studio e decisioni: [RFC 0005](rfc/0005-agentes-e-projetos.md) (in portoghese), fase 2.

`idetea skills` installa nel progetto della cartella attuale le skill di un repository git:

```sh
idetea skills                               # quelle installate in questo progetto
idetea skills list                          # quelle di anthropics/skills (la fonte predefinita)
idetea skills list dono/repo                # quelle di un'altra fonte: proprietario/repo di GitHub, un URL git o una cartella
idetea skills add anthropics/skills pdf frontend-design
idetea skills update                        # che cosa è cambiato nella fonte, con il diff; non cambia ancora niente
idetea skills update --apply                # installa quello che è cambiato
idetea skills remove pdf
```

- **Dove stanno:** una copia in `.agents/skills/<nome>/` (Codex, OpenCode e altri) e un'altra in
  `.claude/skills/<nome>/` (Claude Code). Entrambe vanno in git, insieme a `.idetea/skills.toml`, che annota da quale
  fonte e da quale commit arriva ognuna.
- **Fissate a un commit:** la skill resta com'è stata installata fino a un `update --apply`. L'`update` senza
  `--apply` mostra il diff, così leggi che cosa è cambiato prima.
- **Non viene eseguito niente:** l'IDE copia solo file. La fonte sta in una cache (`~/.cache/idetea/skills/`, oppure
  `$XDG_CACHE_HOME/idetea/skills/`) e si legge dalla cronologia di git. Symlink e sottomoduli di una skill non
  vengono copiati. Nell'elenco, «ha script» segna la skill che porta file che l'agente può eseguire: leggili prima.
- **Copia ritoccata a mano:** `add`, `update --apply` e `remove` rifiutano; `--force` sostituisce (o cancella) lo
  stesso. Anche una cartella con lo stesso nome che non arriva dall'IDE si sostituisce solo con `--force`.

Uscita: 0 (fatto), 1 (errore, o aggiornamento da applicare), 2 (uso errato).

## Progetto nuovo

«Progetto: nuovo…» (riquadro comandi) mostra un modulo nel pannello Interfacce: dove crearlo, il nome della
cartella, il modello, una frase sul progetto, se creare git e quale agente aprire. Dalla riga di comando:

```sh
idetea new meu-app --modelo rust --descricao "um conversor de unidades" --agente claude
```

L'IDE crea la cartella, git, le skill del modello, l'`AGENTS.md` con la **richiesta iniziale** e le istruzioni del
pannello, e un `CLAUDE.md` che importa l'`AGENTS.md`. A creare il progetto è l'agente.

**L'agente in un terminale dell'IDE.** Il modulo offre gli agenti che stanno nel `PATH` (`claude`, `codex`, `gemini`,
`opencode`; il primo trovato arriva già scelto) e «Nessuno». Con un agente, l'IDE scrive nel progetto nuovo una
[richiesta di terminale](terminais-de-agente.md) (il documento è in portoghese) `.idetea/agents/agente.json` con il
comando `<agente> 'Start with the initial request in AGENTS.md.'`. La finestra nuova mostra la richiesta nella scheda
«Richiesta di terminale», come qualsiasi altra: l'agente si apre solo dopo «Consenti ed esegui». Con «Nessuno»,
aprilo tu nella cartella nuova e chiedi di iniziare.

| Modello | Skill |
|---|---|
| `vazio` | `novo-projeto` (l'agente chiede che cos'è il progetto) |
| `rust` | `novo-projeto`, `novo-projeto-rust` |
| `site` | `novo-projeto`, `novo-projeto-site`, `frontend-design` (da `anthropics/skills`) |
| `python` | `novo-projeto`, `novo-projeto-python` |
| `fastapi` | `novo-projeto`, `novo-projeto-python`, `novo-projeto-fastapi` |

Le skill stanno in [`skills/`](../skills/) in questo repository, e i modelli in
[`skills/modelos.toml`](../skills/modelos.toml): un modello nuovo è solo una voce lì.
