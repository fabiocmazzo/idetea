## Erros do documento (aparecem no painel, para o usuário e para o agente corrigir)
ui-too-big = The document has { $size } bytes; the maximum is { $max }.
ui-not-json = The file is not valid JSON: { $error }
ui-not-object = The document must be a JSON object.
ui-wrong-format = "format" must be "idetea-ui" (got { $found }).
ui-unsupported-version = Format version { $version } is not supported (this IDE understands { $supported }).
ui-missing = Missing field "{ $field }".
ui-wrong-type = "{ $field }" must be of type { $expected }.
ui-too-many = "{ $field }" has more than { $max } items.
ui-too-long = "{ $field }" is longer than { $max } bytes.
ui-bad-id = "{ $field }": invalid id ("{ $id }"). Use letters, digits, "_" and "-" (up to 64).
ui-duplicate-id = Duplicate id: "{ $id }".
ui-bad-path = "{ $field }": "{ $path }" is not a valid path (e.g. "/name").
ui-bad-file-path = "{ $field }": "{ $path }" must be a path relative to the project root, without "..".
ui-data-too-big = "data" is larger than { $max } bytes.
ui-data-too-deep = "data" is more than { $max } levels deep.

## Avisos (o documento abre mesmo assim)
ui-unknown-field = Unknown field ignored: "{ $field }".
ui-unknown-value = "{ $field }": unknown value "{ $value }", using the default.
ui-unknown-component = Component "{ $component }" is not supported in this version.
ui-link-error = Could not open { $url }: { $error }
ui-link-ignored = Link ignored (only web, project files and idetea:config): { $url }
ui-comment-label = Comment for the agent (optional)
ui-comment-placeholder = sent along with the button you click
ui-copy = Copy
ui-copied = Copied.
ui-steps-missing = { $count ->
    [one] 1 step left to check.
   *[other] { $count } steps left to check.
}
ui-step-optional = (optional)
ui-requires-not-steps = "{ $field }" must point to the "value" of a "steps"; ignored.
