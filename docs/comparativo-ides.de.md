# Ehrlicher Vergleich: IDE TEA und andere IDEs auf dem Markt

<p><a href="comparativo-ides.md">Português</a> · <a href="comparativo-ides.en.md">English</a> · <a href="comparativo-ides.fr.md">Français</a> · <b>Deutsch</b> ·
<a href="comparativo-ides.it.md">Italiano</a> · <a href="comparativo-ides.ja.md">日本語</a> · <a href="comparativo-ides.zh-CN.md">中文（简体）</a></p>

Dieses Dokument bewertet **IDE TEA** (Stand `v0.15.0`) technisch, direkt und ohne Schminke gegenüber den wichtigsten IDEs und Editoren auf dem Markt:

- **VS Code / VSCodium** (der universelle Standard auf Electron)
- **Cursor / Windsurf** (Forks von VS Code mit Schwerpunkt auf KI)
- **Zed** (native IDE in Rust, Schwerpunkt Geschwindigkeit, Zusammenarbeit und eingebaute KI)
- **JetBrains (RustRover, IntelliJ, PyCharm, CLion)** (schwere semantische IDEs)
- **Neovim / Helix / Lapce** (schlanke modale und native Editoren)

> **Umfang dieser Analyse:** der Fokus ist streng technisch: **Stabilität, Leistung, Editierfunktionen, LSP, Debuggen, Git, Ökosystem und Integration mit Agenten**. Die Größe des Teams oder wer das Projekt pflegt, zählt nicht.

---

## 1. Überblick (direkte Matrix)

| Dimension | IDE TEA | VS Code | Cursor / Windsurf | Zed | JetBrains | Helix / Neovim |
|---|---|---|---|---|---|---|
| **Öffnungszeit (Kaltstart)** | **Ausgezeichnet (< 200 ms)** | Langsam (1–3 s) | Langsam (1,5–4 s) | **Ausgezeichnet (< 150 ms)** | Sehr langsam (5–20 s) | **Sofort (< 50 ms)** |
| **RAM im Leerlauf** | **Niedrig (~60–150 MB)** | Hoch (400 MB–1,5 GB) | Sehr hoch (600 MB–2 GB) | Niedrig (~120–300 MB) | Sehr hoch (1,5–4 GB+) | **Minimal (15–60 MB)** |
| **Riesige Dateien (50 MB+ / minifiziert)** | **Ausgezeichnet (automatischer Leichtmodus)** | Mittel (hängt bei langen Zeilen) | Mittel | Gut | Leidet über 20 MB | Hängt von Plugins ab (Neovim) |
| **Datenschutz / Offline / kein Konto** | **Vollständig (kein Netz außer git push/pull)** | Telemetrie voreingestellt (außer VSCodium) | Verlangt Konto und Cloud | Verlangt ein Konto für KI und Zusammenarbeit | Verlangt Konto / Lizenz | **Vollständig** |
| **Strukturiertes Protokoll für Agenten (deklarative UI, Zustimmung zu Befehlen, Maskottchen)** | **Einzigartig in der Kategorie (nativ über Dateien)** | Gibt es nicht (hängt von Chat-Erweiterungen ab) | Schwerpunkt auf proprietärem Chat/Diff | Schwerpunkt auf Chat/Inline-Hilfe | Schwerpunkt auf Chat/Inline | Gibt es nicht (nur Terminal) |
| **Inline-KI-Vervollständigung („Ghost Text“ / Tab)** | **Gibt es nicht (aus Überzeugung)** | Über Copilot | **Ausgezeichnet (führend)** | Sehr gut (Zeta) | Gut | Über Plugins |
| **Reife des Texteditors** | Gut im Grundlegenden, fortgeschrittene Funktionen fehlen | **Maßstab** | **Maßstab** | Sehr hoch | **Maßstab** | **Maßstab (modal)** |
| **Tiefe von LSP und Refaktorierung** | Hoch | **Vollständig** | **Vollständig** | Hoch | **Unübertroffen (eigener AST)** | Hoch |
| **Debuggen** | Grundlegend (GDB 14+ für Rust/C/C++/Zig, `lldb-dap` unter macOS; natives MoonBit ist experimentell) | **Vollständig (jede Sprache)** | **Vollständig** | Grundlegend/mittel (DAP erst seit Kurzem) | **Ausgezeichnet** | Grundlegend (nvim-dap) |
| **Versionsverwaltung (Git)** | Mittel (Randmarkierungen und Blame; kein 3-Wege-Merge, kein Stash) | **Vollständig** | **Vollständig** | Mittel | **Ausgezeichnet (bester Merge)** | Über LazyGit / Fugitive |
| **Plattformübergreifend** | **Linux, macOS (Apple Silicon), Windows (x86_64)** | **Linux, macOS, Windows, Web** | **Linux, macOS, Windows** | **macOS, Linux, Windows** | **Linux, macOS, Windows** | **Jedes Betriebssystem / SSH** |

