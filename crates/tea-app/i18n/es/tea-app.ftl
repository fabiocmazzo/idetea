## Configuración → Apariencia

settings-theme = Tema
settings-theme-detail = Colores del IDE, del editor y de la terminal.
settings-font-size = Tamaño de fuente
settings-font-size-detail = Editor y terminal.
settings-code-font = Fuente del código
settings-code-font-detail = Editor, números de línea y terminal. JetBrains Mono viene con el IDE; la lista tiene las fuentes monoespaciadas instaladas.
settings-font-weight = Grosor de la fuente
settings-font-weight-detail = Un grosor que la fuente no tiene usa el más cercano.
settings-line-height = Altura de línea
settings-line-height-detail = En múltiplos del tamaño de fuente. La terminal la sigue en la misma proporción.
settings-file-icons = Íconos de archivo
settings-file-icons-detail = Un ícono por lenguaje o tipo de archivo (el conjunto Seti, de VS Code). Apagado: el mismo ícono para todos.
settings-activity-bar = Barra de iconos
settings-activity-bar-detail = Franja en el borde izquierdo para mostrar/ocultar paneles con un clic.
settings-splash = Splash al abrir
settings-splash-detail = Pantalla de inicio (mínimo 2 s). --no-splash la desactiva en una ejecución.
settings-language = Idioma
settings-language-detail = Textos de la interfaz. Automático sigue el idioma del sistema.
settings-language-auto = Automático ({ $language })
settings-language-restart = Se aplica al reiniciar el IDE (ahora: { $language }).

## Comandos do núcleo (título na paleta e na ajuda)
cmd-palette-open = Paleta de comandos
cmd-help-open = Ayuda: atajos y guía
cmd-settings-open = Configuración
cmd-file-new = Archivo: nuevo
cmd-file-save = Archivo: guardar
cmd-file-open = Archivo: abrir…
cmd-file-quick-open = Archivo: abrir por nombre…
cmd-debug-continue = Depuración: iniciar / continuar
cmd-debug-stop = Depuración: detener
cmd-debug-pause = Depuración: pausar
cmd-debug-next = Depuración: siguiente línea
cmd-debug-step-in = Depuración: entrar en la función
cmd-debug-step-out = Depuración: salir de la función
cmd-debug-toggle-breakpoint = Depuración: activar/desactivar breakpoint en la línea
cmd-debug-choose-program = Depuración: elegir el programa…
cmd-file-open-folder = Archivo: abrir carpeta en una ventana nueva…
cmd-file-save-as = Archivo: guardar como…
cmd-file-save-as-path = Archivo: guardar como (escribir la ruta)…
cmd-file-close = Archivo: cerrar pestaña
cmd-tab-next = Pestañas: siguiente
cmd-tab-prev = Pestañas: anterior
cmd-tab-reopen = Pestañas: reabrir la última cerrada
cmd-editor-find = Editor: buscar
cmd-editor-replace = Editor: buscar y reemplazar
cmd-editor-goto-line = Editor: ir a la línea
cmd-editor-undo = Editor: deshacer
cmd-editor-redo = Editor: rehacer
cmd-editor-select-all = Editor: seleccionar todo
cmd-editor-fold-all = Editor: plegar todos los bloques
cmd-editor-unfold-all = Editor: desplegar todos los bloques
cmd-editor-toggle-fold = Editor: plegar o desplegar el bloque del cursor
cmd-editor-toggle-vim = Editor: activar/desactivar modo vim
cmd-lsp-definition = LSP: ir a la definición
cmd-lsp-references = LSP: buscar dónde se usa
cmd-lsp-implementation = LSP: ir a la implementación
cmd-lsp-type-definition = LSP: ir al tipo
cmd-lsp-incoming-calls = LSP: quién llama a esta función
cmd-lsp-outgoing-calls = LSP: qué llama esta función
cmd-nav-file-symbol = Ir a símbolo en el archivo…
cmd-nav-project-symbol = Ir a símbolo en el proyecto…
cmd-nav-back = Navegación: atrás
cmd-nav-forward = Navegación: adelante
cmd-lsp-hover = LSP: mostrar documentación en el cursor
cmd-lsp-format = LSP: formatear archivo
cmd-lsp-restart = LSP: reiniciar servidores
cmd-theme-toggle = Tema: alternar claro/oscuro
cmd-config-open = Config: abrir config global
cmd-config-open-project = Config: abrir config del proyecto
cmd-config-reload = Config: recargar
cmd-view-toggle-left = Diseño: mostrar/ocultar lateral izquierda
cmd-view-toggle-right = Diseño: mostrar/ocultar lateral derecha
cmd-view-toggle-bottom = Diseño: mostrar/ocultar panel inferior
cmd-app-restart = Reiniciar el IDE
cmd-app-quit = Salir

