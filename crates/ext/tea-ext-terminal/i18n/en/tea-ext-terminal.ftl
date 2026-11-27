## Terminal
terminal-start-error = Terminal: could not start the shell: { $error }
cmd-terminal-focus = Terminal: open and focus
cmd-terminal-new = Terminal: new terminal
cmd-terminal-close = Terminal: close the terminal
cmd-terminal-restart = Terminal: restart the shell
terminal-ended-n = Terminal { $number } closed
terminal-link-copied = Address copied (this kind of link does not open from the terminal): { $url }
terminal-link-failed = Could not open { $url }: { $error }
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
agent-request-batch = { $count } terminal requests are waiting. Check each command: "Allow all" runs exactly the ones on this screen, each in its own tab.
agent-request-allow-all = Allow all ({ $count })
agent-request-refuse-all = Refuse all
agent-done = Finished with code { $code }. Enter closes the tab.
run-bad-arg = idetea run: "{ $arg }" is not valid.
run-usage = Usage: idetea run [--title T] [--name name] [--cwd folder] [--worktree branch] [--keep] [--no-wait] [--timeout seconds] -- <command>; idetea run --wait <name>...; idetea run --stop <name>
run-busy = idetea run: there is already a request "{ $name }" (or its status); pick another --name or wait with idetea run --wait { $name }
run-asked = idetea run: request "{ $name }" is in the terminal panel; waiting for the user to allow it and for the command to end
run-exited = idetea run: "{ $name }" ended with code { $code }
run-refused = idetea run: the user refused "{ $name }"
run-invalid = idetea run: request "{ $name }" is invalid: { $error }
run-ended = idetea run: "{ $name }" ended without a code ({ $state })
run-branch = idetea run: the work is on branch { $branch }
run-timeout = idetea run: time is up and "{ $name }" is still in the terminal (idetea run --wait { $name } waits again; --stop ends it)

## Tab state (RFC 0024)
terminal-state-running = Running
terminal-state-unseen = New output since you last looked
terminal-state-attention = Asked for your attention (permission or an answer)
terminal-state-exited = Ended with code { $code }
terminal-state-bell = Rang the bell
terminal-state-turn-done = Finished its turn and is waiting for you
cmd-terminal-next = Terminal: next tab
cmd-terminal-prev = Terminal: previous tab
cmd-terminal-goto = Terminal: go to…
cmd-terminal-next-attention = Terminal: next tab waiting for you
cmd-terminal-rename = Terminal: rename tab
terminal-no-attention = No terminal tab needs attention.
terminal-goto-placeholder = Go to terminal tab…
terminal-rename-placeholder = Tab name
terminal-rename-reset = Back to the default name
terminal-rename-to = Rename to
terminal-close-running-title = Close the tab?
terminal-close-running = { $name } is still running. Closing the tab stops the process.
terminal-close-running-yes = Close and stop
terminal-close-running-no = Cancel
cmd-terminal-find = Terminal: find
terminal-find-placeholder = Find in terminal (Enter: older)
terminal-find-older = Older (up)
terminal-find-newer = Newer (down)
terminal-find-none = No matches
terminal-find-close = Close find
cmd-terminal-split = Terminal: split (or unsplit)
cmd-terminal-resume = Terminal: resume agents from the last session
terminal-resume-nothing = No agents from the last session waiting.
terminal-resume-title = Resume the agents?
terminal-resume-message = { $count ->
    [one] One agent was open when the IDE closed: { $names }.
   *[other] { $count } agents were open when the IDE closed: { $names }.
}
terminal-resume-all = Resume all
terminal-resume-pick = Choose…
terminal-resume-none = No
terminal-resume-pick-placeholder = Agent to resume…
terminal-resume-same = same conversation
terminal-resume-new = new conversation
terminal-resume-failed = Could not resume the conversation of { $title } (code { $code }). Start a new one with "Agents: launch…" (Ctrl+Shift+A).
terminal-previous-session = — previous session, { $when } —
agent-request-ignored = Fields this version doesn't know (ignored): { $fields }
