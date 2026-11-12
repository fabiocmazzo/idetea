## Einstellungen → Darstellung

settings-theme = Design
settings-theme-detail = Farben der IDE, des Editors und des Terminals.
settings-font-size = Schriftgröße
settings-font-size-detail = Editor und Terminal.
settings-code-font = Schriftart für Code
settings-code-font-detail = Editor, Zeilennummern und Terminal. JetBrains Mono liegt der IDE bei; die Liste enthält die installierten nichtproportionalen Schriftarten.
settings-font-weight = Schriftstärke
settings-font-weight-detail = Eine Stärke, die die Schriftart nicht hat, wird durch die nächstliegende ersetzt.
settings-line-height = Zeilenhöhe
settings-line-height-detail = In Vielfachen der Schriftgröße. Das Terminal folgt im selben Verhältnis.
settings-file-icons = Dateisymbole
settings-file-icons-detail = Ein Symbol je Sprache oder Dateityp (der Seti-Satz aus VS Code). Aus: dasselbe Symbol für jede Datei.
settings-activity-bar = Aktivitätsleiste
settings-activity-bar-detail = Leiste am linken Rand, um Bereiche mit einem Klick ein- und auszublenden.
settings-splash = Startbildschirm beim Start
settings-splash-detail = Startbildschirm (mindestens 2 s). --no-splash schaltet ihn für einen Start aus.
settings-language = Sprache
settings-language-detail = Oberflächentext. Automatisch folgt der Systemsprache. Tritt nach einem Neustart der IDE in Kraft.
settings-language-auto = Automatisch ({ $language })
settings-language-restart = Tritt nach einem Neustart der IDE in Kraft (jetzt: { $language }).

## Kernbefehle (Titel in der Befehlspalette und in der Hilfe)
cmd-palette-open = Befehlspalette
cmd-help-open = Hilfe: Tastenkürzel und Anleitung
cmd-settings-open = Einstellungen
cmd-file-new = Datei: neu
cmd-file-save = Datei: speichern
cmd-file-open = Datei: öffnen…
cmd-file-quick-open = Datei: nach Name öffnen…
cmd-debug-continue = Debuggen: starten / fortsetzen
cmd-debug-stop = Debuggen: beenden
cmd-debug-pause = Debuggen: anhalten
cmd-debug-next = Debuggen: nächste Zeile
cmd-debug-step-in = Debuggen: in Funktion springen
cmd-debug-step-out = Debuggen: aus Funktion austreten
cmd-debug-toggle-breakpoint = Debuggen: Haltepunkt in der Zeile umschalten
cmd-debug-choose-program = Debuggen: Programm auswählen…
cmd-file-open-folder = Datei: Ordner in neuem Fenster öffnen…
cmd-file-save-as = Datei: speichern unter…
cmd-file-save-as-path = Datei: speichern unter (Pfad eingeben)…
cmd-file-close = Datei: Registerkarte schließen
cmd-tab-next = Registerkarten: nächste
cmd-tab-prev = Registerkarten: vorherige
cmd-tab-reopen = Registerkarten: zuletzt geschlossene wieder öffnen
cmd-editor-find = Editor: suchen
cmd-editor-replace = Editor: suchen und ersetzen
cmd-editor-goto-line = Editor: gehe zu Zeile
cmd-editor-undo = Editor: rückgängig
cmd-editor-redo = Editor: wiederholen
cmd-editor-select-all = Editor: alles auswählen
cmd-editor-fold-all = Editor: alle Blöcke falten
cmd-editor-unfold-all = Editor: alle Blöcke auffalten
cmd-editor-toggle-fold = Editor: Block am Cursor falten oder auffalten
cmd-editor-toggle-vim = Editor: Vim-Modus umschalten
cmd-editor-zoom-in = Editor: vergrößern
cmd-editor-zoom-out = Editor: verkleinern
cmd-editor-zoom-reset = Editor: Zoom zurücksetzen
cmd-lsp-definition = LSP: gehe zu Definition
cmd-lsp-references = LSP: Verweise suchen
cmd-lsp-implementation = LSP: gehe zu Implementierung
cmd-lsp-type-definition = LSP: gehe zu Typdefinition
cmd-lsp-incoming-calls = LSP: wer diese Funktion aufruft
cmd-lsp-outgoing-calls = LSP: was diese Funktion aufruft
cmd-nav-file-symbol = Gehe zu Symbol in Datei…
cmd-nav-project-symbol = Gehe zu Symbol im Projekt…
cmd-nav-back = Navigation: zurück
cmd-nav-forward = Navigation: vorwärts
cmd-lsp-hover = LSP: Dokumentation am Cursor anzeigen
cmd-lsp-format = LSP: Datei formatieren
cmd-lsp-restart = LSP: Server neu starten
cmd-theme-toggle = Design: hell/dunkel umschalten
cmd-config-open = Konfiguration: globale Konfiguration öffnen
cmd-config-open-project = Konfiguration: Projektkonfiguration öffnen
cmd-config-reload = Konfiguration: neu laden
cmd-view-toggle-left = Layout: linke Seite ein-/ausblenden
cmd-view-toggle-right = Layout: rechte Seite ein-/ausblenden
cmd-view-toggle-bottom = Layout: unteren Bereich ein-/ausblenden
cmd-app-restart = IDE neu starten
cmd-app-quit = Beenden
cmd-mascot-toggle = Maskottchen: in der Ecke anzeigen/einklappen

