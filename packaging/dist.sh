#!/bin/sh
# Gera em dist/:
#   idetea-<versão>-<arch>-linux.tar.gz  executável + install.sh + atalho e ícone
#   IDE_TEA-<versão>-<arch>.AppImage     arquivo único, roda direto (`--install-desktop` põe no menu)
# O executável só depende da libc: Wayland, X11, Vulkan e xkbcommon são carregados do sistema na hora.
# O appimagetool é baixado em target/ na primeira vez (ou use APPIMAGETOOL=/caminho). `--no-appimage`: só o .tar.gz.
set -eu
cd "$(dirname "$0")/.."

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
arch=$(uname -m)
cargo build --locked --profile dist -p tea-app
bin=target/dist/idetea
rm -rf dist
mkdir -p dist

# .tar.gz
name="idetea-$version-$arch-linux"
pkg="dist/$name"
mkdir -p "$pkg"
cp "$bin" assets/idetea.svg LICENSE README.md "$pkg/"
cp packaging/install.sh "$pkg/install.sh"
# Licenças dos recursos de terceiros embutidos no executável (ícones, ícones de arquivo Seti e a JetBrains Mono).
mkdir -p "$pkg/licenses"
cp crates/tea-core/assets/NOTICE.md "$pkg/licenses/NOTICE.md"
cp crates/tea-core/assets/fonts/OFL.txt "$pkg/licenses/JetBrainsMono-OFL.txt"
cp crates/tea-core/assets/seti/LICENSE.txt "$pkg/licenses/Seti-LICENSE.txt"
cp crates/tea-core/assets/syntaxes/LICENSE-ziex.txt "$pkg/licenses/Ziex-LICENSE.txt"
cp crates/tea-core/assets/syntaxes/LICENSE-zig-sublime.txt "$pkg/licenses/sublime-zig-language-LICENSE.txt"
chmod 755 "$pkg/install.sh"
tar -C dist -czf "dist/$name.tar.gz" "$name"
rm -rf "$pkg"

if [ "${1:-}" = "--no-appimage" ]; then
    ls -lh dist
    exit 0
fi

# AppImage
appdir=target/AppDir
rm -rf "$appdir"
mkdir -p "$appdir/usr/bin"
cp "$bin" "$appdir/usr/bin/idetea"
cp packaging/AppRun "$appdir/AppRun"
chmod 755 "$appdir/AppRun"
sed 's|@EXEC@|idetea|' assets/idetea.desktop > "$appdir/idetea.desktop"
cp assets/idetea.svg "$appdir/idetea.svg"
mkdir -p "$appdir/usr/share/doc/idetea"
cp LICENSE crates/tea-core/assets/NOTICE.md "$appdir/usr/share/doc/idetea/"
cp crates/tea-core/assets/fonts/OFL.txt "$appdir/usr/share/doc/idetea/JetBrainsMono-OFL.txt"
cp crates/tea-core/assets/seti/LICENSE.txt "$appdir/usr/share/doc/idetea/Seti-LICENSE.txt"
cp crates/tea-core/assets/syntaxes/LICENSE-ziex.txt "$appdir/usr/share/doc/idetea/Ziex-LICENSE.txt"
cp crates/tea-core/assets/syntaxes/LICENSE-zig-sublime.txt "$appdir/usr/share/doc/idetea/sublime-zig-language-LICENSE.txt"
# PNG de 256 px (gerado do SVG: `rsvg-convert -w 256 -h 256 assets/idetea.svg -o assets/idetea.png`): o
# appimagetool faz o `.DirIcon` com ele.
cp assets/idetea.png "$appdir/idetea.png"
tool="${APPIMAGETOOL:-target/appimagetool-$arch.AppImage}"
if [ ! -x "$tool" ]; then
    curl -fsSL -o "$tool" "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$arch.AppImage"
    chmod 755 "$tool"
fi
# Sem FUSE (CI, contêiner): o appimagetool se extrai e roda.
APPIMAGE_EXTRACT_AND_RUN=1 ARCH="$arch" "$tool" --no-appstream "$appdir" "dist/IDE_TEA-$version-$arch.AppImage"

ls -lh dist
