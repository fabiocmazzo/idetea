<p align="center">
  <img src="assets/idetea.svg" width="112" alt="Ícono del IDE TEA: un vaso de té helado">
</p>

<h1 align="center">IDE TEA</h1>

<p align="center"><b>Just code. Add ice.</b></p>

<p align="center">
  Un IDE rápido y silencioso, en Rust puro, donde los agentes de código preguntan antes de actuar.<br>
  Gratis, siempre: <a href="LICENSE">Apache 2.0</a>. Sin cuenta, sin telemetría, sin marketplace.
</p>

<p align="center">
  <a href="https://github.com/fabiocmazzo/idetea/actions/workflows/ci.yml"><img src="https://github.com/fabiocmazzo/idetea/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://github.com/fabiocmazzo/idetea/releases"><img src="https://img.shields.io/github/v/release/fabiocmazzo/idetea?include_prereleases&amp;label=release" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue" alt="License: Apache 2.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024-orange?logo=rust" alt="Rust 2024"></a>
  <a href="https://github.com/iced-rs/iced"><img src="https://img.shields.io/badge/GUI-iced%200.14-3645ff" alt="GUI: iced 0.14"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey" alt="Platforms: Linux, macOS, Windows">
  <a href="https://codeberg.org/fabiocmazzo/idetea"><img src="https://img.shields.io/badge/Codeberg-mirror-2185D0?logo=codeberg&amp;logoColor=white" alt="Codeberg mirror"></a>
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.pt-BR.md">Português</a> · <b>Español</b> ·
  <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.it.md">Italiano</a> ·
  <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">中文（简体）</a><br>
  <a href="https://idetea.belabs.cloud/es/"><b>Sitio y guías</b></a>
  · <a href="https://github.com/fabiocmazzo/idetea">GitHub</a>
  · <a href="https://codeberg.org/fabiocmazzo/idetea">Codeberg</a> (espejo)
</p>

<p align="center">
  <a href="https://idetea.belabs.cloud/es/#demo">
    <img src="assets/demo-preview.gif" width="880" alt="Demo narrada: abrir un archivo, Rust con resaltado de sintaxis, editor dividido, terminal integrada, y un formulario de agente en el panel Interfaces mientras la mascota Tea pasa a Atención.">
  </a>
</p>

<p align="center">
  <a href="https://idetea.belabs.cloud/es/#demo"><b>Ver la demo de 52 segundos</b></a>
  · narración en inglés, subtítulos en la imagen<br>
  GitHub no reproduce un vídeo del repositorio dentro del README.
  El clip de arriba abre la demo en el sitio, con sonido.
  El archivo está en <a href="assets/idetea-demo.mp4"><code>assets/idetea-demo.mp4</code></a>.
</p>

---

> **Estado:** en desarrollo activo, antes de la 1.0. Compilado y probado en Linux, macOS (Apple Silicon) y Windows (x86_64).
> Habrá asperezas; por favor, avísanos cuando encuentres una.

## Para quién es

Para quien programa con un agente en la terminal al lado del editor: Claude Code, Codex, o cualquier agente que lea
`AGENTS.md` y `CLAUDE.md`. IDE TEA no mete un modelo de chat dentro de la ventana. Le da a ese agente una forma
pequeña, hecha de archivos, de preguntarte, y te deja al mando de cada comando.

## Por qué la gente se queda

**Rápido.** Abre muy por debajo de 200 ms. Un archivo de 50 MB abre en una décima de segundo, y un archivo
minificado con una línea gigante abre en un modo ligero en vez de congelar la ventana.

**Silencioso.** Sin cuenta, sin telemetría, sin marketplace, sin ventanas emergentes que no pediste. La red solo
entra cuando tú lo pides (push, pull, fetch, o una skill que instalas). Apariencia, atajos y paneles viven en un
archivo TOML.

