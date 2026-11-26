## Editor, imagens e PDF
binary-file = é um arquivo binário, não texto
not-valid-in = o arquivo não é { $encoding } válido
not-in-encoding = “{ $ch }” (linha { $line }, coluna { $column }) não existe em { $encoding }; salve como UTF-8 para mantê-lo
untitled = sem título
image-info = { $width } × { $height } px · { $size } · roda do mouse aproxima, arrastar move
image-error = Não foi possível ler a imagem: { $error }
pdf-error = Não foi possível abrir o PDF: { $error }
pdf-page = Página { $current } de { $total }
pdf-fit = Ajustar
pdf-encrypted = PDF protegido por senha (ainda não suportado)
pdf-invalid = arquivo inválido ou danificado

language-image = Imagem
language-light = { $language } (modo leve)

## Marcas do git
hunk-added = 
    { $count ->
        [one] 1 linha nova
       *[other] { $count } linhas novas
    }
hunk-modified = 
    { $count ->
        [one] 1 linha mudada; antes era:
       *[other] { $count } linhas mudadas; antes era:
    }
hunk-deleted = 
    { $count ->
        [one] 1 linha apagada:
       *[other] { $count } linhas apagadas:
    }
hunk-added-note = Não estão no último commit.
hunk-revert = Desfazer mudança
image-saved = gravada às { $time }

## Vista de tabela (RFC 0020)
table-show-text = Texto
table-filter = Filtrar
table-rows = { $count ->
    [one] 1 linha
   *[other] { $count } linhas
}
table-rows-filtered = { $shown } de { $count } linhas
table-sorted = ordenado por { $column } { $arrow }
table-invalid-json = Não é JSON: { $error }
table-broken-row = Esta linha não pôde ser lida; edite no texto.
table-out-of-range = Esta célula não existe.
table-header-row = Primeira linha é o cabeçalho
table-rows-picked = { $count ->
    [one] 1 linha escolhida
   *[other] { $count } linhas escolhidas
}
table-add-row = + Linha
table-delete-rows = Apagar linhas
table-add-column = + Coluna
table-delete-column = Apagar coluna
table-new-column = coluna { $number }
table-duplicate-column = Já existe uma coluna com esse nome.
table-no-header = Sem cabeçalho, as colunas não têm nome.
table-empty-name = O nome da coluna não pode ficar vazio.
