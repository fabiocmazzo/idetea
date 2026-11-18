## Editor, imagens e PDF
not-utf8 = file is not UTF-8 (not supported yet)
untitled = untitled
image-info = { $width } × { $height } px · { $size } · mouse wheel zooms, drag to move
image-error = Could not read the image: { $error }
pdf-error = Could not open the PDF: { $error }
pdf-page = Page { $current } of { $total }
pdf-fit = Fit
pdf-encrypted = password-protected PDF (not supported yet)
pdf-invalid = invalid or damaged file

language-image = Image
language-light = { $language } (light mode)

## Marcas do git
hunk-added = 
    { $count ->
        [one] 1 new line
       *[other] { $count } new lines
    }
hunk-modified = 
    { $count ->
        [one] 1 changed line; it was:
       *[other] { $count } changed lines; it was:
    }
hunk-deleted = 
    { $count ->
        [one] 1 deleted line:
       *[other] { $count } deleted lines:
    }
hunk-added-note = Not in the last commit.
hunk-revert = Revert change
image-saved = saved at { $time }

## Table view (RFC 0020)
table-show-text = Text
table-filter = Filter
table-rows = { $count ->
    [one] 1 row
   *[other] { $count } rows
}
table-rows-filtered = { $shown } of { $count } rows
table-sorted = sorted by { $column } { $arrow }
table-invalid-json = Not JSON: { $error }
table-broken-row = This line could not be read; edit it in the text.
table-out-of-range = This cell does not exist.
table-header-row = First row is the header
table-rows-picked = { $count ->
    [one] 1 row picked
   *[other] { $count } rows picked
}
