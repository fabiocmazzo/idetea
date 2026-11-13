## Impostazioni → Aspetto

settings-theme = Tema
settings-theme-detail = Colori dell'IDE, dell'editor e del terminale.
settings-font-size = Dimensione del carattere
settings-font-size-detail = Editor e terminale.
settings-code-font = Tipo di carattere del codice
settings-code-font-detail = Editor, numeri di riga e terminale. JetBrains Mono è incluso nell'IDE; l'elenco ha i tipi di carattere a spaziatura fissa installati.
settings-font-weight = Spessore del carattere
settings-font-weight-detail = Uno spessore che il tipo di carattere non ha usa quello più vicino.
settings-line-height = Altezza della riga
settings-line-height-detail = In multipli della dimensione del carattere. Il terminale segue nella stessa proporzione.
settings-file-icons = Icone dei file
settings-file-icons-detail = Un'icona per ogni linguaggio o tipo di file (il set Seti di VS Code). Disattivate: la stessa icona per ogni file.
settings-activity-bar = Barra attività
settings-activity-bar-detail = Striscia sul bordo sinistro per mostrare/nascondere i pannelli con un clic.
settings-splash = Schermata iniziale all'avvio
settings-splash-detail = Schermata di avvio (almeno 2 s). --no-splash la disattiva per un'esecuzione.
settings-language = Lingua
settings-language-detail = Testo dell'interfaccia. Automatico segue la lingua del sistema. Ha effetto dopo il riavvio dell'IDE.
settings-language-auto = Automatico ({ $language })
settings-language-restart = Ha effetto dopo il riavvio dell'IDE (ora: { $language }).

## Comandi di base (titolo nel riquadro e nella guida)
cmd-palette-open = Riquadro comandi
cmd-help-open = Guida: scelte rapide e guida
cmd-settings-open = Impostazioni
cmd-file-new = File: nuovo
cmd-file-save = File: salva
cmd-file-open = File: apri…
cmd-file-quick-open = File: apri per nome…
cmd-debug-continue = Debug: avvia / continua
cmd-debug-stop = Debug: interrompi
cmd-debug-pause = Debug: sospendi
cmd-debug-next = Debug: riga successiva
cmd-debug-step-in = Debug: entra nella funzione
cmd-debug-step-out = Debug: esci dalla funzione
cmd-debug-toggle-breakpoint = Debug: attiva/disattiva il punto di interruzione sulla riga
cmd-debug-choose-program = Debug: scegli il programma…
cmd-file-open-folder = File: apri cartella in una nuova finestra…
cmd-file-save-as = File: salva con nome…
cmd-file-save-as-path = File: salva con nome (scrivi il percorso)…
cmd-file-close = File: chiudi scheda
cmd-tab-next = Schede: successiva
cmd-tab-prev = Schede: precedente
cmd-tab-reopen = Schede: riapri l'ultima chiusa
cmd-editor-find = Editor: cerca
cmd-editor-replace = Editor: cerca e sostituisci
cmd-editor-goto-line = Editor: vai alla riga
cmd-editor-undo = Editor: annulla
cmd-editor-redo = Editor: ripeti
cmd-editor-select-all = Editor: seleziona tutto
cmd-editor-fold-all = Editor: riduci tutti i blocchi
cmd-editor-unfold-all = Editor: espandi tutti i blocchi
cmd-editor-toggle-fold = Editor: riduci o espandi il blocco al cursore
cmd-editor-toggle-vim = Editor: attiva/disattiva la modalità vim
cmd-editor-zoom-in = Editor: aumenta zoom
cmd-editor-zoom-out = Editor: riduci zoom
cmd-editor-zoom-reset = Editor: reimposta zoom
cmd-lsp-definition = LSP: vai alla definizione
cmd-lsp-references = LSP: trova i riferimenti
cmd-lsp-implementation = LSP: vai all'implementazione
cmd-lsp-type-definition = LSP: vai alla definizione del tipo
cmd-lsp-incoming-calls = LSP: chi chiama questa funzione
cmd-lsp-outgoing-calls = LSP: che cosa chiama questa funzione
cmd-nav-file-symbol = Vai al simbolo nel file…
cmd-nav-project-symbol = Vai al simbolo nel progetto…
cmd-nav-back = Navigazione: indietro
cmd-nav-forward = Navigazione: avanti
cmd-lsp-hover = LSP: mostra la documentazione al cursore
cmd-lsp-format = LSP: formatta il file
cmd-lsp-restart = LSP: riavvia i server
cmd-theme-toggle = Tema: alterna chiaro/scuro
cmd-config-open = Config: apri la config globale
cmd-config-open-project = Config: apri la config del progetto
cmd-config-reload = Config: ricarica
cmd-view-toggle-left = Layout: mostra/nascondi il lato sinistro
cmd-view-toggle-right = Layout: mostra/nascondi il lato destro
cmd-view-toggle-bottom = Layout: mostra/nascondi il pannello inferiore
cmd-app-restart = Riavvia l'IDE
cmd-app-quit = Esci
cmd-mascot-toggle = Mascotte: mostra/comprimi nell'angolo

