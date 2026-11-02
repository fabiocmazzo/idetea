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
ui-teach-newer-file = { $name } (versión { $version })
ui-teach-newer = Atención: { $files } fue escrito por un IDE más nuevo que este (que tiene la versión { $ours } y no conoce aquella). Escribir devuelve las instrucciones a la versión de este IDE; lo normal es abrir el proyecto con el IDE nuevo.
ui-teach-write-anyway = Escribir de todos modos
