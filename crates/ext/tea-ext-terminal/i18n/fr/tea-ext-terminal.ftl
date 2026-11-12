## Terminal
terminal-start-error = Terminal : impossible de démarrer le shell : { $error }
cmd-terminal-focus = Terminal : ouvrir et donner le focus
cmd-terminal-new = Terminal : nouveau terminal
cmd-terminal-close = Terminal : fermer le terminal
cmd-terminal-restart = Terminal : redémarrer le shell
terminal-ended-n = Terminal { $number } fermé
terminal-not-running = Le shell n'est pas en cours d'exécution.
terminal-start = Démarrer le terminal

## Tâches
terminal-task-error = Tâche : impossible de démarrer : { $error }
terminal-tab-n = Terminal { $number }
terminal-close = Fermer le terminal
terminal-tab-task = Tâche : { $name }
terminal-close-task = Fermer la tâche

## Terminaux demandés par les agents
terminal-tab-agent = Agent : { $title }
terminal-tab-request = { $count ->
    [one] { $count } demande de terminal
   *[other] { $count } demandes de terminal
}
agent-request-notify = Un agent demande un terminal : { $title }. Voyez le panneau Terminal.
agent-request-auto = Demande de terminal "{ $title }" en cours sans confirmation : elle est dans [agents] allow de votre configuration ("{ $entry }").
agent-request-someone = Un agent
agent-request-title = { $who } demande un terminal : { $title }
agent-request-command = Commande (exécutée exactement ainsi) :
agent-request-cwd = Dans le dossier : { $cwd }
agent-request-worktree = S'exécute dans son propre arbre de travail (worktree) : branche { $branch }, dans { $path } (créé à partir de HEAD si la branche n'existe pas)
agent-worktree-kept = Le worktree { $path } a été conservé : { $error }
agent-request-warning = Elle s'exécute avec vos permissions, comme si vous l'aviez saisie. N'autorisez que ce que vous reconnaissez.
agent-request-allow = Autoriser et exécuter
agent-request-refuse = Refuser
agent-request-more = { $count ->
    [one] { $count } demande de plus après celle-ci.
   *[other] { $count } demandes de plus après celle-ci.
}
agent-done = Terminé avec le code { $code }. Entrée ferme l'onglet.
run-bad-arg = idetea run : "{ $arg }" n'est pas valide.
run-usage = Usage : idetea run [--title T] [--name name] [--cwd folder] [--worktree branch] [--keep] [--no-wait] [--timeout seconds] -- <command> ; idetea run --wait <name> ; idetea run --stop <name>
run-busy = idetea run : il y a déjà une demande "{ $name }" (ou son statut) ; choisissez un autre --name ou attendez avec idetea run --wait { $name }
run-asked = idetea run : la demande "{ $name }" est dans le panneau Terminal ; en attente que vous l'autorisiez et que la commande se termine
run-exited = idetea run : "{ $name }" s'est terminé avec le code { $code }
run-refused = idetea run : vous avez refusé "{ $name }"
run-invalid = idetea run : la demande "{ $name }" n'est pas valide : { $error }
run-ended = idetea run : "{ $name }" s'est terminé sans code ({ $state })
run-branch = idetea run : le travail est sur la branche { $branch }
run-timeout = idetea run : le temps est écoulé et "{ $name }" est encore dans le terminal (idetea run --wait { $name } attend à nouveau ; --stop y met fin)
