## Editor, Bilder und PDF
binary-file = das ist eine Binärdatei, kein Text
not-valid-in = die Datei ist kein gültiges { $encoding }
not-in-encoding = „{ $ch }“ (Zeile { $line }, Spalte { $column }) gibt es in { $encoding } nicht; als UTF-8 speichern, um es zu behalten
untitled = unbenannt
image-info = { $width } × { $height } px · { $size } · Mausrad ändert den Zoom, Ziehen verschiebt
image-error = Bild konnte nicht gelesen werden: { $error }
pdf-error = PDF konnte nicht geöffnet werden: { $error }
pdf-page = Seite { $current } von { $total }
pdf-fit = Anpassen
pdf-encrypted = passwortgeschütztes PDF (noch nicht unterstützt)
pdf-invalid = ungültige oder beschädigte Datei

language-image = Bild
language-light = { $language } (Leichtmodus)

## Git-Markierungen
hunk-added = 
    { $count ->
        [one] { $count } neue Zeile
       *[other] { $count } neue Zeilen
    }
hunk-modified = 
    { $count ->
        [one] { $count } geänderte Zeile; vorher:
       *[other] { $count } geänderte Zeilen; vorher:
    }
hunk-deleted = 
    { $count ->
        [one] { $count } gelöschte Zeile:
       *[other] { $count } gelöschte Zeilen:
    }
hunk-added-note = Nicht im letzten Commit.
hunk-revert = Änderung zurücksetzen
image-saved = gespeichert um { $time }

## Tabellenansicht (RFC 0020)
table-show-text = Text
table-filter = Filtern
table-rows = { $count ->
    [one] 1 Zeile
   *[other] { $count } Zeilen
}
table-rows-filtered = { $shown } von { $count } Zeilen
table-sorted = sortiert nach { $column } { $arrow }
table-invalid-json = Kein JSON: { $error }
table-broken-row = Diese Zeile konnte nicht gelesen werden; bearbeiten Sie sie im Text.
table-out-of-range = Diese Zelle gibt es nicht.
table-header-row = Erste Zeile ist die Kopfzeile
table-rows-picked = { $count ->
    [one] 1 Zeile gewählt
   *[other] { $count } Zeilen gewählt
}
