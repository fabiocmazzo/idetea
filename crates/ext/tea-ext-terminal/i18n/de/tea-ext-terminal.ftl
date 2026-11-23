## Terminal
terminal-start-error = Terminal: Shell konnte nicht gestartet werden: { $error }
cmd-terminal-focus = Terminal: öffnen und fokussieren
cmd-terminal-new = Terminal: neues Terminal
cmd-terminal-close = Terminal: Terminal schließen
cmd-terminal-restart = Terminal: Shell neu starten
terminal-ended-n = Terminal { $number } geschlossen
terminal-not-running = Die Shell läuft nicht.
terminal-start = Terminal starten

## Aufgaben
terminal-task-error = Aufgabe: konnte nicht gestartet werden: { $error }
terminal-tab-n = Terminal { $number }
terminal-close = Terminal schließen
terminal-tab-task = Aufgabe: { $name }
terminal-close-task = Aufgabe schließen

## Von Agenten angeforderte Terminals
terminal-tab-agent = Agent: { $title }
terminal-tab-request = { $count ->
    [one] { $count } Terminalanfrage
   *[other] { $count } Terminalanfragen
}
agent-request-notify = Ein Agent bittet um ein Terminal: { $title }. Siehe den Terminalbereich.
agent-request-auto = Terminalanfrage "{ $title }" läuft ohne Bestätigung: sie steht in [agents] allow Ihrer Konfiguration ("{ $entry }").
agent-request-someone = Ein Agent
agent-request-title = { $who } bittet um ein Terminal: { $title }
agent-request-command = Befehl (genau dies wird ausgeführt):
agent-request-cwd = Im Ordner: { $cwd }
agent-request-worktree = Läuft in einem eigenen Worktree (Arbeitsbaum): Branch { $branch }, in { $path } (wird aus HEAD erzeugt, wenn der Branch nicht existiert)
agent-worktree-kept = Der Worktree { $path } wurde behalten: { $error }
agent-request-warning = Er läuft mit Ihren Rechten, als hätten Sie ihn eingegeben. Lassen Sie nur zu, was Sie erkennen.
agent-request-allow = Zulassen und ausführen
agent-request-refuse = Ablehnen
agent-request-more = { $count ->
    [one] Danach noch { $count } Anfrage.
   *[other] Danach noch { $count } Anfragen.
}
agent-done = Beendet mit Code { $code }. Eingabe schließt die Registerkarte.
run-bad-arg = idetea run: "{ $arg }" ist ungültig.
run-usage = Aufruf: idetea run [--title T] [--name Name] [--cwd Ordner] [--worktree Branch] [--keep] [--no-wait] [--timeout Sekunden] -- <Befehl>; idetea run --wait <Name>; idetea run --stop <Name>
run-busy = idetea run: es gibt bereits eine Anfrage "{ $name }" (oder ihren Status); wählen Sie ein anderes --name oder warten Sie mit idetea run --wait { $name }
run-asked = idetea run: Anfrage "{ $name }" liegt im Terminalbereich; es wird gewartet, bis Sie sie zulassen und der Befehl endet
run-exited = idetea run: "{ $name }" endete mit Code { $code }
run-refused = idetea run: Sie haben "{ $name }" abgelehnt
run-invalid = idetea run: Anfrage "{ $name }" ist ungültig: { $error }
run-ended = idetea run: "{ $name }" endete ohne Code ({ $state })
run-branch = idetea run: die Arbeit liegt auf Branch { $branch }
run-timeout = idetea run: die Zeit ist um und "{ $name }" liegt noch im Terminal (idetea run --wait { $name } wartet erneut; --stop beendet es)

## Tab-Status (RFC 0024)
terminal-state-running = Läuft
terminal-state-unseen = Neue Ausgabe seit dem letzten Blick
terminal-state-attention = Bittet um Aufmerksamkeit
terminal-state-exited = Beendet mit Code { $code }
terminal-state-bell = Hat die Glocke geläutet
terminal-state-turn-done = Ist fertig und wartet auf dich
cmd-terminal-next = Terminal: nächster Tab
cmd-terminal-prev = Terminal: vorheriger Tab
cmd-terminal-goto = Terminal: gehe zu…
cmd-terminal-next-attention = Terminal: nächster Tab, der auf dich wartet
cmd-terminal-rename = Terminal: Tab umbenennen
terminal-no-attention = Kein Terminal-Tab braucht Aufmerksamkeit.
terminal-goto-placeholder = Zum Terminal-Tab…
terminal-rename-placeholder = Name des Tabs
terminal-rename-reset = Zurück zum Standardnamen
terminal-rename-to = Umbenennen in
terminal-close-running-title = Tab schließen?
terminal-close-running = { $name } läuft noch. Das Schließen des Tabs beendet den Prozess.
terminal-close-running-yes = Schließen und beenden
terminal-close-running-no = Abbrechen
cmd-terminal-find = Terminal: suchen
terminal-find-placeholder = Im Terminal suchen (Enter: älter)
terminal-find-older = Älter (oben)
terminal-find-newer = Neuer (unten)
terminal-find-none = Keine Treffer
terminal-find-close = Suche schließen
cmd-terminal-split = Terminal: teilen (oder zusammenführen)
cmd-terminal-resume = Terminal: Agenten der letzten Sitzung fortsetzen
terminal-resume-nothing = Keine Agenten der letzten Sitzung warten.
terminal-resume-title = Agenten fortsetzen?
terminal-resume-message = { $count ->
    [one] Ein Agent war offen, als die IDE geschlossen wurde: { $names }.
   *[other] { $count } Agenten waren offen, als die IDE geschlossen wurde: { $names }.
}
terminal-resume-all = Alle fortsetzen
terminal-resume-pick = Auswählen…
terminal-resume-none = Nein
terminal-resume-pick-placeholder = Agent zum Fortsetzen…
terminal-resume-same = dasselbe Gespräch
terminal-resume-new = neues Gespräch
terminal-resume-failed = Das Gespräch von { $title } ließ sich nicht fortsetzen (Code { $code }). Starte ein neues mit „Agenten: starten…“ (Ctrl+Shift+A).
terminal-previous-session = — vorherige Sitzung, { $when } —
