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
  <a href="README.md">English</a> · <a href="README.pt-BR.md">Português</a> · <b>Español</b><br>
  <a href="https://idetea.belabs.cloud/es/"><b>Sitio y guías</b></a>
  · <a href="https://github.com/fabiocmazzo/idetea">GitHub</a>
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

## Instalación

**Linux, macOS y Windows, listo para usar.** Descarga el paquete de tu sistema en
[Releases](https://github.com/fabiocmazzo/idetea/releases): `.tar.gz` o `.AppImage` para Linux x86_64/aarch64,
`.tar.gz` con `IDE TEA.app` para macOS aarch64, o el instalador `-setup.exe` / `.zip` portátil para Windows x86_64.
`SHA256SUMS` tiene las sumas de verificación.

En Linux, `./install.sh` pone `idetea` en `~/.local/bin` y en el menú de aplicaciones. El AppImage funciona tal
cual (`chmod +x` antes) y `--install-desktop` lo pone en el menú. En macOS, `--install-desktop` crea
`~/Applications/IDE TEA.app`.

**Desde el código fuente.** Necesitas [Rust](https://rustup.rs) (estable). Después:

```sh
cargo install --locked --git https://github.com/fabiocmazzo/idetea tea-app
idetea --install-desktop   # opcional: entrada en el menú de aplicaciones e ícono
```

Opcionales: `git` para la integración con git, `gdb` 14+ (o `lldb-dap` en macOS) para depurar, y los servidores de
lenguaje que uses (rust-analyzer, clangd, pyright, typescript-language-server, gopls y lua-language-server ya vienen
configurados).

## Firma de código y privacidad

La firma gratuita de código en Windows es proporcionada por [SignPath.io](https://signpath.io), con certificado de la
[SignPath Foundation](https://signpath.org). Este programa no transfiere ninguna información a redes externas salvo a
petición explícita del usuario. Consulta [`CODE_SIGNING_POLICY.md`](CODE_SIGNING_POLICY.md).

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
y Helix, escrita alrededor de la 0.4.0 (en portugués). Lo que sigue siendo el producto:

- El agente de la terminal tiene un protocolo nativo, hecho de archivos (formularios, comandos que tú apruebas,
  worktrees, la mascota Tea). Eso es lo que los otros editores de esa nota no tienen.

### Tea

La taza de la esquina tiene varias frases para cada humor. Nota un archivo guardado, una pausa larga, la hora
del día y el build del depurador, y de vez en cuando cita un atajo que de verdad existe. `[mascot] chatty` es
`quiet`, `normal` o `chatty`; `[mascot] tips` enciende o apaga las pistas. Habla portugués, inglés, español,
francés, alemán, italiano, japonés y chino simplificado. No roba el foco.
- No hay autocompletado fantasma, a propósito. Si quieres completar con Tab o diffs en línea, ese texto señala a
  Cursor, Windsurf o Zed.
- Las code actions, los inlay hints y la cobertura del depurador van por detrás de VS Code y JetBrains.

La misma nota todavía dice "solo Linux". Las versiones de ahora también salen para macOS (Apple Silicon) y Windows (x86_64).

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
