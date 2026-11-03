# Textos do idetea skills.
skills-bad-arg = idetea skills: "{ $arg }" não vale.
skills-usage = Uso:
    idetea skills                          as skills instaladas neste projeto
    idetea skills list [fonte]             as skills de uma fonte (padrão: anthropics/skills)
    idetea skills add <fonte> <nome>…      instala em .agents/skills/ e .claude/skills/
    idetea skills update [nome…] [--apply] mostra o que mudou na fonte; --apply instala
    idetea skills remove <nome>…           tira do projeto
  A fonte é dono/repo (GitHub), uma URL git ou uma pasta. --force troca uma cópia mexida à mão.
skills-error = erro: { $error }
skills-no-cache = idetea skills: sem pasta de cache (defina HOME ou XDG_CACHE_HOME)
skills-bad-source = fonte que não vale: "{ $source }" (use dono/repo, uma URL git ou uma pasta)
skills-bad-name = { $path }: o nome "{ $name }" não vale (minúsculas, dígitos, - e _); ignorada
skills-duplicate = { $path }: outra skill já se chama "{ $name }"; ignorada
skills-none-installed = Nenhuma skill instalada neste projeto (idetea skills list para ver as de uma fonte).
skills-installed-line = { $name }  ({ $source } @ { $commit })
skills-list-header = { $source } @ { $commit }: { $count ->
    [one] 1 skill
   *[other] { $count } skills
}
skills-files = { $count ->
    [one] 1 arquivo
   *[other] { $count } arquivos
}
skills-tag-scripts = tem scripts
skills-tag-skipped = { $count ->
    [one] 1 link ignorado
   *[other] { $count } links ignorados
}
skills-tag-installed = instalada
skills-tag-installed-at = instalada em { $commit }
skills-add-usage = diga quais skills instalar: idetea skills add <fonte> <nome>…
skills-remove-usage = diga quais skills tirar: idetea skills remove <nome>…
skills-not-found = não há skill "{ $name }" em { $source } (idetea skills list { $source })
skills-not-installed = a skill "{ $name }" não está instalada neste projeto
skills-other-source = a skill "{ $name }" já veio de { $source }; tire antes, ou use --force
skills-refused = recusado (nada mudou): { $reason }
skills-modified = { $path } foi mexida desde a instalação, e eu perderia a sua mudança; para trocar ou tirar mesmo assim, use --force
skills-exists = { $path } já existe e não foi instalada pela IDE; para trocar, use --force
skills-added = instalada: { $name } (@ { $commit }) em .agents/skills/ e .claude/skills/
skills-has-scripts = { $name } traz scripts que o agente pode rodar: leia antes de usar
skills-gone = a skill "{ $name }" não existe mais em { $source }; a cópia do projeto fica como está
skills-up-to-date = { $name }: em dia
skills-update = { $name }: { $old } → { $new }
skills-updated = atualizada: { $name } (@ { $commit })
skills-update-hint = Para instalar o que mudou: idetea skills update --apply
skills-removed = removida: { $name }
