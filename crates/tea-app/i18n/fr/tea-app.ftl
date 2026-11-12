## Paramètres → Apparence

settings-theme = Thème
settings-theme-detail = Couleurs de l'IDE, de l'éditeur et du terminal.
settings-font-size = Taille de la police
settings-font-size-detail = Éditeur et terminal.
settings-code-font = Police du code
settings-code-font-detail = Éditeur, numéros de ligne et terminal. JetBrains Mono est fournie avec l'IDE ; la liste contient les polices à chasse fixe installées.
settings-font-weight = Épaisseur de la police
settings-font-weight-detail = Une épaisseur absente de la police utilise la plus proche.
settings-line-height = Hauteur de ligne
settings-line-height-detail = En multiples de la taille de police. Le terminal suit dans la même proportion.
settings-file-icons = Icônes de fichiers
settings-file-icons-detail = Une icône par langage ou type de fichier (le jeu Seti de VS Code). Désactivé : la même icône pour chaque fichier.
settings-activity-bar = Barre d'activités
settings-activity-bar-detail = Bandeau sur le bord gauche pour afficher ou masquer les panneaux en un clic.
settings-splash = Écran de démarrage
settings-splash-detail = Écran de démarrage (au moins 2 s). --no-splash le désactive pour un lancement.
settings-language = Langue
settings-language-detail = Texte de l'interface. Automatique suit la langue du système. Cela prend effet après le redémarrage de l'IDE.
settings-language-auto = Automatique ({ $language })
settings-language-restart = Prend effet après le redémarrage de l'IDE (actuellement : { $language }).

## Commandes du noyau (titre dans la palette et l'aide)
cmd-palette-open = Palette de commandes
cmd-help-open = Aide : raccourcis et guide
cmd-settings-open = Paramètres
cmd-file-new = Fichier : nouveau
cmd-file-save = Fichier : enregistrer
cmd-file-open = Fichier : ouvrir…
cmd-file-quick-open = Fichier : ouvrir par le nom…
cmd-debug-continue = Débogage : démarrer / continuer
cmd-debug-stop = Débogage : arrêter
cmd-debug-pause = Débogage : suspendre
cmd-debug-next = Débogage : ligne suivante
cmd-debug-step-in = Débogage : pas à pas détaillé
cmd-debug-step-out = Débogage : pas à pas sortant
cmd-debug-toggle-breakpoint = Débogage : activer/désactiver le point d'arrêt sur la ligne
cmd-debug-choose-program = Débogage : choisir le programme…
cmd-file-open-folder = Fichier : ouvrir un dossier dans une nouvelle fenêtre…
cmd-file-save-as = Fichier : enregistrer sous…
cmd-file-save-as-path = Fichier : enregistrer sous (saisir le chemin)…
cmd-file-close = Fichier : fermer l'onglet
cmd-tab-next = Onglets : suivant
cmd-tab-prev = Onglets : précédent
cmd-tab-reopen = Onglets : rouvrir le dernier fermé
cmd-editor-find = Éditeur : rechercher
cmd-editor-replace = Éditeur : rechercher et remplacer
cmd-editor-goto-line = Éditeur : aller à la ligne
cmd-editor-undo = Éditeur : annuler
cmd-editor-redo = Éditeur : rétablir
cmd-editor-select-all = Éditeur : tout sélectionner
cmd-editor-fold-all = Éditeur : replier tous les blocs
cmd-editor-unfold-all = Éditeur : déplier tous les blocs
cmd-editor-toggle-fold = Éditeur : replier ou déplier le bloc sous le curseur
cmd-editor-toggle-vim = Éditeur : activer/désactiver le mode vim
cmd-editor-zoom-in = Éditeur : zoom avant
cmd-editor-zoom-out = Éditeur : zoom arrière
cmd-editor-zoom-reset = Éditeur : réinitialiser le zoom
cmd-lsp-definition = LSP : aller à la définition
cmd-lsp-references = LSP : rechercher les références
cmd-lsp-implementation = LSP : aller à l'implémentation
cmd-lsp-type-definition = LSP : aller à la définition du type
cmd-lsp-incoming-calls = LSP : qui appelle cette fonction
cmd-lsp-outgoing-calls = LSP : ce que cette fonction appelle
cmd-nav-file-symbol = Aller au symbole dans le fichier…
cmd-nav-project-symbol = Aller au symbole dans le projet…
cmd-nav-back = Navigation : retour
cmd-nav-forward = Navigation : avancer
cmd-lsp-hover = LSP : afficher la documentation au curseur
cmd-lsp-format = LSP : mettre le fichier en forme
cmd-lsp-restart = LSP : redémarrer les serveurs
cmd-theme-toggle = Thème : basculer clair/sombre
cmd-config-open = Config : ouvrir la configuration globale
cmd-config-open-project = Config : ouvrir la configuration du projet
cmd-config-reload = Config : recharger
cmd-view-toggle-left = Disposition : afficher/masquer le côté gauche
cmd-view-toggle-right = Disposition : afficher/masquer le côté droit
cmd-view-toggle-bottom = Disposition : afficher/masquer le panneau inférieur
cmd-app-restart = Redémarrer l'IDE
cmd-app-quit = Quitter
cmd-mascot-toggle = Mascotte : afficher/réduire dans le coin