## Avisos, diálogos e menus
file-new-created-on-save = Archivo nuevo: { $path } (se crea al guardar)
file-open-error = Error al abrir { $path }: { $error }
file-changed-on-disk-kept = "{ $name }" cambió en el disco, pero la pestaña tiene cambios: se mantuvo
file-reload-error = Error al recargar { $path }: { $error }
file-changed-on-disk-overwrite = "{ $name }" cambió en el disco, pero la pestaña tiene cambios: guardar lo sobrescribe
debug-still-building = Depuración: todavía compilando…
debug-already-running = Depuración: el programa ya se está ejecutando
debug-building = Depuración: compilando ({ $command })…
unknown-error = error desconocido
debug-build-failed = Depuración: la compilación falló: { $error }
debug-choose-title = ¿Qué programa depurar?
debug-choose-message = El proyecto generó más de un ejecutable.
cancel = Cancelar
debug-running = Depuración: ejecutando { $program }
debug-error = Depuración: { $error }
debug-cancelled = Depuración: cancelada (la compilación termina sola)
debug-stopped = Depuración: detenida
debug-no-source = Depuración: { $name } no tiene código fuente
debug-paused-no-source = Depuración: pausada en { $name } ({ $reason }), sin código fuente
debug-paused-at = Depuración: pausada en { $path }:{ $line } ({ $reason })
debug-exited = Depuración: el programa terminó
debug-exited-code = Depuración: el programa terminó (código { $code })
debug-command-failed = Depuración: { $command } falló: { $message }
debug-gdb-exited = Depuración: gdb terminó ({ $reason })
debug-no-session = Depuración: ninguna sesión en ejecución (F5 inicia)
debug-already-paused = Depuración: el programa ya está pausado
debug-pause-first = Depuración: el programa se está ejecutando (páuselo antes)
nav-nothing-forward = Nada para avanzar
nav-nothing-back = Nada para volver
lsp-no-references = No se encontraron referencias
lsp-no-definition = Definición no encontrada
lsp-no-implementations = No se encontraron implementaciones
lsp-no-type = Tipo no encontrado
lsp-not-used-elsewhere = "{ $symbol }" no se usa en otro lugar
outline-save-first = Guarde el archivo para ver la estructura
outline-reading = Leyendo la estructura…
symbols-type-a-name = Escriba el nombre de una función, tipo, método…
searching = Buscando…
symbols-none-found = No se encontraron símbolos (si el proyecto todavía se está cargando, intente de nuevo)
symbols-none-in-file = Ningún símbolo en este archivo
lsp-save-first = guarde el archivo para usar el servidor de lenguaje
lsp-finding-references = Buscando dónde se usa…
lsp-finding-implementations = Buscando implementaciones…
calls-not-in-function = El cursor no está en una función (o el servidor no conoce sus llamadas)
lsp-formatted = Archivo formateado
lsp-level-error = error:{" "}
lsp-level-warning = aviso:{" "}
lsp-server-stopped = { $server } se detuvo — { $reason }. Use "LSP: reiniciar servidores".
lsp-no-formatting = Formateo no disponible para este archivo
lsp-restarted = Servidores de lenguaje reiniciados
save-changes-title = ¿Guardar los cambios?
save-changes-message = "{ $name }" tiene cambios sin guardar.
save = Guardar
discard = Descartar
picker-already-open = Ya hay una ventana de archivos abierta
save-tab-closed = La pestaña se cerró antes de guardar
folder-opening = Abriendo { $path } en una ventana nueva
folder-open-error = No se pudo abrir la carpeta: { $error }
saved = Guardado: { $path }
save-error = Error al guardar: { $error }
file-deleted-kept = Archivo borrado en el disco; se mantuvo la pestaña con cambios
and-more = … y { $count } más
quit-unsaved-title = ¿Salir con cambios sin guardar?
quit-save-all = Guardar todo y salir
quit-without-saving = Salir sin guardar
save-needs-name = "{ $name }" no tiene nombre: use "Guardar como"
save-file-error = Error al guardar { $path }: { $error }
close-tabs-title = 
    { $count ->
        [one] ¿Cerrar 1 pestaña?
       *[other] ¿Cerrar { $count } pestañas?
    }
