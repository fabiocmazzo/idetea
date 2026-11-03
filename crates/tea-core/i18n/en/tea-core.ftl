## Contagens (painel Referências)
count-references = 
    { $count ->
        [one] { $count } reference
       *[other] { $count } references
    }
count-definitions = 
    { $count ->
        [one] { $count } definition
       *[other] { $count } definitions
    }
count-implementations = 
    { $count ->
        [one] { $count } implementation
       *[other] { $count } implementations
    }
count-types = 
    { $count ->
        [one] { $count } type
       *[other] { $count } types
    }

## Símbolos (estrutura do arquivo)
kind-module = module
kind-class = class
kind-struct = struct
kind-interface = interface/trait
kind-enum = enum
kind-variant = variant
kind-method = method
kind-function = function
kind-constructor = constructor
kind-field = field
kind-property = property
kind-variable = variable
kind-constant = constant
kind-type-parameter = type parameter
kind-block = block
kind-other = symbol
visibility-public = public
visibility-private = private

## Erros do config e dos atalhos
chord-empty = empty shortcut
chord-unknown-modifier = unknown modifier: { $modifier }
chord-no-key = shortcut without a key: { $chord }
config-key-not-text = { $layer }: [keys] { $chord } must be text, got { $value }
config-keys-not-table = { $layer }: [keys] must be a table
config-unknown-section = unknown section: [{ $section }]
config-agents-project = { $layer }: [agents] only works in the user config (~/.config/idetea/config.toml); ignored here
