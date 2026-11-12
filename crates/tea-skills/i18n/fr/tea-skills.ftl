# Textes d'idetea skills (compétences).
skills-bad-arg = idetea skills : "{ $arg }" n'est pas valide.
skills-usage = Usage :
    idetea skills                          les compétences (skills) installées dans ce projet
    idetea skills list [source]            les compétences d'une source (par défaut : anthropics/skills)
    idetea skills add <source> <name>…     installe dans .agents/skills/ et .claude/skills/
    idetea skills update [name…] [--apply] montre ce qui a changé dans la source ; --apply l'installe
    idetea skills remove <name>…           retire du projet
  La source est owner/repo (GitHub), une URL git ou un dossier. --force remplace une copie modifiée à la main.
skills-error = erreur : { $error }
skills-no-cache = idetea skills : pas de dossier de cache (définissez HOME ou XDG_CACHE_HOME)
skills-bad-source = source non valide : "{ $source }" (utilisez owner/repo, une URL git ou un dossier)
skills-bad-name = { $path } : le nom "{ $name }" n'est pas valide (minuscules, chiffres, - et _) ; ignoré
skills-duplicate = { $path } : une autre compétence s'appelle déjà "{ $name }" ; ignorée
skills-none-installed = Aucune compétence installée dans ce projet (idetea skills list pour voir celles d'une source).
skills-installed-line = { $name }  ({ $source } @ { $commit })
skills-list-header = { $source } @ { $commit } : { $count ->
    [one] { $count } compétence
   *[other] { $count } compétences
}
skills-files = { $count ->
    [one] { $count } fichier
   *[other] { $count } fichiers
}
skills-tag-scripts = contient des scripts
skills-tag-skipped = { $count ->
    [one] { $count } lien ignoré
   *[other] { $count } liens ignorés
}
skills-tag-installed = installée
skills-tag-installed-at = installée à { $commit }
skills-add-usage = indiquez les compétences à installer : idetea skills add <source> <name>…
skills-remove-usage = indiquez les compétences à retirer : idetea skills remove <name>…
skills-not-found = il n'y a pas de compétence "{ $name }" dans { $source } (idetea skills list { $source })
skills-not-installed = la compétence "{ $name }" n'est pas installée dans ce projet
skills-other-source = la compétence "{ $name }" venait déjà de { $source } ; retirez-la d'abord, ou utilisez --force
skills-refused = refusé (rien n'a changé) : { $reason }
skills-modified = { $path } a été modifié depuis l'installation, et votre changement serait perdu ; pour le remplacer ou le retirer quand même, utilisez --force
skills-exists = { $path } existe déjà et n'a pas été installé par l'IDE ; pour le remplacer, utilisez --force
skills-added = installée : { $name } (@ { $commit }) dans .agents/skills/ et .claude/skills/
skills-has-scripts = { $name } apporte des scripts que l'agent peut exécuter : lisez-les avant de l'utiliser
skills-gone = la compétence "{ $name }" n'existe plus dans { $source } ; la copie du projet reste en l'état
skills-up-to-date = { $name } : à jour
skills-update = { $name } : { $old } → { $new }
skills-updated = mise à jour : { $name } (@ { $commit })
skills-update-hint = Pour installer ce qui a changé : idetea skills update --apply
skills-removed = retirée : { $name }
