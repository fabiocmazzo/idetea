<p align="center">
  <img src="assets/idetea.svg" width="112" alt="Icona di IDE TEA: un bicchiere di tè freddo">
</p>

<h1 align="center">IDE TEA</h1>

<p align="center"><b>Solo il codice. Il ghiaccio lo metti tu.</b></p>

<p align="center">
  Un IDE veloce e silenzioso, in Rust puro, dove gli agenti di codice chiedono prima di agire.<br>
  Gratis, sempre: <a href="LICENSE">Apache 2.0</a>. Niente account, niente telemetria, niente marketplace.
</p>

<p align="center">
  <a href="https://github.com/fabiocmazzo/idetea/actions/workflows/ci.yml"><img src="https://github.com/fabiocmazzo/idetea/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://github.com/fabiocmazzo/idetea/releases"><img src="https://img.shields.io/github/v/release/fabiocmazzo/idetea?include_prereleases&amp;label=release" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue" alt="License: Apache 2.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024-orange?logo=rust" alt="Rust 2024"></a>
  <a href="https://github.com/iced-rs/iced"><img src="https://img.shields.io/badge/GUI-iced%200.14-3645ff" alt="GUI: iced 0.14"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey" alt="Platforms: Linux, macOS, Windows">
  <a href="https://codeberg.org/fabiocmazzo/idetea"><img src="https://img.shields.io/badge/Codeberg-mirror-2185D0?logo=codeberg&amp;logoColor=white" alt="Codeberg mirror"></a>
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.pt-BR.md">Português</a> · <a href="README.es.md">Español</a> ·
  <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <b>Italiano</b> ·
  <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">中文（简体）</a><br>
  <a href="https://idetea.belabs.cloud/it/"><b>Sito e guide</b></a>
  · <a href="https://github.com/fabiocmazzo/idetea">GitHub</a>
  · <a href="https://codeberg.org/fabiocmazzo/idetea">Codeberg</a> (mirror)
</p>

<p align="center">
  <a href="https://idetea.belabs.cloud/it/#demo">
    <img src="assets/demo-preview.gif" width="880" alt="Demo narrata: apertura rapida, Rust con evidenziazione della sintassi, un editor diviso, il terminale integrato, poi un modulo dell'agente nel pannello Interfacce mentre la mascotte Tea passa ad Attenzione.">
  </a>
</p>

<p align="center">
  <a href="https://idetea.belabs.cloud/it/#demo"><b>Guarda la demo di 52 secondi</b></a>
  · narrazione in inglese, sottotitoli impressi<br>
  GitHub non riproduce un video del repository dentro il README.
  Il clip sopra lo apre sul sito, dove parte con l'audio.
  Il file è <a href="assets/idetea-demo.mp4"><code>assets/idetea-demo.mp4</code></a>.
</p>

---

> **Stato:** in sviluppo attivo, prima della 1.0. Compilato e provato su Linux, macOS (Apple Silicon) e Windows (x86_64).
> Ci saranno spigoli: se ne trovi uno, segnalalo.

## A chi serve

A chi scrive codice con un agente nel terminale, accanto all'editor: Claude Code, Codex, o qualsiasi agente che legga
`AGENTS.md` e `CLAUDE.md`. IDE TEA non mette un modello di chat dentro la finestra. Dà a quell'agente un modo piccolo,
fatto di file, per farti delle domande, e resti tu a decidere ogni comando.

## Perché resti

**Veloce.** Parte ben al di sotto dei 200 ms. Un file da 50 MB si apre in un decimo di secondo, e un file minificato
con una riga enorme si apre in una modalità leggera, invece di bloccare la finestra.

**Silenzioso.** Niente account, niente telemetria, niente marketplace, niente finestre su funzioni che non hai chiesto.
La rete si usa solo quando lo chiedi tu (push, pull, recupera, o una skill che installi). Aspetto, tasti e pannelli
stanno in un unico file TOML, in chiaro.

