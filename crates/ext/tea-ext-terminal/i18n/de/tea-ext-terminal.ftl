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
