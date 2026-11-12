ui-panel = Interfacce
cmd-ui-show = Interfacce: mostra il pannello
ui-empty = Nessuna interfaccia aperta. Un agente può crearne una scrivendo un file in .idetea/ui/ (vedi docs/interfaces/formato.md). Per attivarla o disattivarla: [extensions] interfaces = {"{"} enabled = true {"}"} nella configurazione.
ui-tracked = { $path } è tracciato da git ed è stato ignorato per sicurezza: le interfacce arrivano solo dall'agente in esecuzione su questa macchina, mai da un repository clonato.
ui-invalid = Documento non valido: { $error }
ui-origin = { $path }
ui-origin-source = { $path } · { $source }
ui-waiting = { $count ->
    [one] { $count } risposta inviata; in attesa che l'agente la legga.
   *[other] { $count } risposte inviate; in attesa che l'agente le legga.
}
ui-received = Risposta ricevuta dall'agente.
ui-agent-waiting = L'agente aspetta la risposta.
ui-agent-stopped = { $minutes ->
    [0] L'agente ha appena smesso di aspettare: la risposta resta finché non ricontrolla.
   *[other] L'agente ha smesso di aspettare { $minutes } min fa: la risposta resta finché non ricontrolla.
}
ui-new = Nuova interfaccia dall'agente: { $title }
ui-updated = Interfaccia aggiornata: { $title }
ui-open-later = L'agente ha chiesto di aprire { $path }: è nel pannello Interfacce (non aperta, per non interrompere la scrittura).
ui-write-failed = Impossibile scrivere la risposta: { $error }
ui-prepare-failed = Impossibile preparare la cartella .idetea/ui: { $error }
ui-not-a-file = non è un file normale
ui-surface-mismatch = il campo surface è "{ $found }", ma il file si chiama { $expected }.json
cmd-ui-teach = Interfacce: insegna agli agenti di questo progetto
ui-teach-hint = Perché un agente di codice sappia usare questo pannello, scrivi le istruzioni nell'AGENTS.md del progetto.
ui-teach-button = Insegna agli agenti di questo progetto
ui-teach-title = Insegnare agli agenti di questo progetto?
ui-teach-detail = L'IDE scriverà le istruzioni del pannello Interfacce in: { $files }. Il testo resta fra i marcatori; rieseguirlo aggiorna solo quella parte e lascia il resto. Sono file del progetto: rivedili prima del commit.
ui-teach-file-new = { $name } (nuovo)
ui-teach-write = Scrivi
ui-cancel = Annulla
ui-teach-up-to-date = Le istruzioni per gli agenti sono già aggiornate.
ui-teach-done = Istruzioni scritte in { $files }.
ui-teach-failed = Impossibile scrivere le istruzioni: { $error }
cmd-ui-new-project = Progetto: nuovo…
new-title = Nuovo progetto
new-intro = L'IDE crea la cartella, git, le skill del modello e le istruzioni del pannello, e apre il progetto in una nuova finestra. L'agente crea il progetto vero, seguendo la richiesta iniziale in **AGENTS.md**. Con un agente scelto sotto, la nuova finestra chiede di aprirlo in un terminale dell'IDE (confermi il comando).
new-field-parent = Crea dentro
new-field-name = Nome della cartella
new-field-name-hint = es. my-app
new-field-model = Modello
new-field-description = Che cos'è il progetto (facoltativo)
new-field-description-hint = una frase; va nella richiesta iniziale per l'agente
new-field-git = Crea il repository git
new-field-agent = Agente da aprire in un terminale dell'IDE
new-agent-none = Nessuno (lo apro io)
new-agent-title = Agente: { $agent }
new-bad-agent = nome agente non valido: "{ $agent }" (solo lettere, cifre, - e _)
new-create = Crea e apri
new-model = { $model ->
    [vazio] Vuoto (l'agente chiede di che si tratta)
    [rust] Rust
    [site] Sito web
    [python] Python
    [fastapi] FastAPI (API web Python)
   *[other] { $model }
}
new-creating = Creazione di { $dir } (le skill arrivano dalla rete)…
new-created = Progetto creato in { $dir }; apertura in una nuova finestra.
new-open-failed = Progetto creato in { $dir }, ma la finestra non si è aperta: { $error }
new-failed = Impossibile creare il progetto: { $error }
new-bad-model = modello sconosciuto: { $model }
new-bad-dir = cartella non valida: { $dir } (usa un percorso completo)
new-bad-name = nome cartella non valido: "{ $name }"
new-not-empty = { $dir } esiste già e contiene file: scegli un altro nome
new-done-dir = cartella: { $dir }
new-done-git = git: repository creato
new-done-agents = AGENTS.md (con la richiesta iniziale) e CLAUDE.md, con le istruzioni del pannello
new-done-agent = richiesta di terminale per { $agent }: confermala nella nuova finestra
new-open-hint = Fatto. Aprilo con: idetea { $dir }
new-usage = idetea new: "{ $arg }" non è valido. Uso: idetea new <cartella> [--modelo id] [--descricao testo] [--sem-git] [--agente nome]; modelli: { $models }
cmd-ui-fix = Interfacce: correggi questo progetto
ui-fix-title = Correggere questo progetto?
ui-fix-will = L'IDE farà:
ui-fix-wont = Tanto per sapere (lasciato così):
ui-fix-button = Correggi
ui-fix-nothing = Niente da correggere nelle interfacce di questo progetto.
ui-fix-done = { $count ->
    [one] Corretto: { $count } elemento.
   *[other] Corretti: { $count } elementi.
}
ui-fix-failed = Impossibile correggere: { $error }
ui-fix-hint = Per correggere le voci segnate con "-": idetea ui fix --apply
ui-fix-usage = idetea ui fix: argomento sconosciuto "{ $arg }" (uso: idetea ui fix [--apply])
ui-fix-abandoned = elimina { $path }: nessuno aspetta più la risposta da { $since }
ui-fix-unread = { $path }: { $count ->
    [one] { $count } risposta che l'agente non ha letto
   *[other] { $count } risposte che l'agente non ha letto
} (tenute; chiedi all'agente di leggerle, oppure eliminale a mano)
ui-fix-leftover = elimina { $path }: resto di una scrittura o di un'attesa non finita
ui-fix-check = riscrivi .idetea/ui/check: chiama { $target }, che non esiste più
ui-fix-instructions-older = riscrivi le istruzioni in { $file }: sono alla versione { $found }, più vecchia di questo IDE
ui-fix-instructions-unversioned = riscrivi le istruzioni in { $file }: vengono da un IDE vecchio, senza la riga di versione
ui-fix-instructions-newer = { $file } ha istruzioni di un IDE più nuovo ({ $found }): apri il progetto lì (non torno indietro)
ui-outdated = Questo IDE non è aggiornato (è stato ricompilato dopo l'apertura): le risposte lo dicono all'agente. Riquadro comandi: "Riavvia l'IDE".
ui-audio-failed = Impossibile riprodurre il suono: { $error }
ui-check-ok = { $file }: ok ({ $count ->
        [one] { $count } componente
       *[other] { $count } componenti
    })
ui-check-error = { $file }: errore: { $error }
ui-check-warning = avviso: { $warning }
ui-check-media = media: { $path }: { $problem }
ui-check-name = nome file non valido ("{ $name }"): lettere minuscole, cifre, _ e -, che finisce in .json
ui-check-usage = uso: idetea ui check <file.json>…
ui-check-instructions-older = istruzioni: { $file } ha la versione { $found } e questo IDE ha { $ours }: chiedi di eseguire "Interfacce: insegna agli agenti di questo progetto" e di rileggere il file prima di pubblicare
ui-check-instructions-newer = istruzioni: { $file } ha la versione { $found }, da un IDE più nuovo di questo ({ $ours }): il pannello aperto potrebbe non capire che cosa insegna il file
ui-check-instructions-unversioned = istruzioni: { $file } ha un blocco vecchio, senza la riga di versione; questo IDE ha { $ours }: chiedi di eseguire "Interfacce: insegna agli agenti di questo progetto" e di rileggere il file prima di pubblicare
ui-teach-newer-file = { $name } (versione { $version })
ui-teach-newer = Attenzione: { $files } è stato scritto da un IDE più nuovo di questo (che ha la versione { $ours } e non conosce quello). Scrivere riporta le istruzioni alla versione di questo IDE; di solito conviene aprire il progetto con l'IDE più nuovo.
ui-teach-write-anyway = Scrivi comunque
ask-title = Domanda
ask-choose = Scegli
ask-ok = Conferma
ask-yes = Sì
ask-no = No
ask-not-object = il documento deve essere un oggetto JSON
ask-bad-arg = idetea ask: "{ $arg }" non è valido.
ask-usage = Uso: idetea ask <document.json | -> | --text T [--options "a|b|c" | --confirm | --input Etichetta] [--title T] [--name nome] [--value /pointer] [--json] [--keep] [--timeout secondi]; idetea ask --resume <nome>; idetea ask --close <nome>
ask-invalid = idetea ask: il pannello rifiuterebbe la domanda: { $error }
ask-no-such = idetea ask: non c'è la domanda "{ $name }" nel pannello
ask-timeout = idetea ask: nessuna risposta in tempo; la domanda "{ $name }" resta nel pannello (idetea ask --resume { $name } attende di nuovo)
ask-outdated = idetea ask: la risposta viene da un IDE ricompilato dopo l'apertura (ide.outdated); se stai provando una modifica, chiedi di riavviare l'IDE
ask-comment = commento: { $comment }

# Cronologia delle Interfacce (RFC 0007)
ui-history-panel = Cronologia delle Interfacce
cmd-ui-history = Interfacce: cronologia
cmd-ui-clear-history = Interfacce: cancella la cronologia
ui-history-failed = Impossibile scrivere nella cronologia: { $error }
ui-history-cleared = { $count ->
    [one] { $count } voce rimossa dalla cronologia.
   *[other] { $count } voci rimosse dalla cronologia.
}
ui-history-link = Cronologia ({ $count })
ui-history-empty = La cronologia è vuota.
ui-history-clear-title = Cancellare la cronologia?
ui-history-clear-detail = { $count ->
    [one] Verrà eliminata { $count } voce della cronologia di questo progetto (.idetea/history/).
   *[other] Verranno eliminate { $count } voci della cronologia di questo progetto (.idetea/history/).
} Le interfacce aperte nel pannello restano. Non si può annullare.
ui-history-clear-button = Cancella la cronologia
ui-history-search = Cerca titoli, risposte e commenti
ui-history-open = Aperte ({ $count })
ui-history-waiting-you = in attesa di te
ui-history-answered-open = risposta data, ancora aperta
ui-history-today = Oggi
ui-history-yesterday = Ieri
ui-history-none = Ancora nessuna domanda con risposta. Ogni risposta che dai nel pannello Interfacce resta qui (in .idetea/history/, fuori da git).
ui-history-no-match = Nessun risultato.
ui-history-back = ← Indietro
ui-history-shown = Mostrata: { $time }
ui-history-answered = Risposta: { $time }
ui-history-closed = Chiusa senza risposta: { $time }
ui-history-read = Letta dall'agente: { $time }
ui-history-button = Pulsante: { $action }
ui-history-not-read = L'agente non ha letto questa risposta (il file è ancora in .idetea/ui/responses/, oppure è stato eliminato con l'IDE chiuso).
ui-history-not-read-short = non letta
ui-history-unanswered = nessuna risposta
ui-history-media-changed = { $path } è cambiato o non c'è più dalla risposta: qui vedi la versione attuale.
ui-history-no-audio = la cronologia non riproduce i suoni
history-cli-comment = commento
history-cli-not-read = l'agente non ha letto questa risposta
history-cli-unanswered = nessuna risposta (l'agente ha eliminato la domanda)
history-cli-bad = idetea history: "{ $arg }" non è valido.
history-cli-usage = Uso: idetea history [--name nome] [--since YYYY-MM-DD] [--last N | --all] [--json]
history-cli-empty = idetea history: niente nella cronologia di questo progetto.
