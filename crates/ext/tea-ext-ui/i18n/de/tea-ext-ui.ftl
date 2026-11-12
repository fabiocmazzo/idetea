ui-panel = Interfaces
cmd-ui-show = Interfaces: Bereich anzeigen
ui-empty = Keine Oberfläche offen. Ein Agent kann eine erzeugen, indem er eine Datei nach .idetea/ui/ schreibt (siehe docs/interfaces/formato.md). Ein- oder ausschalten: [extensions] interfaces = {"{"} enabled = true {"}"} in der Konfiguration.
ui-tracked = { $path } wird von git verfolgt und aus Sicherheitsgründen ignoriert: Oberflächen kommen nur vom Agent auf diesem Rechner, nie aus einem geklonten Repository.
ui-invalid = Ungültiges Dokument: { $error }
ui-origin = { $path }
ui-origin-source = { $path } · { $source }
ui-waiting = { $count ->
    [one] { $count } Antwort gesendet; es wird gewartet, bis der Agent sie liest.
   *[other] { $count } Antworten gesendet; es wird gewartet, bis der Agent sie liest.
}
ui-received = Antwort vom Agent empfangen.
ui-agent-waiting = Der Agent wartet auf die Antwort.
ui-agent-stopped = { $minutes ->
    [0] Der Agent hat gerade aufgehört zu warten: die Antwort bleibt, bis er erneut nachsieht.
   *[other] Der Agent hat vor { $minutes } Min. aufgehört zu warten: die Antwort bleibt, bis er erneut nachsieht.
}
ui-new = Neue Oberfläche vom Agent: { $title }
ui-updated = Oberfläche aktualisiert: { $title }
ui-open-later = Der Agent wollte { $path } öffnen: sie liegt im Bereich Interfaces (nicht geöffnet, damit Ihre Eingabe nicht unterbrochen wird).
ui-write-failed = Antwort konnte nicht geschrieben werden: { $error }
ui-prepare-failed = Ordner .idetea/ui konnte nicht vorbereitet werden: { $error }
ui-not-a-file = keine reguläre Datei
ui-surface-mismatch = das Feld surface ist "{ $found }", die Datei heißt aber { $expected }.json
cmd-ui-teach = Interfaces: den Agenten dieses Projekts beibringen
ui-teach-hint = Damit ein Agent diesen Bereich zum Programmieren nutzen kann, schreiben Sie die Anleitung in die AGENTS.md des Projekts.
ui-teach-button = Den Agenten dieses Projekts beibringen
ui-teach-title = Den Agenten dieses Projekts beibringen?
ui-teach-detail = Die IDE schreibt die Anleitung zum Bereich Interfaces nach: { $files }. Der Text bleibt zwischen Markierungen; ein erneuter Lauf aktualisiert nur diesen Teil und lässt den Rest unverändert. Das sind Projektdateien: prüfen Sie sie vor dem Commit.
ui-teach-file-new = { $name } (neu)
ui-teach-write = Schreiben
ui-cancel = Abbrechen
ui-teach-up-to-date = Die Agent-Anleitung ist bereits aktuell.
ui-teach-done = Anleitung geschrieben nach { $files }.
ui-teach-failed = Anleitung konnte nicht geschrieben werden: { $error }
cmd-ui-new-project = Projekt: neu…
new-title = Neues Projekt
new-intro = Die IDE erzeugt den Ordner, git, die Skills der Vorlage und die Bereichsanleitung und öffnet das Projekt in einem neuen Fenster. Ihr Agent erstellt das eigentliche Projekt und folgt der Anfangsanfrage in **AGENTS.md**. Ist unten ein Agent gewählt, bittet das neue Fenster darum, ihn in einem IDE-Terminal zu öffnen (Sie bestätigen den Befehl).
new-field-parent = Erstellen in
new-field-name = Ordnername
new-field-name-hint = z. B. meine-app
new-field-model = Vorlage
new-field-description = Worum es im Projekt geht (optional)
new-field-description-hint = ein Satz; er kommt in die Anfangsanfrage für den Agent
new-field-git = Git-Repository erstellen
new-field-agent = Agent, der in einem IDE-Terminal geöffnet wird
new-agent-none = Keiner (ich öffne ihn selbst)
new-agent-title = Agent: { $agent }
new-bad-agent = ungültiger Agent-Name: "{ $agent }" (nur Buchstaben, Ziffern, - und _)
new-create = Erstellen und öffnen
new-model = { $model ->
    [vazio] Leer (der Agent fragt, was es ist)
    [rust] Rust
    [site] Website
    [python] Python
    [fastapi] FastAPI (Python-Web-API)
   *[other] { $model }
}
new-creating = { $dir } wird erstellt (die Skills kommen über das Netzwerk)…
new-created = Projekt erstellt in { $dir }; es wird in einem neuen Fenster geöffnet.
new-open-failed = Projekt erstellt in { $dir }, aber das Fenster konnte nicht geöffnet werden: { $error }
new-failed = Projekt konnte nicht erstellt werden: { $error }
new-bad-model = unbekannte Vorlage: { $model }
new-bad-dir = ungültiger Ordner: { $dir } (einen vollständigen Pfad verwenden)
new-bad-name = ungültiger Ordnername: "{ $name }"
new-not-empty = { $dir } existiert bereits und enthält Dateien: anderen Namen wählen
new-done-dir = Ordner: { $dir }
new-done-git = git: Repository erstellt
new-done-agents = AGENTS.md (mit der Anfangsanfrage) und CLAUDE.md, mit der Bereichsanleitung
new-done-agent = Terminalanfrage für { $agent }: im neuen Fenster bestätigen
new-open-hint = Fertig. Öffnen mit: idetea { $dir }
new-usage = idetea new: "{ $arg }" ist ungültig. Aufruf: idetea new <Ordner> [--modelo id] [--descricao Text] [--sem-git] [--agente Name]; Vorlagen: { $models }
cmd-ui-fix = Interfaces: dieses Projekt reparieren
ui-fix-title = Dieses Projekt reparieren?
ui-fix-will = Die IDE wird:
ui-fix-wont = Nur zur Information (bleibt so):
ui-fix-button = Reparieren
ui-fix-nothing = Nichts an den Oberflächen dieses Projekts zu reparieren.
ui-fix-done = { $count ->
    [one] Repariert: { $count } Element.
   *[other] Repariert: { $count } Elemente.
}
ui-fix-failed = Konnte nicht repariert werden: { $error }
ui-fix-hint = Um die mit "-" markierten Punkte zu reparieren: idetea ui fix --apply
ui-fix-usage = idetea ui fix: unbekanntes Argument "{ $arg }" (Aufruf: idetea ui fix [--apply])
ui-fix-abandoned = { $path } löschen: seit { $since } wartet niemand mehr auf die Antwort
ui-fix-unread = { $path }: { $count ->
    [one] { $count } Antwort, die der Agent nicht gelesen hat
   *[other] { $count } Antworten, die der Agent nicht gelesen hat
} (behalten; den Agent bitten, sie zu lesen, oder von Hand löschen)
ui-fix-leftover = { $path } löschen: Überbleibsel eines Schreibvorgangs oder eines Wartens, das nicht zu Ende kam
ui-fix-check = .idetea/ui/check neu schreiben: es ruft { $target } auf, das nicht mehr existiert
ui-fix-instructions-older = Anleitung in { $file } neu schreiben: sie ist auf Version { $found }, älter als diese IDE
ui-fix-instructions-unversioned = Anleitung in { $file } neu schreiben: sie stammt von einer alten IDE, ohne Versionszeile
ui-fix-instructions-newer = { $file } enthält eine Anleitung von einer neueren IDE ({ $found }): öffnen Sie das Projekt darin (ich gehe nicht zurück)
ui-outdated = Diese IDE ist veraltet (sie wurde nach dem Öffnen neu gebaut): Antworten teilen das dem Agent mit. Befehlspalette: "IDE neu starten".
ui-audio-failed = Ton konnte nicht abgespielt werden: { $error }
ui-check-ok = { $file }: ok ({ $count ->
        [one] { $count } Komponente
       *[other] { $count } Komponenten
    })
