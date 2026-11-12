# Sprachen

<p><a href="languages.md">English</a> · <a href="languages.fr.md">Français</a> · <b>Deutsch</b> ·
<a href="languages.it.md">Italiano</a> · <a href="languages.ja.md">日本語</a> · <a href="languages.zh-CN.md">中文（简体）</a></p>

Was IDE TEA für jede Sprache tut, und was nicht. Die IDE bindet Werkzeuge ein, die Sie selbst installieren
(Compiler, Sprachserver, Debugger); sie lädt sie nie herunter. Ohne sie öffnen Dateien weiterhin mit Farben,
und das Bearbeiten funktioniert; es fehlen nur die Teile, die das Werkzeug brauchen, und die IDE sagt, was zu installieren ist.

Die vollständige Funktionsliste steht in [features.de.md](features.de.md).

## Überblick

| Sprache | Dateien | Sprachserver (Befehl voreingestellt) | Aufgaben (`Ctrl+Shift+B`) | Debuggen (`F5`) |
|---|---|---|---|---|
| Rust | `.rs` | `rust-analyzer` | cargo | gdb, Build mit cargo |
| C / C++ | `.c` `.h` `.cc` `.cpp` `.cxx` `.hpp` `.hh` | `clangd` | Makefile-Ziele | gdb, mit einem Abschnitt `[debug]` |
| Python | `.py` `.pyi` | `pyright-langserver --stdio` | — | — |
| TypeScript / JavaScript | `.ts` `.tsx` `.js` `.jsx` | `typescript-language-server --stdio` | Skripte aus `package.json` | — |
| Go | `.go` | `gopls` | — | — |
| Lua | `.lua` | `lua-language-server` | — | — |
| Zig | `.zig` `.zon` | `zls` | Schritte aus `build.zig` | gdb, Build mit `zig build` |
| [Ziex](https://ziex.dev) | `.zx` `.mdzx` | `zx lsp` | Schritte aus `build.zig` | — |
| [Hare](#hare) | `.ha` | `hare-lsp -S` | Makefile-Ziele | — |
| [MoonBit](#moonbit) | `.mbt` `.mbti` `.mbt.md` | `moon lsp` | `moon check`, `build`, `test`, `fmt --check` | gdb, Build mit `moon build --target native -g` (experimentell) |

Was ein Sprachserver anbietet (Vervollständigung, Umbenennen, Formatieren …), hängt vom Server ab: Die IDE nutzt, was er
meldet. Jede Sprache bekommt Farben; Hare, MoonBit und Ziex bringen ihre eigene Grammatik mit der IDE mit.

## Hare

[Hare](https://harelang.org) ist eine kleine Systemsprache. Ihr Sprachserver ist jung, und die IDE sagt das, statt
es zu verbergen.

**Installation**

- `hare` aus Ihrer Distribution (unter Arch: `pacman -S hare`).
- `hare-lsp` wird nicht mit Hare ausgeliefert. Bauen Sie es aus [seinem Repository](https://git.sr.ht/~whynothugo/hare-lsp)
  (es braucht die Bibliothek [`hare-json`](https://git.sr.ht/~sircmpwn/hare-json); unter Arch liegt sie im AUR) und legen Sie
  die ausführbare Datei in Ihren `PATH`, zum Beispiel nach `~/.local/bin`:

  ```sh
  make build PREFIX=/usr          # PREFIX: wo die Standardbibliothek von Hare installiert ist
  install -Dm755 hare-lsp -t ~/.local/bin/
  ```

**Geprüft am 2026-11-09** mit Hare 0.26.0.1 und `hare-lsp` `e3db7b8`:

| | |
|---|---|
| Farben | ja |
| Hover | ja |
| Gehe zu Definition (`F12`), über Module hinweg | ja |
| Gliederung und Symbole in der Datei (`Ctrl+Shift+O`) | ja |
| Syntaxfehler | ja |
| Typfehler und andere semantische Prüfungen | nein (im Server noch nicht) |
| Vervollständigung, Verweise, Umbenennen, Formatieren | nein (im Server noch nicht) |
| Inlay Hints | der Server hat sie; die IDE zeigt Inlay Hints noch nicht |
| Aufgaben | Ihre Makefile-Ziele; ein Makefile allein bedeutet nicht Hare, also fügt die IDE keine `hare`-Aufgaben hinzu. Eigene kommen in `[tasks]` |

**Bekanntes Problem:** Der eigene Parser von Hare (`hare::parse`, von `hare-lsp` genutzt) stürzt bei jeder Anweisung ohne `;` ab,
etwa `let x = 1` gefolgt von einer weiteren Zeile. Die IDE sagt es Ihnen beim ersten Mal und holt den Server von
selbst zurück, wenn Sie aufhören zu tippen.

**Ausprobieren:** Öffnen Sie [`examples/languages/hare`](../examples/languages/hare) und führen Sie `check` aus den Aufgaben aus.

## MoonBit

[MoonBit](https://www.moonbitlang.com) liefert seinen Sprachserver mit der Toolchain aus, es ist also nichts weiter zu
installieren.

**Installation:** der [offizielle Installer](https://www.moonbitlang.com/download). Er legt `moon` nach `~/.moon/bin` (oder
`$MOON_HOME/bin`); die IDE schaut auch dort nach, also funktioniert es auch, wenn sie aus dem Anwendungsmenü geöffnet wird.

**Geprüft am 2026-11-09** mit `moon 0.1.20261104`:

| | |
|---|---|
| Farben, einschließlich String-Interpolation | ja |
| Schnittstellendateien `.mbti` | ja |
| Hover | ja |
| Typfehler | ja |
| Vervollständigung, Gehe zu Definition, Formatieren über den Server | was `moon lsp` anbietet; noch nicht einzeln geprüft |
| Aufgaben | `moon check`, `moon build`, `moon test`, `moon fmt --check` |
| Debuggen (`F5`) | experimentell: natives Backend mit gdb, siehe unten |
| `.mbt.md` (Markdown mit MoonBit-Code) | Farben in Blöcken ` ```mbt ` und ` ```moonbit `, Hover darin; `moon test` führt die Blöcke ` ```mbt check ` aus |

Aufgaben laufen im Ordner der `moon.mod`, die der offenen Datei am nächsten liegt, also führt jedes Modul eines Arbeitsbereichs seine eigenen aus;
ohne offene Datei wird die `moon.mod` im Projektstamm verwendet. Die alte `moon.mod.json` schaltet die Aufgaben nicht ein:
MoonBit bittet Sie, sie mit `moon fmt` zu migrieren.

**Debuggen** (geprüft am 2026-11-10, Linux, gdb 18): Liegt eine `moon.mod` im Stamm des offenen Ordners und steht kein Build `[debug]`
in der Konfiguration, führt `F5` `moon build --target native -g` aus und debuggt das Programm, das dabei entstand. Haltepunkte in
`.mbt`-Dateien, Einzelschritte, die Aufrufliste mit den richtigen Dateien und Zeilen und die Werte von Zahlen und anderen einfachen
Variablen funktionieren. Was noch rau ist:

- Funktionsnamen in der Aufrufliste sind die kompilierten, etwa `_M0FP26idetea7example7sum__to` für `sum_to`.
- Zeichenketten erscheinen als UTF-16-Bytes (`"M\000o\000o\000n..."`).
- Direkt nach einem Schritt kann eine Variable als `<optimized out>` erscheinen.
- Nur das native Backend: Programme in Wasm und JavaScript lassen sich aus der IDE nicht debuggen.

In Dateien `.mbt.md` liegt Markdown drumherum und MoonBit in den Codeblöcken; die Markdown-Vorschau funktioniert wie bei jeder
`.md`. Die IDE nutzt dort eine kleine eigene Markdown-Grammatik, deshalb bekommt der übrige Text weniger Farben als in einer
schlichten `.md`.

**Ausprobieren:** Öffnen Sie [`examples/languages/moonbit`](../examples/languages/moonbit), fahren Sie über `sum_to` und führen Sie
`moon test` aus den Aufgaben aus; `sum.mbt.md` hat einen Test in seinem Markdown.

## Eine Sprache, die hier nicht steht

Drei Dinge in der Konfiguration decken die meisten Sprachen ab, ohne die IDE zu ändern:

```toml
# ~/.config/idetea/config.toml oder .idetea/config.toml im Projekt
[lsp.servers.mylang]
command = "mylang-lsp"
args = ["--stdio"]
languages = { ml = "mylang" }    # Dateiendung = languageId, das an den Server geht

[tasks]
"mylang test" = "mylang test"
```

Für Farben legen Sie eine Grammatik `.sublime-syntax` nach `~/.config/idetea/syntaxes/`. Eine Grammatik dort ersetzt auch die
mitgelieferte für dieselbe Endung, und `[lsp.servers.<name>]` kann jeden
Server in der Tabelle oben ändern oder abschalten (`enabled = false`).

Wenn Sie eine Sprache nutzen, die andere IDEs auslassen, und sie gern mitgeliefert hätten, öffnen Sie ein Issue und sagen Sie, an welchem Projekt Sie
sie ausprobieren würden.
