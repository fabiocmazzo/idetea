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
