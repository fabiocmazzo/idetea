## Erros do documento (aparecem no painel, para o usuário e para o agente corrigir)
ui-too-big = El documento tiene { $size } bytes; el máximo es { $max }.
ui-not-json = El archivo no es un JSON válido: { $error }
ui-not-object = El documento debe ser un objeto JSON.
ui-wrong-format = "format" debe ser "idetea-ui" (llegó { $found }).
ui-unsupported-version = La versión { $version } del formato no es compatible (este IDE entiende la { $supported }).
ui-missing = Falta el campo "{ $field }".
ui-wrong-type = "{ $field }" debe ser del tipo { $expected }.
ui-too-many = "{ $field }" tiene más de { $max } elementos.
ui-too-long = "{ $field }" supera los { $max } bytes.
ui-bad-id = "{ $field }": id no válido ("{ $id }"). Usa letras, números, "_" y "-" (hasta 64).
ui-duplicate-id = Id repetido: "{ $id }".
ui-bad-path = "{ $field }": "{ $path }" no es una ruta válida (ej.: "/nombre").
ui-bad-file-path = "{ $field }": "{ $path }" debe ser una ruta relativa a la raíz del proyecto, sin "..".
ui-data-too-big = "data" supera los { $max } bytes.
ui-data-too-deep = "data" tiene más de { $max } niveles.

## Avisos (o documento abre mesmo assim)
ui-unknown-field = Campo desconocido ignorado: "{ $field }".
ui-unknown-value = "{ $field }": valor desconocido "{ $value }", se usa el predeterminado.
ui-unknown-component = El componente "{ $component }" no es compatible en esta versión.
ui-link-error = No se pudo abrir { $url }: { $error }
ui-link-ignored = Enlace ignorado (solo web, archivos del proyecto e idetea:config): { $url }
ui-comment-label = Comentario para el agente (opcional)
ui-comment-placeholder = va junto con el botón que pulses
ui-copy = Copiar
ui-copied = Copiado.
ui-steps-missing = { $count ->
    [one] Falta marcar 1 paso.
   *[other] Faltan marcar { $count } pasos.
}
ui-step-optional = (opcional)
ui-requires-not-steps = "{ $field }" debe apuntar al "value" de un "steps"; ignorado.
ui-too-many-images = El documento tiene más de { $max } imágenes.
ui-image-loading = { $path }: cargando…
ui-image-error = no se pudo leer: { $error }
ui-image-not-file = no es un archivo (los symlinks no se siguen)
ui-image-too-big = supera { $max } MB
ui-image-too-large = { $width }×{ $height } px; el máximo es { $max } por lado
ui-image-total = las imágenes del documento superan { $max } millones de píxeles
ui-too-many-sounds = El documento tiene más de { $max } sonidos.
ui-audio-play = Reproducir
ui-audio-stop = Detener
ui-audio-info = { $seconds } s · { $rate } Hz · { $channels ->
        [one] mono
        [2] estéreo
       *[other] { $channels } canales
    } · { $bits } bits · { $peak }
ui-audio-peak = pico { $db } dBFS
ui-audio-silent = silencio
ui-audio-loop = en bucle
ui-audio-clips = Llega a 0 dBFS: puede estar saturando.
ui-audio-extension = No es un sonido (aceptados: { $list }).
ui-audio-too-big = Mayor que { $max } MB: se reproduce, sin duración ni forma de onda.
ui-audio-unreadable = Sin duración ni forma de onda ({ $error }); se reproduce igual.
ui-audio-no-player = No se encontró un reproductor de sonido (pw-play, paplay, aplay o afplay).
ui-audio-plugin = Los paneles de plugin no reproducen sonido.
