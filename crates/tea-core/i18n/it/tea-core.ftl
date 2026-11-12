## Conteggi (pannello Riferimenti)
count-references = 
    { $count ->
        [one] { $count } riferimento
       *[other] { $count } riferimenti
    }
count-definitions = 
    { $count ->
        [one] { $count } definizione
       *[other] { $count } definizioni
    }
count-implementations = 
    { $count ->
        [one] { $count } implementazione
       *[other] { $count } implementazioni
    }
count-types = 
    { $count ->
        [one] { $count } tipo
       *[other] { $count } tipi
    }

## Simboli (struttura del file)
kind-module = modulo
kind-class = classe
kind-struct = struct
kind-interface = interfaccia/trait
kind-enum = enum
kind-variant = variante
kind-method = metodo
kind-function = funzione
kind-constructor = costruttore
kind-field = campo
kind-property = proprietà
kind-variable = variabile
kind-constant = costante
kind-type-parameter = parametro di tipo
kind-block = blocco
kind-other = simbolo
visibility-public = pubblico
visibility-private = privato

## Errori di config e scelte rapide
chord-empty = scelta rapida vuota
chord-unknown-modifier = modificatore sconosciuto: { $modifier }
chord-no-key = scelta rapida senza tasto: { $chord }
config-key-not-text = { $layer }: [keys] { $chord } deve essere testo, ricevuto { $value }
config-keys-not-table = { $layer }: [keys] deve essere una tabella
config-unknown-section = sezione sconosciuta: [{ $section }]
config-agents-project = { $layer }: [agents] funziona solo nella config utente (~/.config/idetea/config.toml); ignorato qui
config-font-missing = [editor] font = "{ $name }": tipo di carattere non installato, uso { $default }
config-font-weight = [editor] font_weight = "{ $name }": usa uno tra { $choices }
config-file-icons = [ui] file_icons = "{ $value }": usa "seti" o "none"