---

## 2. Stabilität und Verlässlichkeit

### Wo IDE TEA wirklich stark ist
1. **Architektur ohne WebView / Chromium:** anders als VS Code, Cursor und Windsurf gibt es keine Chromium-Renderprozesse und keine Dutzende Node.js-*Extension-Hosts*, die über den Tag Speicher verlieren. Das Programm ist eine einzige Datei, kompiliert in sicherem Rust.
2. **Abschottung von Fehlern in Erweiterungen und externen Prozessen:**
   - Sprachserver (`tea-lsp`), Debugger (`tea-dap` / `gdb`), Aufrufe von `git` und Agenten laufen in getrennten Prozessen. Wenn `rust-analyzer` oder `gdb` hängen bleibt oder stirbt, friert das Fenster der IDE nicht ein.
   - Die internen Erweiterungen sprechen mit dem Kern nur über Nachrichten und Ereignisse (`Request` / `CoreEvent`), ohne veränderbare Verweise auf den Zustand der Anwendung zu teilen.
3. **Schutz vor verlorener Arbeit:**
   - Beim Schließen einer Registerkarte oder beim Beenden der IDE mit ungespeicherten Änderungen wird immer nach Bestätigung gefragt.
   - **Automatische Wiederherstellung nach Abstürzen (`backup.rs`):** Registerkarten mit ungespeicherten Änderungen werden unter `$XDG_STATE_HOME/idetea/backups/<Projekt>/` je PID geschrieben und zur Wiederherstellung angeboten, wenn das Fenster unerwartet schließt.
   - Dateien im Baum zu löschen schickt sie in den **Papierkorb des Systems** (kein unwiderrufliches `unlink`).
   - Externe Änderungen auf der Platte (`git checkout`, `cargo fmt`, Agenten, die Dateien bearbeiten) erkennt die Überwachung: Registerkarten ohne lokale Bearbeitung laden von selbst neu; Registerkarten mit ungespeicherter lokaler Bearbeitung werden **nie** still überschrieben.
   - Binärdateien werden mit einer klaren Warnung abgelehnt, statt den Puffer beim Speichern zu beschädigen. Dateien in anderen Kodierungen (UTF-16, Windows-1252 usw.) werden in ihrer eigenen Kodierung geöffnet und gespeichert; ein Zeichen, das in der Kodierung nicht Platz hat, wird beim Speichern abgelehnt und nie durch `?` ersetzt. Ohne BOM und ohne `charset` in `.editorconfig` fällt eine Shift_JIS-Datei noch auf Windows-1252 zurück (Standard von `[editor] fallback_encoding`) und muss über die Kodierung in der Statusleiste erneut geöffnet werden.
   - Das Protokoll `.idetea/` nutzt atomares Schreiben (`.tmp` + `rename`) und hat einen Befehl zur Selbstheilung (`idetea fix` / *„Interfaces: dieses Projekt reparieren“*).

