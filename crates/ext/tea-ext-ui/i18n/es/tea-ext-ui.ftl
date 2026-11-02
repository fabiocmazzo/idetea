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
ui-write-failed = No se pudo guardar la respuesta: { $error }
ui-prepare-failed = No se pudo preparar la carpeta .idetea/ui: { $error }
ui-not-a-file = no es un archivo común
ui-surface-mismatch = el campo surface es "{ $found }", pero el archivo se llama { $expected }.json
