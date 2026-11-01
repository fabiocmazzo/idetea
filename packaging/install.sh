#!/bin/sh
# Instala a IDE TEA para este usuário: o executável em ~/.local/bin e o atalho (com o ícone) no menu de
# aplicativos. Para desinstalar: `idetea --uninstall-desktop && rm ~/.local/bin/idetea`.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
bin="${XDG_BIN_HOME:-$HOME/.local/bin}"
mkdir -p "$bin"
install -m 755 "$here/idetea" "$bin/idetea"
"$bin/idetea" --install-desktop
case ":$PATH:" in
    *":$bin:"*) ;;
    *) echo "Aviso: $bin não está no PATH; para rodar \`idetea\` no terminal, adicione-o (ex.: no ~/.profile)." ;;
esac
