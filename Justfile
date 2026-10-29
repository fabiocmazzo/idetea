# Tarefas do projeto. `just --list` mostra todas.

set positional-arguments

data := env_var_or_default("XDG_DATA_HOME", env_var("HOME") / ".local/share")
apps := data / "applications"
icons := data / "icons/hicolor/scalable/apps"

# Roda a IDE (debug) com o ícone instalado. Ex.: `just run src/main.rs`
run *args: desktop
    cargo run -p tea-app -- "$@"

# Roda a build de release com o ícone instalado.
run-release *args: desktop
    cargo run --release -p tea-app -- "$@"

# O desktop (COSMIC, GNOME, KDE...) liga a janela ao ícone pelo app id "idetea", que é o nome
# do arquivo .desktop. O atalho fica fora do menu de aplicativos (NoDisplay): aberto pelo menu, a
# raiz do projeto seria a pasta home. Ele só serve para o ícone.

# Instala o ícone e o .desktop do usuário (para o painel mostrar o ícone da janela).
desktop:
    #!/usr/bin/env sh
    set -eu
    mkdir -p "{{apps}}" "{{icons}}"
    cp assets/idetea.svg "{{icons}}/idetea.svg"
    sed "s|@EXEC@|{{justfile_directory()}}/target/release/idetea|" assets/idetea.desktop > "{{apps}}/idetea.desktop.tmp"
    # Só regrava se mudou (evita o desktop recarregar o menu a cada `just run`).
    if cmp -s "{{apps}}/idetea.desktop.tmp" "{{apps}}/idetea.desktop"; then
        rm "{{apps}}/idetea.desktop.tmp"
    else
        mv "{{apps}}/idetea.desktop.tmp" "{{apps}}/idetea.desktop"
        command -v update-desktop-database >/dev/null && update-desktop-database -q "{{apps}}" || true
        command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "{{data}}/icons/hicolor" || true
        echo "Ícone e atalho instalados em {{data}}"
    fi

# Remove o ícone e o atalho instalados por `just desktop`.
undesktop:
    rm -f "{{apps}}/idetea.desktop" "{{icons}}/idetea.svg"
    command -v update-desktop-database >/dev/null && update-desktop-database -q "{{apps}}" || true

# Checagens antes de commitar.
check:
    cargo clippy --workspace --all-targets
    cargo test --workspace
