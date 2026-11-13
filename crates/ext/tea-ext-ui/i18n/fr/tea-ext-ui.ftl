ui-panel = Interfaces
cmd-ui-show = Interfaces : afficher le panneau
ui-empty = Aucune interface ouverte. Un agent peut en créer une en écrivant un fichier dans .idetea/ui/ (voir docs/interfaces/formato.md). Pour l'activer ou la désactiver : [extensions] interfaces = {"{"} enabled = true {"}"} dans la configuration.
ui-tracked = { $path } est suivi par git et a été ignoré par sécurité : les interfaces viennent seulement de l'agent qui tourne sur cette machine, jamais d'un dépôt cloné.
ui-invalid = Document non valide : { $error }
ui-origin = { $path }
ui-origin-source = { $path } · { $source }
ui-waiting = { $count ->
    [one] { $count } réponse envoyée ; en attente que l'agent la lise.
   *[other] { $count } réponses envoyées ; en attente que l'agent les lise.
}
ui-received = Réponse reçue par l'agent.
ui-agent-waiting = L'agent attend la réponse.
ui-agent-stopped = { $minutes ->
    [0] L'agent vient d'arrêter d'attendre : la réponse est conservée jusqu'à ce qu'il revienne.
   *[other] L'agent a arrêté d'attendre il y a { $minutes } min : la réponse est conservée jusqu'à ce qu'il revienne.
}
ui-new = Nouvelle interface de l'agent : { $title }
ui-updated = Interface mise à jour : { $title }
ui-open-later = L'agent a demandé d'ouvrir { $path } : c'est dans le panneau Interfaces (non ouvert, pour ne pas interrompre votre saisie).
ui-write-failed = Impossible d'écrire la réponse : { $error }
ui-prepare-failed = Impossible de préparer le dossier .idetea/ui : { $error }
ui-not-a-file = pas un fichier ordinaire
ui-surface-mismatch = le champ surface est "{ $found }", mais le fichier s'appelle { $expected }.json
cmd-ui-teach = Interfaces : enseigner aux agents de ce projet
ui-teach-hint = Pour qu'un agent de code sache utiliser ce panneau, écrivez les instructions dans le AGENTS.md du projet.
ui-teach-button = Enseigner aux agents de ce projet
ui-teach-title = Enseigner aux agents de ce projet ?
ui-teach-detail = L'IDE écrira les instructions du panneau Interfaces dans : { $files }. Le texte reste entre les marqueurs ; relancer ne met à jour que cette partie et laisse le reste. Ce sont des fichiers du projet : relisez-les avant de valider.
ui-teach-file-new = { $name } (nouveau)
ui-teach-write = Écrire
ui-cancel = Annuler
ui-teach-up-to-date = Les instructions des agents sont déjà à jour.
ui-teach-done = Instructions écrites dans { $files }.
ui-teach-failed = Impossible d'écrire les instructions : { $error }
cmd-ui-new-project = Projet : nouveau…
new-title = Nouveau projet
new-intro = L'IDE crée le dossier, le git, les compétences du modèle et les instructions du panneau, puis ouvre le projet dans une nouvelle fenêtre. Votre agent crée le projet lui-même, en suivant la demande initiale dans **AGENTS.md**. Si un agent est choisi ci-dessous, la nouvelle fenêtre demande de l'ouvrir dans un terminal de l'IDE (vous confirmez la commande).
new-field-parent = Créer dans
new-field-name = Nom du dossier
new-field-name-hint = p. ex. my-app
new-field-model = Modèle
new-field-description = En quoi consiste le projet (facultatif)
new-field-description-hint = une phrase ; elle va dans la demande initiale pour l'agent
new-field-git = Créer le dépôt git
new-field-agent = Agent à ouvrir dans un terminal de l'IDE
new-agent-none = Aucun (je l'ouvrirai)
new-agent-title = Agent : { $agent }
new-bad-agent = nom d'agent non valide : "{ $agent }" (lettres, chiffres, - et _ seulement)
new-create = Créer et ouvrir
new-model = { $model ->
    [vazio] Vide (l'agent demande de quoi il s'agit)
    [rust] Rust
    [site] Site web
    [python] Python
    [fastapi] FastAPI (API web Python)
   *[other] { $model }
}
new-creating = Création de { $dir } (les compétences arrivent par le réseau)…
new-created = Projet créé dans { $dir } ; ouverture dans une nouvelle fenêtre.
new-open-failed = Projet créé dans { $dir }, mais la fenêtre n'a pas pu s'ouvrir : { $error }
new-failed = Impossible de créer le projet : { $error }
new-bad-model = modèle inconnu : { $model }
new-bad-dir = dossier non valide : { $dir } (utilisez un chemin complet)
new-bad-name = nom de dossier non valide : "{ $name }"
new-not-empty = { $dir } existe déjà et contient des fichiers : choisissez un autre nom
new-done-dir = dossier : { $dir }
new-done-git = git : dépôt créé
new-done-agents = AGENTS.md (avec la demande initiale) et CLAUDE.md, avec les instructions du panneau
new-done-agent = demande de terminal pour { $agent } : confirmez-la dans la nouvelle fenêtre
new-open-hint = Terminé. Ouvrez-le avec : idetea { $dir }
new-usage = idetea new : "{ $arg }" n'est pas valide. Usage : idetea new <folder> [--modelo id] [--descricao text] [--sem-git] [--agente name] ; modèles : { $models }
cmd-ui-fix = Interfaces : corriger ce projet
ui-fix-title = Corriger ce projet ?
ui-fix-will = L'IDE va :
ui-fix-wont = Pour information (laissé tel quel) :
ui-fix-button = Corriger
ui-fix-nothing = Rien à corriger dans les interfaces de ce projet.
ui-fix-done = { $count ->
    [one] Corrigé : { $count } élément.
   *[other] Corrigé : { $count } éléments.
}
ui-fix-failed = Impossible de corriger : { $error }
ui-fix-hint = Pour corriger les éléments marqués "-" : idetea ui fix --apply
ui-fix-usage = idetea ui fix : argument inconnu "{ $arg }" (usage : idetea ui fix [--apply])
ui-fix-abandoned = supprimer { $path } : plus personne n'attend la réponse depuis { $since }
ui-fix-unread = { $path } : { $count ->
    [one] { $count } réponse non lue par l'agent
   *[other] { $count } réponses non lues par l'agent
} (conservé ; demandez à l'agent de lire, ou supprimez à la main)
ui-fix-leftover = supprimer { $path } : reste d'une écriture ou d'une attente inachevée
ui-fix-check = réécrire .idetea/ui/check : il appelle { $target }, qui n'existe plus
ui-fix-instructions-older = réécrire les instructions dans { $file } : elles sont à la version { $found }, plus ancienne que cet IDE
ui-fix-instructions-unversioned = réécrire les instructions dans { $file } : elles viennent d'un ancien IDE, sans la ligne de version
ui-fix-instructions-newer = { $file } a des instructions d'un IDE plus récent ({ $found }) : ouvrez le projet avec celui-ci (je ne reviens pas en arrière)
ui-outdated = Cet IDE est obsolète (il a été recompilé après son ouverture) : les réponses le disent à l'agent. Palette : "Redémarrer l'IDE".
ui-audio-failed = Impossible de lire le son : { $error }
ui-check-ok = { $file } : ok ({ $count ->
        [one] { $count } composant
       *[other] { $count } composants
    })
ui-check-error = { $file } : erreur : { $error }
ui-check-warning = avertissement : { $warning }
ui-check-media = média : { $path } : { $problem }
ui-check-name = nom de fichier non valide ("{ $name }") : lettres minuscules, chiffres, _ et -, se terminant par .json
ui-check-usage = usage : idetea ui check <file.json>…
ui-check-instructions-older = instructions : { $file } a la version { $found } et cet IDE a { $ours } : demandez à l'utilisateur d'exécuter "Interfaces : enseigner aux agents de ce projet" et de relire le fichier avant de publier
ui-check-instructions-newer = instructions : { $file } a la version { $found }, d'un IDE plus récent que celui-ci ({ $ours }) : le panneau ouvert peut ne pas comprendre ce que le fichier enseigne
ui-check-instructions-unversioned = instructions : { $file } a un ancien bloc, sans la ligne de version ; cet IDE a { $ours } : demandez à l'utilisateur d'exécuter "Interfaces : enseigner aux agents de ce projet" et de relire le fichier avant de publier
ui-teach-newer-file = { $name } (version { $version })
ui-teach-newer = Avertissement : { $files } a été écrit par un IDE plus récent que celui-ci (qui a la version { $ours } et ne connaît pas l'autre). Écrire ramène les instructions à la version de cet IDE ; en général, ouvrez le projet avec l'IDE plus récent.
ui-teach-write-anyway = Écrire quand même
ask-title = Question
ask-choose = Choisir
ask-ok = Confirmer
ask-yes = Oui
ask-no = Non
ask-not-object = le document doit être un objet JSON
ask-bad-arg = idetea ask : "{ $arg }" n'est pas valide.
ask-usage = Usage : idetea ask <document.json | -> | --text T [--options "a|b|c" | --confirm | --input Label] [--title T] [--name name] [--value /pointer] [--json] [--keep] [--timeout seconds] ; idetea ask --resume <name> ; idetea ask --close <name>
ask-invalid = idetea ask : le panneau refuserait la question : { $error }
ask-no-such = idetea ask : il n'y a pas de question "{ $name }" dans le panneau
ask-timeout = idetea ask : pas de réponse à temps ; la question "{ $name }" reste dans le panneau (idetea ask --resume { $name } attend à nouveau)
ask-outdated = idetea ask : la réponse vient d'un IDE recompilé après son ouverture (ide.outdated) ; si vous testez un changement, demandez un redémarrage de l'IDE
ask-comment = commentaire : { $comment }

# Historique Interfaces (RFC 0007)
ui-history-panel = Historique Interfaces
cmd-ui-history = Interfaces : historique
cmd-ui-clear-history = Interfaces : effacer l'historique
ui-history-failed = Impossible d'écrire dans l'historique : { $error }
ui-history-cleared = { $count ->
    [one] { $count } entrée retirée de l'historique.
   *[other] { $count } entrées retirées de l'historique.
}
ui-history-link = Historique ({ $count })
ui-history-empty = L'historique est vide.
ui-history-clear-title = Effacer l'historique ?
ui-history-clear-detail = { $count ->
    [one] { $count } entrée d'historique de ce projet (.idetea/history/) sera supprimée.
   *[other] Les { $count } entrées d'historique de ce projet (.idetea/history/) seront supprimées.
} Les interfaces ouvertes dans le panneau restent. C'est irréversible.
ui-history-clear-button = Effacer l'historique
ui-history-search = Rechercher dans les titres, réponses et commentaires
ui-history-open = Ouverts ({ $count })
ui-history-waiting-you = en attente de vous
ui-history-answered-open = répondu, encore ouvert
ui-history-today = Aujourd'hui
ui-history-yesterday = Hier
ui-history-none = Aucune question répondue pour l'instant. Chaque réponse donnée dans le panneau Interfaces est conservée ici (dans .idetea/history/, hors de git).
ui-history-no-match = Rien trouvé.
ui-history-back = ← Retour
ui-history-record-decision = Enregistrer comme décision
ui-history-decision-recorded = Décision enregistrée dans { $path } : relisez-la et complétez les conséquences.
ui-history-decision-failed = Impossible d'enregistrer la décision : { $error }
adr-context = Contexte et problème
adr-options = Options envisagées
adr-decision = Décision
adr-chosen = Choisi
adr-consequences = Conséquences
adr-consequences-todo = _À compléter : ce qui s'améliore, ce qui se dégrade._
adr-origin = Enregistrée à partir de la question « { $name } » du panneau Interfaces de l'IDE TEA, répondue le { $date }.
ui-history-shown = Affiché : { $time }
ui-history-answered = Répondu : { $time }
ui-history-closed = Fermé sans réponse : { $time }
ui-history-read = Lu par l'agent : { $time }
ui-history-button = Bouton : { $action }
ui-history-not-read = L'agent n'a pas lu cette réponse (le fichier est encore dans .idetea/ui/responses/, ou il a été supprimé pendant que l'IDE était fermé).
ui-history-not-read-short = non lu
ui-history-unanswered = pas de réponse
ui-history-media-changed = { $path } a changé ou a disparu depuis la réponse : ce que vous voyez ici est la version actuelle.
ui-history-no-audio = l'historique ne lit pas les sons
history-cli-comment = commentaire
history-cli-not-read = l'agent n'a pas lu cette réponse
history-cli-unanswered = pas de réponse (l'agent a supprimé la question)
history-cli-bad = idetea history : "{ $arg }" n'est pas valide.
history-cli-usage = Usage : idetea history [--name name] [--since YYYY-MM-DD] [--last N | --all] [--json]
history-cli-empty = idetea history : rien dans l'historique de ce projet.