### Wo die Stabilität von IDE TEA hinter den großen noch zurückbleibt
1. **Unter Linux hat sie die meisten Nutzungsstunden:** seit 0.5.0 gibt es offizielle Versionen für macOS (Apple Silicon) und Windows (x86_64), mit angepasstem Terminal (`conpty` unter Windows), Aufgaben, Debugger und Tastenkürzeln (`Cmd` gegen `Ctrl`), doch diese beiden Systeme haben noch deutlich weniger echte Nutzung hinter sich. Es gibt keine Version für macOS Intel oder Windows ARM, und der Windows-Installer ist noch nicht codesigniert (SmartScreen warnt beim ersten Start). VS Code, JetBrains und Zed laufen auf allen drei Systemen ausgereift.
2. **Grenzen des Ökosystems `iced` 0.14 für komplexe Anwendungen:**
   - Die IDE arbeitet in einem einzigen Fenster (`iced::application`). Ein Terminal, eine Vorschau oder einen Editor auf einen zweiten Monitor in ein weiteres natives Fenster des Betriebssystems zu ziehen, geht nicht.
   - Es gibt keine fortlaufende lokale Historie je Datei (*Local History* / *Timeline* über Tage ohne Git, wie bei JetBrains und VS Code).
3. **Syntaxhervorhebung auf Regex-Basis (`syntect`) gegenüber `tree-sitter`:**
   - IDE TEA hat zwar einen ausgezeichneten Leichtmodus für Dateien von 50 MB oder riesige minifizierte Zeilen, doch in normalen Codedateien mit Tausenden von Zeilen ist `syntect` (auf Grammatiken `.sublime-syntax` mit Regex) strukturell weniger widerstandsfähig gegen unvollständigen Code als das `tree-sitter`, das Zed, Helix und Neovim nutzen.

---

## 3. Ablauf mit KI-Agenten (wo IDE TEA einzigartig ist, und wo sie nicht mitspielt)

Das ist der tiefste Unterschied in der Produktphilosophie auf dem Markt heute.

### Wie Cursor, Windsurf und Zed KI behandeln
- Sie sind auf **das Modell im Editor** hin gedacht:
  - *Vorhersagende Vervollständigung (Tab / Next Edit Prediction)*, die die nächsten Zeilen rät, während Sie tippen.
  - *Inline Edit (`Ctrl+K`)* und *Composer/Agent Chat*, die farbige Diffs (grün/rot) direkt zwischen den Zeilen des Editors zeigen, um Hunk für Hunk anzunehmen oder abzulehnen.
  - Im Gegenzug binden sie Sie an Cloud-Konten, monatliche Abos oder HTTP-Clients von Anbietern in der IDE.

### Wie IDE TEA KI behandelt (und wo sie gewinnt)
IDE TEA geht davon aus, dass **die besten Agenten schon im Terminal laufen** (Claude Code, Codex, Gemini CLI, Aider, OpenCode) und löst das Problem, das keine andere IDE nativ und offen gelöst hat: **wie diese Agenten mit dem Menschen sprechen, ohne zu einer Textwand im Terminal zu werden**.

1. **Native deklarative Oberflächen (`.idetea/ui/` + `idetea ask`):**
   - Statt mitten in 200 Zeilen Protokoll im Terminal zu fragen, veröffentlicht der Agent ein geprüftes JSON, und die IDE zeichnet native Formulare in `iced`: Auswahlen, Kontrollkästchen, Schrittlisten für manuelle Tests (die Schaltfläche *Zustimmen* wird erst frei, wenn die Schritte erledigt sind), Bild- und Galerie-Vorschauen, animierte Spritesheets und WAV-Audio mit Wellenform.
   - **Keine andere IDE auf dem Markt (VS Code, Cursor, Zed, JetBrains) hat ein deklaratives, dateibasiertes Protokoll, mit dem Terminal-Agenten eine native Oberfläche mit prüfbarem Verlauf (`.idetea/history/`) zeichnen.**
