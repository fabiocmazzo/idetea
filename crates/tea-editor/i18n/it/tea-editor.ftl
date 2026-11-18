## Editor, immagini e PDF
binary-file = è un file binario, non testo
not-valid-in = il file non è { $encoding } valido
not-in-encoding = «{ $ch }» (riga { $line }, colonna { $column }) non esiste in { $encoding }; salva come UTF-8 per mantenerlo
untitled = senza titolo
image-info = { $width } × { $height } px · { $size } · la rotella zooma, trascina per spostare
image-error = Impossibile leggere l'immagine: { $error }
pdf-error = Impossibile aprire il PDF: { $error }
pdf-page = Pagina { $current } di { $total }
pdf-fit = Adatta
pdf-encrypted = PDF protetto da password (non ancora supportato)
pdf-invalid = file non valido o danneggiato

language-image = Immagine
language-light = { $language } (modalità leggera)

## Segni di git
hunk-added = 
    { $count ->
        [one] { $count } nuova riga
       *[other] { $count } nuove righe
    }
hunk-modified = 
    { $count ->
        [one] { $count } riga modificata; prima era:
       *[other] { $count } righe modificate; prima era:
    }
hunk-deleted = 
    { $count ->
        [one] { $count } riga eliminata:
       *[other] { $count } righe eliminate:
    }
hunk-added-note = Non presente nell'ultimo commit.
hunk-revert = Ripristina modifica
image-saved = salvato alle { $time }

## Vista tabella (RFC 0020)
table-show-text = Testo
table-filter = Filtra
table-rows = { $count ->
    [one] 1 riga
   *[other] { $count } righe
}
table-rows-filtered = { $shown } di { $count } righe
table-sorted = ordinato per { $column } { $arrow }
table-invalid-json = Non è JSON: { $error }
table-broken-row = Questa riga non si può leggere; modificala nel testo.
table-out-of-range = Questa cella non esiste.
table-header-row = La prima riga è l’intestazione
table-rows-picked = { $count ->
    [one] 1 riga scelta
   *[other] { $count } righe scelte
}
