## Dokumentfehler (im Bereich, für Sie und den Agent)
ui-too-big = Das Dokument hat { $size } Bytes; das Maximum ist { $max }.
ui-not-json = Die Datei ist kein gültiges JSON: { $error }
ui-not-object = Das Dokument muss ein JSON-Objekt sein.
ui-wrong-format = "format" muss "idetea-ui" sein (erhalten: { $found }).
ui-unsupported-version = Formatversion { $version } wird nicht unterstützt (diese IDE versteht { $supported }).
ui-missing = Feld "{ $field }" fehlt.
ui-wrong-type = "{ $field }" muss vom Typ { $expected } sein.
ui-too-many = "{ $field }" hat mehr als { $max } Einträge.
ui-too-long = "{ $field }" ist länger als { $max } Bytes.
ui-bad-id = "{ $field }": ungültige ID ("{ $id }"). Buchstaben, Ziffern, "_" und "-" verwenden (bis 64).
ui-duplicate-id = Doppelte ID: "{ $id }".
ui-bad-path = "{ $field }": "{ $path }" ist kein gültiger Pfad (z. B. "/name").
ui-bad-file-path = "{ $field }": "{ $path }" muss ein Pfad relativ zum Projektstamm sein, ohne "..".
ui-data-too-big = "data" ist größer als { $max } Bytes.
ui-data-too-deep = "data" ist tiefer als { $max } Ebenen.

## Warnungen (das Dokument öffnet sich trotzdem)
ui-unknown-field = Unbekanntes Feld ignoriert: "{ $field }".
ui-unknown-value = "{ $field }": unbekannter Wert "{ $value }", der Standard wird verwendet.
ui-unknown-component = Komponente "{ $component }" wird in dieser Version nicht unterstützt.
ui-link-error = { $url } konnte nicht geöffnet werden: { $error }
ui-link-ignored = Link ignoriert (nur Web, Projektdateien und idetea:config): { $url }
ui-comment-label = Kommentar für den Agent (optional)
ui-comment-placeholder = wird mit der Schaltfläche mitgesendet, die Sie anklicken
ui-copy = Kopieren
ui-copied = Kopiert.
ui-steps-missing = { $count ->
    [one] Noch { $count } Schritt abhaken.
   *[other] Noch { $count } Schritte abhaken.
}
ui-step-optional = (optional)
ui-requires-unbound = "{ $field }": "{ $path }" ist nicht das "value" einer Komponente; ignoriert.
ui-fields-missing = Noch auszufüllen: { $fields }
ui-too-many-images = Das Dokument hat mehr als { $max } Bilder.
ui-image-loading = { $path }: wird geladen…
ui-image-error = konnte nicht gelesen werden: { $error }
ui-image-not-file = keine Datei (symbolische Links werden nicht verfolgt)
ui-image-too-big = größer als { $max } MB
ui-image-too-large = { $width }×{ $height } px; das Maximum ist { $max } pro Seite
ui-image-total = die Bilder des Dokuments überschreiten { $max } Millionen Pixel
ui-too-many-sounds = Das Dokument hat mehr als { $max } Klänge.
ui-audio-play = Wiedergabe
ui-audio-stop = Stopp
ui-audio-info = { $seconds } s · { $rate } Hz · { $channels ->
        [one] Mono
        [2] Stereo
       *[other] { $channels } Kanäle
    } · { $bits } Bit · { $peak }
ui-audio-peak = Spitze { $db } dBFS
ui-audio-silent = Stille
ui-audio-loop = Wiederholung
ui-audio-clips = Erreicht 0 dBFS: möglicherweise übersteuert.
ui-audio-extension = Kein Klang (zulässig: { $list }).
ui-audio-too-big = Größer als { $max } MB: wird abgespielt, ohne Dauer oder Wellenform.
ui-audio-unreadable = Keine Dauer oder Wellenform ({ $error }); die Wiedergabe funktioniert trotzdem.
ui-audio-no-player = Kein Wiedergabeprogramm gefunden (pw-play, paplay, aplay oder afplay).
ui-audio-plugin = Plugin-Bereiche geben keinen Ton wieder.
ui-sprite-pause = Anhalten
ui-sprite-play = Animieren
ui-sprite-step = Nächstes Bild
ui-sprite-info = Bild { $frame } von { $count } · { $fps } fps
ui-sprite-frame = { $field }: frame_width und frame_height gehören zusammen.
ui-sprite-too-small = Das Blatt ({ $width }×{ $height }) ist kleiner als ein Bild.
ui-open-blocked = { $path }: idetea:open-Links öffnen nur { $list }.
ui-open-not-file = { $path } ist keine reguläre Datei (fehlt, ein Ordner oder eine symbolische Verknüpfung).
ui-select-text = Text auswählen
ui-select-done = Formatiert anzeigen