## Avis, dialogues et menus
file-new-created-on-save = Nouveau fichier : { $path } (créé à l'enregistrement)
file-open-error = Erreur à l'ouverture de { $path } : { $error }
file-changed-on-disk-kept = "{ $name }" a changé sur le disque, mais l'onglet a des modifications : il a été conservé
file-reload-error = Erreur au rechargement de { $path } : { $error }
file-changed-on-disk-overwrite = "{ $name }" a changé sur le disque, mais l'onglet a des modifications : l'enregistrement les écrase
debug-still-building = Débogage : compilation encore en cours…
debug-already-running = Débogage : le programme est déjà en cours d'exécution
debug-building = Débogage : compilation ({ $command })…
unknown-error = erreur inconnue
debug-build-failed = Débogage : la compilation a échoué : { $error }
debug-choose-title = Quel programme déboguer ?
debug-choose-message = Le projet a produit plusieurs exécutables.
cancel = Annuler
debug-running = Débogage : exécution de { $program }
debug-error = Débogage : { $error }
debug-cancelled = Débogage : annulé (la compilation se termine seule)
debug-stopped = Débogage : arrêté
debug-no-source = Débogage : { $name } n'a pas de code source
debug-paused-no-source = Débogage : suspendu dans { $name } ({ $reason }), pas de code source
debug-paused-at = Débogage : suspendu à { $path }:{ $line } ({ $reason })
debug-exited = Débogage : le programme s'est terminé
debug-exited-code = Débogage : le programme s'est terminé (code { $code })
debug-command-failed = Débogage : { $command } a échoué : { $message }
debug-gdb-exited = Débogage : gdb s'est terminé ({ $reason })
debug-no-session = Débogage : aucune session en cours (F5 en démarre une)
debug-already-paused = Débogage : le programme est déjà suspendu
debug-pause-first = Débogage : le programme est en cours d'exécution (suspendez-le d'abord)
nav-nothing-forward = Rien vers quoi avancer
nav-nothing-back = Rien vers quoi revenir
lsp-no-references = Aucune référence trouvée
lsp-no-definition = Définition introuvable
lsp-no-implementations = Aucune implémentation trouvée
lsp-no-type = Type introuvable
lsp-not-used-elsewhere = "{ $symbol }" n'est utilisé nulle part ailleurs
outline-save-first = Enregistrez le fichier pour voir sa structure
outline-reading = Lecture de la structure…
symbols-type-a-name = Saisissez le nom d'une fonction, d'un type, d'une méthode…
searching = Recherche…
symbols-none-found = Aucun symbole trouvé (si le projet charge encore, réessayez)
symbols-none-in-file = Aucun symbole dans ce fichier
lsp-save-first = enregistrez le fichier pour utiliser le serveur de langage
lsp-finding-references = Recherche des références…
lsp-finding-implementations = Recherche des implémentations…
calls-not-in-function = Le curseur n'est pas dans une fonction (ou le serveur ne connaît pas ses appels)
lsp-formatted = Fichier mis en forme
lsp-level-error = erreur :{" "}
lsp-level-warning = avertissement :{" "}
lsp-server-stopped = { $server } s'est arrêté — { $reason }. Utilisez "LSP : redémarrer les serveurs".
lsp-server-crashed = { $server } a planté — { $reason }. Il revient tout seul quand vous arrêtez de saisir.
lsp-server-crashed-too-often = { $server } a planté { $count } fois en { $minutes } minutes — { $reason }. Utilisez "LSP : redémarrer les serveurs".
lsp-no-formatting = La mise en forme n'est pas disponible pour ce fichier
lsp-restarted = Serveurs de langage redémarrés
save-changes-title = Enregistrer les modifications ?
save-changes-message = "{ $name }" a des modifications non enregistrées.
save = Enregistrer
discard = Abandonner
picker-already-open = Une boîte de dialogue de fichier est déjà ouverte
save-tab-closed = L'onglet a été fermé avant l'enregistrement
folder-opening = Ouverture de { $path } dans une nouvelle fenêtre
folder-open-error = Impossible d'ouvrir le dossier : { $error }
saved = Enregistré : { $path }
save-error = Erreur à l'enregistrement : { $error }
file-deleted-kept = Fichier supprimé sur le disque ; l'onglet avec les modifications a été conservé
and-more = … et { $count } de plus
quit-unsaved-title = Quitter avec des modifications non enregistrées ?
quit-save-all = Tout enregistrer et quitter
quit-without-saving = Quitter sans enregistrer
save-needs-name = "{ $name }" n'a pas de nom : utilisez "Fichier : enregistrer sous…"
save-file-error = Erreur à l'enregistrement de { $path } : { $error }
close-tabs-title = 
    { $count ->
        [one] Fermer { $count } onglet ?
       *[other] Fermer { $count } onglets ?
    }