close-tabs-unsaved = Con cambios sin guardar:
save-and-close = Guardar y cerrar
discard-changes = Descartar cambios
tab-close = Cerrar
tab-close-others = Cerrar las demás
tab-close-right = Cerrar a la derecha
tab-close-saved = Cerrar las guardadas
tab-close-all = Cerrar todas
copy-path = Copiar ruta
copy-relative-path = Copiar ruta relativa
reveal-in-tree = Mostrar en el árbol
open-in-file-manager = Abrir en el gestor de archivos
copied = Copiado: { $text }
file-manager-error = No se pudo abrir el gestor de archivos: { $error }
save-then-close = Elija dónde guardar; después cierre la pestaña de nuevo
config-reloaded-problems = Config recargada con problemas: { $problems }
config-reloaded = Config recargada (activar/desactivar extensiones solo se aplica al reiniciar)
config-no-home = No se pudo encontrar la carpeta de config (HOME no está definido)
create-error = Error al crear { $path }: { $error }
config-error = Config: { $error }
write-error = Error al escribir { $path }: { $error }
unknown-command = Comando desconocido: { $id }
keys-record-hint = Use Ctrl, Alt o una tecla especial (F1–F12, PageUp…). Esc cancela.
keys-already-bound = { $key } ya era de "{ $command }"
keys-rebound = { $key } era de "{ $old }"; ahora es de "{ $command }"
keys-bound = { $key } → "{ $command }"
theme-unknown = tema desconocido "{ $name }", usando "dark"
default = predeterminado

## Configurações
settings-title = Configuración
settings-tab-appearance = Apariencia
settings-tab-extensions = Extensiones
settings-tab-shortcuts = Atajos
close-esc = Cerrar (Esc)
settings-no-home = (HOME no está definido)
settings-write-to = Guardar en
settings-target-global = Global
settings-target-project = Proyecto
settings-open-file = Abrir archivo
keys-filter = Filtrar por nombre, comando o tecla…
keys-press = Pulse las teclas… (Esc cancela)
keys-add = Añadir atajo
keys-help = Haga clic en una tecla para quitarla. + graba una tecla nueva (también vale para atajos ya usados: cambian de dueño).
settings-lsp = Servidores de lenguaje
settings-lsp-detail = Activa o desactiva todo el LSP.
settings-servers = Servidores
settings-server-found = encontrado en el PATH
settings-server-not-found = no encontrado en el PATH
settings-lsp-note = Los cambios valen para los servidores iniciados a partir de ahora.
settings-restart-servers = Reiniciar servidores
theme-dark = Oscuro (predeterminado)
theme-light = Claro
slot-left = Izquierda
slot-right = Derecha
slot-bottom = Abajo
slot-status = Barra de estado
settings-on-restart = se aplica al reiniciar el IDE
default-capitalized = Predeterminado
settings-extensions-note = Activar o desactivar una extensión se aplica al reiniciar. Cambiar el lado de un panel se aplica al momento.
ext-filetree = Árbol de archivos
ext-statusbar = Barra de estado
ext-search = Búsqueda en el proyecto
ext-problems = Errores y avisos
ext-terminal = Terminal integrada
ext-git = Cambios, stage, commit y diff
ext-debug = Botones, pila, variables y salida de la depuración
ext-references = Dónde se usa un nombre (Shift+F12)
ext-outline = Funciones y métodos del archivo, con visibilidad (Alt+7)
ext-preview = Vista previa de Markdown y SVG junto al editor (Ctrl+Shift+V)
ext-calls = Quién llama a una función, y a quién llama ella (Alt+Shift+H)
ext-interfaces = Interfaces que un agente escribe en .idetea/ui/ (experimental)