## Hinweise, Dialoge und Menüs
file-new-created-on-save = Neue Datei: { $path } (wird beim Speichern erzeugt)
file-open-error = Fehler beim Öffnen von { $path }: { $error }
file-changed-on-disk-kept = "{ $name }" wurde auf dem Datenträger geändert, die Registerkarte hat aber Änderungen: sie wurde behalten
file-reload-error = Fehler beim Neuladen von { $path }: { $error }
file-changed-on-disk-overwrite = "{ $name }" wurde auf dem Datenträger geändert, die Registerkarte hat aber Änderungen: Speichern überschreibt sie
debug-still-building = Debuggen: wird noch erstellt…
debug-already-running = Debuggen: das Programm läuft bereits
debug-building = Debuggen: wird erstellt ({ $command })…
unknown-error = unbekannter Fehler
debug-build-failed = Debuggen: der Build ist fehlgeschlagen: { $error }
debug-choose-title = Welches Programm debuggen?
debug-choose-message = Das Projekt hat mehr als ein ausführbares Programm erzeugt.
cancel = Abbrechen
debug-running = Debuggen: { $program } läuft
debug-error = Debuggen: { $error }
debug-cancelled = Debuggen: abgebrochen (der Build endet von selbst)
debug-stopped = Debuggen: angehalten
debug-no-source = Debuggen: { $name } hat keinen Quellcode
debug-paused-no-source = Debuggen: angehalten in { $name } ({ $reason }), kein Quellcode
debug-paused-at = Debuggen: angehalten bei { $path }:{ $line } ({ $reason })
debug-exited = Debuggen: das Programm wurde beendet
debug-exited-code = Debuggen: das Programm wurde beendet (Code { $code })
debug-command-failed = Debuggen: { $command } fehlgeschlagen: { $message }
debug-gdb-exited = Debuggen: gdb wurde beendet ({ $reason })
debug-no-session = Debuggen: keine Sitzung aktiv (F5 startet eine)
debug-already-paused = Debuggen: das Programm ist bereits angehalten
debug-pause-first = Debuggen: das Programm läuft (halten Sie es zuerst an)
nav-nothing-forward = Nichts, wohin es vorwärts geht
nav-nothing-back = Nichts, wohin es zurückgeht
lsp-no-references = Keine Verweise gefunden
lsp-no-definition = Definition nicht gefunden
lsp-no-implementations = Keine Implementierungen gefunden
lsp-no-type = Typ nicht gefunden
lsp-not-used-elsewhere = "{ $symbol }" wird sonst nirgends verwendet
outline-save-first = Speichern Sie die Datei, um ihre Struktur zu sehen
outline-reading = Struktur wird gelesen…
symbols-type-a-name = Namen einer Funktion, eines Typs, einer Methode eingeben…
searching = Suche läuft…
symbols-none-found = Keine Symbole gefunden (wenn das Projekt noch lädt, erneut versuchen)
symbols-none-in-file = Keine Symbole in dieser Datei
lsp-save-first = speichern Sie die Datei, um den Sprachserver zu nutzen
lsp-finding-references = Verweise werden gesucht…
lsp-finding-implementations = Implementierungen werden gesucht…
calls-not-in-function = Der Cursor steht nicht in einer Funktion (oder der Server kennt ihre Aufrufe nicht)
lsp-formatted = Datei formatiert
lsp-level-error = Fehler:{" "}
lsp-level-warning = Warnung:{" "}
lsp-server-stopped = { $server } angehalten — { $reason }. Verwenden Sie "LSP: Server neu starten".
lsp-server-crashed = { $server } abgestürzt — { $reason }. Er kommt von selbst zurück, wenn Sie aufhören zu tippen.
lsp-server-crashed-too-often = { $server } ist { $count }-mal in { $minutes } Minuten abgestürzt — { $reason }. Verwenden Sie "LSP: Server neu starten".
lsp-no-formatting = Formatieren ist für diese Datei nicht verfügbar
lsp-restarted = Sprachserver neu gestartet
save-changes-title = Änderungen speichern?
save-changes-message = "{ $name }" hat ungespeicherte Änderungen.
save = Speichern
discard = Verwerfen
picker-already-open = Ein Dateidialog ist bereits offen
save-tab-closed = Die Registerkarte wurde vor dem Speichern geschlossen
folder-opening = { $path } wird in einem neuen Fenster geöffnet
folder-open-error = Ordner konnte nicht geöffnet werden: { $error }
saved = Gespeichert: { $path }
save-error = Fehler beim Speichern: { $error }
file-deleted-kept = Datei auf dem Datenträger gelöscht; die Registerkarte mit Änderungen wurde behalten
and-more = … und { $count } weitere
quit-unsaved-title = Mit ungespeicherten Änderungen beenden?
quit-save-all = Alles speichern und beenden
quit-without-saving = Ohne Speichern beenden
save-needs-name = "{ $name }" hat keinen Namen: verwenden Sie "Speichern unter"
save-file-error = Fehler beim Speichern von { $path }: { $error }
close-tabs-title = 
    { $count ->
        [one] { $count } Registerkarte schließen?
       *[other] { $count } Registerkarten schließen?
    }
close-tabs-unsaved = Mit ungespeicherten Änderungen:
save-and-close = Speichern und schließen
discard-changes = Änderungen verwerfen
tab-close = Schließen
tab-close-others = Andere schließen
tab-close-right = Rechts davon schließen
tab-close-saved = Gespeicherte schließen
tab-close-all = Alle schließen
copy-path = Pfad kopieren
copy-relative-path = Relativen Pfad kopieren
reveal-in-tree = Im Dateibaum anzeigen
open-in-file-manager = Im Dateimanager öffnen
copied = Kopiert: { $text }
file-manager-error = Dateimanager konnte nicht geöffnet werden: { $error }
save-then-close = Wählen Sie, wo gespeichert wird; schließen Sie die Registerkarte danach erneut
config-reloaded-problems = Konfiguration mit Problemen neu geladen: { $problems }
config-reloaded = Konfiguration neu geladen (Erweiterungen ein- oder auszuschalten wirkt erst nach einem Neustart)
config-no-home = Konfigurationsordner nicht gefunden (HOME ist nicht gesetzt)
create-error = Fehler beim Erstellen von { $path }: { $error }
config-error = Konfiguration: { $error }
write-error = Fehler beim Schreiben von { $path }: { $error }
unknown-command = Unbekannter Befehl: { $id }
keys-record-hint = Verwenden Sie Strg, Alt oder eine Sondertaste (F1–F12, Bild-auf …). Esc bricht ab.
keys-already-bound = { $key } war bereits "{ $command }"
keys-rebound = { $key } war "{ $old }"; jetzt ist es "{ $command }"
keys-bound = { $key } → "{ $command }"
theme-unknown = unbekanntes Design "{ $name }", es wird "dark" verwendet
default = Standard

## Einstellungen
settings-title = Einstellungen
settings-tab-appearance = Darstellung
settings-tab-extensions = Erweiterungen
settings-tab-shortcuts = Tastenkürzel
close-esc = Schließen (Esc)
settings-no-home = (HOME ist nicht gesetzt)
settings-write-to = Speichern nach
settings-target-global = Global
settings-target-project = Projekt
settings-open-file = Datei öffnen
keys-filter = Nach Name, Befehl oder Taste filtern…
keys-press = Tasten drücken… (Esc bricht ab)
keys-add = Tastenkürzel hinzufügen
keys-help = Klicken Sie eine Taste an, um sie zu entfernen. + zeichnet eine neue Taste auf (gilt auch für schon belegte Tasten: sie wechseln den Besitzer).
settings-lsp = Sprachserver
settings-lsp-detail = Schaltet LSP als Ganzes ein oder aus.
settings-servers = Server
settings-server-found = in PATH gefunden
settings-server-not-found = in PATH nicht gefunden
settings-lsp-note = Änderungen gelten für Server, die von jetzt an starten.
settings-restart-servers = Server neu starten
theme-dark = Dunkel (Standard)
theme-light = Hell
slot-left = Links
slot-right = Rechts
slot-bottom = Unten
slot-status = Statusleiste
settings-on-restart = wird nach einem Neustart der IDE wirksam
default-capitalized = Standard
settings-extensions-note = Eine Erweiterung ein- oder auszuschalten wirkt nach einem Neustart. Einen Bereich auf eine andere Seite zu verschieben gilt sofort.
ext-filetree = Dateibaum
ext-statusbar = Statusleiste
ext-search = Projektsuche
ext-problems = Fehler und Warnungen
ext-terminal = Integriertes Terminal
ext-git = Änderungen, Stagen, Commit und Diff
ext-debug = Debug-Schaltflächen, Aufrufliste, Variablen und Ausgabe
ext-references = Wo ein Name verwendet wird (Shift+F12)
ext-outline = Funktionen und Methoden der Datei, mit Sichtbarkeit (Alt+7)
ext-preview = Markdown- und SVG-Vorschau neben dem Editor (Ctrl+Shift+V)
ext-calls = Wer eine Funktion aufruft und was sie aufruft (Alt+Shift+H)
ext-interfaces = Oberflächen, die ein Agent nach .idetea/ui/ schreibt (experimentell)