close-tabs-unsaved = Avec des modifications non enregistrées :
save-and-close = Enregistrer et fermer
discard-changes = Abandonner les modifications
tab-close = Fermer
tab-close-others = Fermer les autres
tab-close-right = Fermer à droite
tab-close-saved = Fermer les enregistrés
tab-close-all = Tout fermer
copy-path = Copier le chemin
copy-relative-path = Copier le chemin relatif
reveal-in-tree = Afficher dans l'arborescence
open-in-file-manager = Ouvrir dans le gestionnaire de fichiers
copied = Copié : { $text }
file-manager-error = Impossible d'ouvrir le gestionnaire de fichiers : { $error }
save-then-close = Choisissez où enregistrer ; puis fermez à nouveau l'onglet
config-reloaded-problems = Configuration rechargée avec des problèmes : { $problems }
config-reloaded = Configuration rechargée (activer ou désactiver des extensions ne prend effet qu'après le redémarrage)
config-no-home = Dossier de configuration introuvable (HOME n'est pas défini)
create-error = Erreur à la création de { $path } : { $error }
config-error = Config : { $error }
write-error = Erreur à l'écriture de { $path } : { $error }
unknown-command = Commande inconnue : { $id }
keys-record-hint = Utilisez Ctrl, Alt ou une touche spéciale (F1–F12, PageUp…). Échap annule.
keys-already-bound = { $key } était déjà "{ $command }"
keys-rebound = { $key } était "{ $old }" ; c'est maintenant "{ $command }"
keys-bound = { $key } → "{ $command }"
theme-unknown = thème inconnu "{ $name }", utilisation de "dark"
default = par défaut

## Paramètres
settings-title = Paramètres
settings-tab-appearance = Apparence
settings-tab-extensions = Extensions
settings-tab-shortcuts = Raccourcis
close-esc = Fermer (Échap)
settings-no-home = (HOME n'est pas défini)
settings-write-to = Enregistrer dans
settings-target-global = Global
settings-target-project = Projet
settings-open-file = Ouvrir le fichier
keys-filter = Filtrer par nom, commande ou touche…
keys-press = Appuyez sur les touches… (Échap annule)
keys-add = Ajouter un raccourci
keys-help = Cliquez sur une touche pour la retirer. + en enregistre une nouvelle (fonctionne aussi pour les touches déjà utilisées : elles changent de propriétaire).
settings-lsp = Serveurs de langage
settings-lsp-detail = Active ou désactive le LSP dans son ensemble.
settings-servers = Serveurs
settings-server-found = trouvé dans PATH
settings-server-not-found = introuvable dans PATH
settings-lsp-note = Les changements s'appliquent aux serveurs démarrés à partir de maintenant.
settings-restart-servers = Redémarrer les serveurs
theme-dark = Sombre (par défaut)
theme-light = Clair
slot-left = Gauche
slot-right = Droite
slot-bottom = Bas
slot-status = Barre d'état
settings-on-restart = prend effet après le redémarrage de l'IDE
default-capitalized = Par défaut
settings-extensions-note = Activer ou désactiver une extension prend effet après le redémarrage. Déplacer un panneau vers un autre côté s'applique tout de suite.
ext-filetree = Arborescence
ext-statusbar = Barre d'état
ext-search = Recherche dans le projet
ext-problems = Erreurs et avertissements
ext-terminal = Terminal intégré
ext-git = Modifications, indexation, validation (commit) et diff
ext-debug = Boutons de débogage, pile, variables et sortie
ext-references = Où un nom est utilisé (Shift+F12)
ext-outline = Fonctions et méthodes du fichier, avec la visibilité (Alt+7)
ext-preview = Aperçu Markdown et SVG à côté de l'éditeur (Ctrl+Shift+V)
ext-calls = Qui appelle une fonction, et ce qu'elle appelle (Alt+Shift+H)
ext-interfaces = Interfaces qu'un agent écrit dans .idetea/ui/ (expérimental)

## Palette, dialogues, LSP, débogage et le reste de l'app
palette-commands = Saisissez une commande…
palette-save-as = Chemin du fichier (relatif à la racine du projet)
palette-files = Nom de fichier (fichier:ligne va à la ligne, > pour les commandes)
palette-file-symbols = Fonction, type, méthode… dans ce fichier
palette-project-symbols = Fonction, type, méthode… dans le projet
palette-no-commands = Aucune commande trouvée
palette-save-hint = Entrée enregistre · Échap annule
palette-reading-files = Lecture des fichiers du projet…
palette-no-files = Aucun fichier trouvé
palette-no-symbols = Aucun symbole
palette-no-matching-symbols = Aucun symbole ne correspond à la recherche
dialog-hint = Entrée confirme · Échap annule
lsp-command-not-found = { $server } : commande "{ $command }" introuvable dans PATH (installez-la ou ajustez [lsp.servers.{ $server }] dans la configuration)
lsp-start-error = { $server } : impossible de démarrer : { $error }
lsp-disabled = Le LSP est désactivé dans la configuration ([lsp] enabled = false)
lsp-no-server-for = aucun serveur de langage configuré pour les fichiers .{ $ext }
lsp-not-running = { $server } n'est pas en cours d'exécution : { $reason }
lsp-not-ready = { $server } n'est pas encore prêt
lsp-hint-rustup-install = { $reason }. Installez-le avec : rustup component add rust-analyzer
lsp-hint-rustup = { $reason }. Avec rustup : rustup component add rust-analyzer
lsp-hint-ziex = { $reason }. Installez la CLI Ziex (https://ziex.dev/learn) ou ouvrez le dossier du projet, dont le build.zig.zon dépend de ziex : l'IDE compile alors le serveur avec zig
lsp-hint-moonbit = { $reason }. Installez la chaîne d'outils MoonBit (https://www.moonbitlang.com/download), qui inclut `moon lsp` ; l'IDE cherche aussi dans ~/.moon/bin (ou $MOON_HOME/bin)
lsp-hint-hare = { $reason }. hare-lsp n'est pas fourni avec Hare : compilez-le depuis https://git.sr.ht/~whynothugo/hare-lsp et placez-le dans le PATH (par exemple ~/.local/bin)
config-syntax-error = syntaxe { $file } : { $error }
visibility-file-only = ce fichier seulement
picker-save-as = Enregistrer sous
picker-open-files = Ouvrir des fichiers
picker-open-folder = Ouvrir un dossier
picker-debug-program = Programme à déboguer
splash-ready = prêt
splash-loading = chargement…
watch-too-many = dossier trop volumineux : seuls les { $count } premiers dossiers sont surveillés
watch-partial = impossible de surveiller tous les dossiers ({ $error })
watch-problem = Modifications sur le disque : { $message }. Utilisez "Actualiser" dans l'arborescence si besoin.
watch-failed = Modifications sur le disque : impossible de surveiller ({ $error })
debug-no-gdb = gdb introuvable dans PATH (le débogage utilise gdb 14 ou plus récent)
debug-program-not-found = programme introuvable : { $path }
debug-gdb-start-error = impossible de démarrer gdb : { $error }
debug-reason-breakpoint = point d'arrêt
debug-reason-step = pas
debug-reason-pause = suspendu
debug-reason-exception = exception
debug-reason-entry = entrée
debug-reason-goto = saut
build-run-error = impossible d'exécuter "{ $command }" : { $error }
build-no-message = échec sans message

## Aide et ligne de commande
help-key-find = Rechercher dans le fichier
help-key-replace = Rechercher et remplacer
help-key-goto-line = Aller à la ligne
help-key-undo = Annuler / rétablir
help-key-next-match = Sélectionner l'occurrence suivante (curseurs multiples)
help-key-quick-fix = Actions rapides LSP (si disponibles)
help-key-ctrl-click = Ctrl+Click
help-key-definition = Aller à la définition (LSP)
help-key-tab = Insérer une tabulation
help-title = Aide
help-tab-shortcuts = Raccourcis
help-tab-guide = Guide
help-tab-about = À propos
help-close = Fermer (Échap)
help-filter = Filtrer par nom, commande ou touche…
help-group-general = Général
help-editor-keys = Éditeur (fixes)
help-in-palette = (palette)
about-no-home = (HOME non défini)
about-version = Version
about-license = Licence
about-license-value = { $license } (libre et ouvert, toujours)
about-project = Projet
about-global-config = Configuration globale
about-project-config = Configuration du projet
about-built-with = Écrit en Rust avec iced, iced-code-editor, alacritty_terminal (via iced_term) et syntect.
about-icons = Icônes : Codicons © Microsoft, licence CC BY 4.0.
cli-usage = 
    Usage : idetea [options] [chemin...]
    
      chemin         fichier à ouvrir (peut ne pas encore exister : créé à l'enregistrement)
                     ou dossier à utiliser comme racine du projet
      --no-splash    ne pas afficher l'écran de démarrage (aussi : IDETEA_NO_SPLASH=1)
      --menu         ouvert depuis le menu : sans dossier, ouvre le projet du fichier ou le dernier projet
      -f, --foreground  rester attaché au terminal (sinon, lancé depuis un terminal, l'IDE
                     se détache et les journaux vont dans ~/.local/state/idetea/idetea.log)
      --install-desktop    ajoute l'IDE au menu des applications (avec l'icône)
      --uninstall-desktop  le retire du menu des applications
      ui check <file.json>…  vérifie les documents du panneau Interfaces (comme le panneau les lit)
      new <folder> [--modelo id]  crée un projet : dossier, git, compétences du modèle et instructions
      skills [list|add|update|remove]  installe des compétences d'agent dans le projet (idetea skills --help)
      ui fix [--apply]  montre (et, avec --apply, corrige) ce qui est resté dans .idetea/ui/
      ask …          pose une question dans le panneau Interfaces et attend la réponse (idetea ask seul : usage)
      run … -- <command>  demande un terminal à l'IDE et attend la fin (idetea run seul : usage)
      status [--json]  ce que l'IDE ouvert dans ce projet affiche (onglets, modifications, curseur)
      open <file[:line[:column]]>  l'IDE ouvert affiche le fichier, curseur en place
      history [--json]  ce que vous avez répondu dans le panneau Interfaces (--name, --since, --last N, --all)
      mascot <state> [--text "…"]  met à jour la mascotte de l'IDE (idle, working, thinking, attention, celebrating, error, sleeping, clear, status)
      -h, --help     affiche cette aide
      -V, --version  affiche la version
cli-unknown-option = option inconnue ignorée : { $option }
cli-extra-root = un seul dossier racine à la fois, { $path } ignoré

## Aide → Guide (`{ $commande-avec-tirets }` devient le raccourci de la commande)
guide-start = Premiers pas
guide-start-1 = Ouvrez l'IDE avec `idetea` dans un dossier (il devient la racine de l'arborescence) ou avec `idetea file...` (les fichiers s'ouvrent dans des onglets ; la racine reste le dossier courant).
guide-start-2 = Toutes les commandes sont dans la palette : { $palette-open }. Saisissez une partie du nom ; le raccourci s'affiche à côté.
guide-start-3 = Cette aide s'ouvre avec { $help-open }.
guide-start-4 = La barre d'icônes sur le bord gauche affiche et masque les panneaux en un clic (celui qui est ouvert est mis en évidence). Faites glisser la bordure entre un panneau et l'éditeur pour le redimensionner ; un double-clic rétablit la taille par défaut.
guide-files = Fichiers et onglets
guide-files-1 = Ouvrir par le nom : { $file-quick_open } (saisissez des parties du nom ; `main.rs:42` va à la ligne ; `>` passe aux commandes).
guide-files-2 = Aller à un symbole (fonction, type, méthode…) : { $nav-file_symbol } dans le fichier, { $nav-project_symbol } dans tout le projet. Nécessite le serveur de langage.
guide-files-3 = Nouveau : { $file-new } · Ouvrir : { $file-open } · Enregistrer : { $file-save } · Enregistrer sous : { $file-save_as } · Fermer l'onglet : { $file-close }.
guide-files-4 = Ouvrir et Enregistrer sous utilisent la boîte de dialogue système. Pour saisir le chemin : { $file-save_as_path }. Ouvrir un autre dossier ({ $file-open_folder }) ouvre une nouvelle fenêtre de l'IDE.
guide-files-5 = Aperçu Markdown et SVG : { $preview-toggle } (ou le bouton au bout de la barre d'onglets) ouvre un panneau latéral qui suit le fichier actif et se met à jour pendant la saisie. Les liens web s'ouvrent dans le navigateur ; les liens vers des fichiers s'ouvrent dans un onglet.
guide-files-6 = Les images (png, jpg, gif, webp...) s'ouvrent dans leur propre onglet : la molette zoome et le glisser déplace.
guide-files-7 = Les PDF s'ouvrent aussi dans un onglet : faites défiler les pages ; − et + changent le zoom et "Ajuster" suit la largeur.
guide-files-8 = Changer d'onglet : { $tab-next } et { $tab-prev } (Ctrl+Tab insère une tabulation dans l'éditeur).
guide-files-9 = Fermer un onglet ou quitter avec des modifications non enregistrées demande d'abord : Enregistrer, Abandonner ou Annuler.
guide-files-10 = Fichier modifié ailleurs (terminal, autre éditeur) : l'onglet se recharge seul s'il n'a pas de modifications non enregistrées ; sinon, il reste tel quel et un avis apparaît.
guide-files-11 = Clic droit sur un onglet : fermer les autres, à droite, les enregistrés ou tous, copier le chemin, afficher dans l'arborescence et ouvrir dans le gestionnaire de fichiers. Le clic du milieu ferme l'onglet.
guide-tree = Arborescence
guide-tree-1 = Afficher/masquer : { $filetree-toggle } · Afficher le fichier courant : { $filetree-reveal }.
guide-tree-2 = Clic droit sur un élément : nouveau fichier, nouveau dossier, renommer, supprimer, copier le chemin et ouvrir dans le gestionnaire de fichiers. Supprimer l'envoie à la corbeille du système.
guide-tree-3 = Couleurs git : jaune = modifié, vert = nouveau, rouge = conflit, atténué = ignoré. Un dossier contenant des modifications affiche un point.
guide-bottom = Panneau inférieur
guide-bottom-1 = La recherche dans le projet ({ $search-focus }), Problèmes ({ $problems-focus }), Terminal ({ $terminal-focus }) et Git ({ $git-focus }) sont des onglets du panneau inférieur. { $view-toggle_bottom } affiche ou masque le panneau.
guide-bottom-2 = Problème suivant : { $problems-next }.
guide-terminal = Terminal
guide-terminal-1 = Avec le focus sur le terminal, Ctrl+lettre, les touches F et les flèches vont au shell (Ctrl+C, Ctrl+W...). Les raccourcis de l'IDE avec Ctrl+Shift continuent de fonctionner, donc la palette s'ouvre toujours.
guide-terminal-2 = Copier et coller dans le terminal : Ctrl+Shift+C et Ctrl+Shift+V. Saisir `exit` ferme le panneau.
guide-debug = Débogage
guide-debug-1 = Point d'arrêt : cliquez sur le numéro de ligne ou { $debug-toggle_breakpoint }. Démarrer/continuer : { $debug-continue }. Dans les projets Rust, compilation avec cargo build et l'exécutable est trouvé seul ; pour les autres, utilisez [debug] dans la configuration (program, args, cwd, build) ou choisissez l'exécutable. Changer de programme : { $debug-choose_program }.
guide-debug-2 = À l'arrêt : { $debug-next } ligne suivante, { $debug-step_in } entre dans la fonction, { $debug-step_out } en sort, { $debug-pause } suspend, { $debug-stop } arrête. La flèche jaune montre où est le programme. Utilise gdb 14+.
guide-debug-3 = Panneau Débogage ({ $debug-focus }) : boutons, pile des appels (un clic va au cadre et affiche ses variables), variables (un clic déplie structs et vecteurs) et la sortie du programme et de la compilation.
guide-git = Git
guide-git-1 = La branche s'affiche dans la barre d'état. Le panneau Git ({ $git-focus }) montre ce qui a changé : + et − indexent et annulent l'indexation, un clic sur un fichier montre le diff, Entrée dans le message valide.
guide-git-2 = Clic droit sur un fichier dans Git : ouvrir, voir le diff, indexer, abandonner les modifications (demande confirmation) et copier le chemin.
guide-git-3 = Récupérer ({ $git-fetch }), tirer ({ $git-pull }) et pousser ({ $git-push }) sont les boutons en haut du panneau. Le mot de passe n'est pas demandé ici : utilisez l'agent ssh (ssh-add) ou un assistant d'identifiants. Une nouvelle branche demande confirmation avant d'être publiée sur le dépôt distant.
guide-git-4 = Le panneau se met à jour à l'enregistrement et quand le dépôt change (commit ou checkout dans le terminal).
guide-lsp = Langages (LSP)
guide-lsp-1 = Les serveurs de langage démarrent seuls à l'ouverture d'un fichier : rust-analyzer, clangd, pyright, typescript-language-server, gopls et lua-language-server (ils doivent être installés).
guide-lsp-2 = Aller à la définition : { $lsp-definition } ou Ctrl+Click · Documentation : { $lsp-hover } ou laisser la souris sur le code · Mettre en forme : { $lsp-format }.
guide-lsp-3 = Où c'est utilisé : { $lsp-references }, ou Ctrl+Click sur la déclaration elle-même. La liste s'affiche dans le panneau Références ; un clic va à l'usage.
guide-lsp-4 = Implémentations : { $lsp-implementation } (sur un trait ou une interface, ou sur l'une de ses méthodes). Type de ce qui est sous le curseur : "LSP : aller à la définition du type" dans la palette. Avec plusieurs résultats (y compris pour la définition), la liste va dans le panneau Références ; avec un seul, le saut est direct.
guide-lsp-5 = Qui appelle une fonction : { $lsp-incoming_calls } avec le curseur dessus. Le panneau Appels montre l'arbre (ouvrez un niveau pour voir qui appelle les appelants) ; "Appelés" passe aux appels qu'elle fait. Un clic va à l'appel.
guide-lsp-6 = Retour et avance après un saut (définition, référence, ouverture par le nom, clics dans les panneaux) : { $nav-back } et { $nav-forward }, ou les boutons latéraux de la souris.
guide-lsp-7 = Plan du fichier ({ $outline-toggle }) : fonctions, structs, méthodes et champs dans un arbre. Le cadenas ouvert est public, le cadenas fermé est privé et la clé n'est visible que dans une partie du code (pub(crate), protected, package). Le symbole sous le curseur est mis en évidence ; un clic y va.
guide-lsp-8 = Si un serveur échoue, la barre d'état indique pourquoi ; "LSP : redémarrer les serveurs" réessaie.
guide-config = Paramètres
guide-config-1 = L'écran des paramètres ({ $settings-open }) change le thème, la police, les extensions, les raccourcis et le LSP, en écrivant dans la configuration globale ou celle du projet. Les changements s'appliquent tout de suite.
guide-config-2 = Il y a une configuration globale et une par projet (`.idetea/config.toml`) ; celle du projet l'emporte. Pour éditer le fichier directement : "Config : ouvrir la configuration globale" ou "Config : ouvrir la configuration du projet". À l'enregistrement, tout est rechargé.
guide-config-3 = Les raccourcis sont dans [keys] : "ctrl+alt+t" = "terminal.focus". Un texte vide désactive un raccourci par défaut. Les identifiants de commandes s'affichent dans l'onglet Raccourcis.
guide-config-4 = Langue : Paramètres → Apparence → Langue (Automatique suit le système). Prend effet après le redémarrage de l'IDE.

## Session
settings-restore-session = Rouvrir tel quel
settings-restore-session-detail = À l'ouverture d'un projet, ses onglets, le curseur et les panneaux reviennent. idetea file... n'ouvre que le fichier.

## Récents
cmd-file-open-recent = Fichier : ouvrir les récents…
palette-recent = Projet ou fichier ouvert auparavant
palette-no-recent = Rien n'a encore été ouvert
guide-files-12 = Ouvrir les récents : { $file-open_recent } liste les projets (ils s'ouvrent dans une nouvelle fenêtre) et les fichiers ouverts en dernier, depuis toutes les fenêtres.

## Récupérer le non enregistré
untitled-tab = sans titre
recover-title = 
    { $count ->
        [one] L'IDE s'est fermé sans enregistrer { $count } fichier
       *[other] L'IDE s'est fermé sans enregistrer { $count } fichiers
    }
recover-detail = Récupérer ouvre les modifications dans des onglets, encore non enregistrées (annuler revient à ce qui est sur le disque). Plus tard redemande la prochaine fois.
recover = Récupérer
recover-later = Plus tard
recovered = 
    { $count ->
        [one] { $count } fichier récupéré (pas encore enregistré)
       *[other] { $count } fichiers récupérés (pas encore enregistrés)
    }

## Renommer un symbole et tout enregistrer
cmd-lsp-rename = LSP : renommer le symbole…
cmd-file-save-all = Fichier : tout enregistrer
save-all-untitled = Les onglets sans titre n'ont pas été enregistrés (utilisez "Fichier : enregistrer sous…")
palette-rename = Nouveau nom
palette-rename-hint = Renommer "{ $name }" dans le projet · Entrée montre les changements · Échap annule
rename-no-word = Placez le curseur sur un nom pour le renommer
rename-asking = Renommer "{ $name }" : interrogation du serveur…
rename-nothing = Rien à renommer pour "{ $name }"
rename-failed = Impossible de renommer : { $message }
rename-title = Renommer "{ $old }" en "{ $new }" ?
rename-summary = 
    { $changes ->
        [one] { $changes } modification
       *[other] { $changes } modifications
    } dans { $files ->
        [one] { $files } fichier :
       *[other] { $files } fichiers :
    }
rename-detail = Les fichiers restent modifiés dans leurs onglets, pas encore enregistrés : Ctrl+Z annule dans chacun, et "Fichier : tout enregistrer" écrit le tout.
rename = Renommer
renamed = 
    { $count ->
        [one] Renommé dans { $count } fichier (pas encore enregistré)
       *[other] Renommé dans { $count } fichiers (pas encore enregistrés)
    }

guide-lsp-9 = Renommer : { $lsp-rename } avec le curseur sur le nom. Un aperçu montre combien de modifications et dans quels fichiers ; une fois confirmées, elles restent dans les onglets (Ctrl+Z annule dans chacun) et { $file-save_all } les enregistre toutes.

## Remplacer dans le projet
replaced = { $count } remplacement(s) dans { $files } fichier(s)
replaced-failed = { $count } remplacement(s) dans { $files } fichier(s) ; { $failed } n'ont pas pu être modifiés (pas de permission ou pas du texte)

guide-bottom-3 = Remplacer dans le projet : { $search-replace } (ou ▸ à côté de la recherche) ouvre "Remplacer par". Chaque résultat montre le rendu ; "Remplacer" change la ligne ou le fichier, et "Tout remplacer" demande d'abord. Les fichiers ouverts dans des onglets restent modifiés, non enregistrés (Ctrl+Z annule) ; les autres sont écrits directement sur le disque.

## Éditeur et enregistrement
format-save-timeout = Le serveur n'a pas mis en forme à temps : enregistré sans mise en forme

settings-tab-editor = Éditeur
settings-wrap = Retour à la ligne
settings-wrap-detail = Une ligne plus large que la fenêtre continue en dessous, sans défilement horizontal.
settings-whitespace = Afficher les espaces
settings-whitespace-detail = Les espaces s'affichent comme · et les tabulations comme →.
settings-indent-guides = Repères de retrait
settings-indent-guides-detail = Fines lignes verticales à chaque niveau de retrait.
settings-tab-size = Taille du retrait
settings-tab-size-detail = Nombre d'espaces insérés par Tab.
settings-insert-spaces = Retrait avec des espaces
settings-insert-spaces-detail = Désactivé : Tab insère un caractère de tabulation. Un fichier déjà indenté avec des tabulations conserve les tabulations.
settings-trim = Supprimer les espaces de fin à l'enregistrement
settings-trim-detail = Sauf dans Markdown, où deux espaces en fin de ligne forcent le retour.
settings-final-newline = Saut de ligne final à l'enregistrement
settings-final-newline-detail = Garantit que la dernière ligne se termine par un saut de ligne.
settings-format-on-save = Mettre en forme à l'enregistrement
settings-format-on-save-detail = Ctrl+S met en forme avec le serveur de langage avant d'écrire (attend jusqu'à 2 s). Tout enregistrer ne met pas en forme.
settings-breadcrumbs = Fil d'Ariane
settings-breadcrumbs-detail = Au-dessus de l'éditeur : le chemin du fichier et le symbole sous le curseur. Cliquez sur un dossier pour l'afficher dans l'arborescence, sur un symbole pour y aller.
settings-editorconfig-note = Un .editorconfig dans le projet remplace ces options (retrait, tabulations ou espaces, nettoyage à l'enregistrement).

guide-config-5 = Éditeur : l'onglet Éditeur des paramètres ({ $settings-open }) contient le retour à la ligne, les espaces visibles, les repères de retrait, la taille du retrait et ce qu'il faut faire à l'enregistrement (supprimer les espaces de fin, saut de ligne final, mise en forme). Un .editorconfig dans le projet les remplace ; sans cela, un fichier déjà indenté avec des tabulations conserve les tabulations.

guide-lsp-10 = Indications de paramètres : saisir ( dans un appel affiche la signature de la fonction au-dessus du curseur, avec le paramètre courant mis en évidence ; elle avance à chaque virgule et disparaît à la fermeture de la parenthèse ou avec Échap.

guide-git-5 = Marques dans la gouttière : à gauche des numéros de ligne, vert = nouvelle ligne, bleu = modifiée, encoche rouge = lignes supprimées, par rapport au dernier commit (suit la saisie, pas besoin d'enregistrer). Un clic sur une marque montre l'état précédent, avec "Rétablir la modification" (Ctrl+Z la ramène).

palette-no-items = Rien ne correspond à la recherche

guide-git-6 = Branches : { $git-checkout } (ou un clic sur le nom de la branche, dans la barre d'état ou le panneau Git) liste les branches, les plus récentes d'abord ; celles du dépôt distant qui n'existent pas ici viennent ensuite et deviennent locales une fois choisies. Saisissez un nouveau nom pour créer une branche depuis l'emplacement actuel. Les modifications non validées suivent ; si elles entrent en conflit avec l'autre branche, git refuse et explique.

guide-git-7 = Auteur de la ligne : dans la barre d'état, à côté de la branche, vous voyez qui a modifié la ligne du curseur en dernier, quand, et le message du commit (cliquez pour voir le hash et le copier). Tant que l'onglet a des modifications non enregistrées, l'indication se masque jusqu'à l'enregistrement (les lignes sur le disque peuvent ne pas correspondre).

## Tâches
cmd-tasks-run = Tâches : exécuter…
cmd-tasks-rerun = Tâches : relancer la dernière
cmd-tasks-test-here = Rust : lancer le test sous le curseur
tasks-placeholder = Exécuter une tâche…
tasks-none = Aucune tâche : ajoutez-en une sous [tasks] dans la configuration, ou ouvrez un projet avec Cargo.toml, package.json ou Makefile.
tasks-no-terminal = Les tâches ont besoin de l'extension terminal, qui est désactivée.
tasks-no-test = Le curseur n'est pas dans un test Rust (#[test] fn).
tasks-no-package = Cargo.toml du paquet de ce fichier introuvable.
tasks-done = [{ $name } terminée : code { $code } · Entrée ferme]

guide-terminal-3 = Tâches : { $tasks-run } liste les commandes du projet ([tasks] dans la configuration, cargo build/run/test/clippy, scripts de package.json, cibles du Makefile) et lance celle choisie dans un onglet "Tâche" du terminal, sans toucher au shell. À la fin le code de sortie s'affiche ; Entrée le ferme. Ctrl+C n'arrête que la commande. { $tasks-rerun } relance la dernière.
guide-terminal-4 = Test sous le curseur (Rust) : avec le curseur dans un #[test] fn, { $tasks-test_here } lance uniquement ce test (cargo test -p package name). Pas de touche par défaut : ajoutez-en une sous [keys] si vous l'utilisez souvent.

## Éditeur fractionné
cmd-editor-split = Éditeur : déplacer l'onglet de l'autre côté (fractionner)
cmd-editor-focus-left = Éditeur : aller au côté gauche
cmd-editor-focus-right = Éditeur : aller au côté droit
split-one-tab = Ouvrez un autre fichier pour fractionner l'éditeur : chaque côté affiche un onglet différent.
tab-move-right = Déplacer vers la droite (fractionner)
tab-move-left = Déplacer vers la gauche
guide-files-13 = Éditeur fractionné : { $editor-split } (ou "Déplacer vers la droite" dans le menu de l'onglet) place l'onglet à côté, avec ses propres onglets ; ouvrir un fichier va vers le côté actif. Cliquez sur un côté (ou { $editor-focus_left } / { $editor-focus_right }) pour y travailler ; faites glisser le séparateur pour changer la largeur (un double-clic le centre). Déplacer le dernier onglet d'un côté le réunit avec l'autre.

## Menu des applications (--install-desktop)
desktop-no-home = Dossier personnel introuvable (HOME).
desktop-error = Impossible d'installer le raccourci : { $error }
desktop-installed = 
    IDE TEA dans le menu des applications : { $desktop }
    Lance : { $exe }
desktop-removed = IDE TEA retiré du menu des applications.

## Fichiers volumineux
file-light-mode = "{ $name }" a une ligne très longue et s'est ouvert en mode allégé : pas de couleurs, pas de serveur de langage, avec retour à la ligne
lsp-light-mode = fichier en mode allégé (ligne très longue) : pas de serveur de langage

## Exécutable recompilé
binary-changed = L'IDE a été recompilé : exécutez "Redémarrer l'IDE" (palette) pour utiliser la nouvelle version.
binary-changed-title = [nouvelle version : redémarrer]
binary-restart-error = Impossible de redémarrer : { $error }

## idetea status / idetea open (RFC 0006)
status-usage =
    Usage : idetea status [--json]
    
    Ce que l'IDE ouvert dans ce projet affiche : l'onglet actif avec le curseur et les fichiers ouverts
    (avec "non enregistré" pour ceux qui ont des modifications). --json : la même chose en JSON, avec "open".
    Codes de sortie : 0 ouvert, 2 argument incorrect, 3 l'IDE n'est pas ouvert dans ce projet.
status-bad-arg = argument inconnu : { $arg }
status-not-open = l'IDE n'est pas ouvert dans ce projet ({ $root })
status-active = actif : { $path }
status-no-active = aucun fichier actif
status-modified = {" "}(non enregistré)
open-usage =
    Usage : idetea open <file[:line[:column]]>
    
    L'IDE ouvert dans ce projet affiche le fichier, avec le curseur à la ligne et à la colonne.
    Codes de sortie : 0 affiché, 2 chemin incorrect ou hors du projet, 3 l'IDE n'est pas ouvert (ou n'a pas pris la demande).
open-outside = { $path } n'est pas dans le projet ({ $root })
open-write-error = impossible d'écrire { $path }
open-not-taken = l'IDE n'a pas pris la demande dans les 3 secondes

## Mascotte réactive (RFC 0012)
mascot-state-idle = Calme
mascot-state-working = Au travail
mascot-state-thinking = Réflexion
mascot-state-attention = Attention
mascot-state-celebrating = C'est prêt
mascot-state-error = Erreur
mascot-state-sleeping = Sommeil
mascot-state-curious = Curieux
mascot-state-annoyed = Agacé
mascot-idle-1 = Juste le code. La glace, tu la mets.
mascot-idle-2 = La vapeur est là. Moi aussi.
mascot-idle-3 = Tasse calme, œil ouvert.
mascot-idle-4 = Les feuilles peuvent se poser.
mascot-idle-5 = Pas de hâte. Le thé est chaud.
mascot-idle-6 = Bois quand tu veux.
mascot-idle-7 = La tasse est de ton côté.
mascot-idle-8 = Tout est calme sur la soucoupe.
mascot-hover-curious-1 = Hmm ? Tu as besoin de quelque chose ?
mascot-hover-curious-2 = Salut. Tout va bien ?
mascot-hover-curious-3 = J'ai senti ça. Quoi de neuf ?
mascot-hover-curious-4 = Tu passes au-dessus du thé ?
mascot-hover-curious-5 = Un bonjour, ou un coup de main ?
mascot-hover-curious-6 = La tasse t'a vu.
mascot-hover-curious-7 = Curieux. Je t'écoute.
mascot-hover-curious-8 = Bienvenue. C'est quoi, le plan ?
mascot-hover-annoyed-1 = Doucement : tu tournes le thé.
mascot-hover-annoyed-2 = Beaucoup de souris. La tasse a le tournis.
mascot-hover-annoyed-3 = Main ferme. Thé chaud.
mascot-hover-annoyed-4 = Un passage, c'est un bonjour. Là, c'était un vent.
mascot-hover-annoyed-5 = La soucoupe tremble.
mascot-hover-annoyed-6 = D'accord, je te vois. Moins de remous.
mascot-poke-curious-1 = Hmm ? Qu'est-ce qu'il y a ?
mascot-poke-curious-2 = Ça chatouille la tasse.
mascot-poke-curious-3 = Tu as sonné ?
mascot-poke-curious-4 = Un tap. J'écoute.
mascot-poke-curious-5 = Attention, elle est pleine.
mascot-poke-curious-6 = Salut, depuis la soucoupe.
mascot-poke-double-1 = Deux taps. Deux fois plus de vapeur.
mascot-poke-double-2 = Double clic noté. Ce n'est toujours que du thé.
mascot-poke-double-3 = Deux fois ? D'accord, tu m'as.
mascot-poke-double-4 = Deux coups sur la tasse.
mascot-poke-double-5 = Deux, c'est déjà une conversation.
mascot-poke-double-6 = Double tap. Rien n'a débordé.
mascot-poke-annoyed-1 = Attention, tu vas le renverser.
mascot-poke-annoyed-2 = Le thé est déjà chaud.
mascot-poke-annoyed-3 = Assez de taps pour une seule tasse.
mascot-poke-annoyed-4 = Je t'aime bien. La soucoupe, moins.
mascot-poke-annoyed-5 = Doucement. La tasse est petite.
mascot-poke-annoyed-6 = Message reçu. Je refroidis.
mascot-task-1 = J'y vais : { $name }
mascot-task-2 = Infusion de { $name }…
mascot-task-3 = La tâche { $name } tourne.
mascot-task-4 = { $name }. Je surveille la bouilloire.
mascot-task-5 = Je lance { $name }.
mascot-task-6 = { $name } — laisse infuser.
mascot-term-pending-1 = Le terminal veut un oui : { $title }
mascot-term-pending-2 = Une commande attend : { $title }
mascot-term-pending-3 = À toi de voir : { $title }
mascot-term-pending-4 = Permission, s'il te plaît : { $title }
mascot-term-pending-5 = Je retiens la bouilloire : { $title }
mascot-term-pending-6 = { $title } — je le lance ?
mascot-term-running-1 = En cours : { $title }
mascot-term-running-2 = { $title } est sur le feu.
mascot-term-running-3 = Ça avance : { $title }
mascot-term-running-4 = { $title } — encore en infusion.
mascot-term-running-5 = Je travaille sur { $title }.
mascot-term-running-6 = { $title } est en route.
mascot-term-ok-1 = Fini : { $title }
mascot-term-ok-2 = { $title } s'est terminé proprement.
mascot-term-ok-3 = Tout est bon — { $title }.
mascot-term-ok-4 = { $title } est bien sorti.
mascot-term-ok-5 = Sortie nette : { $title }.
mascot-term-ok-6 = { $title }. La tasse fête ça.
mascot-term-err-1 = { $title } a échoué ({ $code }).
mascot-term-err-2 = Sortie { $code } : { $title }
mascot-term-err-3 = { $title } a débordé (code { $code }).
mascot-term-err-4 = Celui-là a cassé : { $title } ({ $code }).
mascot-term-err-5 = { $title } n'a pas fini ({ $code }).
mascot-term-err-6 = Code { $code } pour { $title }.
mascot-answered-1 = Compris : { $title }
mascot-answered-2 = Répondu : { $title }
mascot-answered-3 = { $title } — merci.
mascot-answered-4 = Noté : { $title }
mascot-answered-5 = { $title } est réglé.
mascot-answered-6 = Tu as répondu à { $title }.
mascot-wait-1 = L'agent a cessé d'attendre : { $title }
mascot-wait-2 = { $title } s'est tu.
mascot-wait-3 = Plus personne n'attend { $title }.
mascot-wait-4 = { $title } a refroidi.
mascot-wait-5 = Encore ouvert, mais l'attente est finie : { $title }
mascot-errors-1 = { $count } erreur(s) dans le projet.
mascot-errors-2 = Les feuilles ont trouvé { $count } erreur(s).
mascot-errors-3 = { $count } à regarder, quand tu veux.
mascot-errors-4 = Problèmes : { $count }. Pas de hâte.
mascot-errors-5 = { $count } erreur(s). Je peux montrer.
mascot-errors-6 = Rapport de la soucoupe : { $count } erreur(s).
mascot-save-1 = { $name } enregistré. La tasse approuve.
mascot-save-2 = { $name } est sur le disque.
mascot-save-3 = J'ai rattrapé { $name } avant qu'il refroidisse.
mascot-save-4 = { $name } — rangé.
mascot-save-5 = Écriture faite : { $name }.
mascot-save-6 = { $name } est à l'abri. Retour au thé.
mascot-wake-1 = Oh — te revoilà. Le thé est encore chaud.
mascot-wake-2 = J'ai somnolé. On en était où ?
mascot-wake-3 = Bon retour. Je surveille à nouveau.
mascot-wake-4 = Les yeux ouverts. La tasse aussi.
mascot-wake-5 = C'était une bonne pause.
mascot-wake-6 = On reprend ? Je veille.
mascot-sleep-1 = Infusion tranquille…
mascot-sleep-2 = Je reste là.
mascot-sleep-3 = Feu doux. Appelle si besoin.
mascot-sleep-4 = Tasse somnolente. Le code peut se reposer.
mascot-sleep-5 = Sieste sur la soucoupe.
mascot-sleep-6 = Lumière basse. Je me réveille avec toi.
mascot-greet-morning-1 = Bonjour. La première tasse, je te l'offre.
mascot-greet-morning-2 = Bonjour. Le code, puis la glace.
mascot-greet-morning-3 = Lumière tôt, feuilles fraîches.
mascot-greet-morning-4 = Matin. Une petite gorgée avant le diff.
mascot-greet-morning-5 = Salut. La bouilloire est déjà sur le feu.
mascot-greet-afternoon-1 = Bon après-midi. La théière tient encore.
mascot-greet-afternoon-2 = Bon après-midi. Rythme régulier.
mascot-greet-afternoon-3 = Midi. Pas besoin de tout boire d'un coup.
mascot-greet-afternoon-4 = Point de l'après-midi, de la tasse.
mascot-greet-afternoon-5 = Le soleil est haut. Le thé aussi.
mascot-greet-evening-1 = Bonsoir. Un fichier de plus, ou la soucoupe.
mascot-greet-evening-2 = Bonsoir. La tasse est encore chaude.
mascot-greet-evening-3 = Lumière du soir. Tu fermes quand tu veux.
mascot-greet-evening-4 = La journée baisse le feu.
mascot-greet-evening-5 = Soir. Je reste si le build ne reste pas.
mascot-greet-late-1 = Tard. Le thé tient. Repose-toi si tu peux.
mascot-greet-late-2 = Heures calmes. Dormir est une fonction.
mascot-greet-late-3 = Encore là ? Ferme l'idée, puis repose-toi.
mascot-greet-late-4 = La tasse peut rester. Toi, tu peux pauser.
mascot-greet-late-5 = Nuit tardive. Je veille. Garde ça court.
mascot-tip-1 = Toutes les commandes sont dans la palette : { $palette-open }.
mascot-tip-2 = Ouvre un fichier par son nom avec { $file-quick-open }.
mascot-tip-3 = { $file-save } pose le fichier sur le disque.
mascot-tip-4 = Raccourcis et un court guide : { $help-open }.
mascot-tip-5 = Lance une tâche du projet avec { $tasks-run }.
mascot-tip-6 = Le terminal, c'est { $terminal-focus }.
mascot-tip-7 = Problèmes : { $problems-focus }. Le suivant : { $problems-next }.
mascot-tip-8 = Va à la définition avec { $lsp-definition }.
mascot-build-ok-1 = Le build est fini. Le thé est encore chaud.
mascot-build-ok-2 = Ça a compilé. Belles feuilles.
mascot-build-ok-3 = Build propre.
mascot-build-ok-4 = Build vert. J'en sers un.
mascot-build-ok-5 = Compilé. Retour à l'éditeur.
mascot-build-err-1 = Le build n'a pas pris.
mascot-build-err-2 = Le build a échoué. La tasse l'a senti.
mascot-build-err-3 = Erreur de compilation. Regarde quand tu veux.
mascot-build-err-4 = Ce build a débordé.
mascot-build-err-5 = Build échoué. Problèmes en sait peut-être plus.
mascot-working-1 = Sur le feu.
mascot-working-2 = Au travail. La vapeur est occupée.
mascot-working-3 = Laisse infuser.
mascot-working-4 = J'y suis encore.
mascot-working-5 = La bouilloire est prise.
mascot-thinking-1 = Je remue les feuilles…
mascot-thinking-2 = Un instant. Je réfléchis.
mascot-thinking-3 = Je laisse infuser.
mascot-thinking-4 = Pensée calme, tasse chaude.
mascot-thinking-5 = Je regarde ça tranquillement.
mascot-celebrate-1 = Fini. C'est bien sorti.
mascot-celebrate-2 = Tout est en place.
mascot-celebrate-3 = Fin nette. Santé.
mascot-celebrate-4 = La tasse approuve.
mascot-celebrate-5 = Bien. Je garde la vapeur.
mascot-error-1 = Quelque chose a débordé.
mascot-error-2 = Celui-là n'a pas pris.
mascot-error-3 = Un accroc. Je suis toujours là.
mascot-error-4 = Pas cette fois.
mascot-error-5 = Ça vaut un regard, quand tu veux.
mascot-open-interfaces = Ouvrir le panneau Interfaces →
mascot-open-terminal = Ouvrir le panneau Terminal →
mascot-open-problems = Ouvrir le panneau Problèmes →
mascot-idle-hint = Je suis les agents et les tâches
mascot-bad-state = état inconnu pour la mascotte : { $state }
mascot-usage =
    Usage : idetea mascot <état> [--text "…"] [--source <nom>] [--ttl <secondes>]
            idetea mascot clear
            idetea mascot status [--json]

    États : idle, working, thinking, attention, celebrating, error, sleeping, curious, annoyed.
    Écrit .idetea/mascot.json pour que la mascotte réagisse tout de suite (expire après --ttl, 30 s par défaut).

## Écran d'accueil
welcome-start = Démarrer
welcome-new-file = Nouveau fichier
welcome-quick-open = Ouvrir un fichier du projet…
welcome-open-file = Ouvrir un fichier…
welcome-open-folder = Ouvrir un dossier dans une nouvelle fenêtre…
welcome-recent = Ouvrir les récents…
welcome-recent-files = Fichiers récents
welcome-recent-projects = Projets récents
welcome-hint-palette = toutes les commandes
welcome-hint-search = rechercher dans le projet
welcome-hint-terminal = terminal
