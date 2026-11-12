# IDE TEA: funzionalità e scelte rapide

[English](features.md) · [Français](features.fr.md) · [Deutsch](features.de.md) · **Italiano** · [日本語](features.ja.md) · [中文（简体）](features.zh-CN.md)

L'elenco completo, per chi vuole il dettaglio tecnico. Per la versione breve, vedi il [README](../README.it.md).

> Prima della 1.0. Compilato e provato su Linux, macOS (Apple Silicon) e Windows (x86_64).

## Editor

- Apri un file per nome (`Ctrl+P`) con ricerca fuzzy; `main.rs:42` salta dritto alla riga.
- Vai a un simbolo (funzione, tipo, metodo…) nel file (`Ctrl+Shift+O`) o nel progetto (`Ctrl+T`).
- Schede con un segno di modifica; apri (`Ctrl+O`), salva e salva con nome, con la finestra di dialogo dei file di sistema.
- Evidenziazione della sintassi per circa 200 linguaggi e tipi di file (TOML, `.env`, INI, Dockerfile, TypeScript, Zig,
  [Ziex](https://ziex.dev) `.zx`…); temi chiaro e scuro (più i temi di iced: Nord, Dracula…). Altri linguaggi:
  metti i file `.sublime-syntax` in `~/.config/idetea/syntaxes/` (letti all'apertura dell'IDE).
- Cerca e sostituisci, vai alla riga, annulla/ripeti, più cursori, riduzione dei blocchi e una modalità vim facoltativa.
- Zoom temporaneo dell'editor (`Ctrl+=` / `Ctrl++`, `Ctrl+-`, `Ctrl+0` o `Ctrl+wheel`), condiviso dalle schede aperte e dalle divisioni;
  il terminale e la dimensione di base configurata restano com'erano.
- Anteprima di Markdown e SVG accanto all'editor (`Ctrl+Shift+V` o il pulsante sulla barra delle schede), aggiornata mentre scrivi.
- Le immagini (png, jpg, gif, webp, bmp, ico…) si aprono in una scheda, con zoom della rotellina e trascinamento.
- Il PDF si apre in una scheda (renderizzato da `hayro`, Rust puro), con zoom e adatta alla larghezza.
- Conserva i CRLF. Un file che non è UTF-8 non viene aperto (con un avviso), invece di essere corrotto.
- File grandi: un file da 50 MB si apre in meno di 0,1 s. Un file con una riga enorme (JSON o JS minificato) si apre in una
  modalità leggera, senza colori e con a capo automatico, così la finestra non si blocca.
- Facoltativo al salvataggio: taglia gli spazi in fondo, newline finale, formatta con il server di linguaggio. Un `.editorconfig`
  nel progetto vince.

## Progetto

- **Albero dei file:** crea, rinomina ed elimina (nel cestino), mostra il file attuale, colori git (modificato, nuovo,
  conflitto, ignorato).
- **Ricerca nel progetto** mentre scrivi, nel rispetto di `.gitignore`, con regex, parola intera e maiuscole/minuscole.
- **Menu del clic destro** sull'albero, sulle schede e sui file nel pannello git.
- **Attività** (`Ctrl+Shift+B`): i tuoi comandi dalla config, più quello che l'IDE trova nel progetto (cargo,
  script di `package.json`, target del Makefile, passi di `build.zig`), con l'output nel terminale. I moduli MoonBit
  hanno anche `moon check`, `build`, `test` e `fmt --check`, eseguiti dal `moon.mod` più vicino al file attivo.
- **Progetto nuovo** («Progetto: nuovo…» o `idetea new <folder>`): cartella, git, le skill di agente di un modello (vuoto,
  Rust, sito, Python, FastAPI) e una richiesta iniziale in `AGENTS.md`. Vedi [skills.it.md](skills.it.md).

## Linguaggi (LSP)

- Errori e avvisi (sottolineatura ondulata e hover del diagnostico nell'editor, pannello Problemi e un conteggio sulla
  barra di stato), completamento, vai alla definizione, formatta.
- Vai all'implementazione (`Ctrl+F12`) e al tipo; con più risultati, l'elenco va nel pannello.
- Trova i riferimenti (`Shift+F12`, o `Ctrl+click` sulla dichiarazione) raggruppati per file, e indietro/avanti dopo ogni
  salto (`Alt+←` / `Alt+→`, o i pulsanti laterali del mouse).
- Gerarchia delle chiamate (`Alt+Shift+H`): chi chiama una funzione, e che cosa chiama, come un albero che si carica mentre lo apri.
- Pannello Struttura (`Alt+7`): funzioni, struct, metodi e campi del file, con quel che è pubblico, privato o
  ristretto (`pub(crate)`, `protected`…), e il simbolo sotto il cursore evidenziato.
- Breadcrumb sopra l'editor: `progetto › cartella › file › impl App › fn update`, il percorso e i simboli dov'è il
  cursore. Una cartella mostra il file nell'albero; un simbolo ci salta. `[editor] breadcrumbs = false` li spegne.
- Tipo di carattere del codice: JetBrains Mono è integrato; funziona qualsiasi tipo di carattere installato (`[editor] font = "Fira Code"`), con le
  sue legature. Anche il peso e l'altezza di riga (`font_weight`, `line_height`), anche in Impostazioni → Aspetto. L'editor,
  i numeri di riga e il terminale usano lo stesso tipo di carattere.
- Icone dei file per linguaggio e tipo (il set Seti di VS Code, integrato, più le icone proprie dell'IDE per Hare e
  MoonBit) nell'albero dei file, nelle schede, in Ctrl+P, nei breadcrumb,
  nei risultati di ricerca e nel pannello git. `[ui] file_icons = "none"` (o Impostazioni → Aspetto) torna a un'icona sola per
  ogni file.
- Maiusc+clic estende la selezione dall'ultimo clic (o dal cursore, dopo averlo spostato con la tastiera).
- Parola per parola: `Ctrl+Left/Right` si sposta (con `Shift`, seleziona), `Ctrl+Backspace/Delete` cancella. `Home` va al
  primo carattere della riga, poi alla colonna 0. `Shift+Tab` toglie il rientro.
- Tasti di VS Code dove prima erano diversi: riduci/espandi il blocco al cursore con `Ctrl+Shift+[` / `]`, riapri
  l'ultima scheda chiusa con `Ctrl+Shift+T`, pannello inferiore con `Ctrl+J`, terminale con `` Ctrl+` `` o `Ctrl+'`.
- Già configurati: rust-analyzer, clangd, pyright, typescript-language-server, gopls, lua-language-server, zls (Zig),
  `zx lsp` ([Ziex](https://ziex.dev); senza la CLI `zx`, un progetto Ziex compila da solo il proprio server con `zig build`),
  `hare-lsp -S` e `moon lsp`. Basta installare il server; puoi cambiare il comando, disattivarlo o aggiungerne altri nella
  config. Il server di Hare è limitato ai diagnostici di parsing, definizione, hover e simboli del documento; il supporto MoonBit
  copre `.mbt`, `.mbti` e `.mbt.md` (Markdown con blocchi MoonBit), nei limiti di quello che sa fare la toolchain installata. Il dettaglio per linguaggio:
  [languages.it.md](languages.it.md).

## Terminale

- Terminale integrato (alacritty) nel pannello inferiore, con i colori e il tipo di carattere del tema.
- Diverse shell in schede («Terminale 1», «Terminale 2»…), più la scheda propria dell'attività; `+` ne apre una, `×` la chiude,
  `exit` chiude la scheda.
- Le scelte rapide della shell (`Ctrl+C`, `Ctrl+W`…) vanno alla shell; le scelte rapide `Ctrl+Shift` dell'IDE continuano a funzionare.

## Git

- Ramo, commit avanti/indietro e numero di modifiche sulla barra di stato.
- Pannello con aggiungi a stage / rimuovi da stage per file o tutti, diff colorato, commit, ignora modifiche (con conferma).
- Push, pull e recupera. L'IDE non aspetta mai una password: usa il tuo ssh agent o il tuo git credential helper.
- Usa il `git` installato, quindi valgono la tua config, gli hook e le credenziali.

## Debug

- Punti di interruzione con un clic sul numero di riga (`F9`); si spostano con il testo mentre modifichi.
- `F5` compila ed esegue: nei progetti Rust usa `cargo build` e trova l'eseguibile; nei progetti Zig,
  `zig build` e gli eseguibili in `zig-out/bin`; nei moduli MoonBit, `moon build --target native -g`
  (sperimentale, vedi [languages.it.md](languages.it.md#moonbit)); altrove, una sezione `[debug]`
  nella config dice il programma, gli argomenti e il comando di build.
- Esegui istruzione, entra ed esci (`F10`, `F11`, `Shift+F11`), sospendi e interrompi, con la riga attuale segnata nell'editor.
- Pannello con lo stack di chiamate (clic per andare al frame), le variabili ad albero, e l'output del programma.
- Usa `gdb` 14 o più recente (Rust, C, C++, Zig e, in modo sperimentale, MoonBit).

## Lavorare con gli agenti

Tutto sono file in chiaro nella cartella `.idetea/` del progetto, così qualsiasi agente che legge e scrive file può usarlo.
«Interfacce: insegna agli agenti di questo progetto» scrive le istruzioni in `AGENTS.md` e `CLAUDE.md`.

- **Pannello Interfacce** (`.idetea/ui/<name>.json`): l'agente scrive un documento, l'IDE lo mostra come un modulo (testo,
  Markdown, select, casella, input, pulsanti, passi per i test manuali, file da modificare, immagini, gallerie, sprite
  animati, suoni con forma d'onda e livello di picco) e scrive la tua risposta in `.idetea/ui/responses/`. Il pannello mostra
  se l'agente sta ancora aspettando. Formato: [docs/interfaces/formato.md](interfaces/formato.md) (per ora in portoghese).
- **Terminali di agente** (`.idetea/agents/<name>.json`): l'agente chiede un terminale con un comando; l'IDE mostra
  la richiesta intera in una scheda «Richiesta di terminale» e la esegue solo dopo che tu la consenti. L'avanzamento va in
  `<name>.status.json` (`pending`, `running`, `exited` con il codice, `refused`, `closed`, `invalid`, `lost`);
  cancellare la richiesta ferma il terminale (se il comando è già finito, la scheda resta finché non la chiudi).
- **Elenco consentiti:** `[agents] allow = ["npm run dev", "claude *"]` nella tua config utente esegue quei comandi senza
  chiedere: il comando esatto, o un prefisso seguito solo da argomenti semplici (niente `;`, `&`, `|`, `$`, backtick,
  reindirizzamenti…). In una config di progetto viene ignorato, con un avviso.
- **Aiutanti nel proprio worktree (albero di lavoro):** `"worktree": "<branch>"` esegue il comando in `.idetea/worktrees/<name>` su quel
  ramo. Ogni comando riceve `IDETEA_ANSWER`, un file nel progetto principale per la risposta. Cancellare la richiesta
  toglie il worktree se non ha niente di non committato; il ramo resta sempre.
- **Comandi per gli agenti:** `idetea ask` (una domanda nel pannello, in attesa della risposta), `idetea run` (un terminale,
  in attesa della fine e della risposta dell'aiutante), `idetea status` (che cosa hai aperto, modifiche non salvate, il cursore),
  `idetea open file:line` (ti mostra un file), `idetea history` (che cosa hai risposto nel pannello) e `idetea mascot`
  (stato della mascotte e fumetto in tempo reale). Fanno il lavoro sui file qui sopra in una chiamata, con le stesse approvazioni.
- **Mascotte reattiva (Tea):** una scheda con la tazza in pixel art nell'angolo in basso a destra (si comprime nella barra di stato, oppure
  si attiva dal riquadro comandi) che osserva `.idetea/` in tempo reale (`idle`, `working`, `thinking`, `attention`,
  `celebrating`, `error`, `sleeping`, più le espressioni interattive `curious` e `annoyed` al passaggio del mouse, al doppio clic
  o a un colpetto). Ogni situazione ha diverse frasi brevi; la stessa frase non compare due volte di seguito. Nota anche un
  salvataggio, la compilazione del debugger che finisce o fallisce, un'inattività lunga (si appisola, e si sveglia quando torni) e l'ora
  del giorno, compreso un accenno gentile a riposare a notte fonda. Un clic sul fumetto apre il pannello Interfacce, Terminale o
  Problemi e non ruba mai il focus. Con `[mascot] tips = true` (e `chatty` diverso da `quiet`) ogni tanto
  cita una scelta rapida che esiste davvero. `[mascot] chatty` vale `quiet`, `normal` o `chatty`. Anche gli agenti o gli hook possono guidarla
  direttamente con `idetea mascot <state> [--text "…"] [--source <name>] [--ttl <secs>]`, e i pacchetti di sprite personalizzati funzionano tramite
  `.idetea/mascot/mascot.toml` o `~/.config/idetea/mascots/<pack>/mascot.toml` (`[mascot] enabled = true`).
- **Seleziona e copia il testo:** «Seleziona il testo» in cima a un'interfaccia (aperta o nella cronologia) ne mostra i testi in un
  editor di sola lettura: seleziona con il mouse, copia con Ctrl+C; «Mostra formattato» torna indietro.
- **Cronologia delle domande:** l'IDE tiene ogni domanda, la tua risposta, il tuo commento e quando l'agente l'ha letta,
  in `.idetea/history/` (fuori da git, finché non la cancelli). Il pannello **Cronologia delle Interfacce** a sinistra (nascosto
  di default) elenca prima le domande aperte, poi quelle con risposta per giorno, con la ricerca; una voce si apre in sola lettura,
  come hai risposto.
- **Correggi questo progetto** («Interfacce: correggi questo progetto»): rimette `.idetea/` in uno stato usabile dopo un crash o una
  modifica andata male. **Skill:** `idetea skills` installa nel progetto le skill di agente da un repository git
  ([skills.it.md](skills.it.md)).

Dettagli dei terminali di agente (per ora in portoghese): [terminais-de-agente.md](terminais-de-agente.md). Skill: [skills.it.md](skills.it.md).

## Interfaccia

- Riquadro comandi con ogni comando e la sua scelta rapida (`Ctrl+Shift+P`).
- Guida (`F1`) con tutte le scelte rapide attuali e una guida breve.
- Impostazioni (`Ctrl+,`): aspetto, estensioni, scelte rapide (registra il tasto che premi) e server di linguaggio.
- Barra attività per mostrare e nascondere i pannelli; i pannelli si ridimensionano con il mouse.
- Schermata di benvenuto quando non c'è nessun file aperto: nuovo file, apri file o cartella, file e progetti recenti, e le scelte
  rapide principali. Scrivere o `Ctrl+N` riporta un editor senza nome.
- Pannelli come isole arrotondate su uno sfondo più scuro, con barre di scorrimento sottili a pillola che svaniscono quando il mouse è lontano;
  `[ui] style = "classic"` tiene tutto su uno sfondo solo, diviso da linee. Il riquadro, i menu e le finestre entrano
  con uno scorrimento morbido; `[ui] animations = false` lo spegne.
- Interfaccia in portoghese, inglese, spagnolo, francese, tedesco, italiano, giapponese e cinese semplificato
  (`[ui] language`, o quella del sistema). Ha effetto dopo il riavvio dell'IDE.

## Non perdere il lavoro

- Chiudere una scheda o uscire con modifiche non salvate chiede sempre prima.
- Eliminare un file lo manda nel cestino di sistema.
- I file cambiati fuori dall'IDE (terminale, `cargo fmt`, un altro editor, `git checkout`) si notano: l'albero e
  git si aggiornano, le schede senza modifiche si ricaricano, e le schede con modifiche vengono tenute, con un avviso.

## Scelte rapide principali

| Scelta rapida | Che cosa |
|---|---|
| `Ctrl+P` | Apri un file per nome |
| `Ctrl+Shift+P` | Riquadro comandi |
| `F1` | Guida: ogni scelta rapida e la guida |
| `Ctrl+,` | Impostazioni |
| `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S` | Apri / salva / salva con nome |
| `Ctrl+B` | Mostra/nascondi l'albero dei file |
| `Ctrl+Shift+F` | Cerca nel progetto |
| `` Ctrl+` `` / `Ctrl+'` | Terminale |
| `Ctrl+Shift+T` | Riapri l'ultima scheda chiusa |
| `Ctrl+Shift+G` | Git |
| `Ctrl+Shift+M` / `F8` | Problemi / problema successivo |
| `F12` | Vai alla definizione |

Si possono cambiare tutte nelle impostazioni o in `~/.config/idetea/config.toml`.

## Compilare dal codice sorgente

```sh
git clone https://github.com/fabiocmazzo/idetea && cd idetea
cargo build --release      # eseguibile in target/release/idetea
```

Con [`just`](https://github.com/casey/just), `just run` installa anche l'icona per il pannello del desktop. In un
terminale, `idetea .` apre l'IDE e restituisce il prompt; `idetea -f .` resta attaccato (con i log).

## Roadmap

- [x] Editor, schede, temi, riquadro comandi, config TOML, albero dei file, barra di stato.
- [x] Ricerca nel progetto, LSP, pannello Problemi, terminale integrato.
- [x] Git (push/pull compresi), menu contestuali, guida, impostazioni, icone, pannelli ridimensionabili.
- [x] Debug con gdb: punti di interruzione, passi, stack di chiamate e variabili.
- [x] Agenti: pannello Interfacce, terminali di agente, elenco consentiti, aiutanti nei worktree, progetto nuovo con le skill.
- [ ] Plugin di terze parti in WASM, senza ricompilare l'IDE (esiste un prototipo dietro la feature `plugins`).
- [ ] Altri adattatori di debug (codelldb, debugpy, delve…).
- [ ] Segni git dentro l'editor; codifiche diverse da UTF-8.

Il piano dettagliato, fase per fase, è in [SPEC.md](../SPEC.md) (in portoghese).
