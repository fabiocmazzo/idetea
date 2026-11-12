## Erreurs du document (dans le panneau, pour vous et pour l'agent)
ui-too-big = Le document fait { $size } octets ; le maximum est { $max }.
ui-not-json = Le fichier n'est pas du JSON valide : { $error }
ui-not-object = Le document doit être un objet JSON.
ui-wrong-format = "format" doit être "idetea-ui" (reçu { $found }).
ui-unsupported-version = La version de format { $version } n'est pas prise en charge (cet IDE comprend { $supported }).
ui-missing = Champ manquant "{ $field }".
ui-wrong-type = "{ $field }" doit être de type { $expected }.
ui-too-many = "{ $field }" a plus de { $max } éléments.
ui-too-long = "{ $field }" dépasse { $max } octets.
ui-bad-id = "{ $field }" : id non valide ("{ $id }"). Utilisez des lettres, des chiffres, "_" et "-" (64 au plus).
ui-duplicate-id = Id en double : "{ $id }".
ui-bad-path = "{ $field }" : "{ $path }" n'est pas un chemin valide (p. ex. "/name").
ui-bad-file-path = "{ $field }" : "{ $path }" doit être un chemin relatif à la racine du projet, sans "..".
ui-data-too-big = "data" dépasse { $max } octets.
ui-data-too-deep = "data" a plus de { $max } niveaux de profondeur.

## Avertissements (le document s'ouvre quand même)
ui-unknown-field = Champ inconnu ignoré : "{ $field }".
ui-unknown-value = "{ $field }" : valeur inconnue "{ $value }", utilisation de la valeur par défaut.
ui-unknown-component = Le composant "{ $component }" n'est pas pris en charge dans cette version.
ui-link-error = Impossible d'ouvrir { $url } : { $error }
ui-link-ignored = Lien ignoré (seulement le web, les fichiers du projet et idetea:config) : { $url }
ui-comment-label = Commentaire pour l'agent (facultatif)
ui-comment-placeholder = envoyé avec le bouton sur lequel vous cliquez
ui-copy = Copier
ui-copied = Copié.
ui-steps-missing = { $count ->
    [one] { $count } étape restante à cocher.
   *[other] { $count } étapes restantes à cocher.
}
ui-step-optional = (facultatif)
ui-requires-unbound = "{ $field }" : "{ $path }" n'est le "value" d'aucun composant ; ignoré.
ui-fields-missing = Encore à remplir : { $fields }
ui-too-many-images = Le document a plus de { $max } images.
ui-image-loading = { $path } : chargement…
ui-image-error = lecture impossible : { $error }
ui-image-not-file = pas un fichier (les liens symboliques ne sont pas suivis)
ui-image-too-big = plus de { $max } Mo
ui-image-too-large = { $width }×{ $height } px ; le maximum est { $max } par côté
ui-image-total = les images du document dépassent { $max } millions de pixels
ui-too-many-sounds = Le document a plus de { $max } sons.
ui-audio-play = Lire
ui-audio-stop = Arrêter
ui-audio-info = { $seconds } s · { $rate } Hz · { $channels ->
        [one] mono
        [2] stéréo
       *[other] { $channels } canaux
    } · { $bits } bits · { $peak }
ui-audio-peak = crête { $db } dBFS
ui-audio-silent = silence
ui-audio-loop = en boucle
ui-audio-clips = Atteint 0 dBFS : il peut saturer.
ui-audio-extension = Pas un son (acceptés : { $list }).
ui-audio-too-big = Plus de { $max } Mo : lecture sans durée ni forme d'onde.
ui-audio-unreadable = Pas de durée ni de forme d'onde ({ $error }) ; la lecture fonctionne quand même.
ui-audio-no-player = Aucun lecteur audio trouvé (pw-play, paplay, aplay ou afplay).
ui-audio-plugin = Les panneaux de plugin ne lisent pas le son.
ui-sprite-pause = Suspendre
ui-sprite-play = Animer
ui-sprite-step = Image suivante
ui-sprite-info = image { $frame } sur { $count } · { $fps } fps
ui-sprite-frame = { $field } : frame_width et frame_height vont ensemble.
ui-sprite-too-small = La feuille ({ $width }×{ $height }) est plus petite qu'une image.
ui-open-blocked = { $path } : les liens idetea:open n'ouvrent que { $list }.
ui-open-not-file = { $path } n'est pas un fichier ordinaire (absent, un dossier ou un lien symbolique).
ui-select-text = Sélectionner le texte
ui-select-done = Afficher mis en forme
