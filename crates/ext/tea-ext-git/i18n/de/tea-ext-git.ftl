## Git
cmd-git-focus = Git: Änderungen anzeigen
cmd-git-refresh = Git: aktualisieren
cmd-git-stage-all = Git: alles stagen
cmd-git-unstage-all = Git: Staging für alles aufheben
cmd-git-commit = Git: committen (fokussiert die Nachricht)
cmd-git-pull = Git: pullen (herunterladen und zusammenführen)
cmd-git-push = Git: pushen (Commits senden)
cmd-git-fetch = Git: abrufen (Remote auf Neuigkeiten prüfen)
git-panel = Git
cancel = Abbrechen
copied = Kopiert: { $text }
git-notice = Git: { $message }
git-wait = Git: bitte warten, noch { $running }
git-fetching = Wird abgerufen…
git-pulling = Wird gepullt…
git-pushing = Wird gepusht…
git-publishing = Wird veröffentlicht…
git-no-repo-notice = Git: dieser Ordner liegt in keinem Repository
git-no-repo = Dieser Ordner liegt in keinem Git-Repository.
git-need-message = Git: Commit-Nachricht schreiben
git-nothing-staged = Git: nichts gestaged (mit + hinzufügen)
git-detached-pull = Git: losgelöster HEAD: zum Pullen einen Branch auschecken
git-detached-push = Git: losgelöster HEAD: zum Pushen einen Branch auschecken
git-no-upstream = Git: dieser Branch verfolgt kein Remote (pushen, um ihn zu veröffentlichen)
git-no-remote = Git: das Repository hat kein Remote (git remote add origin <url>)
git-discarded = Änderungen verworfen in { $path }
git-discard-untracked = Die Datei ist neu (nicht verfolgt) und kommt in den Papierkorb.
git-discard-tracked = Nicht gestagte Änderungen gehen verloren; die Datei kehrt zum Gestagten bzw. zum letzten Commit zurück.
git-discard-title = Änderungen in "{ $path }" verwerfen?
git-discard = Verwerfen
git-publish-title = Branch "{ $branch }" veröffentlichen?
git-publish-detail = Er existiert auf { $remote } noch nicht. Der Push erzeugt den Branch dort und beginnt, ihn zu verfolgen (git push -u { $remote } { $branch }).
git-publish = Veröffentlichen
git-fetch-tip = Abrufen: Remote auf Neuigkeiten prüfen
git-pull-tip = Pullen: herunterladen und zusammenführen{ $count }
git-push-tip = Pushen: Commits senden{ $count }
git-refresh = Aktualisieren
git-commit-placeholder = Commit-Nachricht (Eingabe)
git-staged = Gestaged
git-changes = Änderungen
git-unstage-all = Staging für alles aufheben
git-stage-all = Alles stagen
git-unstage = Staging aufheben
git-stage = Stagen
git-nothing-to-commit = Nichts zu committen.
git-open-file = Datei öffnen
git-view-diff = Diff anzeigen
git-discard-menu = Änderungen verwerfen…
git-copy-path = Pfad kopieren
git-copy-relative = Relativen Pfad kopieren
git-select-file = Wählen Sie eine Datei, um den Diff zu sehen.
git-diff-truncated = … Diff nach { $count } Zeilen abgeschnitten
git-no-diff = Keine Unterschiede (Binärdatei oder nur Berechtigungen).
git-run-error = git konnte nicht ausgeführt werden: { $error }
git-unknown-error = unbekannter Fehler
git-trash-error = konnte nicht in den Papierkorb verschoben werden: { $error }
git-timeout = Zeitüberschreitung ({ $seconds } s) bei der Verbindung zum Remote
git-explain-rejected = das Remote hat Commits, die Sie nicht haben: zuerst pullen
git-explain-diverged = lokaler und Remote-Branch sind auseinandergelaufen: pull.rebase setzen (oder im Terminal auflösen)
git-explain-conflict = Pull mit Konflikten: die markierten Dateien auflösen und committen
git-explain-overwritten = lokale Änderungen würden überschrieben: vor dem Pullen committen oder verwerfen
git-explain-auth = Authentifizierung fehlgeschlagen: einen Credential-Helper einrichten (z. B. gh auth setup-git)
git-explain-ssh = ssh hat die Verbindung abgelehnt: den Schlüssel in den Agent laden (ssh-add) oder den Server im Terminal bestätigen
git-explain-network = Remote nicht erreichbar (Netzwerk oder Adresse)
git-fetch-done = Abruf abgeschlossen
git-up-to-date = bereits aktuell
git-pull-done = Pull abgeschlossen
git-published = Branch { $branch } auf { $remote } veröffentlicht
git-nothing-to-push = nichts zu pushen
git-push-done = Push abgeschlossen

## Branches
cmd-git-checkout = Git: Branch wechseln
git-checkout-placeholder = Zu Branch wechseln… (oder neuen Namen eingeben)
git-create-branch = Branch erstellen
git-branch-current = aktuell · { $date }
git-switched = Jetzt auf Branch { $branch }
git-switch-failed = Wechsel zu { $branch } nicht möglich: { $error }
git-explain-switch-overwritten = nicht committete Änderungen kollidieren mit dem anderen Branch: zuerst committen oder verwerfen

## Zeilenautor (Blame)
git-blame-uncommitted = Sie · noch nicht committet
git-blame-detail = 
    { $author }, { $ago }
    Commit { $hash }
git-blame-copy = Hash kopieren
git-close = Schließen
git-ago-now = gerade eben
git-ago-minutes = 
    { $count ->
        [one] vor { $count } Minute
       *[other] vor { $count } Minuten
    }
git-ago-hours = 
    { $count ->
        [one] vor { $count } Stunde
       *[other] vor { $count } Stunden
    }
git-ago-days = 
    { $count ->
        [one] gestern
       *[other] vor { $count } Tagen
    }
git-ago-months = 
    { $count ->
        [one] vor { $count } Monat
       *[other] vor { $count } Monaten
    }
git-ago-years = 
    { $count ->
        [one] vor { $count } Jahr
       *[other] vor { $count } Jahren
    }
