## Panneau Recherche
search-panel = Rechercher
cmd-search-focus = Recherche : rechercher dans le projet
cmd-search-clear = Recherche : effacer
search-placeholder = Rechercher dans le projet (Entrée relance la recherche)
search-bad-regex = Expression régulière non valide : { $error }
search-running = Recherche…
search-truncated = {" "}(arrêté à { $count })
search-summary = { $results } résultat(s) dans { $files } fichier(s){ $more } · { $scanned } fichiers lus
search-min-chars = Saisissez au moins { $count } caractères

## Remplacer
cmd-search-replace = Recherche : remplacer dans le projet
search-replace-placeholder = Remplacer par (Entrée remplace tout)
search-replace-all = Tout remplacer
search-replace-one = Remplacer
search-cancel = Annuler
search-replace-all-title = Remplacer dans { $files } fichier(s) ?
search-replace-all-message = 
    { $results } résultat(s) de "{ $query }" deviennent "{ $replacement }".
    
    Les fichiers ouverts dans des onglets restent modifiés, non enregistrés (Ctrl+Z annule). Les autres sont écrits directement sur le disque, sans annulation.
search-replace-all-truncated = La recherche s'est arrêtée à { $count } résultats : seuls les fichiers listés changent. Recherchez à nouveau ensuite pour voir le reste.
## Périmètre, filtres et navigation
cmd-search-next = Recherche : résultat suivant
cmd-search-prev = Recherche : résultat précédent
search-filters = Filtres
search-context = Contexte
search-history = Recherches récentes
search-history-empty = Aucune recherche récente
search-include-placeholder = fichiers à inclure (p. ex. src/**, *.rs)
search-exclude-placeholder = fichiers à exclure (p. ex. *.test.rs)
search-bad-glob = Motif invalide : { $error }
search-no-results = Aucun résultat
search-scope-folder = Dans le dossier { $path }
search-scope-file = Dans le fichier { $path }
search-scope-clear = Ne plus limiter la recherche à ce chemin
