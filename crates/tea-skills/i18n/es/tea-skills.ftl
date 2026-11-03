# Textos do idetea skills.
skills-bad-arg = idetea skills: "{ $arg }" no vale.
skills-usage = Uso:
    idetea skills                          las skills instaladas en este proyecto
    idetea skills list [fuente]            las skills de una fuente (por defecto: anthropics/skills)
    idetea skills add <fuente> <nombre>…   instala en .agents/skills/ y .claude/skills/
    idetea skills update [nombre…] [--apply] muestra qué cambió en la fuente; --apply lo instala
    idetea skills remove <nombre>…         las quita del proyecto
  La fuente es dueño/repo (GitHub), una URL git o una carpeta. --force reemplaza una copia editada a mano.
skills-error = error: { $error }
skills-no-cache = idetea skills: sin carpeta de caché (define HOME o XDG_CACHE_HOME)
skills-bad-source = fuente no válida: "{ $source }" (usa dueño/repo, una URL git o una carpeta)
skills-bad-name = { $path }: el nombre "{ $name }" no vale (minúsculas, dígitos, - y _); ignorada
skills-duplicate = { $path }: otra skill ya se llama "{ $name }"; ignorada
skills-none-installed = Ninguna skill instalada en este proyecto (idetea skills list para ver las de una fuente).
skills-installed-line = { $name }  ({ $source } @ { $commit })
skills-list-header = { $source } @ { $commit }: { $count ->
    [one] 1 skill
   *[other] { $count } skills
}
skills-files = { $count ->
    [one] 1 archivo
   *[other] { $count } archivos
}
skills-tag-scripts = tiene scripts
skills-tag-skipped = { $count ->
    [one] 1 enlace ignorado
   *[other] { $count } enlaces ignorados
}
skills-tag-installed = instalada
skills-tag-installed-at = instalada en { $commit }
skills-add-usage = di qué skills instalar: idetea skills add <fuente> <nombre>…
skills-remove-usage = di qué skills quitar: idetea skills remove <nombre>…
skills-not-found = no hay skill "{ $name }" en { $source } (idetea skills list { $source })
skills-not-installed = la skill "{ $name }" no está instalada en este proyecto
skills-other-source = la skill "{ $name }" ya vino de { $source }; quítala antes, o usa --force
skills-refused = rechazado (nada cambió): { $reason }
skills-modified = { $path } cambió desde la instalación, y se perdería tu cambio; para reemplazarla o quitarla igual, usa --force
skills-exists = { $path } ya existe y no la instaló el IDE; para reemplazarla, usa --force
skills-added = instalada: { $name } (@ { $commit }) en .agents/skills/ y .claude/skills/
skills-has-scripts = { $name } trae scripts que el agente puede ejecutar: léelos antes de usarla
skills-gone = la skill "{ $name }" ya no existe en { $source }; la copia del proyecto queda como está
skills-up-to-date = { $name }: al día
skills-update = { $name }: { $old } → { $new }
skills-updated = actualizada: { $name } (@ { $commit })
skills-update-hint = Para instalar lo que cambió: idetea skills update --apply
skills-removed = quitada: { $name }
