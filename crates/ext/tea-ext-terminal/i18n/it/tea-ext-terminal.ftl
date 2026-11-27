## Terminale
terminal-start-error = Terminale: impossibile avviare la shell: { $error }
cmd-terminal-focus = Terminale: apri e metti a fuoco
cmd-terminal-new = Terminale: nuovo terminale
cmd-terminal-close = Terminale: chiudi il terminale
cmd-terminal-restart = Terminale: riavvia la shell
terminal-ended-n = Terminale { $number } chiuso
terminal-link-copied = Indirizzo copiato (questo tipo di link non si apre dal terminale): { $url }
terminal-link-failed = Impossibile aprire { $url }: { $error }
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
agent-request-batch = { $count } richieste di terminale in attesa. Controlla ogni comando: "Consenti tutte" esegue esattamente quelle in questa schermata, ognuna nella sua scheda.
agent-request-allow-all = Consenti tutte ({ $count })
agent-request-refuse-all = Rifiuta tutte
agent-done = Terminato con codice { $code }. Invio chiude la scheda.
run-bad-arg = idetea run: "{ $arg }" non è valido.
run-usage = Uso: idetea run [--title T] [--name nome] [--cwd cartella] [--worktree ramo] [--keep] [--no-wait] [--timeout secondi] -- <comando>; idetea run --wait <nome>...; idetea run --stop <nome>
run-busy = idetea run: c'è già una richiesta "{ $name }" (o il suo stato); scegli un altro --name oppure attendi con idetea run --wait { $name }
run-asked = idetea run: la richiesta "{ $name }" è nel pannello del terminale; in attesa che tu la consenta e che il comando finisca
run-exited = idetea run: "{ $name }" è terminato con codice { $code }
run-refused = idetea run: hai rifiutato "{ $name }"
run-invalid = idetea run: la richiesta "{ $name }" non è valida: { $error }
run-ended = idetea run: "{ $name }" è terminato senza un codice ({ $state })
run-branch = idetea run: il lavoro è sul ramo { $branch }
run-timeout = idetea run: il tempo è scaduto e "{ $name }" è ancora nel terminale (idetea run --wait { $name } attende di nuovo; --stop lo chiude)

## Stato delle schede (RFC 0024)
terminal-state-running = In esecuzione
terminal-state-unseen = Nuovo output dall'ultima volta che l'hai visto
terminal-state-attention = Chiede la tua attenzione
terminal-state-exited = Terminato con il codice { $code }
terminal-state-bell = Ha suonato il campanello
terminal-state-turn-done = Ha finito il suo turno e ti aspetta
cmd-terminal-next = Terminale: scheda successiva
cmd-terminal-prev = Terminale: scheda precedente
cmd-terminal-goto = Terminale: vai a…
cmd-terminal-next-attention = Terminale: prossima scheda che ti aspetta
cmd-terminal-rename = Terminale: rinomina la scheda
terminal-no-attention = Nessuna scheda del terminale chiede attenzione.
terminal-goto-placeholder = Vai alla scheda del terminale…
terminal-rename-placeholder = Nome della scheda
terminal-rename-reset = Torna al nome predefinito
terminal-rename-to = Rinomina in
terminal-close-running-title = Chiudere la scheda?
terminal-close-running = { $name } è ancora in esecuzione. Chiudere la scheda termina il processo.
terminal-close-running-yes = Chiudi e termina
terminal-close-running-no = Annulla
cmd-terminal-find = Terminale: cerca
terminal-find-placeholder = Cerca nel terminale (Invio: più vecchio)
terminal-find-older = Più vecchio (su)
terminal-find-newer = Più recente (giù)
terminal-find-none = Nessun risultato
terminal-find-close = Chiudi la ricerca
cmd-terminal-split = Terminale: dividi (o unisci)
cmd-terminal-resume = Terminale: riprendi gli agenti della sessione precedente
terminal-resume-nothing = Nessun agente della sessione precedente in attesa.
terminal-resume-title = Riprendere gli agenti?
terminal-resume-message = { $count ->
    [one] Un agente era aperto quando l’IDE si è chiuso: { $names }.
   *[other] { $count } agenti erano aperti quando l’IDE si è chiuso: { $names }.
}
terminal-resume-all = Riprendi tutti
terminal-resume-pick = Scegli…
terminal-resume-none = No
terminal-resume-pick-placeholder = Agente da riprendere…
terminal-resume-same = stessa conversazione
terminal-resume-new = nuova conversazione
terminal-resume-failed = Impossibile riprendere la conversazione di { $title } (codice { $code }). Aprine una nuova con «Agenti: avvia…» (Ctrl+Shift+A).
terminal-previous-session = — sessione precedente, { $when } —
agent-request-ignored = Campi che questa versione non conosce (ignorati): { $fields }
