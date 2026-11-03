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
agent-request-warning = Se ejecuta con tus permisos, como si lo escribieras tú. Permite solo lo que reconoces.
agent-request-allow = Permitir y ejecutar
agent-request-refuse = Rechazar
agent-request-more = { $count ->
    [one] Y un pedido más después de este.
   *[other] Y { $count } pedidos más después de este.
}
agent-done = Terminó con código { $code }. Enter cierra la pestaña.
