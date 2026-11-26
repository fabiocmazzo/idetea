# IDE TEA: Funktionen und Tastenkürzel

[English](features.md) · [Français](features.fr.md) · **Deutsch** · [Italiano](features.it.md) · [日本語](features.ja.md) · [中文（简体）](features.zh-CN.md)

Die vollständige Liste, für die technische Leserschaft. Die Kurzfassung steht in der [README](../README.de.md).

> Vor 1.0. Gebaut und getestet auf Linux, macOS (Apple Silicon) und Windows (x86_64).

## Editor

- Eine Datei nach Namen öffnen (`Ctrl+P`) mit unscharfer Suche; `main.rs:42` springt direkt zur Zeile.
- Zu einem Symbol gehen (Funktion, Typ, Methode …) in der Datei (`Ctrl+Shift+O`) oder im Projekt (`Ctrl+T`).
- Registerkarten mit einer Markierung, wenn die Datei geändert ist; öffnen (`Ctrl+O`), speichern und speichern unter, mit dem Dateidialog des Systems.
- Syntaxhervorhebung für etwa 200 Sprachen und Dateitypen (TOML, `.env`, INI, Dockerfile, TypeScript, Zig,
  [Ziex](https://ziex.dev) `.zx` …); helle und dunkle Designs (dazu die Designs von iced: Nord, Dracula …). Weitere Sprachen:
  `.sublime-syntax`-Dateien nach `~/.config/idetea/syntaxes/` legen (gelesen, wenn die IDE öffnet).
- Suchen und Ersetzen, Gehe zu Zeile, Rückgängig/Wiederholen, Mehrfachcursor, Codefaltung und ein optionaler Vim-Modus.
- Vorübergehender Editor-Zoom (`Ctrl+=` / `Ctrl++`, `Ctrl+-`, `Ctrl+0` oder `Ctrl+wheel`), gemeinsam für offene Registerkarten und Teilungen;
  das Terminal und die eingestellte Grundgröße bleiben unverändert.
- Markdown- und SVG-Vorschau neben dem Editor (`Ctrl+Shift+V` oder die Schaltfläche in der Registerkartenleiste), aktualisiert sich beim Tippen.
- Bilder (png, jpg, gif, webp, bmp, ico …) öffnen in einer Registerkarte, mit Zoom per Mausrad und Ziehen.
- PDF öffnet in einer Registerkarte (gerendert von `hayro`, reines Rust), mit Zoom und an die Breite anpassen.
- CSV, TSV und JSONL öffnen als Tabelle (der Knopf in der Tab-Leiste oder `table.toggle` wechselt zum Text): sortieren
  und filtern, ohne die Datei zu ändern; eine Zelle bearbeiten ändert nur dieses Feld im Text (Anführungszeichen
  bleiben, `Strg+Z` macht es rückgängig). In JSONL nimmt eine Zelle JSON an; ein ungültiger Wert wird abgelehnt.
  Spaltenbreite durch Ziehen am Rand; Zeilennummern anklicken (Shift erweitert), `Strg+C` kopiert sie als TSV,
  `Strg+Shift+C` als CSV. `[table] max_mb` (Standard 50) setzt die Größengrenze.
  Zeilen und Spalten über die Leiste: *+ Zeile* nach der gewählten, *Zeilen löschen* (oder `Entf` auf den gewählten
  Zeilen), *+ Spalte* und *Spalte löschen*; zwei Klicks auf einen Spaltennamen benennen ihn um (in JSONL den Schlüssel
  in jedem Objekt). Jede Aktion ist ein Schritt im Rückgängig, der Rest des Texts bleibt, wie er war.
- CRLF bleibt erhalten. Dateien in anderen Kodierungen öffnen und speichern in ihrer eigenen (BOM, UTF-16, `charset`
  aus `.editorconfig`, sonst `[editor] fallback_encoding`, Standard `windows-1252`); ein Klick auf die Kodierung in
  der Statusleiste öffnet neu oder speichert mit einer anderen. Ein Zeichen, das nicht passt, wird beim Speichern
  abgelehnt, nie zu `?`. Binärdateien werden nicht geöffnet.
- Große Dateien: eine Datei von 50 MB öffnet in unter 0,1 s. Eine Datei mit einer riesigen Zeile (minifiziertes JSON oder JS) öffnet in einem Leichtmodus,
  ohne Farben und mit Umbruch, damit das Fenster nicht einfriert.
- Optional beim Speichern: Leerzeichen am Zeilenende entfernen, abschließender Zeilenumbruch, mit dem Sprachserver formatieren. Eine `.editorconfig`
  im Projekt hat Vorrang.

## Projekt

- **Dateibaum:** erstellen, umbenennen und löschen (in den Papierkorb), die aktuelle Datei anzeigen, Git-Farben (geändert, neu,
  Konflikt, ignoriert).
- **Projektsuche** während Sie tippen, unter Beachtung von `.gitignore`, mit Regex, ganzem Wort und Groß-/Kleinschreibung. Rechtsklick auf einen Ordner oder eine Datei sucht nur dort (ein Etikett im Bereich hebt das auf). Einzuschließende und auszuschließende Dateien sind Globs, mit Komma getrennt (`src/**`, `*.test.rs`), hinter „Filter“. Jeder Treffer ist hervorgehoben; die Ersetzungsvorschau streicht die alte Spanne durch und hebt die neue hervor. Auf und Ab gehen durch die Ergebnisse, Eingabe öffnet die Zeile, Links und Rechts klappen eine Datei zu. `F4` und `Umschalt+F4` springen zum nächsten und vorherigen Treffer, auch aus dem Editor. Kontextzeilen kommen aus `[search] context_lines` (Standard 0) und einer Schaltfläche. Die letzten 20 Suchen bleiben im Verlauf.
- **Kontextmenüs** im Baum, auf den Registerkarten und auf den Dateien im Git-Bereich.
- **Aufgaben** (`Ctrl+Shift+B`): eigene Befehle aus der Konfiguration, dazu das, was die IDE im Projekt findet (cargo,
  Skripte aus `package.json`, Makefile-Ziele, Schritte aus `build.zig`), mit der Ausgabe im Terminal. MoonBit-Module bekommen außerdem
  `moon check`, `build`, `test` und `fmt --check`, ausgeführt aus der `moon.mod`, die der aktiven Datei am nächsten liegt.
- **Neues Projekt** („Projekt: neu…“ oder `idetea new <Ordner>`): Ordner, git, die Agent-Skills einer Vorlage (leer,
  Rust, Website, Python, FastAPI) und eine Anfangsanfrage in `AGENTS.md`. Siehe [skills.de.md](skills.de.md).

## Sprachen (LSP)

- Fehler und Warnungen (Wellenunterstreichung und Hover mit der Diagnose im Editor, Bereich Probleme und ein Zähler in der Statusleiste),
  Vervollständigung, Gehe zu Definition, Formatieren.
- Code-Aktionen (`Ctrl+.`): die Korrekturen und Refactorings, die der Server für den Fehler oder die Auswahl
  anbietet (fehlender Import, Funktion erzeugen, Variable extrahieren), die empfohlene zuerst. Die Unterstreichungen
  folgen dem bearbeiteten Text, bis der Server neue Diagnosen schickt.
- Inlay Hints: der abgeleitete Typ von Variablen und die Parameternamen in Aufrufen, grau im Code. Der Cursor springt
  darüber, und sie werden nie gespeichert. Standardmäßig an; `Ctrl+Alt+I` schaltet sie ein und aus, `[lsp]
  inlay_hints = false` dauerhaft.
- Semantische Farben: Der Sprachserver sagt, was jeder Name ist, und Parameter, Felder und Funktionsaufrufe bekommen
  die Farben des Themes dafür, über denen der Grammatik (lokale Variablen behalten die Farbe der Grammatik).
  Standardmäßig an; `[lsp] semantic_tokens = false` schaltet sie aus.
- Gehe zu Implementierung (`Ctrl+F12`) und zum Typ; bei mehreren Ergebnissen geht die Liste in den Bereich.
- Verweise suchen (`Shift+F12` oder `Ctrl+Klick` auf die Deklaration), nach Datei gruppiert, und zurück/vorwärts nach jedem
  Sprung (`Alt+←` / `Alt+→` oder die Seitentasten der Maus).
- Aufrufhierarchie (`Alt+Shift+H`): wer eine Funktion aufruft und was sie aufruft, als Baum, der sich beim Öffnen lädt.
- Bereich Gliederung (`Alt+7`): Funktionen, Strukturen, Methoden und Felder der Datei, mit dem, was öffentlich, privat oder
  eingeschränkt ist (`pub(crate)`, `protected` …), und das Symbol unter dem Cursor ist hervorgehoben.
- Brotkrümel über dem Editor: `Projekt › Ordner › Datei › impl App › fn update`, der Pfad und die Symbole, wo der
  Cursor steht. Ein Ordner zeigt die Datei im Baum; ein Symbol springt dorthin. `[editor] breadcrumbs = false` schaltet sie aus.
- Schriftart für Code: JetBrains Mono liegt der IDE bei; jede installierte Schriftart funktioniert (`[editor] font = "Fira Code"`), mit ihren
  Ligaturen. Schriftstärke und Zeilenhöhe ebenfalls (`font_weight`, `line_height`), auch unter Einstellungen → Darstellung. Der Editor,
  seine Zeilennummern und das Terminal nutzen dieselbe Schriftart.
- Dateisymbole nach Sprache und Typ (der Seti-Satz aus VS Code, mitgeliefert, dazu die eigenen Symbole der IDE für Hare und
  MoonBit) im Dateibaum, in Registerkarten, bei Ctrl+P, in den Brotkrümeln,
  in Suchergebnissen und im Git-Bereich. `[ui] file_icons = "none"` (oder Einstellungen → Darstellung) geht zurück zu einem Symbol für
  jede Datei.
- Shift+Klick erweitert die Auswahl vom letzten Klick (oder vom Cursor, nachdem Sie ihn mit der Tastatur bewegt haben).
- Wort für Wort: `Ctrl+Left/Right` bewegt (mit `Shift` wird ausgewählt), `Ctrl+Backspace/Delete` löscht. `Home` geht zum
  ersten Zeichen der Zeile, dann zu Spalte 0. `Shift+Tab` nimmt die Einrückung weg.
- Tasten wie in VS Code, wo sie früher abwichen: den Block am Cursor falten oder auffalten mit `Ctrl+Shift+[` / `]`, die
  zuletzt geschlossene Registerkarte wieder öffnen mit `Ctrl+Shift+T`, den unteren Bereich mit `Ctrl+J`, das Terminal mit `` Ctrl+` `` oder `Ctrl+'`.
- Schon eingerichtet: rust-analyzer, clangd, pyright, typescript-language-server, gopls, lua-language-server, zls (Zig),
  `zx lsp` ([Ziex](https://ziex.dev); ohne die `zx`-CLI baut ein Ziex-Projekt seinen eigenen Server mit `zig build`),
  `hare-lsp -S` und `moon lsp`. Installieren Sie nur den Server; den Befehl können Sie ändern, ihn abschalten oder weitere in der
  Konfiguration hinzufügen. Der Server von Hare beschränkt sich auf Diagnose aus dem Parser, Definition, Hover und Dokumentensymbole; die Unterstützung für MoonBit
  umfasst `.mbt`, `.mbti` und `.mbt.md` (Markdown mit MoonBit-Blöcken), im Rahmen dessen, was die installierte Toolchain kann. Einzelheiten je Sprache:
  [languages.de.md](languages.de.md).

## Terminal

- Integriertes Terminal (alacritty) im unteren Bereich, mit den Farben und der Schriftart des Designs.
- Mehrere Shells in Registerkarten („Terminal 1“, „Terminal 2“ …), dazu die eigene Registerkarte der Aufgabe; `+` öffnet eine, `×` schließt sie,
  `exit` schließt die Registerkarte.
- Tastenkürzel der Shell (`Ctrl+C`, `Ctrl+W` …) gehen an die Shell; die Tastenkürzel der IDE mit `Ctrl+Shift` funktionieren weiter.
- Jede Registerkarte zeigt einen Punkt mit ihrem Zustand: läuft, neue Ausgabe, die Sie noch nicht gesehen haben,
  bittet um Aufmerksamkeit (die Glocke oder die Hooks eines Agenten) und am Ende der Exit-Code (grün oder rot); der
  Tooltip erklärt ihn. Shell-Registerkarten übernehmen den Titel, den die Shell setzt.
- Agenten erreichen ihre eigene Registerkarte: Jede bekommt `IDETEA_TERMINAL`, und die Hooks von Claude Code, die
  `idetea mascot setup-hooks` installiert, markieren die Registerkarte des Agenten, wenn er um Erlaubnis bittet oder
  seinen Zug beendet.
- `Ctrl+Shift+PgDn` / `Ctrl+Shift+PgUp` wechseln zwischen den Terminal-Registerkarten, auch im Terminal. *Terminal:
  gehe zu…* listet die Registerkarten mit ihrem Zustand; *Terminal: nächster Tab, der auf dich wartet* springt zur
  nächsten, die Sie braucht. Doppelklick auf eine Registerkarte (oder *Terminal: Tab umbenennen*) benennt sie.
- Einen Agenten oder eine Aufgabe zu schließen, die noch läuft, fragt vorher nach.
- Zwei Terminals nebeneinander (`Ctrl+Shift+5` oder die Teilen-Schaltfläche); ein Klick oder Tippen in einem gibt
  ihm den Fokus.
- Suche im Terminal (die Lupe oder *Terminal: suchen*): einfacher Text, der neueste Treffer zuerst.
- `[terminal] scrollback` legt fest, wie viele Zeilen jede Registerkarte behält (standardmäßig 10000).
- `Strg+Klick` (`Cmd+Klick` auf macOS) folgt dem, was das Terminal zeigt. Ein Dateipfad (`src/main.rs:42:5`, `-->`
  von rustc, `File "x.py", line 3` von Python, `at f (x.ts:8:2)` von Node, `x.cs(10,5)`) öffnet sich in einer
  Registerkarte an dieser Zeile, aber nur, wenn die Datei existiert; relative Pfade werden im aktuellen Ordner der
  Shell, im Startordner der Registerkarte und im Projektstamm gesucht. Ein Ordner des Projekts wird im Baum gezeigt.
  `http(s)`- und `mailto`-Links sowie OSC-8-Links (`ls --hyperlink`, `gh`) öffnen sich im System; andere Schemata
  (`ssh:`, `magnet:`) werden stattdessen kopiert. Solange `Strg` gedrückt ist, zeigt ein Kasten das Ziel des Links.
- Eine schmale Bildlaufleiste rechts in jedem Terminal, mit eigenem Streifen, sodass sie nie Text verdeckt: Daumen
  ziehen oder auf die Leiste klicken, um eine Seite zu springen. Auf dem alternativen Bildschirm (vim, less, htop)
  verschwindet sie.
- Terminal-Registerkarten kommen beim erneuten Öffnen der IDE zurück: Shells von selbst, mit ihren Namen; vom
  Starter geöffnete Agenten erst, wenn du zustimmst (*Alle fortsetzen*, *Auswählen…*, *Nein* oder später
  *Terminal: Agenten der letzten Sitzung fortsetzen*). Claude Code und Codex setzen dasselbe Gespräch fort,
  andere Agenten beginnen ein neues.
- Auch jeder andere Agent setzt sein Gespräch fort, wenn sein Starter sagt, wie: `"Mein Agent" = { command =
  "mein-agent", resume = "mein-agent --continue" }` in `[agents.launchers]`.
- Shells öffnen sich wieder in dem Ordner, in dem sie waren (Linux und macOS), und auch die angezeigte
  Registerkarte und die Teilung kommen zurück.
- `[terminal] keep_scrollback = true` behält beim Schließen die letzten Zeilen jeder Registerkarte und zeigt sie
  einmal über dem neuen Prompt. Standardmäßig aus: ein Terminal enthält Passwörter und Tokens. Der Text bleibt
  außerhalb des Projekts und ist nur für dich lesbar.

## Git

- Branch, Commits voraus/zurück und die Zahl der Änderungen in der Statusleiste.
- Bereich mit Stagen und Staging aufheben je Datei oder für alle, farbiger Diff, Commit, Änderungen verwerfen (mit Bestätigung).
- Pushen, Pullen und Abrufen. Die IDE wartet nie auf ein Passwort: sie nutzt Ihren ssh-Agent oder Ihren git-Credential-Helper.
- Markierungen am Rand des Editors bei Zeilen, die seit dem letzten Commit hinzugekommen, geändert oder gelöscht
  sind; ein Klick auf eine Markierung zeigt die alten Zeilen und kann sie rückgängig machen.
- Autor und Commit der Cursorzeile (Blame) in der Statusleiste, mit Details und einer Schaltfläche zum Kopieren des
  Hashs.
- Branch wechseln oder anlegen über die Palette (*Git: Branch wechseln*).
- Nutzt das installierte `git`, also gelten Ihre Konfiguration, Hooks und Zugangsdaten.

## Debuggen

- Haltepunkte mit einem Klick auf die Zeilennummer (`F9`); sie wandern mit dem Text mit, während Sie bearbeiten.
- `F5` baut und startet: in Rust-Projekten nutzt es `cargo build` und findet die ausführbare Datei; in Zig-Projekten
  `zig build` und die ausführbaren Dateien in `zig-out/bin`; in MoonBit-Modulen `moon build --target native -g`
  (experimentell, siehe [languages.de.md](languages.de.md#moonbit)); sonst sagt ein Abschnitt `[debug]`
  in der Konfiguration das Programm, die Argumente und den Build-Befehl.
- Nächste Zeile, in die Funktion springen und aus der Funktion austreten (`F10`, `F11`, `Shift+F11`), anhalten und beenden, mit der aktuellen Zeile im Editor markiert.
- Bereich mit der Aufrufliste (Klick springt zum Frame), Variablen als Baum und der Ausgabe des Programms.
- Nutzt `gdb` 14 oder neuer (Rust, C, C++, Zig und, experimentell, MoonBit), oder `lldb-dap` unter macOS.

## Arbeiten mit Agenten

Alles davon sind schlichte Dateien im Ordner `.idetea/` des Projekts, also kann jeder Agent, der Dateien liest und schreibt, es nutzen.
„Interfaces: den Agenten dieses Projekts beibringen“ schreibt die Anweisungen in `AGENTS.md` und `CLAUDE.md`.

- **Bereich Interfaces** (`.idetea/ui/<Name>.json`): der Agent schreibt ein Dokument, die IDE zeigt es als Formular (Text,
  Markdown, `select`, `checkbox`, `input`, Schaltflächen, Schritte für manuelle Tests, Dateien zum Bearbeiten, Bilder, Galerien, animierte
  Sprites, Klänge mit Wellenform und Spitzenpegel, Tabellen zum Auswählen von Zeilen) und schreibt Ihre Antwort nach `.idetea/ui/responses/`. Der Bereich zeigt,
  ob der Agent noch wartet. Format: [docs/interfaces/formato.md](interfaces/formato.md) (vorerst auf Portugiesisch).
- **Agenten-Terminals** (`.idetea/agents/<Name>.json`): der Agent bittet um ein Terminal mit einem Befehl; die IDE zeigt
  die ganze Anfrage in einer Registerkarte „Terminalanfrage“ und führt sie erst aus, nachdem Sie sie zulassen. Der Fortschritt geht nach
  `<Name>.status.json` (`pending`, `running`, `exited` mit dem Code, `refused`, `closed`, `invalid`, `lost`);
  das Löschen der Anfrage stoppt das Terminal (ist der Befehl schon zu Ende, bleibt die Registerkarte, bis Sie sie schließen).
- **Zulassungsliste:** `[agents] allow = ["npm run dev", "claude *"]` in Ihrer Benutzerkonfiguration führt diese Befehle ohne
  Nachfrage aus: der genaue Befehl oder ein Präfix, dem nur schlichte Argumente folgen (kein `;`, `&`, `|`, `$`, keine Backticks,
  keine Umleitungen …). In einer Projektkonfiguration wird sie ignoriert, mit einer Warnung.
- **Einen Agenten starten** (`Ctrl+Shift+A`, auch auf dem Startbildschirm): die Coding-Agenten im PATH (Claude
  Code, Codex, Gemini CLI, OpenCode, Aider, Cursor Agent) und deine eigenen `[agents.launchers]`, in einem
  Terminal-Tab im Projektordner oder in einem neuen Worktree (`.idetea/worktrees/launch-<Branch>`, wird beim
  Schließen des Tabs entfernt, wenn er sauber ist; der Branch bleibt). `[agents] default` steht oben. Launcher
  zählen nur in deiner Benutzerkonfiguration. Fehlen in `AGENTS.md`/`CLAUDE.md` die Anweisungen dieser IDE, fragt
  sie vorher: schreiben, nur öffnen oder in diesem Projekt nicht mehr fragen.
- **Kontext an den Agenten senden** (`Ctrl+L` im Editor): der Ort des Cursors (oder die Auswahl, mit dem Text in
  einem Codeblock) und die Probleme dieser Zeilen, eingefügt in den Agenten-Tab auf dem Bildschirm oder den zuletzt
  benutzten, nicht abgeschickt. Ohne Agenten-Tab öffnet sich zuerst der Starter. Ein Rechtsklick im Problem-Panel
  sendet ein Problem. `idetea agent [Name] [--worktree <Branch>]` fordert einen Agenten-Tab aus einem Terminal an
  (`--list` zeigt sie).
- **Projektgedächtnis:** Die Anweisungen lehren auch ein gemeinsames Gedächtnis in einfachen Dateien unter git:
  Fakten in `docs/memory/` (eine pro Datei, mit einem Index, den die Agenten zuerst lesen) und
  Architekturentscheidungen als ADRs in `docs/decisions/`. Jeder Agent, der `AGENTS.md`/`CLAUDE.md` liest, nutzt
  es. Siehe [project-memory.md](project-memory.md) (Englisch).
  Das Panel **Projektgedächtnis** listet Fakten und Entscheidungen mit Suche, öffnet und löscht sie und warnt vor
  veralteten Fakten, nicht mehr vorhandenen Pfaden und möglichen Zugangsdaten. Im Interfaces-Verlauf macht **Als
  Entscheidung festhalten** aus einer beantworteten Frage einen ADR.
- **Helfer im eigenen Worktree:** `"worktree": "<Branch>"` führt den Befehl in `.idetea/worktrees/<Name>` auf diesem
  Branch aus. Jeder Befehl bekommt `IDETEA_ANSWER`, eine Datei im Hauptprojekt für die Antwort. Das Löschen der Anfrage
  entfernt den Worktree, wenn er keine uncommitteten Änderungen hat; der Branch bleibt immer.
- **Befehle für Agenten:** `idetea ask` (eine Frage im Bereich, Warten auf die Antwort), `idetea run` (ein Terminal,
  Warten auf das Ende und die Antwort des Helfers), `idetea status` (was Sie offen haben, ungespeicherte Änderungen, der Cursor),
  `idetea open Datei:Zeile` (zeigt Ihnen eine Datei), `idetea history` (was Sie im Bereich geantwortet haben) und `idetea mascot`
  (Zustand des Maskottchens und Sprechblase in Echtzeit). Diese Befehle erledigen die Dateiarbeit oben in einem Aufruf, mit denselben Zustimmungen.
- **Reaktives Maskottchen (Tea):** eine Karte mit einer Teetasse in Pixel-Art in der unteren rechten Ecke (einklappbar in die Statusleiste oder
  über die Befehlspalette umgeschaltet), die `.idetea/` in Echtzeit beobachtet (`idle`, `working`, `thinking`, `attention`,
  `celebrating`, `error`, `sleeping`, dazu die interaktiven Ausdrücke `curious` und `annoyed` beim Darüberfahren, Doppelklick
  oder Anstupsen). Jede Situation hat mehrere kurze Sätze; derselbe Satz kommt nicht zweimal hintereinander. Tea bemerkt auch ein
  Speichern, das Ende oder Scheitern des Builds im Debugger, ein langes Verweilen (sie döst und wacht auf, wenn Sie zurückkommen) und die
  Tageszeit, einschließlich eines leisen Hinweises spätabends, sich auszuruhen. Ein Klick auf die Sprechblase öffnet den Bereich Interfaces, Terminal oder
  Probleme und nimmt nie den Fokus. Sie spricht außerdem einmal über einen Plattenkonflikt bei ungespeicherten
  Änderungen, einen Git-Konflikt, das Ende einer Terminal-Aufgabe, eine beendete Debug-Sitzung, ein Zeichen, das
  die Kodierung nicht hält, und ein Ersetzen in mehreren Dateien. Warnungen und Fehler, die gerade auf null
  gingen, nur bei `chatty`. Die Einstellungen schreiben dieselben Schalter. `idetea mascot setup-hooks`
  richtet Claude-Code-Hooks im Projekt ein und lässt vorhandene Hooks stehen. Mit `[mascot] tips = true` (und `chatty` nicht `quiet`) nennt sie ab und zu
  ein Tastenkürzel, das es wirklich gibt. `[mascot] chatty` ist `quiet`, `normal` oder `chatty`. Agenten oder Hooks können sie auch
  direkt steuern über `idetea mascot <Zustand> [--text "…"] [--source <Name>] [--ttl <Sekunden>]`, und eigene Sprite-Pakete funktionieren über
  `.idetea/mascot/mascot.toml` oder `~/.config/idetea/mascots/<Paket>/mascot.toml` (`[mascot] enabled = true`).
- **Text auswählen und kopieren:** „Text auswählen“ oben an einer Oberfläche (offen oder im Verlauf) zeigt ihre Texte in einem
  Editor nur zum Lesen: mit der Maus auswählen, mit Ctrl+C kopieren; „Formatiert anzeigen“ geht zurück.
- **Bessere Entscheidungen im Panel:** eine `select`-Option (oder ein `gallery`-Eintrag) kann `recommended` sein, mit einer `note` zur Begründung; `number` und `slider` liefern eine JSON-Zahl mit `min`, `max` und einer Warnung außerhalb des Bereichs; `show_if` zeigt eine Komponente nur, wenn ein anderes Feld einen Wert hat (ein einfacher Vergleich, nie ein Ausdruck), und ein verstecktes Feld hält den Button nicht auf; `columns` stellt bis zu 4 Komponenten nebeneinander, im schmalen Panel wieder untereinander.
- **Anhänge in der Antwort:** neben dem Kommentar *Bildschirmfoto der IDE anhängen* (nur das IDE-Fenster, nur auf Klick) und *Datei anhängen…* (bis 8 MB). Sie sehen eine Vorschau und können den Anhang vor dem Antworten entfernen; die IDE kopiert ihn nach `.idetea/ui/responses/<name>/anexos/`, und die Antwort listet ihn auf. Ein Button mit `"attach": "screenshot"` wartet auf einen.
- **Verlauf der Fragen:** die IDE behält jede Frage, Ihre Antwort, Ihren Kommentar und wann der Agent sie gelesen
  hat, in `.idetea/history/` (außerhalb von git, bis Sie ihn leeren). Der Bereich **Verlauf der Interfaces** links (verborgen
  in der Voreinstellung) listet zuerst die offenen Fragen, dann die beantworteten nach Tag, mit Suche; ein Eintrag öffnet nur zum Lesen,
  so, wie Sie geantwortet haben.
- **Dieses Projekt reparieren** („Interfaces: dieses Projekt reparieren“): bringt `.idetea/` nach einem Absturz oder einer
  schlechten Bearbeitung wieder in einen nutzbaren Zustand. **Skills:** `idetea skills` installiert Agent-Skills aus einem git-Repository in das Projekt
  ([skills.de.md](skills.de.md)).

Einzelheiten: [terminais-de-agente.md](terminais-de-agente.md) (vorerst auf Portugiesisch), [skills.de.md](skills.de.md).

## Oberfläche

- Befehlspalette mit jedem Befehl und seinem Tastenkürzel (`Ctrl+Shift+P`).
- Hilfe (`F1`) mit allen aktuellen Tastenkürzeln und einer kurzen Anleitung.
- Einstellungen (`Ctrl+,`): Darstellung (einschließlich des Maskottchens), Erweiterungen, Tastenkürzel (zeichnet die Taste auf, die Sie drücken) und Sprachserver.
- Aktivitätsleiste, um Bereiche zu zeigen und zu verbergen; Bereiche ändern die Größe mit der Maus.
- Willkommensbildschirm, wenn keine Datei offen ist: neue Datei, Datei oder Ordner öffnen, letzte Dateien und Projekte und die wichtigsten
  Tastenkürzel. Tippen oder `Ctrl+N` holt einen unbenannten Editor zurück.
- Bereiche als abgerundete Inseln über einem dunkleren Hintergrund, mit schmalen, pillenförmigen Bildlaufleisten, die verblassen, wenn die Maus weg ist;
  `[ui] style = "classic"` hält alles auf einem Hintergrund, durch Linien geteilt. Palette, Menüs und Dialoge gleiten
  sanft herein; `[ui] animations = false` schaltet das aus.
- Oberfläche auf Portugiesisch, Englisch, Spanisch, Französisch, Deutsch, Italienisch, Japanisch und vereinfachtem Chinesisch
  (`[ui] language` oder die des Systems). Das tritt nach einem Neustart der IDE in Kraft.

## Arbeit nicht verlieren

- Beim Schließen einer Registerkarte oder beim Beenden mit ungespeicherten Änderungen wird immer zuerst gefragt.
- Eine gelöschte Datei landet im Papierkorb des Systems.
- Dateien, die außerhalb der IDE geändert werden (Terminal, `cargo fmt`, ein anderer Editor, `git checkout`), fallen auf: Baum und
  git aktualisieren sich, Registerkarten ohne Änderungen laden neu, und Registerkarten mit Änderungen bleiben, mit einer Warnung.

## Die wichtigsten Tastenkürzel

| Tastenkürzel | Was |
|---|---|
| `Ctrl+P` | Eine Datei nach Namen öffnen |
| `Ctrl+Shift+P` | Befehlspalette |
| `F1` | Hilfe: jedes Tastenkürzel und die Anleitung |
| `Ctrl+,` | Einstellungen |
| `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S` | Öffnen / Speichern / Speichern unter |
| `Ctrl+B` | Dateibaum zeigen oder verbergen |
| `Ctrl+Shift+F` | Im Projekt suchen |
| `F4` / `Umschalt+F4` | Nächstes / vorheriges Suchergebnis |
| `` Ctrl+` `` / `Ctrl+'` | Terminal |
| `Ctrl+Shift+PgDn` / `Ctrl+Shift+PgUp` | Nächste / vorherige Terminal-Registerkarte |
| `Ctrl+Shift+5` | Terminal teilen |
| `Ctrl+Shift+T` | Die zuletzt geschlossene Registerkarte wieder öffnen |
| `Ctrl+Shift+G` | Git |
| `Ctrl+Shift+M` / `F8` | Probleme / nächstes Problem |
| `F12` | Gehe zu Definition |

Alle lassen sich in den Einstellungen oder in `~/.config/idetea/config.toml` ändern.

## Aus dem Quellcode bauen

```sh
git clone https://github.com/fabiocmazzo/idetea && cd idetea
cargo build --release      # ausführbare Datei in target/release/idetea
```

Mit [`just`](https://github.com/casey/just) installiert `just run` auch das Symbol für die Leiste Ihres Desktops. In einem
Terminal öffnet `idetea .` die IDE und gibt die Eingabeaufforderung zurück; `idetea -f .` bleibt verbunden (mit den Protokollen).

## Fahrplan

- [x] Editor, Registerkarten, Designs, Palette, TOML-Konfiguration, Dateibaum, Statusleiste.
- [x] Projektsuche, LSP, Bereich Probleme, integriertes Terminal.
- [x] Git (einschließlich Pushen/Pullen), Kontextmenüs, Hilfe, Einstellungen, Symbole, in der Größe änderbare Bereiche.
- [x] Debuggen mit gdb: Haltepunkte, Einzelschritte, Aufrufliste und Variablen.
- [x] Agenten: Bereich Interfaces, Agenten-Terminals, Zulassungsliste, Helfer in Worktrees, neues Projekt mit Skills.
- [ ] Plugins von Dritten in WASM, ohne die IDE neu zu bauen (ein Prototyp existiert hinter dem Feature `plugins`).
- [ ] Weitere Debug-Adapter (codelldb, debugpy, delve …).
- [x] Andere Kodierungen als UTF-8.
- [ ] Eine 3-Wege-Merge-Ansicht; Stash über die Oberfläche.

Der ausführliche Plan, Phase für Phase, steht in [SPEC.md](../SPEC.md) (auf Portugiesisch).
