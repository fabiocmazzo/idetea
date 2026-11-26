## Editor, imagens e PDF
binary-file = es un archivo binario, no texto
not-valid-in = el archivo no es { $encoding } válido
not-in-encoding = «{ $ch }» (línea { $line }, columna { $column }) no existe en { $encoding }; guarda como UTF-8 para conservarlo
untitled = sin título
image-info = { $width } × { $height } px · { $size } · la rueda del ratón acerca, arrastrar mueve
image-error = No se pudo leer la imagen: { $error }
pdf-error = No se pudo abrir el PDF: { $error }
pdf-page = Página { $current } de { $total }
pdf-fit = Ajustar
pdf-encrypted = PDF protegido con contraseña (aún no compatible)
pdf-invalid = archivo no válido o dañado

language-image = Imagen
language-light = { $language } (modo ligero)

## Marcas do git
hunk-added = 
    { $count ->
        [one] 1 línea nueva
       *[other] { $count } líneas nuevas
    }
hunk-modified = 
    { $count ->
        [one] 1 línea cambiada; antes era:
       *[other] { $count } líneas cambiadas; antes era:
    }
hunk-deleted = 
    { $count ->
        [one] 1 línea borrada:
       *[other] { $count } líneas borradas:
    }
hunk-added-note = No están en el último commit.
hunk-revert = Deshacer cambio
image-saved = guardada a las { $time }

## Vista de tabla (RFC 0020)
table-show-text = Texto
table-filter = Filtrar
table-rows = { $count ->
    [one] 1 fila
   *[other] { $count } filas
}
table-rows-filtered = { $shown } de { $count } filas
table-sorted = ordenado por { $column } { $arrow }
table-invalid-json = No es JSON: { $error }
table-broken-row = Esta línea no se pudo leer; edítala en el texto.
table-out-of-range = Esta celda no existe.
table-header-row = La primera fila es el encabezado
table-rows-picked = { $count ->
    [one] 1 fila elegida
   *[other] { $count } filas elegidas
}
table-add-row = + Fila
table-delete-rows = Borrar filas
table-add-column = + Columna
table-delete-column = Borrar columna
table-new-column = columna { $number }
table-duplicate-column = Ya existe una columna con ese nombre.
table-no-header = Sin cabecera, las columnas no tienen nombre.
table-empty-name = El nombre de la columna no puede quedar vacío.
