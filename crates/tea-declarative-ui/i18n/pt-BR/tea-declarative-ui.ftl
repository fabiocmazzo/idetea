## Erros do documento (aparecem no painel, para o usuário e para o agente corrigir)
ui-too-big = O documento tem { $size } bytes; o máximo é { $max }.
ui-not-json = O arquivo não é um JSON válido: { $error }
ui-not-object = O documento precisa ser um objeto JSON.
ui-wrong-format = "format" precisa ser "idetea-ui" (veio { $found }).
ui-unsupported-version = Versão { $version } do formato não suportada (esta IDE entende a { $supported }).
ui-missing = Falta o campo "{ $field }".
ui-wrong-type = "{ $field }" precisa ser do tipo { $expected }.
ui-too-many = "{ $field }" tem mais de { $max } itens.
ui-too-long = "{ $field }" passa de { $max } bytes.
ui-bad-id = "{ $field }": id inválido ("{ $id }"). Use letras, números, "_" e "-" (até 64).
ui-duplicate-id = Id repetido: "{ $id }".
ui-bad-path = "{ $field }": "{ $path }" não é um caminho válido (ex.: "/nome").
ui-bad-file-path = "{ $field }": "{ $path }" tem que ser um caminho relativo à raiz do projeto, sem "..".
ui-data-too-big = "data" passa de { $max } bytes.
ui-data-too-deep = "data" tem mais de { $max } níveis.

## Avisos (o documento abre mesmo assim)
ui-unknown-field = Campo desconhecido ignorado: "{ $field }".
ui-unknown-value = "{ $field }": valor desconhecido "{ $value }", usando o padrão.
ui-unknown-component = Componente "{ $component }" não suportado nesta versão.
