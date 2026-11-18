## Painel Busca
search-panel = Search
cmd-search-focus = Search: search the project
cmd-search-clear = Search: clear
search-placeholder = Search the project (Enter searches again)
search-bad-regex = Invalid regex: { $error }
search-running = Searching…
search-truncated = {" "}(stopped at { $count })
search-summary = { $results } result(s) in { $files } file(s){ $more } · { $scanned } files read
search-min-chars = Type at least { $count } characters

## Substituir
cmd-search-replace = Search: replace in project
search-replace-placeholder = Replace with (Enter replaces all)
search-replace-all = Replace all
search-replace-one = Replace
search-cancel = Cancel
search-replace-all-title = Replace in { $files } file(s)?
search-replace-all-message = 
    { $results } result(s) of "{ $query }" become "{ $replacement }".
    
    Files open in tabs stay changed, not saved (Ctrl+Z undoes). The others are written straight to disk, with no undo.
search-replace-all-truncated = The search stopped at { $count } results: only the listed files change. Search again afterwards to see the rest.
## Escopo, filtros e navegação
cmd-search-next = Search: next result
cmd-search-prev = Search: previous result
search-filters = Filters
search-context = Context
search-history = Recent searches
search-history-empty = No recent searches
search-include-placeholder = files to include (e.g. src/**, *.rs)
search-exclude-placeholder = files to exclude (e.g. *.test.rs)
search-bad-glob = Invalid pattern: { $error }
search-no-results = No results
search-scope-folder = In folder { $path }
search-scope-file = In file { $path }
search-scope-clear = Stop limiting the search to this path
