ui-panel = Interfaces
cmd-ui-show = Interfaces: show the panel
ui-empty = No interface open. An agent can create one by writing a file to .idetea/ui/ (see docs/interfaces/formato.md). To turn it on or off: [extensions] interfaces = {"{"} enabled = true {"}"} in the configuration.
ui-tracked = { $path } is tracked by git and was ignored for safety: interfaces only come from the agent running on this machine, never from a cloned repository.
ui-invalid = Invalid document: { $error }
ui-origin = { $path }
ui-origin-source = { $path } · { $source }
ui-waiting = { $count ->
    [one] Response sent; waiting for the agent to read it.
   *[other] { $count } responses sent; waiting for the agent to read them.
}
ui-received = Response received by the agent.
ui-agent-waiting = The agent is waiting for the answer.
ui-agent-stopped = { $minutes ->
    [0] The agent just stopped waiting: the answer is kept until it looks again.
   *[other] The agent stopped waiting { $minutes } min ago: the answer is kept until it looks again.
}
ui-new = New interface from the agent: { $title }
ui-updated = Interface updated: { $title }
ui-open-later = The agent asked to open { $path }: it is in the Interfaces panel (not opened to avoid interrupting your typing).
ui-write-failed = Could not write the response: { $error }
ui-prepare-failed = Could not prepare the .idetea/ui folder: { $error }
ui-not-a-file = not a regular file
ui-surface-mismatch = the surface field is "{ $found }", but the file is named { $expected }.json
cmd-ui-teach = Interfaces: teach this project's agents
ui-teach-hint = So that a coding agent knows how to use this panel, write the instructions to the project's AGENTS.md.
ui-teach-button = Teach this project's agents
ui-teach-title = Teach this project's agents?
ui-teach-detail = The IDE will write the Interfaces panel instructions to: { $files }. The text stays between markers; running it again only updates that part and leaves the rest alone. These are project files: review them before committing.
ui-teach-file-new = { $name } (new)
ui-teach-write = Write
ui-cancel = Cancel
ui-teach-up-to-date = The agent instructions are already up to date.
ui-teach-done = Instructions written to { $files }.
ui-teach-failed = Could not write the instructions: { $error }
cmd-ui-new-project = Project: new…
new-title = New project
new-intro = The IDE creates the folder, git, the template's skills and the panel instructions, and opens the project in a new window. Your agent creates the actual project, following the initial request in **AGENTS.md**. With an agent chosen below, the new window asks to open it in an IDE terminal (you confirm the command).
new-field-parent = Create inside
new-field-name = Folder name
new-field-name-hint = e.g. my-app
new-field-model = Template
new-field-description = What the project is (optional)
new-field-description-hint = one sentence; it goes in the initial request for the agent
new-field-git = Create the git repository
new-field-agent = Agent to open in an IDE terminal
new-agent-none = None (I'll open it)
new-agent-title = Agent: { $agent }
new-bad-agent = invalid agent name: "{ $agent }" (only letters, digits, - and _)
new-create = Create and open
new-model = { $model ->
    [vazio] Empty (the agent asks what it is)
    [rust] Rust
    [site] Website
    [python] Python
    [fastapi] FastAPI (Python web API)
   *[other] { $model }
}
new-creating = Creating { $dir } (the skills come over the network)…
new-created = Project created in { $dir }; opening it in a new window.
new-open-failed = Project created in { $dir }, but the window could not be opened: { $error }
new-failed = Could not create the project: { $error }
new-bad-model = unknown template: { $model }
new-bad-dir = invalid folder: { $dir } (use a full path)
new-bad-name = invalid folder name: "{ $name }"
new-not-empty = { $dir } already exists and has files: choose another name
new-done-dir = folder: { $dir }
new-done-git = git: repository created
new-done-agents = AGENTS.md (with the initial request) and CLAUDE.md, with the panel instructions
new-done-agent = terminal request for { $agent }: confirm it in the new window
new-open-hint = Done. Open it with: idetea { $dir }
new-usage = idetea new: "{ $arg }" is not valid. Usage: idetea new <folder> [--modelo id] [--descricao text] [--sem-git] [--agente name]; templates: { $models }
cmd-ui-fix = Interfaces: fix this project
ui-fix-title = Fix this project?
ui-fix-will = The IDE will:
ui-fix-wont = Just so you know (left as is):
ui-fix-button = Fix
ui-fix-nothing = Nothing to fix in this project's interfaces.
ui-fix-done = { $count ->
    [one] Fixed: 1 item.
   *[other] Fixed: { $count } items.
}
ui-fix-failed = Could not fix: { $error }
ui-fix-hint = To fix the items marked "-": idetea ui fix --apply
ui-fix-usage = idetea ui fix: unknown argument "{ $arg }" (usage: idetea ui fix [--apply])
ui-fix-abandoned = delete { $path }: nobody has been waiting for the answer for { $since }
ui-fix-unread = { $path }: { $count ->
    [one] 1 answer the agent has not read
   *[other] { $count } answers the agent has not read
} (kept; ask the agent to read it, or delete it by hand)
ui-fix-leftover = delete { $path }: left over from a write or a wait that did not finish
ui-fix-check = rewrite .idetea/ui/check: it calls { $target }, which no longer exists
ui-fix-instructions-older = rewrite the instructions in { $file }: they are at version { $found }, older than this IDE's
ui-fix-instructions-unversioned = rewrite the instructions in { $file }: they are from an old IDE, without the version line
ui-fix-instructions-newer = { $file } has instructions from a newer IDE ({ $found }): open the project in it (I don't go back)
ui-outdated = This IDE is outdated (it was rebuilt after it opened): responses tell the agent. Palette: "Restart the IDE".
ui-audio-failed = Could not play the sound: { $error }
ui-check-ok = { $file }: ok ({ $count ->
        [one] 1 component
       *[other] { $count } components
    })
ui-check-error = { $file }: error: { $error }
ui-check-warning = warning: { $warning }
ui-check-media = media: { $path }: { $problem }
ui-check-name = invalid file name ("{ $name }"): lowercase letters, digits, _ and -, ending in .json
ui-check-usage = usage: idetea ui check <file.json>…
ui-check-instructions-older = instructions: { $file } has version { $found } and this IDE has { $ours }: ask the user to run "Interfaces: teach this project's agents" and reread the file before publishing
ui-check-instructions-newer = instructions: { $file } has version { $found }, from an IDE newer than this one ({ $ours }): the open panel may not understand what the file teaches
ui-check-instructions-unversioned = instructions: { $file } has an old block, without the version line; this IDE has { $ours }: ask the user to run "Interfaces: teach this project's agents" and reread the file before publishing
ui-teach-newer-file = { $name } (version { $version })
ui-teach-newer = Warning: { $files } was written by a newer IDE than this one (which has version { $ours } and does not know that one). Writing takes the instructions back to this IDE's version; usually you want to open the project with the newer IDE.
ui-teach-write-anyway = Write anyway