2. **Steuerung und Sicherheit der Ausführung (`idetea run`):**
   - Wenn ein Agent ein interaktives Terminal öffnen oder eine Aufgabe abgeben will, zeigt die IDE den genauen Befehl in einer Registerkarte zur Zustimmung. Die Liste der zugelassenen Befehle (`[agents] allow`) wird nur aus der globalen Konfiguration des Benutzers gelesen (`~/.config/idetea/config.toml`) und **niemals** aus dem Repository. Ein bösartiges Repository kann sich damit keine Ausführungsrechte selbst geben.
3. **Abgabe in isolierte Git-Worktrees (`--worktree` + `$IDETEA_ANSWER`):**
   - Die IDE legt Worktrees nativ an und räumt sie auf, damit Unteragenten parallel arbeiten, ohne den Hauptbaum zu verschmutzen.
4. **Natives reaktives Maskottchen (Tea) + `idetea mascot`:**
   - Während Werkzeuge wie *Masko.ai* macOS und einen MCP-Server in der Cloud verlangen, damit ein Maskottchen auf Claude Code reagiert, hat IDE TEA **Tea** nativ eingebaut. Es beobachtet `.idetea/ui/`, `.idetea/agents/`, Aufgaben und LSP in Echtzeit, reagiert auf die Maus (`curious` / `annoyed`) und nimmt Pakete `mascot.toml` an.
5. **Agenten-Starter (`Ctrl+Shift+A`) und Kontext aus dem Editor (`Ctrl+L`):**
   - Der Starter öffnet den gewählten Agenten (Claude Code, Codex, Gemini CLI, OpenCode, Aider, Cursor Agent oder einen eigenen Starter) in einem Terminal-Tab, im Projektordner oder in einem neuen Worktree.
   - `Ctrl+L` im Editor fügt in diesen Tab die Cursorposition (oder die Auswahl) und die LSP-Probleme dieser Zeilen ein, ohne abzuschicken; ein Rechtsklick im Bereich **Probleme** schickt ein einzelnes Problem.
6. **Projektgedächtnis in Dateien (`docs/memory/`, `docs/decisions/`):** die Anweisungen, die die IDE für die Agenten schreibt, lehren ein gemeinsames Gedächtnis in Markdown unter Git (Fakten und ADRs), mit einem eigenen Bereich in der IDE.

### Wo IDE TEA im KI-Ablauf verliert
- **Keine vorhersagende Vervollständigung („Ghost Text“ / Tab):** wer sich auf `Tab` von Cursor/Copilot verlässt, um Boilerplate zu vervollständigen, während er von Hand tippt, findet in IDE TEA nichts Entsprechendes.
- **Keine Inline-Diff-Ansicht der KI im Editor:** wenn ein Agent im Terminal eine Datei bearbeitet, lädt die Datei in der Registerkarte von IDE TEA neu. Um zu sehen, was sich geändert hat, gehen Sie in den Bereich **Git**; es gibt keine schwebenden Schaltflächen *„Accept / Reject hunk“* im Code-Editor wie bei Cursor oder Windsurf.

---

## 4. Code-Editor: was da ist und was fehlt

