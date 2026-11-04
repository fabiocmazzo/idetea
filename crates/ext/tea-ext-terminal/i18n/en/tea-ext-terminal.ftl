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
agent-request-worktree = Runs in its own worktree: branch { $branch }, in { $path } (created from HEAD if the branch does not exist)
agent-worktree-kept = The worktree { $path } was kept: { $error }
agent-request-warning = It runs with your permissions, as if you typed it. Only allow what you recognize.
agent-request-allow = Allow and run
agent-request-refuse = Refuse
agent-request-more = { $count ->
    [one] One more request after this one.
   *[other] { $count } more requests after this one.
}
agent-done = Finished with code { $code }. Enter closes the tab.
run-bad-arg = idetea run: "{ $arg }" is not valid.
run-usage = Usage: idetea run [--title T] [--name name] [--cwd folder] [--worktree branch] [--keep] [--no-wait] [--timeout seconds] -- <command>; idetea run --wait <name>; idetea run --stop <name>
run-busy = idetea run: there is already a request "{ $name }" (or its status); pick another --name or wait with idetea run --wait { $name }
run-asked = idetea run: request "{ $name }" is in the terminal panel; waiting for the user to allow it and for the command to end
run-exited = idetea run: "{ $name }" ended with code { $code }
run-refused = idetea run: the user refused "{ $name }"
run-invalid = idetea run: request "{ $name }" is invalid: { $error }
run-ended = idetea run: "{ $name }" ended without a code ({ $state })
run-branch = idetea run: the work is on branch { $branch }
run-timeout = idetea run: time is up and "{ $name }" is still in the terminal (idetea run --wait { $name } waits again; --stop ends it)