**Tú apruebas lo que se ejecuta.** El agente escribe un JSON pequeño y el IDE muestra un formulario: opciones,
casillas, texto, imágenes, sonidos, una prueba manual paso a paso. El pedido de terminal muestra el comando
completo antes. Un ayudante puede trabajar en su propio worktree de git. Un proyecto nunca se da permiso solo. La
lista de permisos vive en *tu* configuración.

**Completo donde importa.** Varios cursores, zoom temporal, modo vim opcional, árbol de archivos, búsqueda en el
proyecto, terminal integrada, git (stage, diff, commit, push y pull), LSP, depuración con gdb, y vistas previas de
Markdown, SVG, imagen y PDF.

**Pequeño por dentro.** El núcleo solo edita archivos. El árbol, la terminal, git y el depurador son extensiones
que puedes desactivar. Tea, una taza en pixel art, avisa cuando hay una pregunta esperando (un clic abre el panel
correcto) y se pliega en la barra de estado cuando quieres la esquina de vuelta.

Un comando enseña a los agentes del proyecto: "Interfaces: enseñar a los agentes de este proyecto" escribe el
protocolo en `AGENTS.md` y `CLAUDE.md`.

## Funciones en acción

Cinco clips cortos, narración en inglés, subtítulos en la imagen. GitHub no reproduce un vídeo del repositorio
dentro del README. Los pósters abren el clip en el sitio, con sonido. Los archivos están en `assets/`.

<table>
<tr>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/es/#feature-agents"><img src="assets/agents-poster.png" width="440" alt="Agentes: Agentes en tu terminal. Memoria en tu repositorio."></a><br>
    <a href="https://idetea.belabs.cloud/es/#feature-agents"><b>Agentes</b></a> · 57 segundos<br>
    Agentes en tu terminal. Memoria en tu repositorio.<br>
    <a href="assets/idetea-agents.mp4"><code>assets/idetea-agents.mp4</code></a>
  </td>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/es/#feature-tables"><img src="assets/tables-poster.png" width="440" alt="Tablas: Tus datos como tabla. Siguen siendo solo texto."></a><br>
    <a href="https://idetea.belabs.cloud/es/#feature-tables"><b>Tablas</b></a> · 45 segundos<br>
    Tus datos como tabla. Siguen siendo solo texto.<br>
    <a href="assets/idetea-tables.mp4"><code>assets/idetea-tables.mp4</code></a>
  </td>
</tr>
<tr>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/es/#feature-lsp"><img src="assets/lsp-poster.png" width="440" alt="LSP: Soporte profundo de lenguaje, integrado."></a><br>
    <a href="https://idetea.belabs.cloud/es/#feature-lsp"><b>LSP</b></a> · 41 segundos<br>
    Soporte profundo de lenguaje, integrado.<br>
    <a href="assets/idetea-lsp.mp4"><code>assets/idetea-lsp.mp4</code></a>
  </td>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/es/#feature-encodings"><img src="assets/encodings-poster.png" width="440" alt="Codificaciones: Archivos viejos, editor nuevo, no se pierde nada."></a><br>
    <a href="https://idetea.belabs.cloud/es/#feature-encodings"><b>Codificaciones</b></a> · 47 segundos<br>
    Archivos viejos, editor nuevo, no se pierde nada.<br>
    <a href="assets/idetea-encodings.mp4"><code>assets/idetea-encodings.mp4</code></a>
  </td>
</tr>
<tr>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/es/#feature-mascot"><img src="assets/mascot-poster.png" width="440" alt="Mascota: Una taza de té serena, en tu idioma."></a><br>
    <a href="https://idetea.belabs.cloud/es/#feature-mascot"><b>Mascota</b></a> · 52 segundos<br>
    Una taza de té serena, en tu idioma.<br>
    <a href="assets/idetea-mascot.mp4"><code>assets/idetea-mascot.mp4</code></a>
  </td>
  <td width="50%" valign="top"></td>
</tr>
</table>

## Instalación

