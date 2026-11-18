## Éditeur, images et PDF
not-utf8 = le fichier n'est pas en UTF-8 (pas encore pris en charge)
untitled = sans titre
image-info = { $width } × { $height } px · { $size } · la molette zoome, glisser pour déplacer
image-error = Impossible de lire l'image : { $error }
pdf-error = Impossible d'ouvrir le PDF : { $error }
pdf-page = Page { $current } sur { $total }
pdf-fit = Ajuster
pdf-encrypted = PDF protégé par mot de passe (pas encore pris en charge)
pdf-invalid = fichier non valide ou endommagé

language-image = Image
language-light = { $language } (mode allégé)

## Marques git
hunk-added = 
    { $count ->
        [one] { $count } nouvelle ligne
       *[other] { $count } nouvelles lignes
    }
hunk-modified = 
    { $count ->
        [one] { $count } ligne modifiée ; avant :
       *[other] { $count } lignes modifiées ; avant :
    }
hunk-deleted = 
    { $count ->
        [one] { $count } ligne supprimée :
       *[other] { $count } lignes supprimées :
    }
hunk-added-note = Pas dans le dernier commit.
hunk-revert = Rétablir la modification
image-saved = enregistrée à { $time }

## Vue tableau (RFC 0020)
table-show-text = Texte
table-filter = Filtrer
table-rows = { $count ->
    [one] 1 ligne
   *[other] { $count } lignes
}
table-rows-filtered = { $shown } sur { $count } lignes
table-sorted = trié par { $column } { $arrow }
table-invalid-json = Pas du JSON : { $error }
table-broken-row = Cette ligne n’a pas pu être lue ; modifiez-la dans le texte.
table-out-of-range = Cette cellule n’existe pas.
table-header-row = La première ligne est l’en-tête
table-rows-picked = { $count ->
    [one] 1 ligne choisie
   *[other] { $count } lignes choisies
}
