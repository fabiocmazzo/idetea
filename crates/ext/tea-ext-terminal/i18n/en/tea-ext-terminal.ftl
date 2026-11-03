## Terminal
terminal-start-error = Terminal: could not start the shell: { $error }
cmd-terminal-focus = Terminal: open and focus
cmd-terminal-new = Terminal: new terminal
cmd-terminal-close = Terminal: close the terminal
cmd-terminal-restart = Terminal: restart the shell
terminal-ended-n = Terminal { $number } closed
terminal-not-running = The shell is not running.
terminal-start = Start terminal

## Tarefas
terminal-task-error = Task: could not start: { $error }
terminal-tab-n = Terminal { $number }
terminal-close = Close the terminal
terminal-tab-task = Task: { $name }
terminal-close-task = Close task

## Terminais pedidos por agentes
terminal-tab-agent = Agent: { $title }
terminal-tab-request = { $count ->
    [one] Terminal request
   *[other] { $count } terminal requests
}
agent-request-notify = An agent asks for a terminal: { $title }. See the terminal panel.
agent-request-auto = Terminal request "{ $title }" running without confirmation: it is in your config's [agents] allow ("{ $entry }").
agent-request-someone = An agent
agent-request-title = { $who } asks for a terminal: { $title }
agent-request-command = Command (runs exactly this):
agent-request-cwd = In the folder: { $cwd }
agent-request-warning = It runs with your permissions, as if you typed it. Only allow what you recognize.
agent-request-allow = Allow and run
agent-request-refuse = Refuse
agent-request-more = { $count ->
    [one] One more request after this one.
   *[other] { $count } more requests after this one.
}
agent-done = Finished with code { $code }. Enter closes the tab.