**Linux, macOS y Windows, listo para usar.** La versión actual es
[v0.16.0](https://github.com/fabiocmazzo/idetea/releases/tag/v0.16.0).
`SHA256SUMS` en esa página tiene las sumas de verificación. Cambia `VERSION` cuando salga una más nueva, o elige el
archivo en [Releases](https://github.com/fabiocmazzo/idetea/releases).

**Linux (x86_64).** Descomprime y ejecuta `install.sh`. Pone `idetea` en `~/.local/bin` y en el menú de aplicaciones:

```sh
VERSION=0.16.0
curl -fsSL -O "https://github.com/fabiocmazzo/idetea/releases/download/v${VERSION}/idetea-${VERSION}-x86_64-linux.tar.gz"
tar -xzf idetea-${VERSION}-x86_64-linux.tar.gz
cd idetea-${VERSION}-x86_64-linux
./install.sh
```

En ARM, usa `idetea-${VERSION}-aarch64-linux.tar.gz`. Si prefieres un solo archivo, descarga
`IDE_TEA-${VERSION}-x86_64.AppImage` (o `IDE_TEA-${VERSION}-aarch64.AppImage`), haz `chmod +x` y ejecútalo.
`--install-desktop` lo pone en el menú.

**macOS (Apple Silicon).** El archivo incluye `IDE TEA.app`:

```sh
VERSION=0.16.0
curl -fsSL -O "https://github.com/fabiocmazzo/idetea/releases/download/v${VERSION}/idetea-${VERSION}-aarch64-macos.tar.gz"
tar -xzf idetea-${VERSION}-aarch64-macos.tar.gz
cd idetea-${VERSION}-aarch64-macos
./idetea --install-desktop   # crea ~/Applications/IDE TEA.app
```

Todavía no hay paquete para Mac con Intel.

**Windows (x86_64).** Descarga y ejecuta
[IDE_TEA-0.16.0-x86_64-setup.exe](https://github.com/fabiocmazzo/idetea/releases/download/v0.16.0/IDE_TEA-0.16.0-x86_64-setup.exe),
o descomprime el
[idetea-0.16.0-x86_64-windows.zip](https://github.com/fabiocmazzo/idetea/releases/download/v0.16.0/idetea-0.16.0-x86_64-windows.zip)
portátil (`idetea.exe` está dentro). En PowerShell:

```powershell
$Version = "0.16.0"
Invoke-WebRequest -Uri "https://github.com/fabiocmazzo/idetea/releases/download/v$Version/IDE_TEA-$Version-x86_64-setup.exe" -OutFile "IDE_TEA-$Version-x86_64-setup.exe"
Start-Process ".\IDE_TEA-$Version-x86_64-setup.exe"
```

**Desde el código fuente.** Necesitas [Rust](https://rustup.rs) (estable). El paquete es `tea-app`; el binario se llama `idetea`:

```sh
cargo install --locked --git https://github.com/fabiocmazzo/idetea tea-app
idetea --install-desktop   # opcional: entrada en el menú de aplicaciones e ícono
```

Opcionales: `git` para la integración con git, `gdb` 14+ (o `lldb-dap` en macOS) para depurar, y los servidores de
lenguaje que uses (rust-analyzer, clangd, pyright, typescript-language-server, gopls, lua-language-server,
zls, `zx lsp`, `hare-lsp` para Hare y `moon lsp` para MoonBit ya vienen configurados).

## Firma de código y privacidad

El instalador de Windows todavía no tiene firma de código, así que SmartScreen puede avisar la primera vez que se
ejecuta. Este programa no transfiere ninguna información a redes externas salvo a petición explícita del usuario.
Consulta [`CODE_SIGNING_POLICY.md`](CODE_SIGNING_POLICY.md).

## Primeros pasos

```sh
idetea                # abre la carpeta actual como proyecto
idetea src/main.rs    # carpeta actual + este archivo
idetea ~/proyecto     # otra carpeta
idetea new mi-app --model rust --agent claude   # un proyecto nuevo, con un agente listo para empezar
```

Dentro del IDE: `Ctrl+Shift+P` abre la paleta de comandos, `Ctrl+P` abre un archivo por nombre, `F1` muestra todos
los atajos. Las guías están en el [sitio](https://idetea.belabs.cloud/es/).

## Por qué IDE TEA, y cuándo no

[`docs/comparativo-ides.md`](docs/comparativo-ides.md) es una comparación honesta con VS Code, Cursor, Zed, JetBrains
y Helix, actualizada para la 0.15.0 (en portugués). En resumen:

- El agente de la terminal tiene un protocolo nativo, hecho de archivos (formularios, comandos que tú apruebas,
  worktrees, la mascota Tea). Eso es lo que los otros editores de esa nota no tienen.
- No hay autocompletado fantasma, a propósito. Si quieres completar con Tab o diffs en línea, ese texto señala a
  Cursor, Windsurf o Zed.
- La cobertura del depurador va por detrás de VS Code y JetBrains (solo gdb y lldb-dap).

### Tea

La taza de la esquina tiene varias frases para cada humor. Nota un archivo guardado, una pausa larga, la hora
del día y el build del depurador, y de vez en cuando cita un atajo que de verdad existe. `[mascot] chatty` es
`quiet`, `normal` o `chatty`; `[mascot] tips` enciende o apaga las pistas. Toda la interfaz, Tea incluido, está en
portugués, inglés, español, francés, alemán, italiano, japonés y chino simplificado (`[ui] language`, o el idioma
del sistema). También habla una vez de un conflicto en disco con ediciones sin guardar, de un conflicto de git,
del fin de una tarea o de la sesión de depuración, de un carácter que la codificación no admite y de un
reemplazo en varios archivos. Los avisos y los errores que llegan a cero quedan en el modo chatty. Los mismos
interruptores están en Ajustes. `idetea mascot setup-hooks` añade hooks de Claude Code y deja los que ya estaban
en el archivo. No roba el foco.

## Dale una estrella

Si este es el editor que querías, [dale una estrella a IDE TEA](https://github.com/fabiocmazzo/idetea). Es el
proyecto de una persona, y la estrella es como lo encuentra la siguiente.

## Sobre el nombre

Empezó como una broma. El IDE está hecho con **iced**, así que tenía que llamarse **Iced Tea**.

Luego apareció el otro sentido. En portugués, **TEA** es la sigla de *Transtorno do Espectro Autista* (en español,
trastorno del espectro autista, que también es TEA). El autor es autista, el doble sentido pareció justo y el nombre
se quedó. Quizá eso explique algunas cosas del IDE: le gusta lo predecible, el texto simple que se puede leer y el
menor ruido posible.

Más en el [manifiesto](MANIFESTO.es.md).

## Documentación

- [Manifiesto](MANIFESTO.es.md): en qué cree IDE TEA, y lo que no va a hacer.
- [Sitio](https://idetea.belabs.cloud/es/): la demo, la instalación y las guías.
- [Contributing](CONTRIBUTING.md): cómo compilar, probar y abrir un pull request (en inglés).
- [Funciones y atajos](docs/features.md): la lista completa, para quien quiere el detalle técnico (en inglés).
- [Lenguajes](docs/languages.md): lo que el IDE hace en cada lenguaje, incluidos Hare y MoonBit, con
  [ejemplos](examples/languages) para probar (en inglés).
- [Interfaces para agentes](docs/interfaces/formato.md) y [terminales de agente](docs/terminais-de-agente.md) (en
  portugués por ahora).
- [SPEC.md](SPEC.md) y [DESENVOLVIMENTO.md](DESENVOLVIMENTO.md): especificación, arquitectura y cómo compilar,
  probar y escribir una extensión (en portugués por ahora).

## Licencia

IDE TEA es software libre bajo la [Licencia Apache 2.0](LICENSE): úsalo, estúdialo, modifícalo y compártelo,
también comercialmente. Es gratis y lo seguirá siendo.

Los íconos de la interfaz son de la fuente [Codicons](https://github.com/microsoft/vscode-codicons) © Microsoft,
bajo [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) (ver
[`crates/tea-core/assets/NOTICE.md`](crates/tea-core/assets/NOTICE.md)).
