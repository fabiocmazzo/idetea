## Painel Busca
search-panel = Busca
cmd-search-focus = Busca: buscar no projeto
cmd-search-clear = Busca: limpar
search-placeholder = Buscar no projeto (Enter busca de novo)
search-bad-regex = Regex inválida: { $error }
search-running = Buscando…
search-truncated = {" "}(parou em { $count })
search-summary = { $results } resultado(s) em { $files } arquivo(s){ $more } · { $scanned } arquivos lidos
search-min-chars = Digite ao menos { $count } caracteres

## Substituir
cmd-search-replace = Busca: substituir no projeto
search-replace-placeholder = Substituir por (Enter substitui tudo)
search-replace-all = Substituir tudo
search-replace-one = Substituir
search-cancel = Cancelar
search-replace-all-title = Substituir em { $files } arquivo(s)?
search-replace-all-message = 
    { $results } resultado(s) de "{ $query }" viram "{ $replacement }".
    
    Arquivos abertos em abas ficam alterados sem salvar (Ctrl+Z desfaz). Os outros são gravados direto no disco, sem desfazer.
search-replace-all-truncated = A busca parou em { $count } resultados: só os arquivos listados mudam. Depois, busque de novo para ver o resto.
## Escopo, filtros e navegação
cmd-search-next = Busca: próximo resultado
cmd-search-prev = Busca: resultado anterior
search-filters = Filtros
search-context = Contexto
search-history = Buscas recentes
search-history-empty = Nenhuma busca recente
search-include-placeholder = arquivos a incluir (ex.: src/**, *.rs)
search-exclude-placeholder = arquivos a excluir (ex.: *.test.rs)
search-bad-glob = Padrão inválido: { $error }
search-no-results = Nenhum resultado
search-scope-folder = Na pasta { $path }
search-scope-file = No arquivo { $path }
search-scope-clear = Parar de limitar a busca a este caminho
