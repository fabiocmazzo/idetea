# Il manifesto di IDE TEA

<p><a href="MANIFESTO.md">English</a> · <a href="MANIFESTO.pt-BR.md">Português</a> · <a href="MANIFESTO.es.md">Español</a> ·
<a href="MANIFESTO.fr.md">Français</a> · <a href="MANIFESTO.de.md">Deutsch</a> · <b>Italiano</b> ·
<a href="MANIFESTO.ja.md">日本語</a> · <a href="MANIFESTO.zh-CN.md">中文（简体）</a></p>

L'editor è il posto dove passi la giornata. Dovrebbe sembrare una stanza silenziosa, non un centro commerciale.

## 1. Veloce è una funzionalità

Aspettare spezza la concentrazione. IDE TEA si apre in un battito di ciglia, apre i file grandi senza bloccarsi, e
fa il lavoro lento (ricerca, git, server di linguaggio) lontano dallo schermo su cui stai scrivendo. Se qualcosa è
lento, è un bug.

## 2. Silenzioso di default

Niente account. Niente telemetria. Niente finestre su funzioni che non hai chiesto. La rete si usa solo quando lo
chiedi tu (push, pull, recupera). Niente cambia sotto i piedi: le impostazioni stanno in un file TOML in chiaro,
che puoi leggere, versionare e condividere.

## 3. Prevedibile, prima che furbo

Ogni scelta rapida è elencata in un solo posto (`F1`) e si può cambiare. Un pannello resta dove lo metti. Chiudere
qualcosa con modifiche non salvate chiede sempre. Eliminare un file lo manda nel cestino. I file cambiati fuori
dall'IDE si notano, e il lavoro non salvato non viene mai buttato via per «aiutarti».

## 4. Piccolo dentro, completo fuori

Il nucleo modifica solo file. L'albero, il terminale, git, il debugger e tutto il resto sono estensioni che puoi
spegnere. Di un nucleo piccolo ci si fida di più: si legge meglio, ed è più difficile da rompere.

## 5. Gli agenti sono i benvenuti, e resti tu al comando

Gli agenti di codice fanno parte di come si lavora adesso. IDE TEA gli dà un modo semplice, aperto e fatto di file
per parlarti: domande come moduli veri, terminali che possono chiedere, aiutanti nel proprio worktree (albero di
lavoro). Le regole sono sempre le stesse:

- **Lo vedi prima che parta.** Ogni comando che un agente chiede viene mostrato per intero e aspetta la tua
  approvazione, a meno che tu non lo metta nell'elenco consentiti della *tua* config. Un progetto non può mai darsi il
  permesso da solo.
- **La tua risposta è la risposta.** Un'approvazione è una decisione, non un clic per togliersi di mezzo una
  finestra. I pulsanti che contano aspettano che tu abbia fatto i passi.
- **File semplici, senza vincoli.** Il protocollo è JSON in una cartella. Qualsiasi agente che sa leggere e
  scrivere file può usarlo, e ogni messaggio lo puoi leggere tu.

## 6. Gratis, sul serio

IDE TEA è Apache 2.0 e gratuito, e lo resta. Nessun piano a pagamento che nasconde le parti buone.

## 7. Fatto per menti diverse

Il nome ha un secondo senso: in portoghese, TEA è la sigla dell'autismo (disturbo dello spettro autistico). IDE TEA
è fatto da uno sviluppatore autistico, e si vede in quello che conta: poco rumore, testo chiaro, cose che si comportano
sempre allo stesso modo, e spazio per concentrarsi. Se anche la tua mente funziona così, benvenuto. Se no, è
probabile che ti piaccia lo stesso.
