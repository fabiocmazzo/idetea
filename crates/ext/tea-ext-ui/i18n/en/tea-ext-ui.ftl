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
ui-write-failed = Could not write the response: { $error }
ui-prepare-failed = Could not prepare the .idetea/ui folder: { $error }
ui-not-a-file = not a regular file
ui-surface-mismatch = the surface field is "{ $found }", but the file is named { $expected }.json