## Avvisi, finestre e menu
file-new-created-on-save = Nuovo file: { $path } (creato al salvataggio)
file-open-error = Errore nell'aprire { $path }: { $error }
file-changed-on-disk-kept = "{ $name }" è cambiato su disco, ma la scheda ha modifiche: è stata tenuta
file-reload-error = Errore nel ricaricare { $path }: { $error }
file-changed-on-disk-overwrite = "{ $name }" è cambiato su disco, ma la scheda ha modifiche: il salvataggio lo sovrascrive
debug-still-building = Debug: compilazione ancora in corso…
debug-already-running = Debug: il programma è già in esecuzione
debug-building = Debug: compilazione ({ $command })…
unknown-error = errore sconosciuto
debug-build-failed = Debug: la compilazione non è riuscita: { $error }
debug-choose-title = Quale programma sottoporre a debug?
debug-choose-message = Il progetto ha compilato più di un eseguibile.
cancel = Annulla
debug-running = Debug: in esecuzione { $program }
debug-error = Debug: { $error }
debug-cancelled = Debug: annullato (la compilazione finisce da sola)
debug-stopped = Debug: interrotto
debug-no-source = Debug: { $name } non ha codice sorgente
debug-paused-no-source = Debug: in pausa in { $name } ({ $reason }), senza codice sorgente
debug-paused-at = Debug: in pausa a { $path }:{ $line } ({ $reason })
debug-exited = Debug: il programma è uscito
debug-exited-code = Debug: il programma è uscito (codice { $code })
debug-command-failed = Debug: { $command } non è riuscito: { $message }
debug-gdb-exited = Debug: gdb è uscito ({ $reason })
debug-no-session = Debug: nessuna sessione in corso (F5 ne avvia una)
debug-already-paused = Debug: il programma è già in pausa
debug-pause-first = Debug: il programma è in esecuzione (sospendilo prima)
nav-nothing-forward = Niente a cui andare avanti
nav-nothing-back = Niente a cui tornare indietro
lsp-no-references = Nessun riferimento trovato
lsp-no-definition = Definizione non trovata
lsp-no-implementations = Nessuna implementazione trovata
lsp-no-type = Tipo non trovato
lsp-not-used-elsewhere = "{ $symbol }" non si usa da nessun'altra parte
outline-save-first = Salva il file per vederne la struttura
outline-reading = Lettura della struttura…
symbols-type-a-name = Scrivi il nome di una funzione, un tipo, un metodo…
searching = Ricerca…
symbols-none-found = Nessun simbolo trovato (se il progetto sta ancora caricando, riprova)
symbols-none-in-file = Nessun simbolo in questo file
lsp-save-first = salva il file per usare il server di linguaggio
lsp-finding-references = Ricerca dei riferimenti…
lsp-finding-implementations = Ricerca delle implementazioni…
calls-not-in-function = Il cursore non è in una funzione (o il server non ne conosce le chiamate)
lsp-formatted = File formattato
lsp-level-error = errore:{" "}
lsp-level-warning = avviso:{" "}
lsp-server-stopped = { $server } si è fermato — { $reason }. Usa "LSP: riavvia i server".
lsp-server-crashed = { $server } è andato in crash — { $reason }. Torna da solo quando smetti di scrivere.
lsp-server-crashed-too-often = { $server } è andato in crash { $count } volte in { $minutes } minuti — { $reason }. Usa "LSP: riavvia i server".
lsp-no-formatting = La formattazione non è disponibile per questo file
lsp-restarted = Server di linguaggio riavviati
save-changes-title = Salvare le modifiche?
save-changes-message = "{ $name }" ha modifiche non salvate.
save = Salva
discard = Ignora
picker-already-open = Una finestra dei file è già aperta
save-tab-closed = La scheda è stata chiusa prima del salvataggio
folder-opening = Apertura di { $path } in una nuova finestra
folder-open-error = Impossibile aprire la cartella: { $error }
saved = Salvato: { $path }
save-error = Errore nel salvataggio: { $error }
file-deleted-kept = File eliminato su disco; la scheda con le modifiche è stata tenuta
and-more = … e altri { $count }
quit-unsaved-title = Uscire con modifiche non salvate?
quit-save-all = Salva tutto ed esci
quit-without-saving = Esci senza salvare
save-needs-name = "{ $name }" non ha un nome: usa "Salva con nome"
save-file-error = Errore nel salvare { $path }: { $error }
close-tabs-title = 
    { $count ->
        [one] Chiudere { $count } scheda?
       *[other] Chiudere { $count } schede?
    }
close-tabs-unsaved = Con modifiche non salvate:
save-and-close = Salva e chiudi
discard-changes = Ignora modifiche
tab-close = Chiudi
tab-close-others = Chiudi le altre
tab-close-right = Chiudi a destra
tab-close-saved = Chiudi le salvate
tab-close-all = Chiudi tutte
copy-path = Copia percorso
copy-relative-path = Copia percorso relativo
reveal-in-tree = Mostra nell'albero dei file
open-in-file-manager = Apri nel gestore file
copied = Copiato: { $text }
file-manager-error = Impossibile aprire il gestore file: { $error }
save-then-close = Scegli dove salvare; poi chiudi di nuovo la scheda
config-reloaded-problems = Config ricaricata con problemi: { $problems }
config-reloaded = Config ricaricata (attivare o disattivare le estensioni ha effetto solo dopo il riavvio)
config-no-home = Impossibile trovare la cartella della config (HOME non è impostato)
create-error = Errore nella creazione di { $path }: { $error }
config-error = Config: { $error }
write-error = Errore nella scrittura di { $path }: { $error }
unknown-command = Comando sconosciuto: { $id }
keys-record-hint = Usa Ctrl, Alt o un tasto speciale (F1–F12, PageUp…). Esc annulla.
keys-already-bound = { $key } era già "{ $command }"
keys-rebound = { $key } era "{ $old }"; ora è "{ $command }"
keys-bound = { $key } → "{ $command }"
theme-unknown = tema sconosciuto "{ $name }", uso "dark"
default = predefinito

## Impostazioni
settings-title = Impostazioni
settings-tab-appearance = Aspetto
settings-tab-extensions = Estensioni
settings-tab-shortcuts = Scelte rapide
close-esc = Chiudi (Esc)
settings-no-home = (HOME non è impostato)
settings-write-to = Salva in
settings-target-global = Globale
settings-target-project = Progetto
settings-open-file = Apri file
keys-filter = Filtra per nome, comando o tasto…
keys-press = Premi i tasti… (Esc annulla)
keys-add = Aggiungi scelta rapida
keys-help = Clic su un tasto per toglierlo. + registra un tasto nuovo (vale anche per i tasti già in uso: cambiano proprietario).
settings-lsp = Server di linguaggio
settings-lsp-detail = Attiva o disattiva LSP nel suo insieme.
settings-servers = Server
settings-server-found = trovato nel PATH
settings-server-not-found = non trovato nel PATH
settings-lsp-note = Le modifiche valgono per i server avviati da ora in poi.
settings-restart-servers = Riavvia i server
theme-dark = Scuro (predefinito)
theme-light = Chiaro
slot-left = Sinistra
slot-right = Destra
slot-bottom = In basso
slot-status = Barra di stato
settings-on-restart = ha effetto dopo il riavvio dell'IDE
default-capitalized = Predefinito
settings-extensions-note = Attivare o disattivare un'estensione ha effetto dopo il riavvio. Spostare un pannello su un altro lato si applica subito.
ext-filetree = Albero dei file
ext-statusbar = Barra di stato
ext-search = Cerca nel progetto
ext-problems = Errori e avvisi
ext-terminal = Terminale integrato
ext-git = Modifiche, stage, commit e diff
ext-debug = Pulsanti di debug, stack, variabili e output
ext-references = Dove si usa un nome (Maiusc+F12)
ext-outline = Funzioni e metodi del file, con visibilità (Alt+7)
ext-preview = Anteprima Markdown e SVG accanto all'editor (Ctrl+Maiusc+V)
ext-calls = Chi chiama una funzione, e che cosa chiama (Alt+Maiusc+H)
ext-interfaces = Interfacce che un agente scrive in .idetea/ui/ (sperimentale)

