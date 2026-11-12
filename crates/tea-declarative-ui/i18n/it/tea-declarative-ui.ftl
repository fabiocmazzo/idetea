## Errori del documento (nel pannello, per te e per l'agente)
ui-too-big = Il documento ha { $size } byte; il massimo è { $max }.
ui-not-json = Il file non è JSON valido: { $error }
ui-not-object = Il documento deve essere un oggetto JSON.
ui-wrong-format = "format" deve essere "idetea-ui" (ricevuto { $found }).
ui-unsupported-version = La versione del formato { $version } non è supportata (questo IDE capisce { $supported }).
ui-missing = Campo mancante "{ $field }".
ui-wrong-type = "{ $field }" deve essere di tipo { $expected }.
ui-too-many = "{ $field }" ha più di { $max } elementi.
ui-too-long = "{ $field }" è più lungo di { $max } byte.
ui-bad-id = "{ $field }": id non valido ("{ $id }"). Usa lettere, cifre, "_" e "-" (fino a 64).
ui-duplicate-id = Id duplicato: "{ $id }".
ui-bad-path = "{ $field }": "{ $path }" non è un percorso valido (es. "/name").
ui-bad-file-path = "{ $field }": "{ $path }" deve essere un percorso relativo alla radice del progetto, senza "..".
ui-data-too-big = "data" è più grande di { $max } byte.
ui-data-too-deep = "data" è più profondo di { $max } livelli.

## Avvisi (il documento si apre comunque)
ui-unknown-field = Campo sconosciuto ignorato: "{ $field }".
ui-unknown-value = "{ $field }": valore sconosciuto "{ $value }", uso il predefinito.
ui-unknown-component = Il componente "{ $component }" non è supportato in questa versione.
ui-link-error = Impossibile aprire { $url }: { $error }
ui-link-ignored = Collegamento ignorato (solo web, file del progetto e idetea:config): { $url }
ui-comment-label = Commento per l'agente (facoltativo)
ui-comment-placeholder = inviato insieme al pulsante che clicchi
ui-copy = Copia
ui-copied = Copiato.
ui-steps-missing = { $count ->
    [one] { $count } passaggio da spuntare.
   *[other] { $count } passaggi da spuntare.
}
ui-step-optional = (facoltativo)
ui-requires-unbound = "{ $field }": "{ $path }" non è il "value" di nessun componente; ignorato.
ui-fields-missing = Ancora da compilare: { $fields }
ui-too-many-images = Il documento ha più di { $max } immagini.
ui-image-loading = { $path }: caricamento…
ui-image-error = impossibile leggerla: { $error }
ui-image-not-file = non è un file (i symlink non si seguono)
ui-image-too-big = più grande di { $max } MB
ui-image-too-large = { $width }×{ $height } px; il massimo è { $max } per lato
ui-image-total = le immagini del documento superano { $max } milioni di pixel
ui-too-many-sounds = Il documento ha più di { $max } suoni.
ui-audio-play = Riproduci
ui-audio-stop = Interrompi
ui-audio-info = { $seconds } s · { $rate } Hz · { $channels ->
        [one] mono
        [2] stereo
       *[other] { $channels } canali
    } · { $bits } bit · { $peak }
ui-audio-peak = picco { $db } dBFS
ui-audio-silent = silenzio
ui-audio-loop = in ciclo
ui-audio-clips = Arriva a 0 dBFS: potrebbe essere in clipping.
ui-audio-extension = Non è un suono (accettati: { $list }).
ui-audio-too-big = Più grande di { $max } MB: si riproduce, senza durata né forma d'onda.
ui-audio-unreadable = Niente durata né forma d'onda ({ $error }); si riproduce comunque.
ui-audio-no-player = Nessun riproduttore audio trovato (pw-play, paplay, aplay o afplay).
ui-audio-plugin = I pannelli dei plugin non riproducono l'audio.
ui-sprite-pause = Sospendi
ui-sprite-play = Anima
ui-sprite-step = Fotogramma successivo
ui-sprite-info = fotogramma { $frame } di { $count } · { $fps } fps
ui-sprite-frame = { $field }: frame_width e frame_height vanno insieme.
ui-sprite-too-small = Il foglio ({ $width }×{ $height }) è più piccolo di un fotogramma.
ui-open-blocked = { $path }: i collegamenti idetea:open aprono solo { $list }.
ui-open-not-file = { $path } non è un file normale (manca, è una cartella o un symlink).
ui-select-text = Seleziona il testo
ui-select-done = Mostra formattato
