## Terminale
terminal-start-error = Terminale: impossibile avviare la shell: { $error }
cmd-terminal-focus = Terminale: apri e metti a fuoco
cmd-terminal-new = Terminale: nuovo terminale
cmd-terminal-close = Terminale: chiudi il terminale
cmd-terminal-restart = Terminale: riavvia la shell
terminal-ended-n = Terminale { $number } chiuso
terminal-not-running = La shell non è in esecuzione.
terminal-start = Avvia il terminale

## Attività
terminal-task-error = Attività: impossibile avviare: { $error }
terminal-tab-n = Terminale { $number }
terminal-close = Chiudi il terminale
terminal-tab-task = Attività: { $name }
terminal-close-task = Chiudi attività

## Terminali richiesti dagli agenti
terminal-tab-agent = Agente: { $title }
terminal-tab-request = { $count ->
    [one] { $count } richiesta di terminale
   *[other] { $count } richieste di terminale
}
agent-request-notify = Un agente chiede un terminale: { $title }. Vedi il pannello del terminale.
agent-request-auto = Richiesta di terminale "{ $title }" in esecuzione senza conferma: è in [agents] allow della tua config ("{ $entry }").
agent-request-someone = Un agente
agent-request-title = { $who } chiede un terminale: { $title }
agent-request-command = Comando (esegue esattamente questo):
agent-request-cwd = Nella cartella: { $cwd }
agent-request-worktree = Viene eseguito in un proprio worktree (albero di lavoro): ramo { $branch }, in { $path } (creato da HEAD se il ramo non esiste)
agent-worktree-kept = Il worktree { $path } è stato tenuto: { $error }
agent-request-warning = Viene eseguito con i tuoi permessi, come se lo avessi scritto tu. Consenti solo ciò che riconosci.
agent-request-allow = Consenti ed esegui
agent-request-refuse = Rifiuta
agent-request-more = { $count ->
    [one] { $count } altra richiesta dopo questa.
   *[other] { $count } altre richieste dopo questa.
}
agent-done = Terminato con codice { $code }. Invio chiude la scheda.
run-bad-arg = idetea run: "{ $arg }" non è valido.
run-usage = Uso: idetea run [--title T] [--name nome] [--cwd cartella] [--worktree ramo] [--keep] [--no-wait] [--timeout secondi] -- <comando>; idetea run --wait <nome>; idetea run --stop <nome>
run-busy = idetea run: c'è già una richiesta "{ $name }" (o il suo stato); scegli un altro --name oppure attendi con idetea run --wait { $name }
run-asked = idetea run: la richiesta "{ $name }" è nel pannello del terminale; in attesa che tu la consenta e che il comando finisca
run-exited = idetea run: "{ $name }" è terminato con codice { $code }
run-refused = idetea run: hai rifiutato "{ $name }"
run-invalid = idetea run: la richiesta "{ $name }" non è valida: { $error }
run-ended = idetea run: "{ $name }" è terminato senza un codice ({ $state })
run-branch = idetea run: il lavoro è sul ramo { $branch }
run-timeout = idetea run: il tempo è scaduto e "{ $name }" è ancora nel terminale (idetea run --wait { $name } attende di nuovo; --stop lo chiude)