ui-check-error = { $file }: Fehler: { $error }
ui-check-warning = Warnung: { $warning }
ui-check-media = Medien: { $path }: { $problem }
ui-check-name = ungültiger Dateiname ("{ $name }"): Kleinbuchstaben, Ziffern, _ und -, endet auf .json
ui-check-usage = Aufruf: idetea ui check <file.json>…
ui-check-instructions-older = Anleitung: { $file } hat Version { $found } und diese IDE hat { $ours }: bitten Sie, "Interfaces: den Agenten dieses Projekts beibringen" auszuführen und die Datei vor dem Veröffentlichen neu zu lesen
ui-check-instructions-newer = Anleitung: { $file } hat Version { $found }, von einer neueren IDE als dieser ({ $ours }): der offene Bereich versteht möglicherweise nicht, was die Datei lehrt
ui-check-instructions-unversioned = Anleitung: { $file } hat einen alten Block ohne Versionszeile; diese IDE hat { $ours }: bitten Sie, "Interfaces: den Agenten dieses Projekts beibringen" auszuführen und die Datei vor dem Veröffentlichen neu zu lesen
ui-teach-newer-file = { $name } (Version { $version })
ui-teach-newer = Warnung: { $files } wurde von einer neueren IDE geschrieben als dieser (die Version { $ours } hat und jene nicht kennt). Schreiben setzt die Anleitung auf die Version dieser IDE zurück; normalerweise öffnen Sie das Projekt mit der neueren IDE.
ui-teach-write-anyway = Trotzdem schreiben
ask-title = Frage
ask-choose = Auswählen
ask-ok = Bestätigen
ask-yes = Ja
ask-no = Nein
ask-not-object = das Dokument muss ein JSON-Objekt sein
ask-bad-arg = idetea ask: "{ $arg }" ist ungültig.
ask-usage = Aufruf: idetea ask <document.json | -> | --text T [--options "a|b|c" | --confirm | --input Beschriftung] [--title T] [--name Name] [--value /Zeiger] [--json] [--keep] [--timeout Sekunden]; idetea ask --resume <Name>; idetea ask --close <Name>
ask-invalid = idetea ask: der Bereich würde die Frage ablehnen: { $error }
ask-no-such = idetea ask: es gibt keine Frage "{ $name }" im Bereich
ask-timeout = idetea ask: keine Antwort rechtzeitig; die Frage "{ $name }" bleibt im Bereich (idetea ask --resume { $name } wartet erneut)
ask-outdated = idetea ask: die Antwort kam von einer IDE, die nach dem Öffnen neu gebaut wurde (ide.outdated); testen Sie eine Änderung, bitten Sie um einen Neustart der IDE
ask-comment = Kommentar: { $comment }

