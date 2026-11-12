# Linguaggi

<p><a href="languages.md">English</a> · <a href="languages.fr.md">Français</a> · <a href="languages.de.md">Deutsch</a> · <b>Italiano</b> · <a href="languages.ja.md">日本語</a> · <a href="languages.zh-CN.md">中文（简体）</a></p>

Che cosa fa IDE TEA per ogni linguaggio, e che cosa non fa. L'IDE integra strumenti che installi tu (compilatore,
server di linguaggio, debugger); non li scarica mai. Senza, i file si aprono lo stesso con i colori e la modifica
funziona; mancano solo le parti che hanno bisogno dello strumento, e l'IDE dice che cosa installare.

Per l'elenco completo delle funzionalità, vedi [features.it.md](features.it.md).

## Panorama

| Linguaggio | File | Server di linguaggio (comando predefinito) | Attività (`Ctrl+Shift+B`) | Debug (`F5`) |
|---|---|---|---|---|
| Rust | `.rs` | `rust-analyzer` | cargo | gdb, compila con cargo |
| C / C++ | `.c` `.h` `.cc` `.cpp` `.cxx` `.hpp` `.hh` | `clangd` | target del Makefile | gdb, con una sezione `[debug]` |
| Python | `.py` `.pyi` | `pyright-langserver --stdio` | — | — |
| TypeScript / JavaScript | `.ts` `.tsx` `.js` `.jsx` | `typescript-language-server --stdio` | script di `package.json` | — |
| Go | `.go` | `gopls` | — | — |
| Lua | `.lua` | `lua-language-server` | — | — |
| Zig | `.zig` `.zon` | `zls` | passi di `build.zig` | gdb, compila con `zig build` |
| [Ziex](https://ziex.dev) | `.zx` `.mdzx` | `zx lsp` | passi di `build.zig` | — |
| [Hare](#hare) | `.ha` | `hare-lsp -S` | target del Makefile | — |
| [MoonBit](#moonbit) | `.mbt` `.mbti` `.mbt.md` | `moon lsp` | `moon check`, `build`, `test`, `fmt --check` | gdb, compila con `moon build --target native -g` (sperimentale) |

Che cosa offre un server di linguaggio (completamento, rinomina, formattazione...) dipende dal server: l'IDE usa
quello che il server annuncia. Ogni linguaggio ha i colori; Hare, MoonBit e Ziex portano con l'IDE la propria
grammatica.

## Hare

[Hare](https://harelang.org) è un linguaggio di sistema piccolo. Il suo server di linguaggio è giovane, e l'IDE lo
dice, invece di nasconderlo.

**Installazione**

- `hare` dalla tua distribuzione (su Arch: `pacman -S hare`).
- `hare-lsp` non arriva insieme a Hare. Compilalo dal [suo repository](https://git.sr.ht/~whynothugo/hare-lsp)
  (gli serve la libreria [`hare-json`](https://git.sr.ht/~sircmpwn/hare-json); su Arch è nell'AUR), poi metti
  l'eseguibile nel `PATH`, per esempio in `~/.local/bin`:

  ```sh
  make build PREFIX=/usr          # PREFIX: dove è installata la libreria standard di Hare
  install -Dm755 hare-lsp -t ~/.local/bin/
  ```

**Verificato il 2026-11-09** con Hare 0.26.0.1 e `hare-lsp` `e3db7b8`:

| | |
|---|---|
| Colori | sì |
| Hover | sì |
| Vai alla definizione (`F12`), fra i moduli | sì |
| Struttura e simboli nel file (`Ctrl+Shift+O`) | sì |
| Errori di sintassi | sì |
| Errori di tipo e altri controlli semantici | no (non ancora nel server) |
| Completamento, riferimenti, rinomina, formattazione | no (non ancora nel server) |
| Inlay hint | il server li ha; l'IDE non mostra ancora gli inlay hint |
| Attività | i target del tuo Makefile; un Makefile da solo non vuol dire Hare, quindi l'IDE non aggiunge attività `hare`. Aggiungi le tue in `[tasks]` |

**Problema noto:** il parser di Hare (`hare::parse`, usato da `hare-lsp`) va in crash su qualsiasi istruzione a cui
manca il `;`, per esempio `let x = 1` seguito da un'altra riga. L'IDE te lo dice la prima volta e riporta su il
server da solo quando smetti di scrivere.

**Provalo:** apri [`examples/languages/hare`](../examples/languages/hare) ed esegui `check` dalle attività.

## MoonBit

[MoonBit](https://www.moonbitlang.com) porta il server di linguaggio insieme alla toolchain, quindi non c'è altro
da installare.

**Installazione:** l'[installer ufficiale](https://www.moonbitlang.com/download). Mette `moon` in `~/.moon/bin` (o
`$MOON_HOME/bin`); l'IDE guarda anche lì, quindi funziona anche se lo apri dal menu delle applicazioni.

**Verificato il 2026-11-09** con `moon 0.1.20261104`:

| | |
|---|---|
| Colori, interpolazione delle stringhe compresa | sì |
| File di interfaccia `.mbti` | sì |
| Hover | sì |
| Errori di tipo | sì |
| Completamento, vai alla definizione, formattazione tramite il server | quello che offre `moon lsp`; non ancora controllato voce per voce |
| Attività | `moon check`, `moon build`, `moon test`, `moon fmt --check` |
| Debug (`F5`) | sperimentale: backend nativo con gdb, vedi sotto |
| `.mbt.md` (Markdown con codice MoonBit) | colori nei blocchi ` ```mbt ` e ` ```moonbit `; hover al loro interno; `moon test` esegue i blocchi ` ```mbt check ` |

Le attività partono nella cartella del `moon.mod` più vicino al file aperto, così ogni modulo di un'area di lavoro
esegue le proprie; senza un file aperto si usa il `moon.mod` della radice del progetto. Il vecchio `moon.mod.json`
non accende le attività: MoonBit chiede di migrarlo con `moon fmt`.

**Debug** (verificato il 2026-11-10, Linux, gdb 18): con un `moon.mod` nella radice della cartella aperta e senza
una build `[debug]` nella config, `F5` esegue `moon build --target native -g` e fa il debug del programma compilato.
Funzionano i punti di interruzione nei file `.mbt`, l'esecuzione passo passo, lo stack di chiamate con i file e le
righe giusti, e i valori dei numeri e di altre variabili semplici. Quel che è ancora grezzo:

- I nomi delle funzioni nello stack di chiamate sono quelli compilati, per esempio `_M0FP26idetea7example7sum__to` per `sum_to`.
- Le stringhe compaiono come byte UTF-16 (`"M\000o\000o\000n..."`).
- Subito dopo un passo, una variabile può comparire come `<optimized out>`.
- Solo il backend nativo: i programmi Wasm e JavaScript non si possono mettere in debug dall'IDE.

Nei file `.mbt.md`, Markdown intorno e MoonBit dentro i blocchi di codice; l'anteprima Markdown funziona come per
qualsiasi `.md`. Lì l'IDE usa una piccola grammatica Markdown sua, quindi il resto del testo ha meno colori che in
un `.md` normale.

**Provalo:** apri [`examples/languages/moonbit`](../examples/languages/moonbit), passa il mouse su `sum_to` ed esegui
`moon test` dalle attività; `sum.mbt.md` ha un test dentro il Markdown.

## Un linguaggio che non c'è

Tre cose nella config coprono la maggior parte dei linguaggi, senza cambiare l'IDE:

```toml
# ~/.config/idetea/config.toml, oppure .idetea/config.toml nel progetto
[lsp.servers.mylang]
command = "mylang-lsp"
args = ["--stdio"]
languages = { ml = "mylang" }    # estensione del file = languageId inviato al server

[tasks]
"mylang test" = "mylang test"
```

Per i colori, metti una grammatica `.sublime-syntax` in `~/.config/idetea/syntaxes/`. Una grammatica lì sostituisce
anche quella integrata per la stessa estensione, e `[lsp.servers.<name>]` può cambiare o spegnere (`enabled = false`)
qualsiasi server della tabella sopra.

Se usi un linguaggio che le altre IDE lasciano fuori e lo vorresti integrato, apri una issue e di' su quale
progetto lo proveresti.
