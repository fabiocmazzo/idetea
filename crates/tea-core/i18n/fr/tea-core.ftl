## Compteurs (panneau Références)
count-references = 
    { $count ->
        [one] { $count } référence
       *[other] { $count } références
    }
count-definitions = 
    { $count ->
        [one] { $count } définition
       *[other] { $count } définitions
    }
count-implementations = 
    { $count ->
        [one] { $count } implémentation
       *[other] { $count } implémentations
    }
count-types = 
    { $count ->
        [one] { $count } type
       *[other] { $count } types
    }

## Symboles (structure du fichier)
kind-module = module
kind-class = classe
kind-struct = struct
kind-interface = interface/trait
kind-enum = enum
kind-variant = variante
kind-method = méthode
kind-function = fonction
kind-constructor = constructeur
kind-field = champ
kind-property = propriété
kind-variable = variable
kind-constant = constante
kind-type-parameter = paramètre de type
kind-block = bloc
kind-other = symbole
visibility-public = public
visibility-private = privé

## Erreurs de configuration et de raccourcis
chord-empty = raccourci vide
chord-unknown-modifier = modificateur inconnu : { $modifier }
chord-no-key = raccourci sans touche : { $chord }
config-key-not-text = { $layer } : [keys] { $chord } doit être du texte, reçu { $value }
config-keys-not-table = { $layer } : [keys] doit être une table
config-unknown-section = section inconnue : [{ $section }]
config-agents-project = { $layer } : [agents] ne fonctionne que dans la configuration utilisateur (~/.config/idetea/config.toml) ; ignoré ici
config-font-missing = [editor] font = "{ $name }" : police non installée, utilisation de { $default }
config-font-weight = [editor] font_weight = "{ $name }" : utilisez l'une de { $choices }
config-file-icons = [ui] file_icons = "{ $value }" : utilisez "seti" ou "none"
