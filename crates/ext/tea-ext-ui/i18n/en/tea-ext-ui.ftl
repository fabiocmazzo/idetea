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
ui-outdated = This IDE is outdated (it was rebuilt after it opened): responses tell the agent. Palette: "Restart the IDE".
