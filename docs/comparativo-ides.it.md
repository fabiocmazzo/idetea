# Confronto onesto: IDE TEA e gli altri IDE sul mercato

<p><a href="comparativo-ides.md">Português</a> · <a href="comparativo-ides.fr.md">Français</a> · <a href="comparativo-ides.de.md">Deutsch</a> · <b>Italiano</b> · <a href="comparativo-ides.ja.md">日本語</a> · <a href="comparativo-ides.zh-CN.md">中文（简体）</a></p>

Questo documento è una valutazione tecnica, diretta e senza infiocchettature di **IDE TEA** (stato di allora, `v0.4.0`) rispetto ai principali IDE ed editor sul mercato:

- **VS Code / VSCodium** (lo standard universale basato su Electron)
- **Cursor / Windsurf** (fork di VS Code centrati sull'IA)
- **Zed** (IDE nativo in Rust, centrato su velocità, collaborazione e IA integrata)
- **JetBrains (RustRover, IntelliJ, PyCharm, CLion)** (IDE semantici pesanti)
- **Neovim / Helix / Lapce** (editor modali/nativi snelli)

> **Ambito di questa analisi:** il focus è strettamente tecnico — **stabilità, prestazioni, funzionalità di modifica, LSP, debug, Git, ecosistema e integrazione con gli agenti**. Non tiene conto delle dimensioni del team né di chi mantiene il progetto.

> Le versioni successive a questa fotografia della 0.4.0 escono anche per macOS (Apple Silicon) e Windows (x86_64).

---

## 1. Visione d'insieme (matrice diretta)

| Dimensione | IDE TEA | VS Code | Cursor / Windsurf | Zed | JetBrains | Helix / Neovim |
|---|---|---|---|---|---|---|
| **Tempo di apertura (cold start)** | **Eccellente (< 200 ms)** | Lento (1–3 s) | Lento (1,5–4 s) | **Eccellente (< 150 ms)** | Molto lento (5–20 s) | **Istantaneo (< 50 ms)** |
| **Consumo di RAM a riposo** | **Basso (~60–150 MB)** | Alto (400 MB–1,5 GB) | Molto alto (600 MB–2 GB) | Basso (~120–300 MB) | Molto alto (1,5–4 GB+) | **Minimo (15–60 MB)** |
| **File enormi (50 MB+ / minificati)** | **Eccellente (modalità leggera automatica)** | Medio (si blocca sulle righe lunghe) | Medio | Buono | Fatica sopra i 20 MB | Dipende dai plugin (Neovim) |
| **Privacy / offline / zero account** | **Totale (zero rete salvo git push/pull)** | Telemetria di default (tranne VSCodium) | Richiede account e cloud | Chiede un account per IA e collaborazione | Richiede account / licenza | **Totale** |
| **Protocollo strutturato per gli agenti (UI dichiarativa, approvazione dei comandi, mascotte)** | **Unico nella categoria (nativo, su file)** | Non c'è (dipende da estensioni di chat) | Centrato su chat/diff proprietari | Centrato su chat/assistenza in linea | Centrato su chat/in linea | Non c'è (solo terminale) |
| **Autocompletamento IA in linea («Ghost text» / Tab)** | **Non c'è (per filosofia)** | Tramite Copilot | **Eccellente (in testa)** | Molto buono (Zeta) | Buono | Tramite plugin |
| **Maturità dell'editor di testo** | Buona sulle basi, mancano funzioni avanzate | **Riferimento** | **Riferimento** | Molto alta | **Riferimento** | **Riferimento (modale)** |
| **Profondità di LSP e refactoring** | Media (senza Code Action e Inlay Hint) | **Completa** | **Completa** | Alta | **Impareggiabile (AST proprio)** | Alta |
| **Debug** | Di base (GDB 14+ per Rust/C/C++/Zig; MoonBit nativo è sperimentale) | **Completo (qualsiasi linguaggio)** | **Completo** | Di base/medio (DAP recente) | **Eccellente** | Di base (nvim-dap) |
| **Controllo di versione (Git)** | Di base/medio (senza gutter diff né merge a 3 vie) | **Completo** | **Completo** | Medio | **Eccellente (merge migliore)** | Tramite LazyGit / Fugitive |
| **Supporto multipiattaforma** | **Solo Linux (Wayland), testato oggi** | **Linux, macOS, Windows, Web** | **Linux, macOS, Windows** | **macOS, Linux, Windows** | **Linux, macOS, Windows** | **Qualsiasi SO / SSH** |

---

## 2. Stabilità e affidabilità

### Dove IDE TEA è forte davvero
1. **Architettura senza WebView / Chromium:** a differenza di VS Code, Cursor e Windsurf, non ci sono processi di rendering Chromium né decine di *extension host* Node.js che perdono memoria nel corso della giornata. Il binario è uno solo, compilato in Rust sicuro.
2. **Isolamento dei guasti di estensioni e processi esterni:**
   - I server di linguaggio (`tea-lsp`), il debugger (`tea-dap` / `gdb`), le chiamate `git` e gli agenti girano in processi separati. Se `rust-analyzer` o `gdb` si blocca o muore, la finestra dell'IDE non si congela.
   - Le estensioni interne parlano con il nucleo solo per messaggi ed eventi (`Request` / `CoreEvent`), senza condividere riferimenti mutabili dello stato dell'applicazione.
3. **Protezione contro la perdita del lavoro:**
   - Chiudere una scheda o uscire dall'IDE con modifiche non salvate chiede sempre conferma.
   - **Recupero automatico dai crash (`backup.rs`):** le schede con modifiche non salvate vengono scritte in `$XDG_STATE_HOME/idetea/backups/<progetto>/` per PID e proposte per il ripristino se la finestra si chiude all'improvviso.
   - Cancellare file nell'albero li manda nel **cestino di sistema** (non fa un `unlink` irreversibile).
   - I cambiamenti esterni sul disco (`git checkout`, `cargo fmt`, agenti che modificano file) li vede il watcher: le schede senza modifica locale si ricaricano da sole; le schede con una modifica locale non salvata **non** vengono mai sovrascritte in silenzio.
   - I file binari o non UTF-8 vengono rifiutati con un avviso chiaro, invece di corrompere il buffer al salvataggio.
   - Il protocollo `.idetea/` usa una scrittura atomica (`.tmp` + `rename`) e ha un comando di autoripristino (`idetea fix` / *«Interfacce: correggi questo progetto»*).

### Dove la stabilità di IDE TEA è ancora indietro rispetto alle grandi
1. **Provata solo su Linux (Wayland):** oggi IDE TEA è sviluppato e validato su Linux. Su macOS e Windows ci sono differenze reali di pacchettizzazione, tipi di carattere, scelte rapide (`Cmd` contro `Ctrl`), PTY del terminale (`conpty` su Windows) e percorsi che non hanno ancora passato la prova del fuoco. VS Code, JetBrains e Zed girano in modo maturo su tutti e tre i sistemi.
2. **Limiti dell'ecosistema `iced` 0.14 per applicazioni complesse:**
   - L'IDE lavora in una finestra sola (`iced::application`). Non si può staccare un terminale, un'anteprima o un editor su un secondo monitor, in un'altra finestra nativa del sistema.
   - Non c'è una cronologia locale continua per file (*Local History* / *Timeline* lungo i giorni, senza Git, come in JetBrains e VS Code).
3. **Evidenziazione della sintassi basata su regex (`syntect`) contro `tree-sitter`:**
   - IDE TEA ha una modalità leggera eccellente per i file da 50 MB o le righe minificate enormi, ma nei file di codice normali con migliaia di righe `syntect` (basato su grammatiche `.sublime-syntax` con regex) è strutturalmente meno resistente al codice incompleto di `tree-sitter`, usato da Zed, Helix e Neovim.

---

## 3. Flusso con gli agenti di IA (dove IDE TEA è unica, e dove non compete)

Questa è la differenza di filosofia di prodotto più profonda sul mercato oggi.

### Come Cursor, Windsurf e Zed trattano l'IA
- Sono pensati per **il modello dentro l'editor**:
  - *Autocompletamento predittivo (Tab / Next Edit Prediction)* che indovina le righe successive mentre scrivi.
  - *Modifica in linea (`Ctrl+K`)* e *Composer/Agent Chat* che mostrano diff colorati (verde/rosso) direttamente fra le righe dell'editor, per accettare o rifiutare hunk per hunk.
  - In cambio, ti legano ad account nel cloud, abbonamenti mensili o client HTTP dei provider dentro l'IDE.

### Come IDE TEA tratta l'IA (e dove vince)
IDE TEA parte dal fatto che **i migliori agenti girano già nel terminale** (Claude Code, Codex, Gemini CLI, Aider, OpenCode) e risolve il problema che nessun altro IDE ha risolto in modo nativo e aperto: **come questi agenti parlano con la persona senza diventare un muro di testo nel terminale**.

1. **Interfacce dichiarative native (`.idetea/ui/` + `idetea ask`):**
   - Invece di chiedere in mezzo a 200 righe di log nel terminale, l'agente pubblica un JSON validato e l'IDE disegna moduli nativi in `iced`: selettori, caselle, elenchi di passi di un test manuale (il pulsante *Approva* si abilita solo dopo aver fatto i passi), anteprime di immagine/galleria, spritesheet animati e audio WAV con forma d'onda.
   - **Nessun altro IDE sul mercato (VS Code, Cursor, Zed, JetBrains) ha un protocollo dichiarativo basato su file con cui gli agenti nel terminale disegnano UI nativa, con una cronologia verificabile (`.idetea/history/`).**
2. **Governo e sicurezza dell'esecuzione (`idetea run`):**
   - Quando un agente vuole aprire un terminale interattivo o delegare un compito, l'IDE mostra il comando esatto in una scheda di approvazione. L'elenco dei comandi consentiti (`[agents] allow`) si legge solo dalla config globale dell'utente (`~/.config/idetea/config.toml`) e **mai** dal repository: un repository malevolo non può concedersi da solo il permesso di eseguire.
3. **Delega in git worktree isolati (alberi di lavoro) (`--worktree` + `$IDETEA_ANSWER`):**
   - L'IDE gestisce in modo nativo la creazione e la pulizia dei worktree, così i subagenti lavorano in parallelo senza sporcare l'albero principale.
4. **Mascotte reattiva nativa (Tea) + `idetea mascot`:**
   - Mentre strumenti come *Masko.ai* chiedono macOS più un server MCP nel cloud per avere una mascotte che reagisce a Claude Code, IDE TEA ha **Tea** integrato in modo nativo, che osserva `.idetea/ui/`, `.idetea/agents/`, le attività e l'LSP in tempo reale, reagisce al mouse (`curious` / `annoyed`) e accetta pacchetti `mascot.toml`.

### Dove IDE TEA perde nel flusso di IA
- **Zero autocompletamento predittivo («Ghost text» / Tab):** per chi dipende dal `Tab` di Cursor/Copilot per completare il boilerplate mentre scrive a mano, IDE TEA non ha un equivalente.
- **Niente diff inline dell'IA nell'editor:** quando un agente nel terminale modifica un file, il file si ricarica nella scheda di IDE TEA. Per vedere che cosa è cambiato vai al pannello **Git**; non ci sono pulsanti *«Accept / Reject hunk»* che galleggiano dentro l'editor, come in Cursor o Windsurf.
- **Passare il contesto dall'editor all'agente è ancora manuale:** l'agente può eseguire `idetea status` per scoprire il file attivo e la riga del cursore, ma non c'è ancora una scelta rapida nell'IDE (per esempio `Ctrl+L`) che mandi un brano selezionato o un errore dell'LSP dritto nel prompt dell'agente aperto nel terminale.

---

## 4. Editor di codice: che cosa c'è e che cosa manca

### Quel che in IDE TEA è già solido
- Apertura rapida dei file (`Ctrl+P` con ricerca fuzzy e salto `file:riga`), simboli nel file (`Ctrl+Shift+O`) e nel progetto (`Ctrl+T`).
- Più cursori, selezione parola per parola (`Ctrl+Frecce`), `Shift+clic`, riduzione del codice (folding), annulla/ripeti, cerca e sostituisci nel file e **sostituzione in tutto il progetto (`Ctrl+Shift+H`)** con anteprima riga per riga e supporto alle regex.
- Zoom temporaneo dell'editor (`Ctrl+=`, `Ctrl+-`, `Ctrl+0`, `Ctrl+rotellina`) sincronizzato fra schede e divisioni, senza toccare il resto dell'interfaccia.
- Divisione dell'editor fianco a fianco (split), breadcrumb cliccabili (`progetto › cartella › file › simbolo`), icone Seti per linguaggio e tipi di carattere personalizzabili con legature (`JetBrains Mono` integrato).
- Modalità Vim facoltativa.
- Schede native per le immagini (con zoom e trascinamento), PDF (renderizzato in Rust puro tramite `hayro`) e anteprima fianco a fianco di Markdown e SVG in tempo reale.

### Quel che manca nell'editor rispetto a VS Code, Zed e JetBrains
1. **Snippet:** non c'è un motore di snippet (tab-stop `$1`, `$2`) per espandere blocchi rapidi di codice.
2. **Selezioni strutturali per AST (Expand/Shrink Selection):** in Zed, Helix e JetBrains una scelta rapida espande la selezione da parola → espressione → blocco `if` → funzione intera, tramite `tree-sitter`/AST.
3. **Codifiche legacy:** apre solo UTF-8 (conservando LF/CRLF). Chi lavora con file `ISO-8859-1` / `Windows-1252` non può modificarli in IDE TEA.
4. **Più finestre / più progetti nella stessa istanza:** ogni progetto è un processo `idetea` separato; non ci sono «Multi-root Workspaces» nella stessa finestra.

---

## 5. Intelligenza di linguaggio (LSP) e refactoring

### Quel che IDE TEA già fa molto bene
- Diagnostici in tempo reale con **sottolineatura ondulata nel testo dell'editor**, tooltip dell'errore al passaggio del mouse, conteggio sulla barra di stato e pannello **Problemi** (`Ctrl+Shift+M` / `F8`).
- Autocompletamento, firma dei parametri, vai alla definizione (`F12`), vai al tipo e vai all'implementazione (`Ctrl+F12`).
- Trova i riferimenti (`Shift+F12` o `Ctrl+clic` sulla dichiarazione) raggruppati per file, con cronologia di navigazione Indietro/Avanti (`Alt+←` / `Alt+→` e i pulsanti laterali del mouse).
- **Gerarchia delle chiamate (`Alt+Shift+H`):** chi chiama la funzione e che cosa chiama.
- **Pannello Struttura (`Alt+7`):** albero dei simboli del file, con la visibilità (`pub`, `pub(crate)`, privato) e il simbolo sotto il cursore evidenziato.
- **Rinomina simbolo (`F2`)** in tutto il progetto, con una finestra che mostra in anteprima quanti file e quante modifiche saranno toccati, prima di applicare.
- Formattazione al salvataggio e rispetto di `.editorconfig`.

### Lacune importanti di LSP rispetto a VS Code, Zed e JetBrains
1. **Senza Code Action / Quick Fix (`Ctrl+.` / lampadina dell'LSP):**
   - Questa è oggi la **mancanza pratica più grande nel lavoro quotidiano con Rust/TypeScript/Go**: quando `rust-analyzer` suggerisce *«Import `std::collections::HashMap`»*, *«Add missing match arms»* o *«Fill struct fields»*, in IDE TEA vedi il messaggio diagnostico, ma non puoi applicare il *Quick Fix* (`textDocument/codeAction`) con una scelta rapida.
2. **Senza Inlay Hint:**
   - Non mostra i tipi inferiti in linea (`let x: Vec<String> = ...`) né i nomi dei parametri nelle chiamate di funzione, funzione molto usata in Rust e TypeScript su VS Code, Zed e RustRover.
3. **Senza Semantic Token dell'LSP:**
   - I colori della sintassi vengono solo dalla grammatica `.sublime-syntax` (`syntect`), senza arricchimento semantico dell'LSP (per esempio distinguere dal colore del server di linguaggio una variabile mutabile da una immutabile, o una macro da una funzione).

---

## 6. Debug

### Che cosa fa IDE TEA oggi
- Integrazione nativa con `gdb` 14+ (via DAP): punti di interruzione cliccando sul numero di riga (`F9`) che seguono il testo se modifichi il file, `F5` (compila ed esegue in automatico nei progetti Rust, Zig e, in modo sperimentale, MoonBit), Esegui istruzione (`F10`), Entra (`F11`), Esci (`Shift+F11`), Sospendi e Interrompi.
- Pannello con lo stack di chiamate (il clic porta alla riga del frame), albero delle variabili locali ispezionabile e output del programma.

### Dove perde rispetto a VS Code e JetBrains
- **Legato a GDB 14+ (Rust, C, C++, Zig e, in modo sperimentale, MoonBit):** non supporta ancora adattatori DAP per Python (`debugpy`), Go (`delve`), JavaScript/TypeScript (`js-debug`) o `codelldb` (macOS). Hare non ha un debugger nell'IDE.
- **Funzioni avanzate di debug assenti:** niente punti di interruzione condizionali (*«fermati solo quando `i == 42`»*), logpoint, finestra di *Watch Expressions* arbitrarie (REPL durante il punto di interruzione) né valori delle variabili disegnati in linea accanto al codice nell'editor.

---

## 7. Git e controllo di versione

### Che cosa fa IDE TEA oggi
- Barra di stato con il ramo attuale, commit avanti/indietro (`ahead`/`behind`) e totale dei file modificati.
- Pannello Git con aggiungi a stage / rimuovi da stage per file o tutto, diff colorato, casella del messaggio di commit, ignora modifiche con conferma, e pulsanti di `push`, `pull` e `fetch` che usano il `git` e le credenziali/SSH di sistema.
- Colori di stato Git nell'albero dei file.

### Dove perde rispetto a VS Code, Zed e JetBrains
1. **Senza segni Git nel margine dell'editor (Gutter Diff):** mentre modifichi un file, non ci sono quelle barre verdi/azzurre/rosse accanto ai numeri di riga che mostrano quali righe sono state aggiunte o cambiate dall'ultimo commit.
2. **Senza Git Blame in linea:** non mostra chi ha cambiato la riga attuale e in quale commit.
3. **Senza risolutore visuale dei conflitti di merge (merge a 3 vie):** in caso di conflitto si risolve modificando i marcatori `<<<<<<<` nel testo, oppure dal terminale.
4. **Senza cambio/creazione di rami o Stash dall'interfaccia:** per cambiare ramo, creare un ramo o fare `git stash` si usa il terminale integrato.

---

## 8. Riassunto onesto: quando scegliere IDE TEA oggi (e quando no)

### IDE TEA è già migliore, o unica, quando:
1. **Programmi su Linux accanto ad agenti di terminale (Claude Code, Codex, Gemini CLI, ecc.)** e vuoi che l'agente faccia domande strutturate, mostri piani e test manuali, chieda un permesso esplicito per eseguire comandi, usi worktree isolati e abbia una mascotte reattiva (**Tea**) senza dipendere da cloud o abbonamento.
2. **Dai priorità a velocità, prevedibilità e poco rumore sensoriale:** si apre in meno di 200 ms, consuma poca memoria, non ha account, non ha telemetria, non cambia l'interfaccia da sola e rispetta file TOML semplici.
3. **Lavori su progetti in Rust, Zig, C/C++, Python, Go, Hare, MoonBit o Web** dove il flusso principale è modificare in fretta, navigare simboli e riferimenti dell'LSP, eseguire attività e test nel terminale integrato e rivedere i commit.

### Un altro IDE è ancora meglio quando:
1. **Vuoi l'autocompletamento predittivo di IA a ogni tasto (`Tab` / Ghost text) o la modifica in linea per diff:** **Cursor**, **Windsurf** o **Zed** seguono meglio questo stile.
2. **Dipendi molto dai refactoring assistiti dalle *Code Action* (`Ctrl+.`), dagli *Inlay Hint* o dai debugger di Python/JS/Go nell'interfaccia:** **VS Code** o **JetBrains** sono molto più completi oggi.
3. **Lavori su macOS o Windows, o dipendi da SSH Remote Development / DevContainers:** **VS Code** e **Zed** hanno già un supporto maturo per questi scenari.

---

## 9. I 5 passi successivi di maggior impatto (per avvicinarsi ai leader senza perdere l'anima di IDE TEA)

Se l'obiettivo è ridurre la distanza funzionale da VS Code e Zed tenendo il binario leggero e la filosofia del manifesto:

1. **Code Action / Quick Fix dell'LSP (`Ctrl+.`):** poter applicare i *quick fix* e gli *import* automatici che il server LSP sa già calcolare. È la voce n. 1 che manca di più quando si programma in Rust/TS.
2. **Segni Git nell'editor (Gutter Diff) + cambio di ramo dalla barra di stato:** mostrare nel margine dell'editor le righe aggiunte o modificate, e permettere di cliccare il nome del ramo nella barra di stato per cambiare ramo nel riquadro comandi.
3. **Scelta rapida «Invia selezione / diagnostico all'agente» (`Ctrl+L`):** chiudere il cerchio fra l'editor/LSP e l'agente che gira nel terminale integrato.
4. **Inlay Hint dell'LSP (tipi inferiti e nomi dei parametri, facoltativi):** mostrare i suggerimenti di tipo in linea, con una scelta rapida per accenderli e spegnerli.
5. **Portabilità su macOS e Windows + altri adattatori DAP (`lldb-dap`, `debugpy`, `delve`):** aprire l'IDE a chi sviluppa fuori da Linux e supportare il debug oltre GDB.
