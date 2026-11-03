ui-panel = Interfaces
cmd-ui-show = Interfaces: mostrar el panel
ui-empty = Ninguna interfaz abierta. Un agente puede crear una escribiendo un archivo en .idetea/ui/ (ver docs/interfaces/formato.md). Para activarla o desactivarla: [extensions] interfaces = {"{"} enabled = true {"}"} en la configuración.
ui-tracked = { $path } está versionado en git y se ignoró por seguridad: las interfaces solo vienen del agente que corre en esta máquina, nunca de un repositorio clonado.
ui-invalid = Documento inválido: { $error }
ui-origin = { $path }
ui-origin-source = { $path } · { $source }
ui-waiting = { $count ->
    [one] Respuesta enviada; esperando que el agente la lea.
   *[other] { $count } respuestas enviadas; esperando que el agente las lea.
}
ui-received = Respuesta recibida por el agente.
ui-agent-waiting = El agente está esperando la respuesta.
ui-agent-stopped = { $minutes ->
    [0] El agente acaba de dejar de esperar: la respuesta queda guardada hasta que vuelva a mirar.
   *[other] El agente dejó de esperar hace { $minutes } min: la respuesta queda guardada hasta que vuelva a mirar.
}
ui-new = Nueva interfaz del agente: { $title }
ui-updated = Interfaz actualizada: { $title }
ui-open-later = El agente pidió abrir { $path }: está en el panel Interfaces (no se abrió para no interrumpir la escritura).
ui-write-failed = No se pudo guardar la respuesta: { $error }
ui-prepare-failed = No se pudo preparar la carpeta .idetea/ui: { $error }
ui-not-a-file = no es un archivo común
ui-surface-mismatch = el campo surface es "{ $found }", pero el archivo se llama { $expected }.json
cmd-ui-teach = Interfaces: enseñar a los agentes de este proyecto
ui-teach-hint = Para que un agente de código sepa usar este panel, guarda las instrucciones en el AGENTS.md del proyecto.
ui-teach-button = Enseñar a los agentes de este proyecto
ui-teach-title = ¿Enseñar a los agentes de este proyecto?
ui-teach-detail = El IDE va a guardar las instrucciones del panel Interfaces en: { $files }. El texto queda entre marcadores; ejecutarlo de nuevo solo actualiza esa parte y no toca el resto. Son archivos del proyecto: revísalos antes del commit.
ui-teach-file-new = { $name } (nuevo)
ui-teach-write = Guardar
ui-cancel = Cancelar
ui-teach-up-to-date = Las instrucciones para agentes ya están actualizadas.
ui-teach-done = Instrucciones guardadas en { $files }.
ui-teach-failed = No se pudieron guardar las instrucciones: { $error }
cmd-ui-new-project = Proyecto: nuevo…
new-title = Nuevo proyecto
new-intro = El IDE crea la carpeta, git, las skills del modelo y las instrucciones del panel, y abre el proyecto en una ventana nueva. Quien crea el proyecto de verdad es tu agente: el pedido inicial queda en **AGENTS.md**; abre el agente allí y pídele que empiece.
new-field-parent = Crear dentro de
new-field-name = Nombre de la carpeta
new-field-name-hint = p. ej.: mi-app
new-field-model = Modelo
new-field-description = Qué es el proyecto (opcional)
new-field-description-hint = una frase; va en el pedido inicial para el agente
new-field-git = Crear el repositorio git
new-create = Crear y abrir
new-model = { $model ->
    [vazio] Vacío (el agente pregunta qué es)
    [rust] Rust
    [site] Sitio web
    [python] Python
    [fastapi] FastAPI (API web en Python)
   *[other] { $model }
}
new-creating = Creando { $dir } (las skills vienen por la red)…
new-created = Proyecto creado en { $dir }; abriéndolo en una ventana nueva.
new-open-failed = Proyecto creado en { $dir }, pero no se pudo abrir la ventana: { $error }
new-failed = No se pudo crear el proyecto: { $error }
new-bad-model = modelo desconocido: { $model }
new-bad-dir = carpeta no válida: { $dir } (usa una ruta completa)
new-bad-name = nombre de carpeta no válido: "{ $name }"
new-not-empty = { $dir } ya existe y tiene archivos: elige otro nombre
new-done-dir = carpeta: { $dir }
new-done-git = git: repositorio creado
new-done-agents = AGENTS.md (con el pedido inicial) y CLAUDE.md, con las instrucciones del panel
new-open-hint = Listo. Ábrelo con: idetea { $dir }
new-usage = idetea new: "{ $arg }" no vale. Uso: idetea new <carpeta> [--modelo id] [--descricao texto] [--sem-git]; modelos: { $models }
cmd-ui-fix = Interfaces: arreglar este proyecto
ui-fix-title = ¿Arreglar este proyecto?
ui-fix-will = El IDE va a:
ui-fix-wont = Solo para que lo sepas (no lo toco):
ui-fix-button = Arreglar
ui-fix-nothing = Nada que arreglar en las interfaces de este proyecto.
ui-fix-done = { $count ->
    [one] Arreglado: 1 elemento.
   *[other] Arreglado: { $count } elementos.
}
ui-fix-failed = No se pudo arreglar: { $error }
ui-fix-hint = Para arreglar los elementos con "-": idetea ui fix --apply
ui-fix-usage = idetea ui fix: argumento desconocido "{ $arg }" (uso: idetea ui fix [--apply])
ui-fix-abandoned = borrar { $path }: nadie espera la respuesta desde hace { $since }
ui-fix-unread = { $path }: { $count ->
    [one] 1 respuesta que el agente no leyó
   *[other] { $count } respuestas que el agente no leyó
} (se guarda; pide al agente que la lea, o bórrala a mano)
ui-fix-leftover = borrar { $path }: sobró de una escritura o espera que no terminó
ui-fix-check = reescribir .idetea/ui/check: llama a { $target }, que ya no existe
ui-fix-instructions-older = reescribir las instrucciones de { $file }: están en la versión { $found }, más vieja que la de este IDE
ui-fix-instructions-unversioned = reescribir las instrucciones de { $file }: son de un IDE antiguo, sin la línea de versión
ui-fix-instructions-newer = { $file } tiene instrucciones de un IDE más nuevo ({ $found }): abre el proyecto en él (no vuelvo atrás)
ui-outdated = Este IDE está desactualizado (fue recompilado después de abrir): las respuestas avisan al agente. Paleta: "Reiniciar el IDE".
ui-audio-failed = No se pudo reproducir el sonido: { $error }
ui-check-ok = { $file }: ok ({ $count ->
        [one] 1 componente
       *[other] { $count } componentes
    })