## Paleta, diálogos, LSP, debug e o resto do app
palette-commands = Escriba un comando…
palette-save-as = Ruta del archivo (relativa a la raíz del proyecto)
palette-files = Nombre del archivo (archivo:línea va a la línea, > para comandos)
palette-file-symbols = Función, tipo, método… de este archivo
palette-project-symbols = Función, tipo, método… del proyecto
palette-no-commands = No se encontraron comandos
palette-save-hint = Enter guarda · Esc cancela
palette-reading-files = Leyendo los archivos del proyecto…
palette-no-files = No se encontraron archivos
palette-no-symbols = Ningún símbolo
palette-no-matching-symbols = Ningún símbolo coincide con la búsqueda
dialog-hint = Enter confirma · Esc cancela
lsp-command-not-found = { $server }: comando "{ $command }" no encontrado en el PATH (instálelo o ajuste [lsp.servers.{ $server }] en la config)
lsp-start-error = { $server }: no se pudo iniciar: { $error }
lsp-disabled = LSP desactivado en la config ([lsp] enabled = false)
lsp-no-server-for = ningún servidor de lenguaje configurado para archivos .{ $ext }
lsp-not-running = { $server } no se está ejecutando: { $reason }
lsp-not-ready = { $server } todavía no está listo
lsp-hint-rustup-install = { $reason }. Instálelo con: rustup component add rust-analyzer
lsp-hint-rustup = { $reason }. Con rustup: rustup component add rust-analyzer
lsp-hint-ziex = { $reason }. Instala el CLI de Ziex (https://ziex.dev/learn) o abre la carpeta del proyecto, cuyo build.zig.zon depende de ziex: así el IDE compila el servidor con zig
config-syntax-error = sintaxis { $file }: { $error }
visibility-file-only = solo en este archivo
picker-save-as = Guardar como
picker-open-files = Abrir archivos
picker-open-folder = Abrir carpeta
picker-debug-program = Programa a depurar
splash-ready = listo
splash-loading = cargando…
watch-too-many = carpeta demasiado grande: solo se observan las primeras { $count } carpetas
watch-partial = no se pudieron observar todas las carpetas ({ $error })
watch-problem = Cambios en el disco: { $message }. Use "Actualizar" en el árbol cuando lo necesite.
watch-failed = Cambios en el disco: no se pudo observar ({ $error })
debug-no-gdb = gdb no encontrado en el PATH (la depuración usa gdb 14 o más reciente)
debug-program-not-found = programa no encontrado: { $path }
debug-gdb-start-error = no se pudo iniciar gdb: { $error }
debug-reason-breakpoint = breakpoint
debug-reason-step = paso
debug-reason-pause = pausado
debug-reason-exception = excepción
debug-reason-entry = inicio
debug-reason-goto = salto
build-run-error = no se pudo ejecutar "{ $command }": { $error }
build-no-message = falló sin mensaje

## Ajuda e linha de comando
help-key-find = Buscar en el archivo
help-key-replace = Buscar y reemplazar
help-key-goto-line = Ir a la línea
help-key-undo = Deshacer / rehacer
help-key-next-match = Seleccionar la siguiente coincidencia (multicursor)
help-key-quick-fix = Acciones rápidas del LSP (si las hay)
help-key-ctrl-click = Ctrl+Clic
help-key-definition = Ir a la definición (LSP)
help-key-tab = Insertar tabulación
help-title = Ayuda
help-tab-shortcuts = Atajos
help-tab-guide = Guía
help-tab-about = Acerca de
help-close = Cerrar (Esc)
help-filter = Filtrar por nombre, comando o tecla…
help-group-general = General
help-editor-keys = Editor (fijos)
help-in-palette = (paleta)
about-no-home = (HOME no definido)
about-version = Versión
about-license = Licencia
about-license-value = { $license } (libre y gratuita, siempre)
about-project = Proyecto
about-global-config = Config global
about-project-config = Config del proyecto
about-built-with = Hecha en Rust con iced, iced-code-editor, alacritty_terminal (vía iced_term) y syntect.
about-icons = Iconos: Codicons © Microsoft, licencia CC BY 4.0.
cli-usage = 
    Uso: idetea [opciones] [ruta...]
    
      ruta           archivo a abrir (puede no existir: se crea al guardar)
                     o carpeta a usar como raíz del proyecto
      --no-splash    no muestra la pantalla de inicio (también: IDETEA_NO_SPLASH=1)
      --menu         abierta desde el menú: sin carpeta, abre el proyecto del archivo o el último proyecto
      -f, --foreground  queda atada a la terminal (si no, abierta desde una terminal, el IDE
                     se suelta y el log va a ~/.local/state/idetea/idetea.log)
      --install-desktop    añade el IDE al menú de aplicaciones (con el icono)
      --uninstall-desktop  lo quita del menú de aplicaciones
      ui check <archivo.json>…  revisa documentos del panel Interfaces (como los lee el panel)
      new <carpeta> [--modelo id]  crea un proyecto nuevo: carpeta, git, skills del modelo e instrucciones
      skills [list|add|update|remove]  instala skills de agentes en el proyecto (idetea skills --help)
      ui fix [--apply]  muestra (y, con --apply, arregla) lo que quedó atrás en .idetea/ui/
      ask …          pregunta en el panel Interfaces y espera la respuesta (idetea ask solo: el uso)
      run … -- <comando>  pide una terminal al IDE y espera el final (idetea run solo: el uso)
      status [--json]  lo que muestra el IDE abierto en este proyecto (pestañas, cambios, cursor)
      open <archivo[:línea[:columna]]>  el IDE abierto muestra el archivo, con el cursor en su lugar
      history [--json]  lo que respondiste en el panel Interfaces (--name, --since, --last N, --all)
      -h, --help     muestra esta ayuda
      -V, --version  muestra la versión
cli-unknown-option = opción desconocida ignorada: { $option }
cli-extra-root = solo una carpeta raíz a la vez, se ignora { $path }

## Ajuda → Guia (`{ $comando-com-hifens }` vira o atalho do comando)
guide-start = Primeros pasos
guide-start-1 = Abra el IDE con `idetea` en una carpeta (se vuelve la raíz del árbol) o con `idetea archivo...` (los archivos se abren en pestañas; la raíz sigue siendo la carpeta actual).
guide-start-2 = Todos los comandos están en la paleta: { $palette-open }. Escriba parte del nombre; el atajo aparece al lado.
guide-start-3 = Esta ayuda se abre con { $help-open }.
guide-start-4 = La barra de iconos del borde izquierdo muestra y oculta los paneles con un clic (el abierto queda resaltado). Arrastre el borde entre un panel y el editor para cambiar el tamaño; doble clic vuelve al predeterminado.
guide-files = Archivos y pestañas
guide-files-1 = Abrir por nombre: { $file-quick_open } (escriba partes del nombre; `main.rs:42` va a la línea; `>` cambia a los comandos).
guide-files-2 = Ir a un símbolo (función, tipo, método…): { $nav-file_symbol } en el archivo, { $nav-project_symbol } en todo el proyecto. Necesita el servidor de lenguaje.
guide-files-3 = Nuevo: { $file-new } · Abrir: { $file-open } · Guardar: { $file-save } · Guardar como: { $file-save_as } · Cerrar pestaña: { $file-close }.
guide-files-4 = Abrir y Guardar como usan el diálogo de archivos del sistema. Para escribir la ruta: { $file-save_as_path }. Abrir otra carpeta ({ $file-open_folder }) abre una nueva ventana del IDE.
guide-files-5 = Vista previa de Markdown y SVG: { $preview-toggle } (o el botón al final de la barra de pestañas) abre un panel al lado que sigue al archivo activo y se actualiza mientras escribe. Los enlaces web se abren en el navegador; los enlaces a archivos se abren en una pestaña.
guide-files-6 = Las imágenes (png, jpg, gif, webp...) se abren en su propia pestaña: la rueda del ratón acerca y arrastrar mueve.
guide-files-7 = Los PDF también se abren en una pestaña: desplácese por las páginas; − y + cambian el zoom y "Ajustar" sigue el ancho.
guide-files-8 = Cambiar de pestaña: { $tab-next } y { $tab-prev } (Ctrl+Tab inserta una tabulación en el editor).
guide-files-9 = Cerrar una pestaña o salir con cambios pregunta antes: Guardar, Descartar o Cancelar.
guide-files-10 = Archivo cambiado desde fuera (terminal, otro editor): la pestaña se recarga sola si no tiene cambios; si los tiene, queda como está y aparece un aviso.
guide-files-11 = Clic derecho en una pestaña: cerrar las otras, las de la derecha, las guardadas o todas, copiar ruta, mostrar en el árbol y abrir en el gestor de archivos. El clic central cierra la pestaña.
guide-tree = Árbol de archivos
guide-tree-1 = Mostrar/ocultar: { $filetree-toggle } · Mostrar el archivo actual: { $filetree-reveal }.
guide-tree-2 = Clic derecho en un elemento: nuevo archivo, nueva carpeta, renombrar, eliminar, copiar ruta y abrir en el gestor de archivos. Eliminar lo envía a la papelera del sistema.
guide-tree-3 = Colores de git: amarillo = modificado, verde = nuevo, rojo = conflicto, atenuado = ignorado. Una carpeta con cambios dentro muestra un punto.
guide-bottom = Panel inferior
guide-bottom-1 = Búsqueda en el proyecto ({ $search-focus }), Problemas ({ $problems-focus }), Terminal ({ $terminal-focus }) y Git ({ $git-focus }) son pestañas del panel inferior. { $view-toggle_bottom } muestra/oculta el panel.
guide-bottom-2 = Siguiente problema: { $problems-next }.
guide-terminal = Terminal
guide-terminal-1 = Con el foco en la terminal, Ctrl+letra, las teclas F y las flechas van al shell (Ctrl+C, Ctrl+W...). Los atajos del IDE con Ctrl+Shift siguen funcionando, así que la paleta siempre se abre.
guide-terminal-2 = Copiar y pegar en la terminal: Ctrl+Shift+C y Ctrl+Shift+V. Escribir `exit` cierra el panel.
guide-debug = Depuración
guide-debug-1 = Breakpoint: clic en el número de línea o { $debug-toggle_breakpoint }. Iniciar/continuar: { $debug-continue }. En proyectos Rust compila con cargo build y encuentra el ejecutable solo; en los demás, use [debug] en la config (program, args, cwd, build) o elija el ejecutable. Cambiar el programa: { $debug-choose_program }.
guide-debug-2 = Detenido: { $debug-next } siguiente línea, { $debug-step_in } entra en la función, { $debug-step_out } sale, { $debug-pause } pausa, { $debug-stop } detiene. La flecha amarilla muestra dónde está el programa. Usa gdb 14+.
guide-debug-3 = Panel de depuración ({ $debug-focus }): botones, pila de llamadas (clic va al marco y muestra sus variables), variables (clic abre structs y vectores) y la salida del programa y de la compilación.
guide-git = Git
guide-git-1 = La rama aparece en la barra de estado. El panel Git ({ $git-focus }) muestra lo que cambió: + y − agregan y quitan del stage, clic en un archivo muestra el diff, Enter en el mensaje hace el commit.
guide-git-2 = Clic derecho en un archivo de Git: abrir, ver diff, stage, descartar cambios (pide confirmación) y copiar ruta.
guide-git-3 = Fetch ({ $git-fetch }), pull ({ $git-pull }) y push ({ $git-push }) están en los botones de arriba del panel. La contraseña no se pide aquí: use el agente ssh (ssh-add) o un credential helper. Una rama nueva pregunta antes de publicarse en el remoto.
guide-git-4 = El panel se actualiza al guardar y cuando el repositorio cambia (commit o checkout hechos en la terminal).
guide-lsp = Lenguajes (LSP)
guide-lsp-1 = Los servidores de lenguaje se inician solos al abrir un archivo: rust-analyzer, clangd, pyright, typescript-language-server, gopls y lua-language-server (deben estar instalados).
guide-lsp-2 = Ir a la definición: { $lsp-definition } o Ctrl+Clic · Documentación: { $lsp-hover } o el ratón quieto sobre el código · Formatear: { $lsp-format }.
guide-lsp-3 = Dónde se usa: { $lsp-references }, o Ctrl+Clic en la propia declaración. La lista aparece en el panel Referencias; el clic lleva al uso.
guide-lsp-4 = Implementaciones: { $lsp-implementation } (en un trait o interfaz, o en uno de sus métodos). Tipo de lo que está en el cursor: "LSP: ir al tipo" en la paleta. Con más de un resultado (incluso en la definición), la lista va al panel Referencias; con uno solo, salta directo.
guide-lsp-5 = Quién llama a una función: { $lsp-incoming_calls } con el cursor en ella. El panel Llamadas muestra el árbol (abra un nivel para ver quién llama a quien la llama); "Qué llama" cambia a las llamadas que ella hace. El clic lleva a la llamada.
guide-lsp-6 = Atrás y adelante después de un salto (definición, referencia, abrir por nombre, clics en los paneles): { $nav-back } y { $nav-forward }, o los botones laterales del ratón.
guide-lsp-7 = Estructura del archivo ({ $outline-toggle }): funciones, structs, métodos y campos en árbol. El candado abierto es público, el cerrado es privado y la llave es visible solo en parte (pub(crate), protected, paquete). El símbolo donde está el cursor queda resaltado; el clic lleva a él.
guide-lsp-8 = Si un servidor falla, la barra de estado muestra el motivo; "LSP: reiniciar servidores" lo intenta de nuevo.
guide-config = Configuración
guide-config-1 = La pantalla de configuración ({ $settings-open }) cambia tema, fuente, extensiones, atajos y LSP, y guarda en la config global o en la del proyecto. Los cambios se aplican al momento.
guide-config-2 = Hay una config global y una por proyecto (`.idetea/config.toml`); la del proyecto gana. Para editar el archivo directamente: "Config: abrir config global" o "Config: abrir config del proyecto". Al guardar, todo se recarga.
guide-config-3 = Los atajos están en [keys]: "ctrl+alt+t" = "terminal.focus". Un texto vacío desactiva un atajo predeterminado. Los ids de los comandos aparecen en la pestaña Atajos.
guide-config-4 = Idioma: Configuración → Apariencia → Idioma (Automático sigue el sistema). Se aplica al reiniciar el IDE.

## Sessão
settings-restore-session = Reabrir como estaba
settings-restore-session-detail = Al abrir un proyecto, vuelven las pestañas, el cursor y los paneles. idetea archivo... abre solo el archivo.

## Recentes
cmd-file-open-recent = Archivo: abrir reciente…
palette-recent = Proyecto o archivo abierto antes
palette-no-recent = Nada abierto antes todavía
guide-files-12 = Abrir reciente: { $file-open_recent } lista los proyectos (se abren en una ventana nueva) y los archivos abiertos por última vez, de todas las ventanas.

## Recuperar não salvos
untitled-tab = sin título
recover-title = 
    { $count ->
        [one] El IDE se cerró sin guardar 1 archivo
       *[other] El IDE se cerró sin guardar { $count } archivos
    }
recover-detail = Recuperar abre los cambios en pestañas, aún sin guardar (deshacer vuelve a lo que está en el disco). Después pregunta de nuevo la próxima vez.
recover = Recuperar
recover-later = Después
recovered = 
    { $count ->
        [one] 1 archivo recuperado (aún sin guardar)
       *[other] { $count } archivos recuperados (aún sin guardar)
    }

## Renomear símbolo e salvar todos
cmd-lsp-rename = LSP: renombrar símbolo…
cmd-file-save-all = Archivo: guardar todos
save-all-untitled = Las pestañas sin nombre no se guardaron (use Guardar como)
palette-rename = Nuevo nombre
palette-rename-hint = Renombrar "{ $name }" en todo el proyecto · Enter muestra lo que cambia · Esc cancela
rename-no-word = Ponga el cursor en un nombre para renombrarlo
rename-asking = Renombrar "{ $name }": preguntando al servidor…
rename-nothing = Nada que renombrar en "{ $name }"
rename-failed = No se pudo renombrar: { $message }
rename-title = ¿Renombrar "{ $old }" a "{ $new }"?
rename-summary = 
    { $changes ->
        [one] 1 cambio
       *[other] { $changes } cambios
    } en { $files ->
        [one] 1 archivo:
       *[other] { $files } archivos:
    }
rename-detail = Los archivos quedan cambiados en sus pestañas, aún sin guardar: Ctrl+Z deshace en cada uno, y "Archivo: guardar todos" guarda todo.
rename = Renombrar
renamed = 
    { $count ->
        [one] Renombrado en 1 archivo (aún sin guardar)
       *[other] Renombrado en { $count } archivos (aún sin guardar)
    }

guide-lsp-9 = Renombrar: { $lsp-rename } con el cursor en el nombre. Una vista previa muestra cuántos cambios y en qué archivos; al confirmar, quedan cambiados en sus pestañas (Ctrl+Z deshace en cada uno) y { $file-save_all } los guarda todos.

## Substituir no projeto
replaced = { $count } reemplazo(s) en { $files } archivo(s)
replaced-failed = { $count } reemplazo(s) en { $files } archivo(s); { $failed } no se pudieron cambiar (sin permiso o no es texto)

guide-bottom-3 = Reemplazar en el proyecto: { $search-replace } (o ▸ junto a la búsqueda) abre "Reemplazar por". Cada resultado muestra cómo queda; "Reemplazar" cambia la línea o el archivo, y "Reemplazar todo" pide confirmación. Los archivos abiertos en pestañas quedan cambiados sin guardar (Ctrl+Z deshace); los demás se guardan directo en el disco.

## Editor e salvar
format-save-timeout = El servidor no formateó a tiempo: guardado sin formatear

settings-tab-editor = Editor
settings-wrap = Ajustar líneas largas
settings-wrap-detail = Una línea más ancha que la ventana sigue debajo, sin desplazamiento lateral.
settings-whitespace = Mostrar espacios
settings-whitespace-detail = Los espacios se ven como · y los tabs como →.
settings-indent-guides = Guías de sangría
settings-indent-guides-detail = Líneas verticales finas en cada nivel de sangría.
settings-tab-size = Tamaño de la sangría
settings-tab-size-detail = Cuántos espacios inserta Tab.
settings-insert-spaces = Sangrar con espacios
settings-insert-spaces-detail = Desactivado: Tab inserta un carácter tab. Un archivo que ya usa tabs sigue con tabs.
settings-trim = Quitar espacios al final de las líneas al guardar
settings-trim-detail = Salvo en Markdown, donde dos espacios al final cortan la línea.
settings-final-newline = Salto de línea al final del archivo al guardar
settings-final-newline-detail = Asegura que la última línea termine con un salto de línea.
settings-format-on-save = Formatear al guardar
settings-format-on-save-detail = Ctrl+S formatea con el servidor de lenguaje antes de guardar (espera hasta 2 s). Guardar todos no formatea.
settings-breadcrumbs = Breadcrumbs
settings-breadcrumbs-detail = Encima del editor: la ruta del archivo y el símbolo donde está el cursor. Clic en una carpeta para mostrarla en el árbol, en un símbolo para ir a él.
settings-editorconfig-note = Un .editorconfig en el proyecto prevalece sobre estas opciones (sangría, tabs o espacios, limpieza al guardar).

guide-config-5 = Editor: la pestaña Editor de la configuración ({ $settings-open }) tiene ajuste de línea, espacios visibles, guías de sangría, tamaño de la sangría y qué hacer al guardar (quitar espacios al final, salto de línea final, formatear). Un .editorconfig en el proyecto prevalece; sin él, un archivo que ya usa tabs sigue con tabs.

guide-lsp-10 = Pistas de parámetros: al escribir ( en una llamada, la firma de la función aparece encima del cursor, con el parámetro actual resaltado; avanza con cada coma y desaparece al cerrar el paréntesis o con Esc.

guide-git-5 = Marcas en el margen: a la izquierda de los números de línea, verde = línea nueva, azul = cambiada, marca roja = líneas borradas, comparando con el último commit (sigue lo que escribe, sin guardar). Al hacer clic en la marca se ve cómo era, con "Deshacer cambio" (Ctrl+Z lo trae de vuelta).

palette-no-items = Nada coincide con la búsqueda

guide-git-6 = Ramas: { $git-checkout } (o haga clic en el nombre de la rama, en la barra de estado o en el panel Git) lista las ramas, la más reciente primero; las remotas que aún no existen aquí van después y, al elegirlas, se vuelven locales. Escriba un nombre nuevo para crear la rama desde donde está. Los cambios sin commit van con usted; si chocan con la otra rama, git se niega y lo explica.

guide-git-7 = Autor de la línea: en la barra de estado, junto a la rama, aparece quién cambió por última vez la línea del cursor, cuándo y el mensaje del commit (haga clic para ver el hash y copiarlo). Con la pestaña cambiada y sin guardar, desaparece hasta guardar (las líneas del disco pueden no coincidir).

## Tarefas
cmd-tasks-run = Tareas: ejecutar…
cmd-tasks-rerun = Tareas: ejecutar la última otra vez
cmd-tasks-test-here = Rust: ejecutar la prueba en el cursor
tasks-placeholder = Ejecutar tarea…
tasks-none = Ninguna tarea: cree una en [tasks] en la configuración, o abra un proyecto con Cargo.toml, package.json o Makefile.
tasks-no-terminal = Las tareas necesitan la extensión del terminal, que está desactivada.
tasks-no-test = El cursor no está en una prueba Rust (#[test] fn).
tasks-no-package = No encontré el Cargo.toml del paquete de este archivo.
tasks-done = [{ $name } terminó: código { $code } · Enter cierra]

guide-terminal-3 = Tareas: { $tasks-run } lista los comandos del proyecto ([tasks] en la configuración, cargo build/run/test/clippy, scripts de package.json, objetivos del Makefile) y ejecuta el elegido en una pestaña "Tarea" del terminal, sin tocar el shell. Al final aparece el código de salida; Enter la cierra. Ctrl+C detiene solo el comando. { $tasks-rerun } ejecuta la última otra vez.
guide-terminal-4 = Prueba en el cursor (Rust): con el cursor dentro de un #[test] fn, { $tasks-test_here } ejecuta solo esa prueba (cargo test -p paquete nombre). Sin atajo por defecto: cree uno en [keys] si la usa mucho.

## Editor dividido
cmd-editor-split = Editor: mover la pestaña al otro lado (dividir)
cmd-editor-focus-left = Editor: ir al lado izquierdo
cmd-editor-focus-right = Editor: ir al lado derecho
split-one-tab = Abra otro archivo para dividir el editor: cada lado muestra una pestaña distinta.
tab-move-right = Mover a la derecha (dividir)
tab-move-left = Mover a la izquierda
guide-files-13 = Editor dividido: { $editor-split } (o "Mover a la derecha" en el menú de la pestaña) pone la pestaña al lado, con sus propias pestañas; abrir un archivo va al lado con foco. Haga clic en un lado (o { $editor-focus_left } / { $editor-focus_right }) para trabajar en él; arrastre la división para cambiar el ancho (doble clic la centra). Mover la última pestaña de un lado de vuelta lo junta todo.

## Menu de aplicativos (--install-desktop)
desktop-no-home = No encontré la carpeta personal (HOME).
desktop-error = No se pudo instalar el acceso directo: { $error }
desktop-installed = 
    IDE TEA en el menú de aplicaciones: { $desktop }
    Abre: { $exe }
desktop-removed = IDE TEA quitado del menú de aplicaciones.

## Arquivos grandes
file-light-mode = "{ $name }" tiene una línea muy larga y se abrió en modo ligero: sin colores, sin servidor de lenguaje y con ajuste de línea
lsp-light-mode = archivo en modo ligero (línea muy larga): sin servidor de lenguaje

## Executável recompilado
binary-changed = El IDE fue recompilado: ejecute "Reiniciar el IDE" (paleta) para usar la versión nueva.
binary-changed-title = [versión nueva: reinicie]
binary-restart-error = No se pudo reiniciar: { $error }

## idetea status / idetea open (RFC 0006)
status-usage =
    Uso: idetea status [--json]
    
    Lo que muestra el IDE abierto en este proyecto: la pestaña activa con el cursor y los archivos
    abiertos (con "sin guardar" en los que tienen cambios). --json: lo mismo en JSON, con "open".
    Códigos: 0 abierto, 2 argumento inválido, 3 el IDE no está abierto en este proyecto.
status-bad-arg = argumento desconocido: { $arg }
status-not-open = el IDE no está abierto en este proyecto ({ $root })
status-active = activo: { $path }
status-no-active = ningún archivo activo
status-modified = {" "}(sin guardar)
open-usage =
    Uso: idetea open <archivo[:línea[:columna]]>
    
    El IDE abierto en este proyecto muestra el archivo, con el cursor en la línea y la columna.
    Códigos: 0 mostrado, 2 ruta inválida o fuera del proyecto, 3 el IDE no está abierto (o no tomó el pedido).
open-outside = { $path } no está dentro del proyecto ({ $root })
open-write-error = no se pudo escribir { $path }
open-not-taken = el IDE no tomó el pedido en 3 segundos
