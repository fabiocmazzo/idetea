# Testi di idetea skills.
skills-bad-arg = idetea skills: "{ $arg }" non è valido.
skills-usage = Uso:
    idetea skills                          le skill installate in questo progetto
    idetea skills list [source]            le skill di una sorgente (predefinito: anthropics/skills)
    idetea skills add <source> <name>…     installa in .agents/skills/ e .claude/skills/
    idetea skills update [name…] [--apply] mostra che cosa è cambiato nella sorgente; --apply lo installa
    idetea skills remove <name>…           rimuove dal progetto
  La sorgente è owner/repo (GitHub), un URL git o una cartella. --force sostituisce una copia modificata a mano.
skills-error = errore: { $error }
skills-no-cache = idetea skills: nessuna cartella di cache (imposta HOME o XDG_CACHE_HOME)
skills-bad-source = sorgente non valida: "{ $source }" (usa owner/repo, un URL git o una cartella)
skills-bad-name = { $path }: il nome "{ $name }" non è valido (minuscole, cifre, - e _); saltato
skills-duplicate = { $path }: un'altra skill si chiama già "{ $name }"; saltata
skills-none-installed = Nessuna skill installata in questo progetto (idetea skills list per vederne una sorgente).
skills-installed-line = { $name }  ({ $source } @ { $commit })
skills-list-header = { $source } @ { $commit }: { $count ->
    [one] { $count } skill
   *[other] { $count } skill
}
skills-files = { $count ->
    [one] { $count } file
   *[other] { $count } file
}
skills-tag-scripts = ha script
skills-tag-skipped = { $count ->
    [one] { $count } collegamento saltato
   *[other] { $count } collegamenti saltati
}
skills-tag-installed = installata
skills-tag-installed-at = installata a { $commit }
skills-add-usage = indica quali skill installare: idetea skills add <source> <name>…
skills-remove-usage = indica quali skill rimuovere: idetea skills remove <name>…
skills-not-found = non c'è la skill "{ $name }" in { $source } (idetea skills list { $source })
skills-not-installed = la skill "{ $name }" non è installata in questo progetto
skills-other-source = la skill "{ $name }" veniva già da { $source }; rimuovila prima, oppure usa --force
skills-refused = rifiutato (niente è cambiato): { $reason }
skills-modified = { $path } è cambiato dopo l'installazione e la modifica andrebbe persa; per sostituirlo o rimuoverlo comunque, usa --force
skills-exists = { $path } esiste già e non è stato installato dall'IDE; per sostituirlo, usa --force
skills-added = installata: { $name } (@ { $commit }) in .agents/skills/ e .claude/skills/
skills-has-scripts = { $name } porta script che l'agente può eseguire: leggili prima di usarla
skills-gone = la skill "{ $name }" non esiste più in { $source }; la copia del progetto resta com'è
skills-up-to-date = { $name }: aggiornata
skills-update = { $name }: { $old } → { $new }
skills-updated = aggiornata: { $name } (@ { $commit })
skills-update-hint = Per installare che cosa è cambiato: idetea skills update --apply
skills-removed = rimossa: { $name }
