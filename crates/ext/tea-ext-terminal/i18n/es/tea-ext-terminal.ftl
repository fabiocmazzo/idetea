## Terminal
terminal-start-error = Terminal: no se pudo iniciar el shell: { $error }
cmd-terminal-focus = Terminal: abrir y enfocar
cmd-terminal-new = Terminal: nueva terminal
cmd-terminal-close = Terminal: cerrar la terminal
cmd-terminal-restart = Terminal: reiniciar el shell
terminal-ended-n = Terminal { $number } cerrada
terminal-not-running = El shell no se está ejecutando.
terminal-start = Iniciar terminal

## Tarefas
terminal-task-error = Tarea: no se pudo iniciar: { $error }
terminal-tab-n = Terminal { $number }
terminal-close = Cerrar la terminal
terminal-tab-task = Tarea: { $name }
terminal-close-task = Cerrar tarea

## Terminais pedidos por agentes
terminal-tab-agent = Agente: { $title }
terminal-tab-request = { $count ->
    [one] Pedido de terminal
   *[other] { $count } pedidos de terminal
}
agent-request-notify = Un agente pide una terminal: { $title }. Míralo en el panel de la terminal.
agent-request-auto = Pedido de terminal "{ $title }" ejecutándose sin confirmar: está en el [agents] allow de tu config ("{ $entry }").
agent-request-someone = Un agente
agent-request-title = { $who } pide una terminal: { $title }
agent-request-command = Comando (ejecuta exactamente esto):
agent-request-cwd = En la carpeta: { $cwd }
agent-request-worktree = Se ejecuta en su propio worktree: rama { $branch }, en { $path } (creada desde HEAD si la rama no existe)
agent-worktree-kept = El worktree { $path } se quedó: { $error }
agent-request-warning = Se ejecuta con tus permisos, como si lo escribieras tú. Permite solo lo que reconoces.
agent-request-allow = Permitir y ejecutar
agent-request-refuse = Rechazar
agent-request-more = { $count ->
    [one] Y un pedido más después de este.
   *[other] Y { $count } pedidos más después de este.
}
agent-done = Terminó con código { $code }. Enter cierra la pestaña.
run-bad-arg = idetea run: "{ $arg }" no vale.
run-usage = Uso: idetea run [--title T] [--name nombre] [--cwd carpeta] [--worktree rama] [--keep] [--no-wait] [--timeout segundos] -- <comando>; idetea run --wait <nombre>; idetea run --stop <nombre>
run-busy = idetea run: ya hay un pedido "{ $name }" (o su estado); elige otro --name o espera con idetea run --wait { $name }
run-asked = idetea run: pedido "{ $name }" en el panel de la terminal; esperando que el usuario lo permita y que el comando termine
run-exited = idetea run: "{ $name }" terminó con código { $code }
run-refused = idetea run: el usuario rechazó "{ $name }"
run-invalid = idetea run: el pedido "{ $name }" no es válido: { $error }
run-ended = idetea run: "{ $name }" terminó sin código ({ $state })
run-branch = idetea run: el trabajo quedó en la rama { $branch }
run-timeout = idetea run: se acabó el tiempo y "{ $name }" sigue en la terminal (idetea run --wait { $name } vuelve a esperar; --stop lo termina)

## Estado de las pestañas (RFC 0024)
terminal-state-running = En ejecución
terminal-state-unseen = Salida nueva desde la última vez que lo viste
terminal-state-attention = Pidió tu atención
terminal-state-exited = Terminó con el código { $code }
terminal-state-bell = Sonó la campana
terminal-state-turn-done = Terminó su turno y te espera
cmd-terminal-next = Terminal: pestaña siguiente
cmd-terminal-prev = Terminal: pestaña anterior
cmd-terminal-goto = Terminal: ir a…
cmd-terminal-next-attention = Terminal: siguiente pestaña que te espera
cmd-terminal-rename = Terminal: renombrar la pestaña
terminal-no-attention = Ninguna pestaña de terminal pide atención.
terminal-goto-placeholder = Ir a la pestaña de terminal…
terminal-rename-placeholder = Nombre de la pestaña
terminal-rename-reset = Volver al nombre predeterminado
terminal-rename-to = Renombrar a
terminal-close-running-title = ¿Cerrar la pestaña?
terminal-close-running = { $name } sigue en ejecución. Cerrar la pestaña detiene el proceso.
terminal-close-running-yes = Cerrar y detener
terminal-close-running-no = Cancelar
cmd-terminal-find = Terminal: buscar
terminal-find-placeholder = Buscar en la terminal (Enter: más antigua)
terminal-find-older = Más antigua (arriba)
terminal-find-newer = Más nueva (abajo)
terminal-find-none = Sin resultados
terminal-find-close = Cerrar la búsqueda
cmd-terminal-split = Terminal: dividir (o unir)
cmd-terminal-resume = Terminal: retomar los agentes de la sesión anterior
terminal-resume-nothing = Ningún agente de la sesión anterior en espera.
terminal-resume-title = ¿Retomar los agentes?
terminal-resume-message = { $count ->
    [one] Un agente estaba abierto cuando se cerró el IDE: { $names }.
   *[other] { $count } agentes estaban abiertos cuando se cerró el IDE: { $names }.
}
terminal-resume-all = Retomar todos
terminal-resume-pick = Elegir…
terminal-resume-none = No
terminal-resume-pick-placeholder = Agente para retomar…
