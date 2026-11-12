## Zählungen (Bereich Verweise)
count-references = 
    { $count ->
        [one] { $count } Verweis
       *[other] { $count } Verweise
    }
count-definitions = 
    { $count ->
        [one] { $count } Definition
       *[other] { $count } Definitionen
    }
count-implementations = 
    { $count ->
        [one] { $count } Implementierung
       *[other] { $count } Implementierungen
    }
count-types = 
    { $count ->
        [one] { $count } Typ
       *[other] { $count } Typen
    }

## Symbole (Dateistruktur)
kind-module = Modul
kind-class = Klasse
kind-struct = Struktur
kind-interface = Schnittstelle/Trait
kind-enum = Enumeration
kind-variant = Variante
kind-method = Methode
kind-function = Funktion
kind-constructor = Konstruktor
kind-field = Feld
kind-property = Eigenschaft
kind-variable = Variable
kind-constant = Konstante
kind-type-parameter = Typparameter
kind-block = Block
kind-other = Symbol
visibility-public = öffentlich
visibility-private = privat

## Fehler von Konfiguration und Tastenkürzeln
chord-empty = leeres Tastenkürzel
chord-unknown-modifier = unbekannter Modifikator: { $modifier }
chord-no-key = Tastenkürzel ohne Taste: { $chord }
config-key-not-text = { $layer }: [keys] { $chord } muss Text sein, erhalten: { $value }
config-keys-not-table = { $layer }: [keys] muss eine Tabelle sein
config-unknown-section = unbekannter Abschnitt: [{ $section }]
config-agents-project = { $layer }: [agents] gilt nur in der Benutzerkonfiguration (~/.config/idetea/config.toml); hier ignoriert
config-font-missing = [editor] font = "{ $name }": Schriftart nicht installiert, es wird { $default } verwendet
config-font-weight = [editor] font_weight = "{ $name }": eines von { $choices } verwenden
config-file-icons = [ui] file_icons = "{ $value }": "seti" oder "none" verwenden
