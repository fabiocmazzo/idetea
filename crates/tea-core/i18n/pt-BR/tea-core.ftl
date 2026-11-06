## Contagens (painel Referências)
count-references = 
    { $count ->
        [one] { $count } referência
       *[other] { $count } referências
    }
count-definitions = 
    { $count ->
        [one] { $count } definição
       *[other] { $count } definições
    }
count-implementations = 
    { $count ->
        [one] { $count } implementação
       *[other] { $count } implementações
    }
count-types = 
    { $count ->
        [one] { $count } tipo
       *[other] { $count } tipos
    }

## Símbolos (estrutura do arquivo)
kind-module = módulo
kind-class = classe
kind-struct = struct
kind-interface = interface/trait
kind-enum = enum
kind-variant = variante
kind-method = método
kind-function = função
kind-constructor = construtor
kind-field = campo
kind-property = propriedade
kind-variable = variável
kind-constant = constante
kind-type-parameter = parâmetro de tipo
kind-block = bloco
kind-other = símbolo
visibility-public = público
visibility-private = privado

## Erros do config e dos atalhos
chord-empty = atalho vazio
chord-unknown-modifier = modificador desconhecido: { $modifier }
chord-no-key = atalho sem tecla: { $chord }
config-key-not-text = { $layer }: [keys] { $chord } deve ser texto, veio { $value }
config-keys-not-table = { $layer }: [keys] deve ser uma tabela
config-unknown-section = seção desconhecida: [{ $section }]
config-agents-project = { $layer }: [agents] só vale no config do usuário (~/.config/idetea/config.toml); ignorado aqui
config-font-missing = [editor] font = "{ $name }": fonte não instalada, usando { $default }
config-font-weight = [editor] font_weight = "{ $name }": use um destes: { $choices }
config-file-icons = [ui] file_icons = "{ $value }": use "seti" ou "none"
