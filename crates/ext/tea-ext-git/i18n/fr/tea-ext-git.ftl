## Git
cmd-git-focus = Git : afficher les modifications
cmd-git-refresh = Git : actualiser
cmd-git-stage-all = Git : tout indexer
cmd-git-unstage-all = Git : tout désindexer
cmd-git-commit = Git : valider (place le curseur dans le message)
cmd-git-pull = Git : tirer (télécharger et fusionner)
cmd-git-push = Git : pousser (envoyer les commits)
cmd-git-fetch = Git : récupérer (voir les nouveautés sur le dépôt distant)
git-panel = Git
cancel = Annuler
copied = Copié : { $text }
git-notice = Git : { $message }
git-wait = Git : veuillez patienter, encore { $running }
git-fetching = Récupération…
git-pulling = Tirage…
git-pushing = Poussée…
git-publishing = Publication…
git-no-repo-notice = Git : ce dossier n'est pas dans un dépôt
git-no-repo = Ce dossier n'est pas dans un dépôt git.
git-need-message = Git : écrivez le message de validation
git-nothing-staged = Git : rien d'indexé (utilisez + pour ajouter)
git-detached-pull = Git : HEAD détachée : extrayez une branche pour tirer
git-detached-push = Git : HEAD détachée : extrayez une branche pour pousser
git-no-upstream = Git : cette branche ne suit aucun dépôt distant (poussez pour la publier)
git-no-remote = Git : le dépôt n'a pas de remote (git remote add origin <url>)
git-discarded = modifications abandonnées dans { $path }
git-discard-untracked = Le fichier est nouveau (non suivi) et va à la corbeille.
git-discard-tracked = Les modifications non indexées seront perdues ; le fichier revient à ce qui est indexé ou dans le dernier commit.
git-discard-title = Abandonner les modifications dans "{ $path }" ?
git-discard = Abandonner
git-publish-title = Publier la branche "{ $branch }" ?
git-publish-detail = Elle n'existe pas encore sur { $remote }. La poussée crée la branche là-bas et commence à la suivre (git push -u { $remote } { $branch }).
git-publish = Publier
git-fetch-tip = Récupérer : voir les nouveautés sur le dépôt distant
git-pull-tip = Tirer : télécharger et fusionner{ $count }
git-push-tip = Pousser : envoyer les commits{ $count }
git-refresh = Actualiser
git-commit-placeholder = Message de validation (Entrée)
git-staged = Indexé
git-changes = Modifications
git-unstage-all = Tout désindexer
git-stage-all = Tout indexer
git-unstage = Annuler l'indexation
git-stage = Indexer
git-nothing-to-commit = Rien à valider.
git-open-file = Ouvrir le fichier
git-view-diff = Voir le diff
git-discard-menu = Abandonner les modifications…
git-copy-path = Copier le chemin
git-copy-relative = Copier le chemin relatif
git-select-file = Sélectionnez un fichier pour voir son diff.
git-diff-truncated = … diff coupé à { $count } lignes
git-no-diff = Aucune différence (fichier binaire ou permissions seulement).
git-run-error = impossible de lancer git : { $error }
git-unknown-error = erreur inconnue
git-trash-error = impossible de mettre à la corbeille : { $error }
git-timeout = délai dépassé ({ $seconds } s) en parlant au dépôt distant
git-explain-rejected = le dépôt distant a des commits que vous n'avez pas : tirez d'abord
git-explain-diverged = les branches locale et distante ont divergé : réglez pull.rebase (ou résolvez dans le terminal)
git-explain-conflict = tirage avec conflits : résolvez les fichiers marqués et validez
git-explain-overwritten = des modifications locales seraient écrasées : validez ou abandonnez-les avant de tirer
git-explain-auth = échec de l'authentification : configurez un assistant d'identifiants (p. ex. gh auth setup-git)
git-explain-ssh = ssh a refusé la connexion : chargez la clé dans l'agent (ssh-add) ou confirmez le serveur dans le terminal
git-explain-network = impossible de joindre le dépôt distant (réseau ou adresse)
git-fetch-done = récupération terminée
git-up-to-date = déjà à jour
git-pull-done = tirage terminé
git-published = branche { $branch } publiée sur { $remote }
git-nothing-to-push = rien à pousser
git-push-done = poussée terminée

## Branches
cmd-git-checkout = Git : changer de branche
git-checkout-placeholder = Passer à la branche… (ou saisissez un nouveau nom)
git-create-branch = Créer une branche
git-branch-current = actuelle · { $date }
git-switched = Vous êtes sur la branche { $branch }
git-switch-failed = Impossible de passer à { $branch } : { $error }
git-explain-switch-overwritten = des modifications non validées entrent en conflit avec l'autre branche : validez ou abandonnez-les d'abord

## Blame
git-blame-uncommitted = Vous · pas encore validé
git-blame-detail = 
    { $author }, { $ago }
    Commit { $hash }
git-blame-copy = Copier le hash
git-close = Fermer
git-ago-now = à l'instant
git-ago-minutes = 
    { $count ->
        [one] il y a { $count } minute
       *[other] il y a { $count } minutes
    }
git-ago-hours = 
    { $count ->
        [one] il y a { $count } heure
       *[other] il y a { $count } heures
    }
git-ago-days = 
    { $count ->
        [one] il y a { $count } jour
       *[other] il y a { $count } jours
    }
git-ago-months = 
    { $count ->
        [one] il y a { $count } mois
       *[other] il y a { $count } mois
    }
git-ago-years = 
    { $count ->
        [one] il y a { $count } an
       *[other] il y a { $count } ans
    }