ui-check-error = { $file }: error: { $error }
ui-check-warning = aviso: { $warning }
ui-check-media = medios: { $path }: { $problem }
ui-check-name = el nombre del archivo no vale ("{ $name }"): minúsculas, dígitos, _ y -, terminando en .json
ui-check-usage = uso: idetea ui check <archivo.json>…
ui-check-instructions-older = instrucciones: { $file } tiene la versión { $found } y este IDE tiene la { $ours }: pide al usuario que ejecute "Interfaces: enseñar a los agentes de este proyecto" y vuelve a leer el archivo antes de publicar
ui-check-instructions-newer = instrucciones: { $file } tiene la versión { $found }, de un IDE más nuevo que este ({ $ours }): el panel abierto puede no entender lo que enseña el archivo
ui-check-instructions-unversioned = instrucciones: { $file } tiene un bloque antiguo, sin la línea de versión; este IDE tiene la { $ours }: pide al usuario que ejecute "Interfaces: enseñar a los agentes de este proyecto" y vuelve a leer el archivo antes de publicar
ui-teach-newer-file = { $name } (versión { $version })
ui-teach-newer = Atención: { $files } fue escrito por un IDE más nuevo que este (que tiene la versión { $ours } y no conoce aquella). Escribir devuelve las instrucciones a la versión de este IDE; lo normal es abrir el proyecto con el IDE nuevo.
ui-teach-write-anyway = Escribir de todos modos