### Was in IDE TEA schon solide ist
- Schnelles Öffnen von Dateien (`Ctrl+P` mit unscharfer Suche und Sprung `Datei:Zeile`), Symbole in der Datei (`Ctrl+Shift+O`) und im Projekt (`Ctrl+T`).
- Mehrfachcursor, Auswahl Wort für Wort (`Ctrl+Left`/`Ctrl+Right`), `Shift+Klick`, Codefaltung, Rückgängig/Wiederholen, Suche und Ersetzen in der Datei und **Ersetzen im ganzen Projekt (`Ctrl+Shift+H`)** mit Vorschau Zeile für Zeile und Regex.
- Projektsuche (`Ctrl+Shift+F`) mit Bereich auf Ordner oder Datei (Rechtsklick, mit einem Chip im Panel zum Aufheben), Include- und Exclude-Globs (`src/**`, `*.test.rs`), `F4` / `Shift+F4` zum nächsten und vorherigen Treffer auch aus dem Editor und Kontextzeilen (`[search] context_lines`, Standard 0).
- CSV, TSV und JSONL öffnen sich als Tabelle (sortieren und filtern, ohne die Datei anzufassen; eine Zelle bearbeiten ändert im Text nur dieses Feld, `Ctrl+Z` macht es rückgängig). Die Schaltfläche in der Tab-Leiste führt zurück zum Text.
- Vorübergehender Zoom des Editors (`Ctrl+=`, `Ctrl+-`, `Ctrl+0`, `Ctrl+Scroll`), abgestimmt zwischen Registerkarten und Teilungen, ohne den Rest der Oberfläche zu verändern.
- Teilung des Editors nebeneinander, klickbare Brotkrümel (`Projekt › Ordner › Datei › Symbol`), Seti-Symbole je Sprache und anpassbare Schriften mit Ligaturen (`JetBrains Mono` mitgeliefert).
- Optionaler Vim-Modus.
- Native Registerkarten für Bilder (mit Zoom und Ziehen), PDF (in reinem Rust über `hayro` gerendert) und eine Vorschau von Markdown und SVG nebeneinander in Echtzeit.

### Was im Editor gegenüber VS Code, Zed und JetBrains fehlt
1. **Snippets:** es gibt keine Snippet-Maschine (Tabstopps `$1`, `$2`), um kurze Codeblöcke aufzuklappen.
2. **Strukturelle Auswahl nach AST (Auswahl erweitern/verkleinern):** in Zed, Helix und JetBrains erweitert ein Tastenkürzel die Auswahl vom Wort zur Ausdrucksebene, zum Block `if` und zur ganzen Funktion, über `tree-sitter`/AST.
3. ~~Alte Kodierungen~~ — **erledigt:** Dateien in Windows-1252, ISO-8859-*, UTF-16, Shift_JIS, GBK und anderen öffnen und speichern in ihrer Kodierung; die Statusleiste zeigt sie und öffnet neu oder speichert mit einer anderen.
4. **Mehrere Fenster / mehrere Projekte in derselben Instanz:** jedes Projekt ist ein eigener Prozess `idetea`; es gibt im selben Fenster keine Multi-Root-Arbeitsbereiche.

---

## 5. Sprachintelligenz (LSP) und Refaktorierung

### Was IDE TEA schon sehr gut liefert
- Diagnosen in Echtzeit mit **Wellenunterstreichung im Text des Editors**, Fehler-Tooltip beim Hover, Zähler in der Statusleiste und Bereich **Probleme** (`Ctrl+Shift+M` / `F8`).
- Vervollständigen, Parametersignatur, Gehe zu Definition (`F12`), Gehe zum Typ und Gehe zu Implementierung (`Ctrl+F12`).
- Verweise suchen (`Shift+F12` oder `Ctrl+Klick` auf die Deklaration), nach Datei gruppiert, mit dem Navigationsverlauf Zurück/Vorwärts (`Alt+←` / `Alt+→` und die Seitentasten der Maus).
- **Aufrufhierarchie (`Alt+Shift+H`):** wer die Funktion aufruft und was sie aufruft.
- **Bereich Struktur / Gliederung (`Alt+7`):** Symbolbaum der Datei mit Sichtbarkeit (`pub`, `pub(crate)`, privat) und Hervorhebung des Symbols unter dem Cursor.
- **Symbol umbenennen (`F2`)** im ganzen Projekt, mit einem Dialog, der vorher zeigt, wie viele Dateien und Änderungen betroffen sind.
- Formatieren beim Speichern und Respekt vor `.editorconfig`.

