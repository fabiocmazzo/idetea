## Contagens (painel Referências)
count-references = 
    { $count ->
        [one] { $count } referencia
       *[other] { $count } referencias
    }
count-definitions = 
    { $count ->
        [one] { $count } definición
       *[other] { $count } definiciones
    }
count-implementations = 
    { $count ->
        [one] { $count } implementación
       *[other] { $count } implementaciones
    }
count-types = 
    { $count ->
        [one] { $count } tipo
       *[other] { $count } tipos
    }

## Símbolos (estrutura do arquivo)
kind-module = módulo
kind-class = clase
kind-struct = struct
kind-interface = interfaz/trait
kind-enum = enum
kind-variant = variante
kind-method = método
kind-function = función
kind-constructor = constructor
kind-field = campo
kind-property = propiedad
kind-variable = variable
kind-constant = constante
kind-type-parameter = parámetro de tipo
kind-block = bloque
kind-other = símbolo
visibility-public = público
visibility-private = privado

## Erros do config e dos atalhos
chord-empty = atajo vacío
chord-unknown-modifier = modificador desconocido: { $modifier }
chord-no-key = atajo sin tecla: { $chord }
config-key-not-text = { $layer }: [keys] { $chord } debe ser texto, llegó { $value }
config-keys-not-table = { $layer }: [keys] debe ser una tabla
config-unknown-section = sección desconocida: [{ $section }]
config-agents-project = { $layer }: [agents] solo vale en la config del usuario (~/.config/idetea/config.toml); ignorado aquí
config-font-missing = [editor] font = "{ $name }": fuente no instalada, usando { $default }
config-font-weight = [editor] font_weight = "{ $name }": usa uno de estos: { $choices }