**Decidi tu che cosa viene eseguito.** Un agente scrive un piccolo file JSON e l'IDE mostra un modulo: opzioni,
caselle, testo, immagini, suoni, test manuali passo per passo. Una richiesta di terminale mostra prima il comando
intero. Un aiutante può girare nel proprio worktree di git (albero di lavoro). Un progetto non può mai darsi il
permesso da solo. L'elenco consentiti sta nella *tua* config.

**Completo dove conta.** Più cursori, zoom temporaneo, una modalità vim facoltativa, albero dei file, ricerca nel
progetto, terminale integrato, git (aggiungi a stage, diff, commit, push e pull), LSP, debug con gdb, e anteprime di
Markdown, SVG, immagini e PDF.

**Piccolo dentro.** Il nucleo modifica solo file. L'albero, il terminale, git e il debugger sono estensioni che puoi
spegnere. Tea, una tazza in pixel art, tiene d'occhio una domanda in attesa (un clic apre il pannello giusto) e si
comprime nella barra di stato quando rivuoi l'angolo.

Un comando insegna agli agenti del progetto: «Interfacce: insegna agli agenti di questo progetto» scrive il
protocollo in `AGENTS.md` e `CLAUDE.md`.

## Le funzioni in azione

Cinque clip brevi, narrazione in inglese, sottotitoli impressi. GitHub non riproduce un video del repository
dentro il README. I poster aprono il clip sul sito, con l'audio. I file sono in `assets/`.

<table>
<tr>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/it/#feature-agents"><img src="assets/agents-poster.png" width="440" alt="Agenti: Agenti nel tuo terminale. Memoria nel tuo repository."></a><br>
    <a href="https://idetea.belabs.cloud/it/#feature-agents"><b>Agenti</b></a> · 57 secondi<br>
    Agenti nel tuo terminale. Memoria nel tuo repository.<br>
    <a href="assets/idetea-agents.mp4"><code>assets/idetea-agents.mp4</code></a>
  </td>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/it/#feature-tables"><img src="assets/tables-poster.png" width="440" alt="Tabelle: I tuoi dati come tabella. Restano solo testo."></a><br>
    <a href="https://idetea.belabs.cloud/it/#feature-tables"><b>Tabelle</b></a> · 45 secondi<br>
    I tuoi dati come tabella. Restano solo testo.<br>
    <a href="assets/idetea-tables.mp4"><code>assets/idetea-tables.mp4</code></a>
  </td>
</tr>
<tr>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/it/#feature-lsp"><img src="assets/lsp-poster.png" width="440" alt="LSP: Supporto linguistico profondo, già integrato."></a><br>
    <a href="https://idetea.belabs.cloud/it/#feature-lsp"><b>LSP</b></a> · 41 secondi<br>
    Supporto linguistico profondo, già integrato.<br>
    <a href="assets/idetea-lsp.mp4"><code>assets/idetea-lsp.mp4</code></a>
  </td>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/it/#feature-encodings"><img src="assets/encodings-poster.png" width="440" alt="Codifiche: Vecchi file, nuovo editor, non si perde nulla."></a><br>
    <a href="https://idetea.belabs.cloud/it/#feature-encodings"><b>Codifiche</b></a> · 47 secondi<br>
    Vecchi file, nuovo editor, non si perde nulla.<br>
    <a href="assets/idetea-encodings.mp4"><code>assets/idetea-encodings.mp4</code></a>
  </td>
</tr>
<tr>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/it/#feature-mascot"><img src="assets/mascot-poster.png" width="440" alt="Mascotte: Una tazza di tè calma, nella tua lingua."></a><br>
    <a href="https://idetea.belabs.cloud/it/#feature-mascot"><b>Mascotte</b></a> · 52 secondi<br>
    Una tazza di tè calma, nella tua lingua.<br>
    <a href="assets/idetea-mascot.mp4"><code>assets/idetea-mascot.mp4</code></a>
  </td>
  <td width="50%" valign="top"></td>
</tr>
</table>

## Installazione

