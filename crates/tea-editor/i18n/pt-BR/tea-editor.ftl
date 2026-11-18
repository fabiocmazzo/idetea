## Editor, imagens e PDF
not-utf8 = arquivo não é UTF-8 (ainda não suportado)
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
