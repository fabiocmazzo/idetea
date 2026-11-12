## Git
cmd-git-focus = Git: mostra le modifiche
cmd-git-refresh = Git: aggiorna
cmd-git-stage-all = Git: aggiungi tutto a stage
cmd-git-unstage-all = Git: rimuovi tutto da stage
cmd-git-commit = Git: esegui commit (mette a fuoco il messaggio)
cmd-git-pull = Git: pull (scarica e unisci)
cmd-git-push = Git: push (invia i commit)
cmd-git-fetch = Git: recupera (controlla le novità sul remoto)
git-panel = Git
cancel = Annulla
copied = Copiato: { $text }
git-notice = Git: { $message }
git-wait = Git: attendi, ancora { $running }
git-fetching = Recupero…
git-pulling = Pull…
git-pushing = Push…
git-publishing = Pubblicazione…
git-no-repo-notice = Git: questa cartella non è in un repository
git-no-repo = Questa cartella non è in un repository git.
git-need-message = Git: scrivi il messaggio di commit
git-nothing-staged = Git: niente in stage (usa + per aggiungere)
git-detached-pull = Git: HEAD scollegato: passa a un ramo per il pull
git-detached-push = Git: HEAD scollegato: passa a un ramo per il push
git-no-upstream = Git: questo ramo non segue un remoto (push per pubblicarlo)
git-no-remote = Git: il repository non ha un remoto (git remote add origin <url>)
git-discarded = modifiche ignorate in { $path }
git-discard-untracked = Il file è nuovo (non tracciato) e va nel cestino.
git-discard-tracked = Le modifiche non in stage andranno perse; il file torna a quanto è in stage o nell'ultimo commit.
git-discard-title = Ignorare le modifiche in "{ $path }"?
git-discard = Ignora
git-publish-title = Pubblicare il ramo "{ $branch }"?
git-publish-detail = Non esiste ancora su { $remote }. Il push crea lì il ramo e inizia a seguirlo (git push -u { $remote } { $branch }).
git-publish = Pubblica
git-fetch-tip = Recupera: controlla le novità sul remoto
git-pull-tip = Pull: scarica e unisci{ $count }
git-push-tip = Push: invia i commit{ $count }
git-refresh = Aggiorna
git-commit-placeholder = Messaggio di commit (Invio)
git-staged = In stage
git-changes = Modifiche
git-unstage-all = Rimuovi tutto da stage
git-stage-all = Aggiungi tutto a stage
git-unstage = Rimuovi da stage
git-stage = Aggiungi a stage
git-nothing-to-commit = Niente da committare.
git-open-file = Apri file
git-view-diff = Vedi diff
git-discard-menu = Ignora modifiche…
git-copy-path = Copia percorso
git-copy-relative = Copia percorso relativo
git-select-file = Seleziona un file per vederne il diff.
git-diff-truncated = … diff tagliato a { $count } righe
git-no-diff = Nessuna differenza (file binario o solo i permessi).
git-run-error = impossibile eseguire git: { $error }
git-unknown-error = errore sconosciuto
git-trash-error = impossibile spostare nel cestino: { $error }
git-timeout = tempo scaduto ({ $seconds } s) nel parlare col remoto
git-explain-rejected = il remoto ha commit che non hai: fai prima il pull
git-explain-diverged = i rami locale e remoto sono divergenti: imposta pull.rebase (oppure risolvi nel terminale)
git-explain-conflict = pull con conflitti: risolvi i file segnati ed esegui il commit
git-explain-overwritten = le modifiche locali verrebbero sovrascritte: esegui il commit o ignorale prima del pull
git-explain-auth = autenticazione non riuscita: configura un credential helper (es. gh auth setup-git)
git-explain-ssh = ssh ha rifiutato la connessione: carica la chiave nell'agent (ssh-add) oppure conferma il server nel terminale
git-explain-network = impossibile raggiungere il remoto (rete o indirizzo)
git-fetch-done = recupero completato
git-up-to-date = già aggiornato
git-pull-done = pull completato
git-published = ramo { $branch } pubblicato su { $remote }
git-nothing-to-push = niente da inviare con push
git-push-done = push completato

## Rami
cmd-git-checkout = Git: cambia ramo
git-checkout-placeholder = Passa al ramo… (oppure scrivi un nome nuovo)
git-create-branch = Crea ramo
git-branch-current = attuale · { $date }
git-switched = Ora sul ramo { $branch }
git-switch-failed = Impossibile passare a { $branch }: { $error }
git-explain-switch-overwritten = le modifiche non committate sono in conflitto con l'altro ramo: esegui il commit o ignorale prima

## Blame
git-blame-uncommitted = Tu · non ancora nel commit
git-blame-detail = 
    { $author }, { $ago }
    Commit { $hash }
git-blame-copy = Copia hash
git-close = Chiudi
git-ago-now = proprio ora
git-ago-minutes = 
    { $count ->
        [one] { $count } minuto fa
       *[other] { $count } minuti fa
    }
git-ago-hours = 
    { $count ->
        [one] { $count } ora fa
       *[other] { $count } ore fa
    }
git-ago-days = 
    { $count ->
        [one] { $count } giorno fa
       *[other] { $count } giorni fa
    }
git-ago-months = 
    { $count ->
        [one] { $count } mese fa
       *[other] { $count } mesi fa
    }
git-ago-years = 
    { $count ->
        [one] { $count } anno fa
       *[other] { $count } anni fa
    }