### Wichtige LSP-Lücken gegenüber VS Code, Zed und JetBrains
1. ~~Keine Code Actions / Quick Fixes (`Ctrl+.` / die Glühbirne des LSP)~~ — **erledigt:** `Ctrl+.` listet die Korrekturen und Refactorings des Servers für den Fehler oder die Auswahl (fehlender Import, Funktion erzeugen, Variable extrahieren).
2. ~~Keine Inlay Hints~~ — **erledigt:** Der abgeleitete Typ von Variablen und die Parameternamen erscheinen grau im Code; `Ctrl+Alt+I` schaltet sie ein und aus.
3. ~~Keine Semantic Tokens des LSP~~ — **erledigt:** Parameter, Felder und Funktionsaufrufe bekommen die Farbe des Themes über der Grammatik, standardmäßig an (`[lsp] semantic_tokens`). Mit dem Standardtheme ist der Unterschied bei Aufrufen gering, und veränderliche Variablen ändern ihre Farbe nur in einem Theme mit eigener Farbe dafür.

---

## 6. Debuggen

### Was IDE TEA heute tut
- Native Anbindung an `gdb` 14+ (oder `lldb-dap` unter macOS), über DAP: Haltepunkte per Klick auf die Zeilennummer (`F9`), die dem Text folgen, wenn Sie die Datei bearbeiten, `F5` (kompiliert und startet automatisch in Projekten in Rust, Zig und, experimentell, MoonBit), nächste Zeile (`F10`), in die Funktion springen (`F11`), aus der Funktion austreten (`Shift+F11`), anhalten und beenden.
- Bereich mit Aufrufliste (ein Klick führt zur Zeile des Frames), inspizierbarem Baum der lokalen Variablen und Ausgabe des Programms.

### Wo sie gegenüber VS Code und JetBrains verliert
- **Gebunden an GDB 14+ (Rust, C, C++, Zig und, experimentell, MoonBit) und an `lldb-dap` unter macOS:** noch keine DAP-Adapter für Python (`debugpy`), Go (`delve`), JavaScript/TypeScript (`js-debug`) oder `codelldb`. Hare hat in der IDE keinen Debugger.
- **Fortgeschrittene Debug-Funktionen fehlen:** keine bedingten Haltepunkte (*„nur anhalten, wenn `i == 42`“*), keine Logpoints, kein Fenster für beliebige *Watch Expressions* (REPL am Haltepunkt) und keine Variablenwerte, die inline neben dem Code im Editor gezeichnet werden.

---

## 7. Git und Versionsverwaltung

### Was IDE TEA heute tut
- Statusleiste mit aktuellem Branch, Commits voraus/zurück (`ahead`/`behind`) und der Gesamtzahl geänderter Dateien.
- Git-Bereich mit Stagen und Staging aufheben je Datei oder insgesamt, farbiger Diff, Feld für die Commit-Nachricht, Änderungen verwerfen mit Bestätigung und Schaltflächen für `push`, `pull` und `fetch` mit dem `git` und den Zugangsdaten/SSH des Systems.
- Git-Statusfarben im Dateibaum.
- Markierungen am Rand des Editors bei Zeilen, die seit dem letzten Commit hinzugekommen, geändert oder gelöscht sind; ein Klick auf die Markierung zeigt den alten Abschnitt und kann ihn rückgängig machen.
- Autor und Commit der Cursorzeile (Blame) in der Statusleiste, mit Details und Kopieren des Hashs.
- Branch wechseln und anlegen über die Palette.

### Wo sie gegenüber VS Code, Zed und JetBrains verliert
1. **Kein visueller Konfliktauflöser (3-Wege-Merge):** bei einem Konflikt lösen Sie auf, indem Sie die Markierungen `<<<<<<<` im Text bearbeiten oder das Terminal nutzen.
2. **Kein Stash über die Oberfläche:** für `git stash` nutzen Sie das integrierte Terminal.
3. **Blame nur für die Cursorzeile:** er steht in der Statusleiste; es gibt keine Anmerkung neben jeder Zeile und keine Blame-Ansicht der ganzen Datei.

