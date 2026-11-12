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
# do arquivo .desktop. Sem ele, a janela fica sem ícone no painel.

# Um atalho já instalado (ex.: por `idetea --install-desktop` de uma versão instalada) é mantido.

# Instala o ícone e, se ainda não houver atalho, o .desktop apontando para a build de release daqui.
desktop:
    #!/usr/bin/env sh
    set -eu
    mkdir -p "{{apps}}" "{{icons}}"
    cmp -s assets/idetea.svg "{{icons}}/idetea.svg" || cp assets/idetea.svg "{{icons}}/idetea.svg"
    if [ ! -e "{{apps}}/idetea.desktop" ]; then
        sed "s|@EXEC@|{{justfile_directory()}}/target/release/idetea|" assets/idetea.desktop > "{{apps}}/idetea.desktop"
        command -v update-desktop-database >/dev/null && update-desktop-database -q "{{apps}}" || true
        command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "{{data}}/icons/hicolor" || true
        echo "Ícone e atalho instalados em {{data}}"
    fi

# Remove o ícone e o atalho instalados por `just desktop`.
undesktop:
    rm -f "{{apps}}/idetea.desktop" "{{icons}}/idetea.svg"
    command -v update-desktop-database >/dev/null && update-desktop-database -q "{{apps}}" || true

# Gera o .tar.gz e o AppImage em dist/ (o mesmo que o CI publica a cada versão).
dist *args:
    packaging/dist.sh "$@"

# Checagens antes de commitar.
check:
    cargo clippy --workspace --all-targets
    cargo test --workspace

# Grava o vídeo narrado da demo (Xvfb + ffmpeg). Saída: dist/idetea-demo.mp4.
# Com OPENROUTER_API_KEY no ambiente, usa a voz sage. Sem a chave, usa o Piper local.
# A chave não é impressa. Ver tools/demo-video/README.md.
demo *args:
    ./tools/demo-video/demo.sh "$@"
