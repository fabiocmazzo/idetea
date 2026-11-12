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
    *)
        lang=$(printf '%s' "${LC_ALL:-${LANG:-C}}" | tr '[:upper:]' '[:lower:]')
        case "$lang" in
            zh*) echo "提示：$bin 不在 PATH 中。要在终端里运行 \`idetea\`，请把它加进去（例如写在 ~/.profile）。" ;;
            ja*) echo "注意: $bin が PATH にありません。ターミナルで \`idetea\` を実行するには追加してください（例: ~/.profile）。" ;;
            fr*) echo "Avertissement : $bin n'est pas dans le PATH. Pour lancer \`idetea\` dans un terminal, ajoutez-le (par exemple dans ~/.profile)." ;;
            de*) echo "Hinweis: $bin ist nicht im PATH. Damit \`idetea\` im Terminal läuft, fügen Sie ihn hinzu (z. B. in ~/.profile)." ;;
            it*) echo "Avviso: $bin non è nel PATH. Per eseguire \`idetea\` da un terminale, aggiungilo (per esempio in ~/.profile)." ;;
            es*) echo "Aviso: $bin no está en el PATH; para ejecutar \`idetea\` en la terminal, añádelo (p. ej. en ~/.profile)." ;;
            en*) echo "Warning: $bin is not on PATH. To run \`idetea\` from a terminal, add it (for example in ~/.profile)." ;;
            *) echo "Aviso: $bin não está no PATH; para rodar \`idetea\` no terminal, adicione-o (ex.: no ~/.profile)." ;;
        esac
        ;;
esac