**Linux, macOS e Windows, pronto all'uso.** La versione attuale è
[v0.22.0](https://github.com/fabiocmazzo/idetea/releases/tag/v0.22.0).
`SHA256SUMS` in quella pagina ha le somme di controllo. Cambia `VERSION` quando ne esce una più nuova, oppure scegli
il file dalle [Release](https://github.com/fabiocmazzo/idetea/releases).

**Linux (x86_64).** Decomprimi ed esegui `install.sh`. Mette `idetea` in `~/.local/bin` e nel menu delle applicazioni:

```sh
VERSION=0.22.0
curl -fsSL -O "https://github.com/fabiocmazzo/idetea/releases/download/v${VERSION}/idetea-${VERSION}-x86_64-linux.tar.gz"
tar -xzf idetea-${VERSION}-x86_64-linux.tar.gz
cd idetea-${VERSION}-x86_64-linux
./install.sh
```

Su ARM usa `idetea-${VERSION}-aarch64-linux.tar.gz`. Se preferisci un solo file, scarica
`IDE_TEA-${VERSION}-x86_64.AppImage` (o `IDE_TEA-${VERSION}-aarch64.AppImage`), poi `chmod +x` ed eseguilo.
`--install-desktop` lo aggiunge al menu.

**macOS (Apple Silicon).** L'archivio contiene `IDE TEA.app`:

```sh
VERSION=0.22.0
curl -fsSL -O "https://github.com/fabiocmazzo/idetea/releases/download/v${VERSION}/idetea-${VERSION}-aarch64-macos.tar.gz"
tar -xzf idetea-${VERSION}-aarch64-macos.tar.gz
cd idetea-${VERSION}-aarch64-macos
./idetea --install-desktop   # crea ~/Applications/IDE TEA.app
```

Non c'è ancora un pacchetto per i Mac Intel.

**Windows (x86_64).** Scarica ed esegui
[IDE_TEA-0.22.0-x86_64-setup.exe](https://github.com/fabiocmazzo/idetea/releases/download/v0.22.0/IDE_TEA-0.22.0-x86_64-setup.exe),
oppure decomprimi lo
[idetea-0.22.0-x86_64-windows.zip](https://github.com/fabiocmazzo/idetea/releases/download/v0.22.0/idetea-0.22.0-x86_64-windows.zip)
portatile (`idetea.exe` è dentro). In PowerShell:

```powershell
$Version = "0.22.0"
Invoke-WebRequest -Uri "https://github.com/fabiocmazzo/idetea/releases/download/v$Version/IDE_TEA-$Version-x86_64-setup.exe" -OutFile "IDE_TEA-$Version-x86_64-setup.exe"
Start-Process ".\IDE_TEA-$Version-x86_64-setup.exe"
```

**Dal codice sorgente.** Serve [Rust](https://rustup.rs) (stable). Il pacchetto è `tea-app`; il binario si chiama `idetea`:

```sh
cargo install --locked --git https://github.com/fabiocmazzo/idetea tea-app
idetea --install-desktop   # facoltativo: voce nel menu e icona
```

Facoltativi: `git` per l'integrazione git, `gdb` 14+ (o `lldb-dap` su macOS) per il debug, e i server di linguaggio
che usi (rust-analyzer, clangd, pyright, typescript-language-server, gopls, lua-language-server, zls, `zx lsp`, `hare-lsp` per Hare e `moon lsp` per MoonBit sono già configurati).

## Firma del codice e privacy

L'installer per Windows non ha ancora la firma del codice, quindi SmartScreen può mostrare un avviso al primo avvio.
Questo programma non trasferisce informazioni ad altri sistemi in rete, se non su richiesta esplicita di chi lo usa.
Vedi [`CODE_SIGNING_POLICY.md`](CODE_SIGNING_POLICY.md).

## Per iniziare

```sh
idetea                # apre la cartella attuale come progetto
idetea src/main.rs    # cartella attuale + questo file
idetea ~/progetto     # un'altra cartella
idetea new my-app --model rust --agent claude     # un progetto nuovo, con un agente pronto a iniziarlo
```

Dentro l'IDE: `Ctrl+Shift+P` apre il riquadro comandi, `Ctrl+P` apre un file per nome, `F1` mostra ogni scelta rapida.
Le guide sono sul [sito](https://idetea.belabs.cloud/it/).

## Perché IDE TEA, e quando no

[`docs/comparativo-ides.it.md`](docs/comparativo-ides.it.md) è un confronto onesto con VS Code, Cursor, Zed, JetBrains
e Helix, aggiornato alla 0.15.0. In breve:

- Gli agenti nel terminale hanno un protocollo nativo, fatto di file (moduli, comandi approvati, worktree, la
  mascotte Tea). È la cosa che gli altri editor di quella nota non hanno.
- Niente autocompletamento fantasma, di proposito. Se vuoi completare con Tab o i diff in linea, quel documento
  indica Cursor, Windsurf o Zed.
- La copertura del debugger è indietro rispetto a VS Code e JetBrains (solo gdb e lldb-dap).

### Tea

La tazza nell'angolo ha diverse frasi per ogni umore. Nota un salvataggio, una pausa lunga, l'ora del giorno e la
compilazione del debugger, e ogni tanto cita una scelta rapida che esiste davvero. `[mascot] chatty` vale `quiet`,
`normal` o `chatty`; `[mascot] tips` accende o spegne i suggerimenti. Tutta l'interfaccia, Tea compreso, è in
portoghese, inglese, spagnolo, francese, tedesco, italiano, giapponese e cinese semplificato (`[ui] language`, oppure
la lingua del sistema). Parla anche una volta di un conflitto sul disco con modifiche non salvate, di un
conflitto git, della fine di un'attività o della sessione di debug, di un carattere che la codifica non regge e
di una sostituzione in più file. Avvisi ed errori che arrivano a zero restano nella modalità chatty. Gli stessi
interruttori sono in Impostazioni. `idetea mascot setup-hooks` aggiunge gli hook di Claude Code e lascia quelli
già nel file. Non prende il focus.

## Metti una stella al repository

Se questo è l'editor che volevi, [metti una stella a IDE TEA](https://github.com/fabiocmazzo/idetea). È il progetto
di una persona, e la stella è il modo in cui lo trova la persona successiva.

## Sul nome

È nato da una battuta. L'IDE è fatto con **iced**, quindi doveva chiamarsi **Iced Tea**.

Poi è saltato fuori l'altro senso. In portoghese, **TEA** è la sigla di *Transtorno do Espectro Autista*: il
disturbo dello spettro autistico. L'autore è autistico, il doppio senso gli è sembrato giusto, e il nome è rimasto.
Forse spiega anche qualcosa dell'IDE: preferisce la prevedibilità, il testo semplice che si può leggere, e
meno rumore possibile.

Il resto è nel [manifesto](MANIFESTO.it.md).

## Documentazione

- [Manifesto](MANIFESTO.it.md): in che cosa crede IDE TEA, e che cosa non farà.
- [Sito](https://idetea.belabs.cloud/it/): la demo, l'installazione e le guide.
- [Contributing](CONTRIBUTING.md): come compilare, provare e aprire una pull request (in inglese).
- [Funzionalità e scelte rapide](docs/features.it.md): l'elenco completo.
- [Linguaggi](docs/languages.it.md): che cosa fa l'IDE per ogni linguaggio, Hare e MoonBit compresi, con
  [esempi](examples/languages) da provare.
- [Interfacce per gli agenti](docs/interfaces/formato.md) e [terminali di agente](docs/terminais-de-agente.md) (per
  ora in portoghese).
- [SPEC.md](SPEC.md) e [DESENVOLVIMENTO.md](DESENVOLVIMENTO.md): specifica, architettura e come compilare,
  provare e scrivere un'estensione (per ora in portoghese).

## Licenza

IDE TEA è software libero sotto la [Licenza Apache 2.0](LICENSE): usalo, studialo, modificalo e condividilo,
anche a scopo commerciale. È gratuito e lo resta.

Le icone dell'interfaccia vengono da [Codicons](https://github.com/microsoft/vscode-codicons) © Microsoft, sotto
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) (vedi
[`crates/tea-core/assets/NOTICE.md`](crates/tea-core/assets/NOTICE.md)).