# Verlauf der Interfaces (RFC 0007)
ui-history-panel = Verlauf der Interfaces
cmd-ui-history = Interfaces: Verlauf
cmd-ui-clear-history = Interfaces: Verlauf löschen
ui-history-failed = Verlauf konnte nicht geschrieben werden: { $error }
ui-history-cleared = { $count ->
    [one] { $count } Eintrag aus dem Verlauf entfernt.
   *[other] { $count } Einträge aus dem Verlauf entfernt.
}
ui-history-link = Verlauf ({ $count })
ui-history-empty = Der Verlauf ist leer.
ui-history-clear-title = Verlauf löschen?
ui-history-clear-detail = { $count ->
    [one] Der Verlaufeintrag dieses Projekts (.idetea/history/) wird gelöscht.
   *[other] Die { $count } Verlaufeinträge dieses Projekts (.idetea/history/) werden gelöscht.
} Die im Bereich offenen Oberflächen bleiben. Das lässt sich nicht rückgängig machen.
ui-history-clear-button = Verlauf löschen
ui-history-search = Titel, Antworten und Kommentare suchen
ui-history-open = Offen ({ $count })
ui-history-waiting-you = wartet auf Sie
ui-history-answered-open = beantwortet, noch offen
ui-history-today = Heute
ui-history-yesterday = Gestern
ui-history-none = Noch keine beantworteten Fragen. Jede Antwort im Bereich Interfaces bleibt hier (in .idetea/history/, außerhalb von git).
ui-history-no-match = Nichts gefunden.
ui-history-back = ← Zurück
ui-history-shown = Angezeigt: { $time }
ui-history-answered = Beantwortet: { $time }
ui-history-closed = Ohne Antwort geschlossen: { $time }
ui-history-read = Vom Agent gelesen: { $time }
ui-history-button = Schaltfläche: { $action }
ui-history-not-read = Der Agent hat diese Antwort nicht gelesen (die Datei liegt noch in .idetea/ui/responses/, oder sie wurde gelöscht, während die IDE geschlossen war).
ui-history-not-read-short = nicht gelesen
ui-history-unanswered = keine Antwort
ui-history-media-changed = { $path } hat sich seit der Antwort geändert oder fehlt: was Sie hier sehen, ist die aktuelle Version.
ui-history-no-audio = der Verlauf spielt keine Klänge ab
history-cli-comment = Kommentar
history-cli-not-read = der Agent hat diese Antwort nicht gelesen
history-cli-unanswered = keine Antwort (der Agent hat die Frage gelöscht)
history-cli-bad = idetea history: "{ $arg }" ist ungültig.
history-cli-usage = Aufruf: idetea history [--name Name] [--since YYYY-MM-DD] [--last N | --all] [--json]
history-cli-empty = idetea history: nichts im Verlauf dieses Projekts.