## Befehlspalette, Dialoge, LSP, Debuggen und der Rest der App
palette-commands = Befehl eingeben…
palette-save-as = Dateipfad (relativ zum Projektstamm)
palette-files = Dateiname (datei:zeile springt zur Zeile, > für Befehle)
palette-file-symbols = Funktion, Typ, Methode … in dieser Datei
palette-project-symbols = Funktion, Typ, Methode … im Projekt
palette-no-commands = Keine Befehle gefunden
palette-save-hint = Eingabe speichert · Esc bricht ab
palette-reading-files = Projektdateien werden gelesen…
palette-no-files = Keine Dateien gefunden
palette-no-symbols = Keine Symbole
palette-no-matching-symbols = Keine Symbole passen zur Suche
dialog-hint = Eingabe bestätigt · Esc bricht ab
lsp-command-not-found = { $server }: Befehl "{ $command }" nicht in PATH gefunden (installieren oder [lsp.servers.{ $server }] in der Konfiguration anpassen)
lsp-start-error = { $server }: konnte nicht gestartet werden: { $error }
lsp-disabled = LSP ist in der Konfiguration aus ([lsp] enabled = false)
lsp-no-server-for = kein Sprachserver für Dateien .{ $ext } konfiguriert
lsp-not-running = { $server } läuft nicht: { $reason }
lsp-not-ready = { $server } ist noch nicht bereit
lsp-hint-rustup-install = { $reason }. Installieren mit: rustup component add rust-analyzer
lsp-hint-rustup = { $reason }. Mit rustup: rustup component add rust-analyzer
lsp-hint-ziex = { $reason }. Installieren Sie die Ziex-CLI (https://ziex.dev/learn) oder öffnen Sie den Projektordner, dessen build.zig.zon von ziex abhängt: dann baut die IDE den Server mit zig
lsp-hint-moonbit = { $reason }. Installieren Sie die MoonBit-Werkzeuge (https://www.moonbitlang.com/download), die `moon lsp` enthalten; die IDE sucht auch in ~/.moon/bin (oder $MOON_HOME/bin)
lsp-hint-hare = { $reason }. hare-lsp liegt Hare nicht bei: bauen Sie es aus https://git.sr.ht/~whynothugo/hare-lsp und legen Sie es in den PATH (zum Beispiel ~/.local/bin)
config-syntax-error = Syntax { $file }: { $error }
visibility-file-only = nur diese Datei
picker-save-as = Speichern unter
picker-open-files = Dateien öffnen
picker-open-folder = Ordner öffnen
picker-debug-program = Programm zum Debuggen
splash-ready = bereit
splash-loading = wird geladen…
watch-too-many = Ordner zu groß: nur die ersten { $count } Ordner werden überwacht
watch-partial = nicht alle Ordner konnten überwacht werden ({ $error })
watch-problem = Änderungen auf dem Datenträger: { $message }. Verwenden Sie bei Bedarf "Aktualisieren" im Dateibaum.
watch-failed = Änderungen auf dem Datenträger: Überwachen nicht möglich ({ $error })
debug-no-gdb = gdb nicht in PATH gefunden (das Debuggen verwendet gdb 14 oder neuer)
debug-program-not-found = Programm nicht gefunden: { $path }
debug-gdb-start-error = gdb konnte nicht gestartet werden: { $error }
debug-reason-breakpoint = Haltepunkt
debug-reason-step = Schritt
debug-reason-pause = angehalten
debug-reason-exception = Ausnahme
debug-reason-entry = Eintritt
debug-reason-goto = Sprung
build-run-error = "{ $command }" konnte nicht ausgeführt werden: { $error }
build-no-message = fehlgeschlagen ohne Meldung

## Hilfe und Befehlszeile
help-key-find = In Datei suchen
help-key-replace = Suchen und ersetzen
help-key-goto-line = Gehe zu Zeile
help-key-undo = Rückgängig / wiederholen
help-key-next-match = Nächstes Vorkommen auswählen (Mehrfachcursor)
help-key-quick-fix = LSP-Schnellaktionen (wenn verfügbar)
help-key-ctrl-click = Ctrl+Klick
help-key-definition = Gehe zu Definition (LSP)
help-key-tab = Tabulator einfügen
help-title = Hilfe
help-tab-shortcuts = Tastenkürzel
help-tab-guide = Anleitung
help-tab-about = Info
help-close = Schließen (Esc)
help-filter = Nach Name, Befehl oder Taste filtern…
help-group-general = Allgemein
help-editor-keys = Editor (fest)
help-in-palette = (Befehlspalette)
about-no-home = (HOME nicht gesetzt)
about-version = Version
about-license = Lizenz
about-license-value = { $license } (frei und offen, immer)
about-project = Projekt
about-global-config = Globale Konfiguration
about-project-config = Projektkonfiguration
about-built-with = In Rust gebaut mit iced, iced-code-editor, alacritty_terminal (über iced_term) und syntect.
about-icons = Symbole: Codicons © Microsoft, Lizenz CC BY 4.0.
cli-usage = 
    Aufruf: idetea [Optionen] [Pfad...]
    
      Pfad           zu öffnende Datei (darf noch fehlen: wird beim Speichern erzeugt)
                     oder Ordner als Projektstamm
      --no-splash    Startbildschirm nicht anzeigen (auch: IDETEA_NO_SPLASH=1)
      --menu         aus dem Menü geöffnet: ohne Ordner öffnet sich das Projekt der Datei oder das letzte Projekt
      -f, --foreground  am Terminal angehängt bleiben (sonst löst sich die IDE, aus einem Terminal gestartet,
                     und die Protokolle gehen nach ~/.local/state/idetea/idetea.log)
      --install-desktop    fügt die IDE dem Anwendungsmenü hinzu (mit Symbol)
      --uninstall-desktop  entfernt sie aus dem Anwendungsmenü
      ui check <file.json>…  prüft Dokumente des Bereichs Interfaces (so, wie der Bereich sie liest)
      new <Ordner> [--modelo id]  erzeugt ein neues Projekt: Ordner, git, die Skills der Vorlage und die Anleitung
      skills [list|add|update|remove]  installiert Agent-Skills im Projekt (idetea skills --help)
      ui fix [--apply]  zeigt (und repariert mit --apply), was in .idetea/ui/ liegen blieb
      ask …          fragt im Bereich Interfaces und wartet auf die Antwort (idetea ask allein: Aufruf)
      run … -- <Befehl>  bittet die IDE um ein Terminal und wartet auf das Ende (idetea run allein: Aufruf)
      status [--json]  was die in diesem Projekt geöffnete IDE zeigt (Registerkarten, Änderungen, Cursor)
      open <Datei[:Zeile[:Spalte]]>  die geöffnete IDE zeigt die Datei, mit dem Cursor an der Stelle
      history [--json]  was Sie im Bereich Interfaces geantwortet haben (--name, --since, --last N, --all)
      mascot <Zustand> [--text "…"]  aktualisiert das IDE-Maskottchen (idle, working, thinking, attention, celebrating, error, sleeping, clear, status)
      -h, --help     diese Hilfe anzeigen
      -V, --version  die Version anzeigen
cli-unknown-option = unbekannte Option ignoriert: { $option }
cli-extra-root = nur ein Stammordner auf einmal, { $path } wird ignoriert
## Hilfe → Anleitung (`{ $befehl-mit-bindestrichen }` wird zum Tastenkürzel des Befehls)
guide-start = Erste Schritte
guide-start-1 = Öffnen Sie die IDE mit `idetea` in einem Ordner (er wird zum Stamm des Baums) oder mit `idetea datei...` (die Dateien öffnen sich in Registerkarten; der Stamm bleibt der aktuelle Ordner).
guide-start-2 = Jeder Befehl liegt in der Befehlspalette: { $palette-open }. Tippen Sie einen Teil des Namens; das Tastenkürzel steht daneben.
guide-start-3 = Diese Hilfe öffnet sich mit { $help-open }.
guide-start-4 = Die Symbolleiste am linken Rand blendet die Bereiche mit einem Klick ein und aus (der offene ist hervorgehoben). Ziehen Sie den Rand zwischen einem Bereich und dem Editor, um die Größe zu ändern; ein Doppelklick stellt den Standard wieder her.
guide-files = Dateien und Registerkarten
guide-files-1 = Nach Name öffnen: { $file-quick_open } (Teile des Namens tippen; `main.rs:42` springt zur Zeile; `>` wechselt zu den Befehlen).
guide-files-2 = Zu einem Symbol (Funktion, Typ, Methode …): { $nav-file_symbol } in der Datei, { $nav-project_symbol } im ganzen Projekt. Der Sprachserver wird gebraucht.
guide-files-3 = Neu: { $file-new } · Öffnen: { $file-open } · Speichern: { $file-save } · Speichern unter: { $file-save_as } · Registerkarte schließen: { $file-close }.
guide-files-4 = Öffnen und Speichern unter nutzen den Dateidialog des Systems. Um den Pfad zu tippen: { $file-save_as_path }. Einen anderen Ordner öffnen ({ $file-open_folder }) öffnet ein neues IDE-Fenster.
guide-files-5 = Markdown- und SVG-Vorschau: { $preview-toggle } (oder die Schaltfläche am Ende der Registerkartenleiste) öffnet einen Seitenbereich, der der aktiven Datei folgt und sich beim Tippen aktualisiert. Weblinks öffnen sich im Browser; Links auf Dateien öffnen sich in einer Registerkarte.
guide-files-6 = Bilder (png, jpg, gif, webp …) öffnen sich in einer eigenen Registerkarte: das Mausrad ändert den Zoom, Ziehen verschiebt.
guide-files-7 = PDFs öffnen sich ebenfalls in einer Registerkarte: durch die Seiten blättern; − und + ändern den Zoom und "Anpassen" folgt der Breite.
guide-files-8 = Registerkarten wechseln: { $tab-next } und { $tab-prev } (Ctrl+Tab fügt im Editor einen Tabulator ein).
guide-files-9 = Beim Schließen einer Registerkarte oder beim Beenden mit ungespeicherten Änderungen wird zuerst gefragt: Speichern, Verwerfen oder Abbrechen.
guide-files-10 = Datei außerhalb geändert (Terminal, anderer Editor): die Registerkarte lädt sich selbst neu, wenn sie keine ungespeicherten Änderungen hat; sonst bleibt sie, wie sie ist, und ein Hinweis erscheint.
guide-files-11 = Rechtsklick auf eine Registerkarte: andere schließen, rechts davon, gespeicherte oder alle, Pfad kopieren, im Baum anzeigen und im Dateimanager öffnen. Ein Mittelklick schließt die Registerkarte.
guide-tree = Dateibaum
guide-tree-1 = Ein-/ausblenden: { $filetree-toggle } · Die aktuelle Datei anzeigen: { $filetree-reveal }.
guide-tree-2 = Rechtsklick auf ein Element: neue Datei, neuer Ordner, umbenennen, löschen, Pfad kopieren und im Dateimanager öffnen. Löschen verschiebt in den Papierkorb des Systems.
guide-tree-3 = Git-Farben: gelb = geändert, grün = neu, rot = Konflikt, abgeblendet = ignoriert. Ein Ordner mit Änderungen darin zeigt einen Punkt.
guide-bottom = Unterer Bereich
guide-bottom-1 = Projektsuche ({ $search-focus }), Probleme ({ $problems-focus }), Terminal ({ $terminal-focus }) und Git ({ $git-focus }) sind Registerkarten im unteren Bereich. { $view-toggle_bottom } blendet den Bereich ein und aus.
guide-bottom-2 = Nächstes Problem: { $problems-next }.
guide-terminal = Terminal
guide-terminal-1 = Liegt der Fokus auf dem Terminal, gehen Strg+Buchstabe, Funktionstasten und Pfeile an die Shell (Ctrl+C, Ctrl+W …). IDE-Tastenkürzel mit Ctrl+Shift funktionieren weiter, die Befehlspalette öffnet also immer.
guide-terminal-2 = Kopieren und Einfügen im Terminal: Ctrl+Shift+C und Ctrl+Shift+V. `exit` tippen schließt den Bereich.
guide-debug = Debuggen
guide-debug-1 = Haltepunkt: auf die Zeilennummer klicken oder { $debug-toggle_breakpoint }. Starten/fortsetzen: { $debug-continue }. In Rust-Projekten wird mit cargo build gebaut und das Programm von selbst gefunden; für andere [debug] in der Konfiguration (program, args, cwd, build) oder das Programm auswählen. Programm wechseln: { $debug-choose_program }.
guide-debug-2 = Angehalten: { $debug-next } nächste Zeile, { $debug-step_in } springt in die Funktion, { $debug-step_out } tritt aus, { $debug-pause } hält an, { $debug-stop } beendet. Der gelbe Pfeil zeigt, wo das Programm steht. Verwendet gdb 14+.
guide-debug-3 = Bereich Debuggen ({ $debug-focus }): Schaltflächen, Aufrufliste (ein Klick springt zum Frame und zeigt dessen Variablen), Variablen (ein Klick klappt Strukturen und Vektoren auf) sowie die Ausgabe von Programm und Build.
guide-git = Git
guide-git-1 = Der Branch steht in der Statusleiste. Der Git-Bereich ({ $git-focus }) zeigt, was sich geändert hat: + und − stagen und heben das Staging auf, ein Klick auf eine Datei zeigt den Diff, Eingabe in der Nachricht committet.
guide-git-2 = Rechtsklick auf eine Datei in Git: öffnen, Diff anzeigen, stagen, Änderungen verwerfen (fragt nach) und Pfad kopieren.
guide-git-3 = Abrufen ({ $git-fetch }), pullen ({ $git-pull }) und pushen ({ $git-push }) sind die Schaltflächen oben im Bereich. Das Passwort wird hier nicht abgefragt: den ssh-Agent (ssh-add) oder einen Credential-Helper verwenden. Ein neuer Branch fragt, bevor er auf dem Remote veröffentlicht wird.
guide-git-4 = Der Bereich aktualisiert sich beim Speichern und wenn sich das Repository ändert (Commit oder Branchwechsel im Terminal).
guide-lsp = Sprachen (LSP)
guide-lsp-1 = Sprachserver starten von selbst, wenn Sie eine Datei öffnen: rust-analyzer, clangd, pyright, typescript-language-server, gopls und lua-language-server (sie müssen installiert sein).
guide-lsp-2 = Gehe zu Definition: { $lsp-definition } oder Ctrl+Klick · Dokumentation: { $lsp-hover } oder den Mauszeiger auf dem Code ruhen lassen · Formatieren: { $lsp-format }.
guide-lsp-3 = Wo es verwendet wird: { $lsp-references }, oder Ctrl+Klick auf die Deklaration selbst. Die Liste erscheint im Bereich Verweise; ein Klick springt zur Verwendung.
guide-lsp-4 = Implementierungen: { $lsp-implementation } (auf einem Trait oder einer Schnittstelle, oder auf einer ihrer Methoden). Typ dessen, was unter dem Cursor steht: "LSP: gehe zu Typdefinition" in der Befehlspalette. Bei mehr als einem Ergebnis (auch für die Definition) geht die Liste in den Bereich Verweise; bei nur einem wird direkt dorthin gesprungen.
guide-lsp-5 = Wer eine Funktion aufruft: { $lsp-incoming_calls } mit dem Cursor darauf. Der Bereich Aufrufe zeigt den Baum (eine Ebene öffnen, um zu sehen, wer die Aufrufer aufruft); "Aufgerufene" wechselt zu den Aufrufen, die sie selbst macht. Ein Klick springt zum Aufruf.
guide-lsp-6 = Zurück und vorwärts nach einem Sprung (Definition, Verweis, nach Name öffnen, Klicks in den Bereichen): { $nav-back } und { $nav-forward }, oder die Seitentasten der Maus.
guide-lsp-7 = Dateigliederung ({ $outline-toggle }): Funktionen, Strukturen, Methoden und Felder in einem Baum. Das offene Schloss ist öffentlich, das geschlossene privat, und der Schlüssel ist nur in einem Teil des Codes sichtbar (pub(crate), protected, package). Das Symbol unter dem Cursor ist hervorgehoben; ein Klick springt dorthin.
guide-lsp-8 = Wenn ein Server fehlschlägt, zeigt die Statusleiste den Grund; "LSP: Server neu starten" versucht es erneut.
guide-config = Einstellungen
guide-config-1 = Der Einstellungsbildschirm ({ $settings-open }) ändert Design, Schriftart, Erweiterungen, Tastenkürzel und LSP und schreibt in die globale oder die Projektkonfiguration. Änderungen gelten sofort.
guide-config-2 = Es gibt eine globale Konfiguration und eine je Projekt (`.idetea/config.toml`); die des Projekts gewinnt. Um die Datei direkt zu bearbeiten: "Konfiguration: globale Konfiguration öffnen" oder "Konfiguration: Projektkonfiguration öffnen". Beim Speichern wird alles neu geladen.
guide-config-3 = Tastenkürzel stehen in [keys]: "ctrl+alt+t" = "terminal.focus". Leerer Text schaltet ein Standard-Tastenkürzel aus. Befehls-IDs stehen auf der Registerkarte Tastenkürzel.
guide-config-4 = Sprache: Einstellungen → Darstellung → Sprache (Automatisch folgt dem System). Tritt nach einem Neustart der IDE in Kraft.

## Sitzung
settings-restore-session = So wieder öffnen, wie es war
settings-restore-session-detail = Beim Öffnen eines Projekts kommen seine Registerkarten, der Cursor und die Bereiche zurück. idetea datei... öffnet nur die Datei.

## Zuletzt geöffnet
cmd-file-open-recent = Datei: zuletzt geöffnet…
palette-recent = Zuvor geöffnetes Projekt oder Datei
palette-no-recent = Noch nichts zuvor geöffnet
guide-files-12 = Zuletzt geöffnet: { $file-open_recent } listet die Projekte (sie öffnen sich in einem neuen Fenster) und die zuletzt geöffneten Dateien, aus jedem Fenster.

## Ungespeichertes wiederherstellen
untitled-tab = unbenannt
recover-title = 
    { $count ->
        [one] Die IDE wurde geschlossen, ohne { $count } Datei zu speichern
       *[other] Die IDE wurde geschlossen, ohne { $count } Dateien zu speichern
    }
recover-detail = Wiederherstellen öffnet die Änderungen in Registerkarten, noch ungespeichert (Rückgängig geht auf das zurück, was auf dem Datenträger liegt). Später fragt beim nächsten Mal erneut.
recover = Wiederherstellen
recover-later = Später
recovered = 
    { $count ->
        [one] { $count } Datei wiederhergestellt (noch nicht gespeichert)
       *[other] { $count } Dateien wiederhergestellt (noch nicht gespeichert)
    }

## Symbol umbenennen und alles speichern
cmd-lsp-rename = LSP: Symbol umbenennen…
cmd-file-save-all = Datei: alle speichern
save-all-untitled = Unbenannte Registerkarten wurden nicht gespeichert (verwenden Sie Speichern unter)
palette-rename = Neuer Name
palette-rename-hint = "{ $name }" im Projekt umbenennen · Eingabe zeigt, was sich ändert · Esc bricht ab
rename-no-word = Setzen Sie den Cursor auf einen Namen, um ihn umzubenennen
rename-asking = "{ $name }" umbenennen: der Server wird gefragt…
rename-nothing = Nichts umzubenennen für "{ $name }"
rename-failed = Umbenennen nicht möglich: { $message }
rename-title = "{ $old }" in "{ $new }" umbenennen?
rename-summary = 
    { $changes ->
        [one] { $changes } Änderung
       *[other] { $changes } Änderungen
    } in { $files ->
        [one] { $files } Datei:
       *[other] { $files } Dateien:
    }
rename-detail = Die Dateien bleiben in ihren Registerkarten geändert, noch nicht gespeichert: Ctrl+Z macht in jeder rückgängig, und "Datei: alle speichern" schreibt alles.
rename = Umbenennen
renamed = 
    { $count ->
        [one] Umbenannt in { $count } Datei (noch nicht gespeichert)
       *[other] Umbenannt in { $count } Dateien (noch nicht gespeichert)
    }

guide-lsp-9 = Umbenennen: { $lsp-rename } mit dem Cursor auf dem Namen. Eine Vorschau zeigt, wie viele Änderungen und in welchen Dateien; nach dem Bestätigen bleiben sie in ihren Registerkarten geändert (Ctrl+Z macht in jeder rückgängig) und { $file-save_all } speichert sie alle.

## Im Projekt ersetzen
replaced = { $count } Ersetzung(en) in { $files } Datei(en)
replaced-failed = { $count } Ersetzung(en) in { $files } Datei(en); { $failed } konnten nicht geändert werden (keine Berechtigung oder kein Text)

guide-bottom-3 = Im Projekt ersetzen: { $search-replace } (oder ▸ neben der Suche) öffnet "Ersetzen durch". Jedes Ergebnis zeigt, wie es aussehen wird; "Ersetzen" ändert die Zeile oder die Datei, und "Alle ersetzen" fragt zuerst. In Registerkarten geöffnete Dateien bleiben geändert, nicht gespeichert (Ctrl+Z macht rückgängig); die anderen werden direkt auf den Datenträger geschrieben.

## Editor und Speichern
format-save-timeout = Der Server hat nicht rechtzeitig formatiert: ohne Formatieren gespeichert

settings-tab-editor = Editor
settings-wrap = Lange Zeilen umbrechen
settings-wrap-detail = Eine Zeile, die breiter als das Fenster ist, läuft unten weiter, ohne seitliches Scrollen.
settings-whitespace = Leerzeichen anzeigen
settings-whitespace-detail = Leerzeichen erscheinen als · und Tabulatoren als →.
settings-indent-guides = Einzugsführungslinien
settings-indent-guides-detail = Dünne senkrechte Linien auf jeder Einzugsebene.
settings-tab-size = Einzugsgröße
settings-tab-size-detail = Wie viele Leerzeichen Tab einfügt.
settings-insert-spaces = Mit Leerzeichen einrücken
settings-insert-spaces-detail = Aus: Tab fügt ein Tabulatorzeichen ein. Eine Datei, die schon mit Tabulatoren eingerückt ist, behält Tabulatoren.
settings-trim = Leerzeichen am Zeilenende beim Speichern entfernen
settings-trim-detail = Außer in Markdown, wo zwei Leerzeichen am Ende die Zeile umbrechen.
settings-final-newline = Abschließender Zeilenumbruch beim Speichern
settings-final-newline-detail = Stellt sicher, dass die letzte Zeile mit einem Zeilenumbruch endet.
settings-format-on-save = Beim Speichern formatieren
settings-format-on-save-detail = Ctrl+S formatiert mit dem Sprachserver, bevor geschrieben wird (wartet bis zu 2 s). Alle speichern formatiert nicht.
settings-breadcrumbs = Brotkrümel
settings-breadcrumbs-detail = Über dem Editor: der Pfad der Datei und das Symbol, an dem der Cursor steht. Ein Klick auf einen Ordner zeigt ihn im Baum, ein Klick auf ein Symbol springt dorthin.
settings-editorconfig-note = Eine .editorconfig im Projekt überschreibt diese Optionen (Einzug, Tabulatoren oder Leerzeichen, Aufräumen beim Speichern).

guide-config-5 = Editor: die Registerkarte Editor in den Einstellungen ({ $settings-open }) hat Zeilenumbruch, sichtbare Leerzeichen, Einzugsführungslinien, Einzugsgröße und was beim Speichern geschieht (Leerzeichen am Ende entfernen, abschließender Zeilenumbruch, formatieren). Eine .editorconfig im Projekt überschreibt das; ohne eine behält eine Datei, die schon mit Tabulatoren eingerückt ist, die Tabulatoren.

guide-lsp-10 = Parameterhinweise: ( in einem Aufruf tippen zeigt die Funktionssignatur über dem Cursor, mit dem aktuellen Parameter hervorgehoben; sie rückt mit jedem Komma weiter und verschwindet, wenn die Klammer schließt oder mit Esc.

guide-git-5 = Markierungen im Bundsteg: links der Zeilennummern, grün = neue Zeile, blau = geändert, rote Kerbe = gelöschte Zeilen, verglichen mit dem letzten Commit (folgt dem Tippen, Speichern ist nicht nötig). Ein Klick auf eine Markierung zeigt, wie es war, mit "Änderung zurücksetzen" (Ctrl+Z holt es zurück).

palette-no-items = Nichts passt zur Suche

guide-git-6 = Branches: { $git-checkout } (oder auf den Branchnamen klicken, in der Statusleiste oder im Git-Bereich) listet Branches, die neuesten zuerst; Remote-Branches, die es hier nicht gibt, kommen danach und werden lokal, wenn sie gewählt werden. Einen neuen Namen tippen erzeugt einen Branch von der Stelle, an der Sie sind. Nicht committete Änderungen kommen mit; kollidieren sie mit dem anderen Branch, lehnt git ab und erklärt es.

guide-git-7 = Zeilenautor: in der Statusleiste, neben dem Branch, sehen Sie, wer die Cursorzeile zuletzt geändert hat, wann, und die Commit-Nachricht (Klick zeigt den Hash und kopiert ihn). Solange die Registerkarte ungespeicherte Änderungen hat, bleibt das verborgen, bis Sie speichern (Zeilen auf dem Datenträger können abweichen).

## Aufgaben
cmd-tasks-run = Aufgaben: ausführen…
cmd-tasks-rerun = Aufgaben: die letzte erneut ausführen
cmd-tasks-test-here = Rust: Test am Cursor ausführen
tasks-placeholder = Aufgabe ausführen…
tasks-none = Keine Aufgaben: fügen Sie eine unter [tasks] in der Konfiguration hinzu, oder öffnen Sie ein Projekt mit Cargo.toml, package.json oder Makefile.
tasks-no-terminal = Aufgaben brauchen die Terminal-Erweiterung, die ausgeschaltet ist.
tasks-no-test = Der Cursor steht nicht in einem Rust-Test (#[test] fn).
tasks-no-package = Die Cargo.toml des Pakets dieser Datei wurde nicht gefunden.
tasks-done = [{ $name } beendet: Code { $code } · Eingabe schließt]

guide-terminal-3 = Aufgaben: { $tasks-run } listet die Projektbefehle ([tasks] in der Konfiguration, cargo build/run/test/clippy, Skripte aus package.json, Makefile-Ziele) und führt das Gewählte in einer Registerkarte "Aufgabe" des Terminals aus, ohne die Shell anzutasten. Am Ende erscheint der Exitcode; Eingabe schließt sie. Ctrl+C hält nur den Befehl an. { $tasks-rerun } führt den letzten erneut aus.
guide-terminal-4 = Test am Cursor (Rust): steht der Cursor in einem #[test] fn, führt { $tasks-test_here } nur diesen Test aus (cargo test -p Paket Name). Kein Standard-Tastenkürzel: fügen Sie unter [keys] eines hinzu, wenn Sie es oft nutzen.

## Geteilter Editor
cmd-editor-split = Editor: Registerkarte auf die andere Seite verschieben (teilen)
cmd-editor-focus-left = Editor: zur linken Seite
cmd-editor-focus-right = Editor: zur rechten Seite
split-one-tab = Öffnen Sie eine weitere Datei, um den Editor zu teilen: jede Seite zeigt eine andere Registerkarte.
tab-move-right = Nach rechts verschieben (teilen)
tab-move-left = Nach links verschieben
guide-files-13 = Editor teilen: { $editor-split } (oder "Nach rechts verschieben" im Registerkartenmenü) legt die Registerkarte daneben, mit eigenen Registerkarten; eine Datei zu öffnen geht auf die fokussierte Seite. Klicken Sie eine Seite an (oder { $editor-focus_left } / { $editor-focus_right }), um dort zu arbeiten; ziehen Sie die Trennlinie, um die Breite zu ändern (Doppelklick zentriert sie). Die letzte Registerkarte einer Seite zurückzuschieben fügt alles wieder zusammen.

## Anwendungsmenü (--install-desktop)
desktop-no-home = Persönlicher Ordner nicht gefunden (HOME).
desktop-error = Die Verknüpfung konnte nicht installiert werden: { $error }
desktop-installed = 
    IDE TEA im Anwendungsmenü: { $desktop }
    Startet: { $exe }
desktop-removed = IDE TEA aus dem Anwendungsmenü entfernt.

## Große Dateien
file-light-mode = "{ $name }" hat eine sehr lange Zeile und wurde im Leichtmodus geöffnet: keine Farben, kein Sprachserver und mit Zeilenumbruch
lsp-light-mode = Datei im Leichtmodus (sehr lange Zeile): kein Sprachserver

## Neu gebautes Programm
binary-changed = Die IDE wurde neu gebaut: führen Sie "IDE neu starten" aus (Befehlspalette), um die neue Version zu verwenden.
binary-changed-title = [neue Version: neu starten]
binary-restart-error = Neustart nicht möglich: { $error }

## idetea status / idetea open (RFC 0006)
status-usage =
    Aufruf: idetea status [--json]
    
    Was die in diesem Projekt geöffnete IDE zeigt: die aktive Registerkarte mit dem Cursor und die offenen Dateien
    (mit "ungespeichert" bei denen mit Änderungen). --json: dasselbe als JSON, mit "open".
    Exitcodes: 0 geöffnet, 2 ungültiges Argument, 3 die IDE ist in diesem Projekt nicht geöffnet.
status-bad-arg = unbekanntes Argument: { $arg }
status-not-open = die IDE ist in diesem Projekt nicht geöffnet ({ $root })
status-active = aktiv: { $path }
status-no-active = keine aktive Datei
status-modified = {" "}(ungespeichert)
open-usage =
    Aufruf: idetea open <Datei[:Zeile[:Spalte]]>
    
    Die in diesem Projekt geöffnete IDE zeigt die Datei, mit dem Cursor in Zeile und Spalte.
    Exitcodes: 0 angezeigt, 2 ungültiger Pfad oder außerhalb des Projekts, 3 die IDE ist nicht geöffnet (oder hat die Anfrage nicht angenommen).
open-outside = { $path } liegt nicht im Projekt ({ $root })
open-write-error = { $path } konnte nicht geschrieben werden
open-not-taken = die IDE hat die Anfrage nicht innerhalb von 3 Sekunden angenommen

## Reaktives Maskottchen (RFC 0012)
mascot-state-idle = Ruhig
mascot-state-working = Beschäftigt
mascot-state-thinking = Nachdenken
mascot-state-attention = Achtung
mascot-state-celebrating = Fertig
mascot-state-error = Fehler
mascot-state-sleeping = Schlaf
mascot-state-curious = Neugierig
mascot-state-annoyed = Genervt
mascot-idle-1 = Nur der Code. Eis darfst du selbst zugeben.
mascot-idle-2 = Der Dampf ist da. Ich auch.
mascot-idle-3 = Ruhige Tasse, wacher Blick.
mascot-idle-4 = Die Blätter dürfen sich setzen.
mascot-idle-5 = Keine Eile. Der Tee ist warm.
mascot-idle-6 = Trink, wenn du soweit bist.
mascot-idle-7 = Die Tasse ist auf deiner Seite.
mascot-idle-8 = Alles ruhig auf der Untertasse.
mascot-hover-curious-1 = Hmm? Brauchst du etwas?
mascot-hover-curious-2 = Hallo. Alles in Ordnung?
mascot-hover-curious-3 = Hab ich gespürt. Was gibt's?
mascot-hover-curious-4 = Schwebst du über dem Tee?
mascot-hover-curious-5 = Ein Hallo, oder eine Hand?
mascot-hover-curious-6 = Die Tasse hat dich gesehen.
mascot-hover-curious-7 = Neugierig. Nur zu.
mascot-hover-curious-8 = Willkommen. Was ist der Plan?
mascot-hover-annoyed-1 = Langsam — du rührst den Tee um.
mascot-hover-annoyed-2 = Viel Maus. Der Tasse ist schwindelig.
mascot-hover-annoyed-3 = Ruhige Hand. Heißer Tee.
mascot-hover-annoyed-4 = Einmal vorbei ist ein Hallo. Das war ein Sturm.
mascot-hover-annoyed-5 = Die Untertasse klappert.
mascot-hover-annoyed-6 = Schon gut, ich sehe dich. Weniger Wirbel.
mascot-poke-curious-1 = Hmm? Was ist?
mascot-poke-curious-2 = Das kitzelt die Tasse.
mascot-poke-curious-3 = Du hast geläutet?
mascot-poke-curious-4 = Ein Tippen. Ich höre.
mascot-poke-curious-5 = Vorsicht, sie ist voll.
mascot-poke-curious-6 = Hallo von der Untertasse.
mascot-poke-double-1 = Zwei Taps. Doppelt Dampf.
mascot-poke-double-2 = Doppelklick notiert. Immer noch nur Tee.
mascot-poke-double-3 = Zweimal? Gut, du hast mich.
mascot-poke-double-4 = Zweimal an die Tasse geklopft.
mascot-poke-double-5 = Zwei ist schon ein Gespräch.
mascot-poke-double-6 = Doppeltipp. Nichts verschüttet.
mascot-poke-annoyed-1 = Vorsicht, gleich läuft er über.
mascot-poke-annoyed-2 = Der Tee ist schon heiß.
mascot-poke-annoyed-3 = Genug Getippe für eine Tasse.
mascot-poke-annoyed-4 = Ich mag dich. Die Untertasse weniger.
mascot-poke-annoyed-5 = Langsam. Die Tasse ist klein.
mascot-poke-annoyed-6 = Verstanden. Ich kühle ab.
mascot-task-1 = Bin dran: { $name }
mascot-task-2 = { $name } zieht…
mascot-task-3 = Aufgabe { $name } läuft.
mascot-task-4 = { $name }. Ich passe auf den Kessel auf.
mascot-task-5 = Starte { $name }.
mascot-task-6 = { $name } — lass es ziehen.
mascot-term-pending-1 = Das Terminal will ein Ja: { $title }
mascot-term-pending-2 = Ein Befehl wartet: { $title }
mascot-term-pending-3 = Deine Entscheidung: { $title }
mascot-term-pending-4 = Erlaubnis, bitte: { $title }
mascot-term-pending-5 = Ich halte den Kessel: { $title }
mascot-term-pending-6 = { $title } — soll ich starten?
mascot-term-running-1 = Läuft: { $title }
mascot-term-running-2 = { $title } steht auf dem Feuer.
mascot-term-running-3 = In Arbeit: { $title }
mascot-term-running-4 = { $title } — zieht noch.
mascot-term-running-5 = Arbeite an { $title }.
mascot-term-running-6 = { $title } ist unterwegs.
mascot-term-ok-1 = Fertig: { $title }
mascot-term-ok-2 = { $title } ist sauber durch.
mascot-term-ok-3 = Alles gut — { $title }.
mascot-term-ok-4 = { $title } ist gut geworden.
mascot-term-ok-5 = Sauberer Abgang: { $title }.
mascot-term-ok-6 = { $title }. Die Tasse freut sich.
mascot-term-err-1 = { $title } ist fehlgeschlagen ({ $code }).
mascot-term-err-2 = Exit { $code }: { $title }
mascot-term-err-3 = { $title } ist übergelaufen (Code { $code }).
mascot-term-err-4 = Das ist schiefgegangen: { $title } ({ $code }).
mascot-term-err-5 = { $title } wurde nicht fertig ({ $code }).
mascot-term-err-6 = Code { $code } von { $title }.
mascot-answered-1 = Verstanden: { $title }
mascot-answered-2 = Beantwortet: { $title }
mascot-answered-3 = { $title } — danke.
mascot-answered-4 = Notiert: { $title }
mascot-answered-5 = { $title } ist erledigt.
mascot-answered-6 = Du hast { $title } beantwortet.
mascot-wait-1 = Der Agent wartet nicht mehr: { $title }
mascot-wait-2 = { $title } ist still geworden.
mascot-wait-3 = Auf { $title } wartet niemand mehr.
mascot-wait-4 = { $title } ist abgekühlt.
mascot-wait-5 = Noch offen, aber die Wartezeit ist vorbei: { $title }
mascot-errors-1 = { $count } Fehler im Projekt.
mascot-errors-2 = Die Blätter haben { $count } Fehler gefunden.
mascot-errors-3 = { $count } zum Ansehen, wenn du magst.
mascot-errors-4 = Probleme: { $count }. Keine Eile.
mascot-errors-5 = { $count } Fehler. Ich kann zeigen.
mascot-errors-6 = Bericht der Untertasse: { $count } Fehler.
mascot-save-1 = { $name } gespeichert. Die Tasse nickt.
mascot-save-2 = { $name } liegt auf der Platte.
mascot-save-3 = { $name } erwischt, bevor es abkühlte.
mascot-save-4 = { $name } — verstaut.
mascot-save-5 = Geschrieben: { $name }.
mascot-save-6 = { $name } ist sicher. Zurück zum Tee.
mascot-wake-1 = Oh — du bist zurück. Der Tee ist noch warm.
mascot-wake-2 = Ich war weggedöst. Wo waren wir?
mascot-wake-3 = Willkommen zurück. Ich passe wieder auf.
mascot-wake-4 = Augen auf. Die Tasse auch.
mascot-wake-5 = Das war eine gute Pause.
mascot-wake-6 = Weiter? Ich halte Wache.
mascot-sleep-1 = Zieht leise…
mascot-sleep-2 = Ich bleibe hier.
mascot-sleep-3 = Kleine Flamme. Ruf, wenn du mich brauchst.
mascot-sleep-4 = Müde Tasse. Der Code darf ruhen.
mascot-sleep-5 = Nickerchen auf der Untertasse.
mascot-sleep-6 = Licht gedimmt. Ich wache auf, wenn du es tust.
mascot-greet-morning-1 = Morgen. Die erste Tasse geht auf mich.
mascot-greet-morning-2 = Guten Morgen. Erst Code, dann Eis.
mascot-greet-morning-3 = Frühes Licht, frische Blätter.
mascot-greet-morning-4 = Morgen. Ein kleiner Schluck vor dem Diff.
mascot-greet-morning-5 = Hallo. Der Kessel steht schon auf.
mascot-greet-afternoon-1 = Nachmittag. Die Kanne hält noch.
mascot-greet-afternoon-2 = Guten Tag. Gleichmäßiges Tempo.
mascot-greet-afternoon-3 = Mittag. Nicht auf ex trinken.
mascot-greet-afternoon-4 = Nachmittagsgruß von der Tasse.
mascot-greet-afternoon-5 = Die Sonne steht hoch. Der Tee auch.
mascot-greet-evening-1 = Abend. Noch eine Datei, oder die Untertasse.
mascot-greet-evening-2 = Guten Abend. Die Tasse ist noch warm.
mascot-greet-evening-3 = Abendlicht. Mach Schluss, wenn du magst.
mascot-greet-evening-4 = Der Tag dreht die Flamme runter.
mascot-greet-evening-5 = Abend. Ich bleibe, falls der Build es nicht tut.
mascot-greet-late-1 = Spät. Der Tee hält. Ruh dich aus, wenn du kannst.
mascot-greet-late-2 = Stille Stunden. Schlaf ist eine Funktion.
mascot-greet-late-3 = Noch wach? Schließ den Gedanken, dann Pause.
mascot-greet-late-4 = Die Tasse kann bleiben. Du darfst pausieren.
mascot-greet-late-5 = Tiefe Nacht. Ich passe auf. Halt es kurz.
mascot-tip-1 = Jeder Befehl liegt in der Palette: { $palette-open }.
mascot-tip-2 = Eine Datei nach Namen öffnen: { $file-quick-open }.
mascot-tip-3 = { $file-save } legt die Datei auf die Platte.
mascot-tip-4 = Tastenkürzel und eine kurze Hilfe: { $help-open }.
mascot-tip-5 = Eine Projektaufgabe starten: { $tasks-run }.
mascot-tip-6 = Das Terminal ist { $terminal-focus }.
mascot-tip-7 = Probleme: { $problems-focus }. Das nächste ist { $problems-next }.
mascot-tip-8 = Zur Definition springen: { $lsp-definition }.
mascot-build-ok-1 = Der Build ist durch. Der Tee ist noch heiß.
mascot-build-ok-2 = Kompiliert. Gute Blätter.
mascot-build-ok-3 = Sauberer Build.
mascot-build-ok-4 = Grüner Build. Ich schenke ein.
mascot-build-ok-5 = Kompiliert. Zurück zum Editor.
mascot-build-err-1 = Der Build hat nicht gezogen.
mascot-build-err-2 = Build fehlgeschlagen. Die Tasse hat's gemerkt.
mascot-build-err-3 = Kompilierfehler. Schau, wenn du magst.
mascot-build-err-4 = Der Build ist übergelaufen.
mascot-build-err-5 = Build fehlgeschlagen. Probleme weiß vielleicht mehr.
mascot-working-1 = Auf dem Feuer.
mascot-working-2 = Am Arbeiten. Der Dampf ist beschäftigt.
mascot-working-3 = Lass es ziehen.
mascot-working-4 = Bin noch dran.
mascot-working-5 = Der Kessel ist belegt.
mascot-thinking-1 = Ich rühre die Blätter…
mascot-thinking-2 = Einen Moment. Ich denke.
mascot-thinking-3 = Es zieht noch.
mascot-thinking-4 = Ruhiger Gedanke, warme Tasse.
mascot-thinking-5 = Ich schau mir das in Ruhe an.
mascot-celebrate-1 = Fertig. Das ist gut geworden.
mascot-celebrate-2 = Alles bereit.
mascot-celebrate-3 = Sauberes Ende. Prost.
mascot-celebrate-4 = Die Tasse nickt.
mascot-celebrate-5 = Gut. Ich halte den Dampf.
mascot-error-1 = Etwas ist übergelaufen.
mascot-error-2 = Das hat nicht gezogen.
mascot-error-3 = Ein Haken. Ich bin noch da.
mascot-error-4 = Diesmal nicht.
mascot-error-5 = Einen Blick wert, wenn du magst.
mascot-open-interfaces = Panel Interfaces öffnen →
mascot-open-terminal = Panel Terminal öffnen →
mascot-open-problems = Panel Probleme öffnen →
mascot-idle-hint = Ich begleite Agenten und Aufgaben
mascot-bad-state = unbekannter Maskottchen-Zustand: { $state }
mascot-usage =
    Aufruf: idetea mascot <zustand> [--text "…"] [--source <name>] [--ttl <sekunden>]
            idetea mascot clear
            idetea mascot status [--json]

    Zustände: idle, working, thinking, attention, celebrating, error, sleeping, curious, annoyed.
    Schreibt .idetea/mascot.json, damit das Maskottchen sofort reagiert (läuft nach --ttl ab, Standard 30 s).

## Willkommensbildschirm
welcome-start = Start
welcome-new-file = Neue Datei
welcome-quick-open = Projektdatei öffnen…
welcome-open-file = Datei öffnen…
welcome-open-folder = Ordner in einem neuen Fenster öffnen…
welcome-recent = Zuletzt geöffnet…
welcome-recent-files = Letzte Dateien
welcome-recent-projects = Letzte Projekte
welcome-hint-palette = alle Befehle
welcome-hint-search = Projekt durchsuchen
welcome-hint-terminal = Terminal
