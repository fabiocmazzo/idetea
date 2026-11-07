#!/bin/sh
# Gera as sintaxes do ZX (Ziex) embutidas na IDE.
# Uso: tools/zx-syntax.sh <clone do ziex> <clone do sublime-zig-language>
#
# - zx.sublime-syntax: a gramática do VS Code do Ziex, convertida (tools/tm2sublime.py).
# - zig.sublime-syntax: a do Zig que o ZX inclui (scope:source.zig). As sintaxes registradas formam um conjunto
#   à parte e não enxergam as do editor, então o Zig vem junto, escondido e sem extensões: os `.zig` continuam
#   com a sintaxe do editor (a mesma gramática, pelo `bat`).
set -eu
Z=${1:?caminho do clone do ziex}
ZIG=${2:?caminho do clone do sublime-zig-language}
OUT=crates/tea-core/assets/syntaxes
python3 -I tools/tm2sublime.py "$Z/ide/vscode/syntaxes/zx.tmLanguage.json" \
    --name ZX --ext zx --ext mdzx \
    --inject-into source.zig --guard zig-guards --inject zx-block \
    --header "Gerado por tools/zx-syntax.sh a partir de ide/vscode/syntaxes/zx.tmLanguage.json do Ziex
(https://github.com/ziex-dev/ziex, commit $(git -C "$Z" rev-parse --short HEAD), licença MIT: LICENSE-ziex.txt).
Não edite à mão: rode o script de novo." > "$OUT/zx.sublime-syntax"
cp "$Z/LICENSE" "$OUT/LICENSE-ziex.txt"
{
    echo "%YAML 1.2"
    echo "---"
    echo "# Copiado por tools/zx-syntax.sh de Syntaxes/Zig.sublime-syntax do sublime-zig-language"
    echo "# (https://github.com/ziglang/sublime-zig-language, commit $(git -C "$ZIG" rev-parse --short HEAD), licença MIT:"
    echo "# LICENSE-zig-sublime.txt). Mudanças: escondida e sem extensões (só para o ZX incluir)."
    sed -e '1,2d' -e '/^#/d' "$ZIG/Syntaxes/Zig.sublime-syntax" \
        | awk '/^file_extensions:/ { print "file_extensions: []"; print "hidden: true"; skip = 1; next }
               skip && /^  - / { next }
               { skip = 0; print }'
} > "$OUT/zig.sublime-syntax"
cp "$ZIG/LICENSE" "$OUT/LICENSE-zig-sublime.txt"