## Riquadro, finestre, LSP, debug e il resto
palette-commands = Scrivi un comando…
palette-save-as = Percorso del file (relativo alla radice del progetto)
palette-files = Nome del file (file:riga va alla riga, > per i comandi)
palette-file-symbols = Funzione, tipo, metodo… in questo file
palette-project-symbols = Funzione, tipo, metodo… nel progetto
palette-no-commands = Nessun comando trovato
palette-save-hint = Invio salva · Esc annulla
palette-reading-files = Lettura dei file del progetto…
palette-no-files = Nessun file trovato
palette-no-symbols = Nessun simbolo
palette-no-matching-symbols = Nessun simbolo corrisponde alla ricerca
dialog-hint = Invio conferma · Esc annulla
lsp-command-not-found = { $server }: comando "{ $command }" non trovato nel PATH (installalo o sistema [lsp.servers.{ $server }] nella config)
lsp-start-error = { $server }: impossibile avviare: { $error }
lsp-disabled = LSP è disattivato nella config ([lsp] enabled = false)
lsp-no-server-for = nessun server di linguaggio configurato per i file .{ $ext }
lsp-not-running = { $server } non è in esecuzione: { $reason }
lsp-not-ready = { $server } non è ancora pronto
lsp-hint-rustup-install = { $reason }. Installalo con: rustup component add rust-analyzer
lsp-hint-rustup = { $reason }. Con rustup: rustup component add rust-analyzer
lsp-hint-ziex = { $reason }. Installa la CLI di Ziex (https://ziex.dev/learn) oppure apri la cartella del progetto, il cui build.zig.zon dipende da ziex: allora l'IDE compila il server con zig
lsp-hint-moonbit = { $reason }. Installa la toolchain MoonBit (https://www.moonbitlang.com/download), che include `moon lsp`; l'IDE cerca anche in ~/.moon/bin (o $MOON_HOME/bin)
lsp-hint-hare = { $reason }. hare-lsp non è incluso in Hare: compilalo da https://git.sr.ht/~whynothugo/hare-lsp e mettilo nel PATH (per esempio, ~/.local/bin)
config-syntax-error = sintassi { $file }: { $error }
visibility-file-only = solo questo file
picker-save-as = Salva con nome
picker-open-files = Apri file
picker-open-folder = Apri cartella
picker-debug-program = Programma da sottoporre a debug
splash-ready = pronto
splash-loading = caricamento…
watch-too-many = cartella troppo grande: si osservano solo le prime { $count } cartelle
watch-partial = impossibile osservare tutte le cartelle ({ $error })
watch-problem = Modifiche su disco: { $message }. Usa "Aggiorna" nell'albero dei file quando serve.
watch-failed = Modifiche su disco: impossibile osservare ({ $error })
debug-no-gdb = gdb non trovato nel PATH (il debug usa gdb 14 o più recente)
debug-program-not-found = programma non trovato: { $path }
debug-gdb-start-error = impossibile avviare gdb: { $error }
debug-reason-breakpoint = punto di interruzione
debug-reason-step = istruzione
debug-reason-pause = in pausa
debug-reason-exception = eccezione
debug-reason-entry = ingresso
debug-reason-goto = salto
build-run-error = impossibile eseguire "{ $command }": { $error }
build-no-message = fallito senza messaggio

## Guida e riga di comando
help-key-find = Cerca nel file
help-key-replace = Cerca e sostituisci
help-key-goto-line = Vai alla riga
help-key-undo = Annulla / ripeti
help-key-next-match = Seleziona l'occorrenza successiva (multi-cursore)
help-key-quick-fix = Azioni rapide LSP (se disponibili)
help-key-ctrl-click = Ctrl+clic
help-key-definition = Vai alla definizione (LSP)
help-key-tab = Inserisci una tabulazione
help-title = Guida
help-tab-shortcuts = Scelte rapide
help-tab-guide = Guida
help-tab-about = Informazioni
help-close = Chiudi (Esc)
help-filter = Filtra per nome, comando o tasto…
help-group-general = Generale
help-editor-keys = Editor (fisse)
help-in-palette = (riquadro comandi)
about-no-home = (HOME non impostato)
about-version = Versione
about-license = Licenza
about-license-value = { $license } (libera e aperta, sempre)
about-project = Progetto
about-global-config = Config globale
about-project-config = Config del progetto
about-built-with = Scritto in Rust con iced, iced-code-editor, alacritty_terminal (via iced_term) e syntect.
about-icons = Icone: Codicons © Microsoft, licenza CC BY 4.0.
cli-usage = 
    Uso: idetea [opzioni] [percorso...]
    
      percorso       file da aprire (può non esistere ancora: si crea al salvataggio)
                     o cartella da usare come radice del progetto
      --no-splash    non mostra la schermata iniziale (anche: IDETEA_NO_SPLASH=1)
      --menu         aperto dal menu: senza cartella, apre il progetto del file o l'ultimo progetto
      -f, --foreground  resta attaccato al terminale (altrimenti, avviato da un terminale, l'IDE
                     si stacca e i log vanno in ~/.local/state/idetea/idetea.log)
      --install-desktop    aggiunge l'IDE al menu delle applicazioni (con l'icona)
      --uninstall-desktop  lo rimuove dal menu delle applicazioni
      ui check <file.json>…  controlla i documenti del pannello Interfacce (come li legge il pannello)
      new <folder> [--modelo id]  crea un progetto nuovo: cartella, git, skill del modello e istruzioni
      skills [list|add|update|remove]  installa le skill degli agenti nel progetto (idetea skills --help)
      ui fix [--apply]  mostra (e, con --apply, corregge) che cosa è rimasto in .idetea/ui/
      ask …          chiede nel pannello Interfacce e attende la risposta (idetea ask da solo: uso)
      run … -- <command>  chiede un terminale all'IDE e attende la fine (idetea run da solo: uso)
      status [--json]  che cosa mostra l'IDE aperto in questo progetto (schede, modifiche, cursore)
      open <file[:line[:column]]>  l'IDE aperto mostra il file, con il cursore al suo posto
      history [--json]  che cosa hai risposto nel pannello Interfacce (--name, --since, --last N, --all)
      mascot <state> [--text "…"]  aggiorna la mascotte dell'IDE (idle, working, thinking, attention, celebrating, error, sleeping, clear, status)
      -h, --help     mostra questo aiuto
      -V, --version  mostra la versione
cli-unknown-option = opzione sconosciuta ignorata: { $option }
cli-extra-root = una sola cartella radice alla volta, si ignora { $path }

## Guida → Guida (`{ $comando-com-hifens }` diventa la scelta rapida del comando)
guide-start = Per iniziare
guide-start-1 = Apri l'IDE con `idetea` in una cartella (diventa la radice dell'albero) o con `idetea file...` (i file si aprono in schede; la radice resta la cartella corrente).
guide-start-2 = Ogni comando è nel riquadro: { $palette-open }. Scrivi parte del nome; la scelta rapida compare accanto.
guide-start-3 = Questa guida si apre con { $help-open }.
guide-start-4 = La barra delle icone sul bordo sinistro mostra e nasconde i pannelli con un clic (quello aperto è evidenziato). Trascina il bordo fra un pannello e l'editor per ridimensionarlo; doppio clic ripristina la misura predefinita.
guide-files = File e schede
guide-files-1 = Apri per nome: { $file-quick_open } (scrivi parti del nome; `main.rs:42` va alla riga; `>` passa ai comandi).
guide-files-2 = Vai a un simbolo (funzione, tipo, metodo…): { $nav-file_symbol } nel file, { $nav-project_symbol } in tutto il progetto. Serve il server di linguaggio.
guide-files-3 = Nuovo: { $file-new } · Apri: { $file-open } · Salva: { $file-save } · Salva con nome: { $file-save_as } · Chiudi scheda: { $file-close }.
guide-files-4 = Apri e Salva con nome usano la finestra di sistema. Per scrivere il percorso: { $file-save_as_path }. Aprire un'altra cartella ({ $file-open_folder }) apre una nuova finestra dell'IDE.
guide-files-5 = Anteprima Markdown e SVG: { $preview-toggle } (o il pulsante in fondo alla barra delle schede) apre un pannello laterale che segue il file attivo e si aggiorna mentre scrivi. I collegamenti web si aprono nel browser; quelli ai file, in una scheda.
guide-files-6 = Le immagini (png, jpg, gif, webp...) si aprono in una scheda propria: la rotella zooma e il trascinamento sposta.
guide-files-7 = Anche i PDF si aprono in una scheda: scorri le pagine; − e + cambiano lo zoom e "Adatta" segue la larghezza.
guide-files-8 = Cambia scheda: { $tab-next } e { $tab-prev } (Ctrl+Tab inserisce una tabulazione nell'editor).
guide-files-9 = Chiudere una scheda o uscire con modifiche non salvate chiede prima: Salva, Ignora o Annulla.
guide-files-10 = File cambiato fuori (terminale, un altro editor): la scheda si ricarica da sola se non ha modifiche non salvate; se le ha, resta com'è e compare un avviso.
guide-files-11 = Clic destro su una scheda: chiudi le altre, a destra, le salvate o tutte, copia il percorso, mostra nell'albero e apri nel gestore file. Il clic centrale chiude la scheda.
guide-tree = Albero dei file
guide-tree-1 = Mostra/nascondi: { $filetree-toggle } · Mostra il file corrente: { $filetree-reveal }.
guide-tree-2 = Clic destro su un elemento: nuovo file, nuova cartella, rinomina, elimina, copia percorso e apri nel gestore file. Elimina lo manda nel cestino di sistema.
guide-tree-3 = Colori git: giallo = modificato, verde = nuovo, rosso = conflitto, attenuato = ignorato. Una cartella con modifiche dentro mostra un punto.
guide-bottom = Pannello inferiore
guide-bottom-1 = Cerca nel progetto ({ $search-focus }), Problemi ({ $problems-focus }), Terminale ({ $terminal-focus }) e Git ({ $git-focus }) sono schede del pannello inferiore. { $view-toggle_bottom } mostra/nasconde il pannello.
guide-bottom-2 = Problema successivo: { $problems-next }.
guide-terminal = Terminale
guide-terminal-1 = Con il focus sul terminale, Ctrl+lettera, i tasti F e le frecce vanno alla shell (Ctrl+C, Ctrl+W...). Le scelte rapide dell'IDE con Ctrl+Maiusc continuano a funzionare, così il riquadro comandi si apre sempre.
guide-terminal-2 = Copia e incolla nel terminale: Ctrl+Maiusc+C e Ctrl+Maiusc+V. Scrivere `exit` chiude il pannello.
guide-debug = Debug
guide-debug-1 = Punto di interruzione: clic sul numero di riga o { $debug-toggle_breakpoint }. Avvia/continua: { $debug-continue }. Nei progetti Rust compila con cargo build e trova l'eseguibile da solo; per gli altri, usa [debug] nella config (program, args, cwd, build) oppure scegli l'eseguibile. Cambia programma: { $debug-choose_program }.
guide-debug-2 = In pausa: { $debug-next } riga successiva, { $debug-step_in } entra nella funzione, { $debug-step_out } esce, { $debug-pause } sospende, { $debug-stop } interrompe. La freccia gialla mostra dov'è il programma. Usa gdb 14+.
guide-debug-3 = Pannello Debug ({ $debug-focus }): pulsanti, stack di chiamate (il clic va al frame e mostra le sue variabili), variabili (il clic espande struct e vettori) e l'output del programma e della compilazione.
guide-git = Git
guide-git-1 = Il ramo compare nella barra di stato. Il pannello Git ({ $git-focus }) mostra che cosa è cambiato: + e − aggiungono e rimuovono dallo stage, il clic su un file mostra il diff, Invio nel messaggio esegue il commit.
guide-git-2 = Clic destro su un file in Git: apri, vedi il diff, aggiungi a stage, ignora modifiche (chiede conferma) e copia il percorso.
guide-git-3 = Recupera ({ $git-fetch }), pull ({ $git-pull }) e push ({ $git-push }) sono i pulsanti in alto nel pannello. Qui non si chiede la password: usa l'agent ssh (ssh-add) o un credential helper. Un ramo nuovo chiede conferma prima di essere pubblicato sul remoto.
guide-git-4 = Il pannello si aggiorna al salvataggio e quando il repository cambia (commit o checkout fatti nel terminale).
guide-lsp = Linguaggi (LSP)
guide-lsp-1 = I server di linguaggio partono da soli quando apri un file: rust-analyzer, clangd, pyright, typescript-language-server, gopls e lua-language-server (devono essere installati).
guide-lsp-2 = Vai alla definizione: { $lsp-definition } o Ctrl+clic · Documentazione: { $lsp-hover } o tieni il mouse sul codice · Formatta: { $lsp-format }.
guide-lsp-3 = Dove si usa: { $lsp-references }, o Ctrl+clic sulla dichiarazione stessa. L'elenco compare nel pannello Riferimenti; il clic va all'uso.
guide-lsp-4 = Implementazioni: { $lsp-implementation } (su un trait o un'interfaccia, o su uno dei suoi metodi). Il tipo di ciò che è sotto il cursore: "LSP: vai alla definizione del tipo" nel riquadro comandi. Con più di un risultato (anche per la definizione), l'elenco va nel pannello Riferimenti; con uno solo, ci si salta direttamente.
guide-lsp-5 = Chi chiama una funzione: { $lsp-incoming_calls } con il cursore su di essa. Il pannello Chiamate mostra l'albero (apri un livello per vedere chi chiama i chiamanti); "Chiamati" passa alle chiamate che fa. Il clic va alla chiamata.
guide-lsp-6 = Indietro e avanti dopo un salto (definizione, riferimento, apri per nome, clic nei pannelli): { $nav-back } e { $nav-forward }, o i tasti laterali del mouse.
guide-lsp-7 = Struttura del file ({ $outline-toggle }): funzioni, struct, metodi e campi in un albero. Il lucchetto aperto è pubblico, quello chiuso è privato e la chiave è visibile solo in parte del codice (pub(crate), protected, package). Il simbolo sotto il cursore è evidenziato; il clic ci va.
guide-lsp-8 = Se un server fallisce, la barra di stato dice perché; "LSP: riavvia i server" riprova.
guide-config = Impostazioni
guide-config-1 = La schermata delle impostazioni ({ $settings-open }) cambia tema, tipo di carattere, estensioni, scelte rapide e LSP, scrivendo nella config globale o in quella del progetto. Le modifiche si applicano subito.
guide-config-2 = C'è una config globale e una per progetto (`.idetea/config.toml`); vince quella del progetto. Per modificare il file direttamente: "Config: apri la config globale" o "Config: apri la config del progetto". Al salvataggio, tutto si ricarica.
guide-config-3 = Le scelte rapide stanno in [keys]: "ctrl+alt+t" = "terminal.focus". Un testo vuoto disattiva una scelta rapida predefinita. Gli id dei comandi si vedono nella scheda Scelte rapide.
guide-config-4 = Lingua: Impostazioni → Aspetto → Lingua (Automatico segue il sistema). Ha effetto dopo il riavvio dell'IDE.

## Sessione
settings-restore-session = Riapri com'era
settings-restore-session-detail = Quando un progetto si apre, tornano le sue schede, il cursore e i pannelli. idetea file... apre solo il file.

## Recenti
cmd-file-open-recent = File: apri recenti…
palette-recent = Progetto o file aperto in precedenza
palette-no-recent = Ancora niente di aperto in precedenza
guide-files-12 = Apri recenti: { $file-open_recent } elenca i progetti (si aprono in una nuova finestra) e i file aperti per ultimi, da ogni finestra.

## Recupera i non salvati
untitled-tab = senza titolo
recover-title = 
    { $count ->
        [one] L'IDE si è chiuso senza salvare { $count } file
       *[other] L'IDE si è chiuso senza salvare { $count } file
    }
recover-detail = Recupera apre le modifiche in schede, ancora non salvate (annulla torna a quanto c'è su disco). Più tardi chiede di nuovo la prossima volta.
recover = Recupera
recover-later = Più tardi
recovered = 
    { $count ->
        [one] { $count } file recuperato (non ancora salvato)
       *[other] { $count } file recuperati (non ancora salvati)
    }

## Rinomina simbolo e salva tutto
cmd-lsp-rename = LSP: rinomina simbolo…
cmd-file-save-all = File: salva tutto
save-all-untitled = Le schede senza titolo non sono state salvate (usa Salva con nome)
palette-rename = Nuovo nome
palette-rename-hint = Rinomina "{ $name }" in tutto il progetto · Invio mostra che cosa cambia · Esc annulla
rename-no-word = Metti il cursore su un nome per rinominarlo
rename-asking = Rinomina "{ $name }": richiesta al server…
rename-nothing = Niente da rinominare per "{ $name }"
rename-failed = Impossibile rinominare: { $message }
rename-title = Rinominare "{ $old }" in "{ $new }"?
rename-summary = 
    { $changes ->
        [one] { $changes } modifica
       *[other] { $changes } modifiche
    } in { $files ->
        [one] { $files } file:
       *[other] { $files } file:
    }
rename-detail = I file restano modificati nelle schede, non ancora salvati: Ctrl+Z annulla in ognuna, e "File: salva tutto" scrive tutto.
rename = Rinomina
renamed = 
    { $count ->
        [one] Rinominato in { $count } file (non ancora salvato)
       *[other] Rinominato in { $count } file (non ancora salvati)
    }

guide-lsp-9 = Rinomina: { $lsp-rename } con il cursore sul nome. Un'anteprima mostra quante modifiche e in quali file; una volta confermate, restano modificate nelle schede (Ctrl+Z annulla in ognuna) e { $file-save_all } le salva tutte.

## Sostituisci nel progetto
replaced = { $count } sostituzioni in { $files } file
replaced-failed = { $count } sostituzioni in { $files } file; { $failed } non si sono potute cambiare (permesso mancante o non è testo)

guide-bottom-3 = Sostituisci nel progetto: { $search-replace } (o ▸ accanto alla ricerca) apre "Sostituisci con". Ogni risultato mostra come diventerà; "Sostituisci" cambia la riga o il file, e "Sostituisci tutto" chiede prima. I file aperti nelle schede restano modificati, non salvati (Ctrl+Z annulla); gli altri vengono scritti direttamente su disco.

## Editor e salvataggio
format-save-timeout = Il server non ha formattato in tempo: salvato senza formattare

settings-tab-editor = Editor
settings-wrap = A capo automatico
settings-wrap-detail = Una riga più larga della finestra continua sotto, senza scorrimento orizzontale.
settings-whitespace = Mostra gli spazi
settings-whitespace-detail = Gli spazi si vedono come · e le tabulazioni come →.
settings-indent-guides = Guide di rientro
settings-indent-guides-detail = Linee verticali sottili a ogni livello di rientro.
settings-tab-size = Dimensione del rientro
settings-tab-size-detail = Quanti spazi inserisce Tab.
settings-insert-spaces = Rientro con spazi
settings-insert-spaces-detail = Disattivato: Tab inserisce un carattere di tabulazione. Un file già rientrato con tabulazioni le mantiene.
settings-trim = Elimina gli spazi finali al salvataggio
settings-trim-detail = Tranne in Markdown, dove due spazi finali vanno a capo.
settings-final-newline = Nuova riga finale al salvataggio
settings-final-newline-detail = Fa sì che l'ultima riga finisca con un a capo.
settings-format-on-save = Formatta al salvataggio
settings-format-on-save-detail = Ctrl+S formatta con il server di linguaggio prima di scrivere (attende fino a 2 s). Salva tutto non formatta.
settings-breadcrumbs = Breadcrumb
settings-breadcrumbs-detail = Sopra l'editor: il percorso del file e il simbolo dov'è il cursore. Clic su una cartella per mostrarla nell'albero, su un simbolo per saltarci.
settings-editorconfig-note = Un .editorconfig nel progetto sostituisce queste opzioni (rientro, tabulazioni o spazi, pulizia al salvataggio).

guide-config-5 = Editor: la scheda Editor nelle impostazioni ({ $settings-open }) ha a capo automatico, spazi visibili, guide di rientro, dimensione del rientro e che cosa fare al salvataggio (elimina gli spazi finali, nuova riga finale, formatta). Un .editorconfig nel progetto le sostituisce; senza, un file già rientrato con tabulazioni le mantiene.

guide-lsp-10 = Suggerimenti sui parametri: scrivere ( in una chiamata mostra la firma della funzione sopra il cursore, con il parametro corrente evidenziato; avanza a ogni virgola e sparisce quando la parentesi si chiude o con Esc.

guide-git-5 = Segni nel gutter: a sinistra dei numeri di riga, verde = riga nuova, blu = modificata, tacca rossa = righe eliminate, rispetto all'ultimo commit (segue quello che scrivi, senza salvare). Un clic su un segno mostra com'era, con "Ripristina modifica" (Ctrl+Z la riporta).

palette-no-items = Niente corrisponde alla ricerca

guide-git-6 = Rami: { $git-checkout } (o clic sul nome del ramo, nella barra di stato o nel pannello Git) elenca i rami, i più recenti prima; quelli remoti che qui non esistono vengono dopo e diventano locali quando li scegli. Scrivi un nome nuovo per creare un ramo da dove sei. Le modifiche non committate vengono con te; se sono in conflitto con l'altro ramo, git rifiuta e spiega.

guide-git-7 = Autore della riga: nella barra di stato, accanto al ramo, vedi chi ha cambiato per ultimo la riga del cursore, quando, e il messaggio di commit (clic per vedere l'hash e copiarlo). Finché la scheda ha modifiche non salvate resta nascosto, fino al salvataggio (le righe su disco possono non coincidere).

## Attività
cmd-tasks-run = Attività: esegui…
cmd-tasks-rerun = Attività: esegui di nuovo l'ultima
cmd-tasks-test-here = Rust: esegui il test al cursore
tasks-placeholder = Esegui attività…
tasks-none = Nessuna attività: aggiungine una sotto [tasks] nella config, oppure apri un progetto con Cargo.toml, package.json o Makefile.
tasks-no-terminal = Le attività richiedono l'estensione terminale, che è disattivata.
tasks-no-test = Il cursore non è in un test Rust (#[test] fn).
tasks-no-package = Impossibile trovare il Cargo.toml del package di questo file.
tasks-done = [{ $name } terminata: codice { $code } · Invio chiude]

guide-terminal-3 = Attività: { $tasks-run } elenca i comandi del progetto ([tasks] nella config, cargo build/run/test/clippy, script di package.json, target del Makefile) ed esegue quello scelto in una scheda "Attività" del terminale, senza toccare la shell. Alla fine compare il codice di uscita; Invio la chiude. Ctrl+C ferma solo il comando. { $tasks-rerun } esegue di nuovo l'ultima.
guide-terminal-4 = Test al cursore (Rust): con il cursore dentro una #[test] fn, { $tasks-test_here } esegue solo quel test (cargo test -p package name). Nessun tasto predefinito: aggiungine uno sotto [keys] se lo usi spesso.

## Editor diviso
cmd-editor-split = Editor: sposta la scheda sull'altro lato (dividi)
cmd-editor-focus-left = Editor: vai al lato sinistro
cmd-editor-focus-right = Editor: vai al lato destro
split-one-tab = Apri un altro file per dividere l'editor: ogni lato mostra una scheda diversa.
tab-move-right = Sposta a destra (dividi)
tab-move-left = Sposta a sinistra
guide-files-13 = Editor diviso: { $editor-split } (o "Sposta a destra" nel menu della scheda) mette la scheda di fianco, con le sue schede; aprire un file va nel lato con il focus. Clic su un lato (o { $editor-focus_left } / { $editor-focus_right }) per lavorare lì; trascina il divisore per cambiare la larghezza (doppio clic lo centra). Spostare indietro l'ultima scheda di un lato riunisce tutto.

## Menu applicazioni (--install-desktop)
desktop-no-home = Impossibile trovare la cartella home (HOME).
desktop-error = Impossibile installare la scelta rapida: { $error }
desktop-installed = 
    IDE TEA nel menu delle applicazioni: { $desktop }
    Esegue: { $exe }
desktop-removed = IDE TEA rimosso dal menu delle applicazioni.

## File grandi
file-light-mode = "{ $name }" ha una riga molto lunga e si è aperto in modalità leggera: niente colori, niente server di linguaggio e con a capo automatico
lsp-light-mode = file in modalità leggera (riga molto lunga): niente server di linguaggio

## Eseguibile ricompilato
binary-changed = L'IDE è stato ricompilato: esegui "Riavvia l'IDE" (riquadro comandi) per usare la nuova versione.
binary-changed-title = [nuova versione: riavvia]
binary-restart-error = Impossibile riavviare: { $error }

## idetea status / idetea open (RFC 0006)
status-usage =
    Uso: idetea status [--json]
    
    Che cosa mostra l'IDE aperto in questo progetto: la scheda attiva con il cursore e i file aperti
    (con "non salvato" su quelli con modifiche). --json: lo stesso in JSON, con "open".
    Codici di uscita: 0 aperto, 2 argomento non valido, 3 l'IDE non è aperto in questo progetto.
status-bad-arg = argomento sconosciuto: { $arg }
status-not-open = l'IDE non è aperto in questo progetto ({ $root })
status-active = attivo: { $path }
status-no-active = nessun file attivo
status-modified = {" "}(non salvato)
open-usage =
    Uso: idetea open <file[:line[:column]]>
    
    L'IDE aperto in questo progetto mostra il file, con il cursore sulla riga e la colonna.
    Codici di uscita: 0 mostrato, 2 percorso non valido o fuori dal progetto, 3 l'IDE non è aperto (o non ha preso la richiesta).
open-outside = { $path } non è dentro il progetto ({ $root })
open-write-error = impossibile scrivere { $path }
open-not-taken = l'IDE non ha preso la richiesta entro 3 secondi

## Mascotte reattiva (RFC 0012)
mascot-state-idle = Calmo
mascot-state-working = Al lavoro
mascot-state-thinking = Pensiero
mascot-state-attention = Attenzione
mascot-state-celebrating = Fatto
mascot-state-error = Errore
mascot-state-sleeping = Sonno
mascot-state-curious = Curioso
mascot-state-annoyed = Infastidito
mascot-idle-1 = Solo il codice. Il ghiaccio lo metti tu.
mascot-idle-2 = C'è vapore. Ci sono anch'io.
mascot-idle-3 = Tazza quieta, occhio aperto.
mascot-idle-4 = Le foglie possono posarsi.
mascot-idle-5 = Senza fretta. Il tè è caldo.
mascot-idle-6 = Bevi quando vuoi.
mascot-idle-7 = La tazza è dalla tua parte.
mascot-idle-8 = Tutto calmo sul piattino.
mascot-hover-curious-1 = Hmm? Ti serve qualcosa?
mascot-hover-curious-2 = Ciao. Tutto a posto?
mascot-hover-curious-3 = L'ho sentito. Che c'è?
mascot-hover-curious-4 = Passi sopra il tè?
mascot-hover-curious-5 = Un ciao, o una mano?
mascot-hover-curious-6 = La tazza ti ha visto.
mascot-hover-curious-7 = Curioso. Vai pure.
mascot-hover-curious-8 = Benvenuto. Qual è il piano?
mascot-hover-annoyed-1 = Piano: stai girando il tè.
mascot-hover-annoyed-2 = Troppo mouse. Alla tazza gira la testa.
mascot-hover-annoyed-3 = Mano ferma. Tè caldo.
mascot-hover-annoyed-4 = Un passaggio è un ciao. Quello era un vento.
mascot-hover-annoyed-5 = Il piattino trema.
mascot-hover-annoyed-6 = Va bene, ti vedo. Meno vortice.
mascot-poke-curious-1 = Hmm? Che c'è?
mascot-poke-curious-2 = Questo fa il solletico alla tazza.
mascot-poke-curious-3 = Hai suonato?
mascot-poke-curious-4 = Un tocco. Ti ascolto.
mascot-poke-curious-5 = Attento, è piena.
mascot-poke-curious-6 = Ciao, dal piattino.
mascot-poke-double-1 = Due tocchi. Il doppio del vapore.
mascot-poke-double-2 = Doppio clic annotato. È sempre tè.
mascot-poke-double-3 = Due volte? Va bene, mi hai.
mascot-poke-double-4 = Due colpi sulla tazza.
mascot-poke-double-5 = Due è già una conversazione.
mascot-poke-double-6 = Doppio tocco. Non è caduto nulla.
mascot-poke-annoyed-1 = Attento, lo rovesci.
mascot-poke-annoyed-2 = Il tè è già caldo.
mascot-poke-annoyed-3 = Basta tocchi per una tazza sola.
mascot-poke-annoyed-4 = Tu mi stai simpatico. Il piattino meno.
mascot-poke-annoyed-5 = Piano. La tazza è piccola.
mascot-poke-annoyed-6 = Messaggio ricevuto. Mi raffreddo.
mascot-task-1 = Ci sono: { $name }
mascot-task-2 = In infusione { $name }…
mascot-task-3 = L'attività { $name } è in corso.
mascot-task-4 = { $name }. Tengo d'occhio il bollitore.
mascot-task-5 = Avvio { $name }.
mascot-task-6 = { $name } — lascialo in infusione.
mascot-term-pending-1 = Il terminale vuole un sì: { $title }
mascot-term-pending-2 = Un comando aspetta: { $title }
mascot-term-pending-3 = Decidi tu: { $title }
mascot-term-pending-4 = Permesso, per favore: { $title }
mascot-term-pending-5 = Tengo il bollitore: { $title }
mascot-term-pending-6 = { $title } — lo avvio?
mascot-term-running-1 = In esecuzione: { $title }
mascot-term-running-2 = { $title } è sul fuoco.
mascot-term-running-3 = In corso: { $title }
mascot-term-running-4 = { $title } — ancora in infusione.
mascot-term-running-5 = Lavoro su { $title }.
mascot-term-running-6 = { $title } è in cammino.
mascot-term-ok-1 = Fatto: { $title }
mascot-term-ok-2 = { $title } è finito pulito.
mascot-term-ok-3 = Tutto a posto — { $title }.
mascot-term-ok-4 = { $title } è uscito bene.
mascot-term-ok-5 = Uscita pulita: { $title }.
mascot-term-ok-6 = { $title }. La tazza festeggia.
mascot-term-err-1 = { $title } non è riuscito ({ $code }).
mascot-term-err-2 = Uscita { $code }: { $title }
mascot-term-err-3 = { $title } è traboccato (codice { $code }).
mascot-term-err-4 = Quello si è rotto: { $title } ({ $code }).
mascot-term-err-5 = { $title } non ha finito ({ $code }).
mascot-term-err-6 = Codice { $code } da { $title }.
mascot-answered-1 = Capito: { $title }
mascot-answered-2 = Risposto: { $title }
mascot-answered-3 = { $title } — grazie.
mascot-answered-4 = Segnato: { $title }
mascot-answered-5 = { $title } è sistemato.
mascot-answered-6 = Hai risposto a { $title }.
mascot-wait-1 = L'agente ha smesso di aspettare: { $title }
mascot-wait-2 = { $title } è rimasto in silenzio.
mascot-wait-3 = Nessuno aspetta più { $title }.
mascot-wait-4 = { $title } si è raffreddato.
mascot-wait-5 = Ancora aperto, ma l'attesa è finita: { $title }
mascot-errors-1 = { $count } errore/i nel progetto.
mascot-errors-2 = Le foglie hanno trovato { $count } errore/i.
mascot-errors-3 = { $count } da guardare, quando vuoi.
mascot-errors-4 = Problemi: { $count }. Senza fretta.
mascot-errors-5 = { $count } errore/i. Posso indicare.
mascot-errors-6 = Rapporto del piattino: { $count } errore/i.
mascot-save-1 = Salvato { $name }. La tazza approva.
mascot-save-2 = { $name } è sul disco.
mascot-save-3 = Ho preso { $name } prima che si raffreddasse.
mascot-save-4 = { $name } — messo via.
mascot-save-5 = Scrittura fatta: { $name }.
mascot-save-6 = { $name } è al sicuro. Si torna al tè.
mascot-wake-1 = Oh — sei tornato. Il tè è ancora caldo.
mascot-wake-2 = Mi ero assopito. Dov'eravamo?
mascot-wake-3 = Bentornato. Ricomincio a guardare.
mascot-wake-4 = Occhi aperti. Anche la tazza.
mascot-wake-5 = È stata una buona pausa.
mascot-wake-6 = Si riprende? Io sto di guardia.
mascot-sleep-1 = In infusione, in silenzio…
mascot-sleep-2 = Resto qui.
mascot-sleep-3 = Fuoco basso. Chiama se hai bisogno.
mascot-sleep-4 = Tazza assonnata. Il codice può riposare.
mascot-sleep-5 = Pisolino sul piattino.
mascot-sleep-6 = Luce bassa. Mi sveglio quando ti svegli.
mascot-greet-morning-1 = Buongiorno. La prima tazza la offro io.
mascot-greet-morning-2 = Buongiorno. Codice, poi ghiaccio.
mascot-greet-morning-3 = Luce presto, foglie nuove.
mascot-greet-morning-4 = Mattina. Un sorso breve prima del diff.
mascot-greet-morning-5 = Ciao. Il bollitore è già sul fuoco.
mascot-greet-afternoon-1 = Buon pomeriggio. La teiera regge ancora.
mascot-greet-afternoon-2 = Buon pomeriggio. Ritmo costante.
mascot-greet-afternoon-3 = Mezzogiorno. Non berlo tutto d'un fiato.
mascot-greet-afternoon-4 = Controllo del pomeriggio, dalla tazza.
mascot-greet-afternoon-5 = Il sole è alto. Anche il tè.
mascot-greet-evening-1 = Buonasera. Un file in più, o il piattino.
mascot-greet-evening-2 = Buonasera. La tazza è ancora calda.
mascot-greet-evening-3 = Luce della sera. Chiudi quando vuoi.
mascot-greet-evening-4 = La giornata abbassa il fuoco.
mascot-greet-evening-5 = Sera. Io resto, se il build non resta.
mascot-greet-late-1 = Tardi. Il tè regge. Riposa, se puoi.
mascot-greet-late-2 = Ore quiete. Dormire è una funzione.
mascot-greet-late-3 = Ancora sveglio? Chiudi il pensiero, poi riposa.
mascot-greet-late-4 = La tazza può restare. Tu puoi fare pausa.
mascot-greet-late-5 = Notte fonda. Io veglio. Tienilo breve.
mascot-tip-1 = Ogni comando è nella tavolozza: { $palette-open }.
mascot-tip-2 = Apri un file per nome con { $file-quick-open }.
mascot-tip-3 = { $file-save } mette il file sul disco.
mascot-tip-4 = Scorciatoie e una guida breve: { $help-open }.
mascot-tip-5 = Avvia un'attività del progetto con { $tasks-run }.
mascot-tip-6 = Il terminale è { $terminal-focus }.
mascot-tip-7 = Problemi: { $problems-focus }. Il prossimo è { $problems-next }.
mascot-tip-8 = Vai alla definizione con { $lsp-definition }.
mascot-build-ok-1 = Il build è finito. Il tè è ancora caldo.
mascot-build-ok-2 = Ha compilato. Buone foglie.
mascot-build-ok-3 = Build pulito.
mascot-build-ok-4 = Build verde. Ne verso uno.
mascot-build-ok-5 = Compilato. Si torna all'editor.
mascot-build-err-1 = Il build non ha preso.
mascot-build-err-2 = Il build è fallito. La tazza l'ha sentito.
mascot-build-err-3 = Errore di compilazione. Guardalo quando vuoi.
mascot-build-err-4 = Quel build è traboccato.
mascot-build-err-5 = Build fallito. Problemi forse ne sa di più.
mascot-working-1 = Sul fuoco.
mascot-working-2 = Al lavoro. Il vapore è occupato.
mascot-working-3 = Lascialo in infusione.
mascot-working-4 = Ci sono ancora.
mascot-working-5 = Il bollitore è occupato.
mascot-thinking-1 = Mescolo le foglie…
mascot-thinking-2 = Un momento. Sto pensando.
mascot-thinking-3 = Lo lascio in infusione.
mascot-thinking-4 = Pensiero quieto, tazza calda.
mascot-thinking-5 = Lo guardo con calma.
mascot-celebrate-1 = Fatto. È uscito bene.
mascot-celebrate-2 = Tutto a posto.
mascot-celebrate-3 = Chiusura pulita. Salute.
mascot-celebrate-4 = La tazza approva.
mascot-celebrate-5 = Bene. Tengo il vapore.
mascot-error-1 = Qualcosa è traboccato.
mascot-error-2 = Quello non ha preso.
mascot-error-3 = Un intoppo. Sono ancora qui.
mascot-error-4 = Stavolta no.
mascot-error-5 = Merita uno sguardo, quando vuoi.
mascot-open-interfaces = Apri il pannello Interfacce →
mascot-open-terminal = Apri il pannello Terminale →
mascot-open-problems = Apri il pannello Problemi →
mascot-idle-hint = Seguo agenti e attività
mascot-bad-state = stato sconosciuto per il mascotte: { $state }
mascot-usage =
    Uso: idetea mascot <stato> [--text "…"] [--source <nome>] [--ttl <secondi>]
         idetea mascot clear
         idetea mascot status [--json]

    Stati: idle, working, thinking, attention, celebrating, error, sleeping, curious, annoyed.
    Scrive .idetea/mascot.json così il mascotte reagisce subito (scade dopo --ttl, 30 s di default).

## Schermata iniziale
welcome-start = Inizia
welcome-new-file = Nuovo file
welcome-quick-open = Apri un file del progetto…
welcome-open-file = Apri file…
welcome-open-folder = Apri cartella in una nuova finestra…
welcome-recent = Apri recenti…
welcome-recent-files = File recenti
welcome-recent-projects = Progetti recenti
welcome-hint-palette = tutti i comandi
welcome-hint-search = cerca nel progetto
welcome-hint-terminal = terminale

## Lançador de agentes (RFC 0016)
cmd-agents-launch = Agenti: avvia…
launcher-placeholder = Avvia agente…
launcher-worktree = { $name } · in un nuovo worktree…
launcher-branch-placeholder = Branch del worktree per { $name } (es.: fix/login)
launcher-branch-create = Crea worktree sul branch
launcher-none = Nessun agente nel PATH ({ $known }). Installane uno, o aggiungi il tuo in [agents.launchers] nella config utente.
launcher-teach-title = Istruire prima gli agenti?
launcher-teach-detail = L’AGENTS.md/CLAUDE.md di questo progetto non ha ancora le istruzioni di questo IDE (o ne ha una versione vecchia). Senza, { $name } non sa usare il pannello Interfacce né chiedere terminali.
launcher-teach-write = Scrivi e avvia
launcher-teach-open = Avvia soltanto
launcher-teach-skip = Non chiedere in questo progetto
launcher-teach-done = Istruzioni scritte in { $files }.
launcher-teach-failed = Impossibile scrivere le istruzioni: { $error }
welcome-launch-agent = Avvia un agente