---

## 8. Ehrliche Zusammenfassung: wann Sie IDE TEA heute wählen (und wann nicht)

### IDE TEA ist schon besser oder einzigartig, wenn Sie:
1. **Neben Terminal-Agenten programmieren (Claude Code, Codex, Gemini CLI und andere), unter Linux, macOS oder Windows,** und wollen, dass der Agent strukturierte Fragen stellt, Pläne und manuelle Tests zeigt, ausdrücklich um Erlaubnis bittet, bevor er Befehle ausführt, isolierte Worktrees nutzt und ein reaktives Maskottchen hat (**Tea**), ohne Cloud oder Abo.
2. **Geschwindigkeit, Vorhersehbarkeit und wenig sensorischen Lärm voranstellen:** öffnet in unter 200 ms, verbraucht wenig Speicher, hat kein Konto, keine Telemetrie, ändert die Oberfläche nicht von selbst und respektiert schlichte TOML-Dateien.
3. **Mit Projekten in Rust, Zig, C/C++, Python, Go, Hare, MoonBit oder dem Web arbeiten**, bei denen der Hauptfluss ist: schnell bearbeiten, über Symbole und Verweise des LSP navigieren, Aufgaben und Tests im integrierten Terminal laufen lassen und Commits prüfen.

### Eine andere IDE ist noch besser, wenn Sie:
1. **Vorhersagende KI-Vervollständigung bei jedem Tastendruck wollen (`Tab` / Ghost Text) oder Bearbeitung inline per Diff:** **Cursor**, **Windsurf** oder **Zed** bedienen diesen Stil besser.
2. **Stark auf Debugger für Python/JS/Go in der Oberfläche angewiesen sind:** **VS Code** oder **JetBrains** sind heute deutlich vollständiger.
3. **SSH Remote Development / DevContainers brauchen oder macOS Intel / Windows ARM nutzen:** **VS Code** und **Zed** haben für diese Fälle schon eine ausgereifte Unterstützung.

---

## 9. Erledigt und noch offen (an die Spitze heranrücken, ohne die Seele von IDE TEA zu verlieren)

Dieser Abschnitt nannte einmal fünf nächste Schritte; vier sind erledigt. Was erledigt ist und was offen bleibt, bei schlankem Programm und der Philosophie des Manifests:

**Schon erledigt**

1. ~~Code Actions / Quick Fixes des LSP (`Ctrl+.`)~~ — **erledigt:** die Liste öffnet sich in der Palette, die empfohlene zuerst, und die Änderungen bleiben ungespeichert in den Tabs.
2. ~~Git-Markierungen im Editor (Gutter-Diff) und Branchwechsel~~ — **schon erledigt:** Randmarkierungen sowie Branch wechseln und anlegen über die Palette.
3. ~~Tastenkürzel „Auswahl / Diagnose an den Agenten schicken“ (`Ctrl+L`)~~ — **in 0.8.0 erledigt**, zusammen mit dem Agenten-Starter (`Ctrl+Shift+A`).
4. ~~Inlay Hints des LSP~~ — **erledigt:** abgeleitete Typen und Parameternamen im Code, standardmäßig an, mit `Ctrl+Alt+I` zum Ein- und Ausschalten.

**Noch offen**

1. **Weitere DAP-Adapter (`debugpy`, `delve`, `js-debug`, `codelldb`):** Debuggen jenseits von GDB und `lldb-dap` unterstützen. Die Übertragbarkeit auf macOS und Windows, die auch zu diesem Punkt gehörte, kam mit 0.5.0.
2. **3-Wege-Merge-Ansicht und Stash über die Oberfläche** (heute über das Terminal; siehe Abschnitt 7).
3. **Plugins von Dritten in WASM**, ohne die IDE neu zu bauen (ein Prototyp existiert hinter dem Feature `plugins`).
