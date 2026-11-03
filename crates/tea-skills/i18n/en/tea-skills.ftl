# Textos do idetea skills.
skills-bad-arg = idetea skills: "{ $arg }" is not valid.
skills-usage = Usage:
    idetea skills                          the skills installed in this project
    idetea skills list [source]            the skills of a source (default: anthropics/skills)
    idetea skills add <source> <name>…     installs into .agents/skills/ and .claude/skills/
    idetea skills update [name…] [--apply] shows what changed in the source; --apply installs it
    idetea skills remove <name>…           removes from the project
  The source is owner/repo (GitHub), a git URL or a folder. --force replaces a copy edited by hand.
skills-error = error: { $error }
skills-no-cache = idetea skills: no cache folder (set HOME or XDG_CACHE_HOME)
skills-bad-source = invalid source: "{ $source }" (use owner/repo, a git URL or a folder)
skills-bad-name = { $path }: the name "{ $name }" is not valid (lowercase, digits, - and _); skipped
skills-duplicate = { $path }: another skill is already called "{ $name }"; skipped
skills-none-installed = No skills installed in this project (idetea skills list to see a source's).
skills-installed-line = { $name }  ({ $source } @ { $commit })
skills-list-header = { $source } @ { $commit }: { $count ->
    [one] 1 skill
   *[other] { $count } skills
}
skills-files = { $count ->
    [one] 1 file
   *[other] { $count } files
}
skills-tag-scripts = has scripts
skills-tag-skipped = { $count ->
    [one] 1 link skipped
   *[other] { $count } links skipped
}
skills-tag-installed = installed
skills-tag-installed-at = installed at { $commit }
skills-add-usage = say which skills to install: idetea skills add <source> <name>…
skills-remove-usage = say which skills to remove: idetea skills remove <name>…
skills-not-found = there is no skill "{ $name }" in { $source } (idetea skills list { $source })
skills-not-installed = the skill "{ $name }" is not installed in this project
skills-other-source = the skill "{ $name }" already came from { $source }; remove it first, or use --force
skills-refused = refused (nothing changed): { $reason }
skills-modified = { $path } was changed since it was installed, and your change would be lost; to replace or remove it anyway, use --force
skills-exists = { $path } already exists and was not installed by the IDE; to replace it, use --force
skills-added = installed: { $name } (@ { $commit }) into .agents/skills/ and .claude/skills/
skills-has-scripts = { $name } brings scripts the agent may run: read them before using it
skills-gone = the skill "{ $name }" no longer exists in { $source }; the project's copy stays as is
skills-up-to-date = { $name }: up to date
skills-update = { $name }: { $old } → { $new }
skills-updated = updated: { $name } (@ { $commit })
skills-update-hint = To install what changed: idetea skills update --apply
skills-removed = removed: { $name }
