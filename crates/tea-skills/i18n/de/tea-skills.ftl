# Texte von idetea skills.
skills-bad-arg = idetea skills: "{ $arg }" ist ungültig.
skills-usage = Aufruf:
    idetea skills                          die in diesem Projekt installierten Skills
    idetea skills list [Quelle]            die Skills einer Quelle (Standard: anthropics/skills)
    idetea skills add <Quelle> <Name>…     installiert nach .agents/skills/ und .claude/skills/
    idetea skills update [Name…] [--apply] zeigt, was sich in der Quelle geändert hat; --apply installiert es
    idetea skills remove <Name>…           entfernt sie aus dem Projekt
  Die Quelle ist owner/repo (GitHub), eine Git-URL oder ein Ordner. --force ersetzt eine von Hand geänderte Kopie.
skills-error = Fehler: { $error }
skills-no-cache = idetea skills: kein Cache-Ordner (HOME oder XDG_CACHE_HOME setzen)
skills-bad-source = ungültige Quelle: "{ $source }" (owner/repo, eine Git-URL oder einen Ordner verwenden)
skills-bad-name = { $path }: der Name "{ $name }" ist ungültig (Kleinbuchstaben, Ziffern, - und _); übersprungen
skills-duplicate = { $path }: ein anderer Skill heißt bereits "{ $name }"; übersprungen
skills-none-installed = Keine Skills in diesem Projekt installiert (idetea skills list zeigt die einer Quelle).
skills-installed-line = { $name }  ({ $source } @ { $commit })
skills-list-header = { $source } @ { $commit }: { $count ->
    [one] { $count } Skill
   *[other] { $count } Skills
}
skills-files = { $count ->
    [one] { $count } Datei
   *[other] { $count } Dateien
}
skills-tag-scripts = enthält Skripte
skills-tag-skipped = { $count ->
    [one] { $count } Link übersprungen
   *[other] { $count } Links übersprungen
}
skills-tag-installed = installiert
skills-tag-installed-at = installiert bei { $commit }
skills-add-usage = geben Sie an, welche Skills installiert werden sollen: idetea skills add <source> <name>…
skills-remove-usage = geben Sie an, welche Skills entfernt werden sollen: idetea skills remove <name>…
skills-not-found = es gibt keinen Skill "{ $name }" in { $source } (idetea skills list { $source })
skills-not-installed = der Skill "{ $name }" ist in diesem Projekt nicht installiert
skills-other-source = der Skill "{ $name }" stammt bereits aus { $source }; entfernen Sie ihn zuerst, oder verwenden Sie --force
skills-refused = abgelehnt (nichts geändert): { $reason }
skills-modified = { $path } wurde seit der Installation geändert, und Ihre Änderung ginge verloren; zum Ersetzen oder Entfernen trotzdem --force verwenden
skills-exists = { $path } existiert bereits und wurde nicht von der IDE installiert; zum Ersetzen --force verwenden
skills-added = installiert: { $name } (@ { $commit }) nach .agents/skills/ und .claude/skills/
skills-has-scripts = { $name } bringt Skripte mit, die der Agent ausführen kann: lesen Sie sie vor der Verwendung
skills-gone = der Skill "{ $name }" existiert in { $source } nicht mehr; die Kopie im Projekt bleibt, wie sie ist
skills-up-to-date = { $name }: aktuell
skills-update = { $name }: { $old } → { $new }
skills-updated = aktualisiert: { $name } (@ { $commit })
skills-update-hint = Zum Installieren der Änderungen: idetea skills update --apply
skills-removed = entfernt: { $name }
