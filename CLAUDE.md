<!-- idetea-ui:begin (IDE TEA: "Interfaces" command; edit outside these markers) -->
## Questions through the IDE (Interfaces panel)

When you need me to choose between options, confirm something or fill in a few fields, you can create an
interface in the TEA IDE instead of asking in the chat. This text is enough; the full format is at
https://github.com/fabiocmazzo/idetea/blob/main/docs/interfaces/formato.md

Talk to me in the language I use with you, including every text you put in an interface (titles, labels,
options, buttons). These instructions are in English only because they are meant for agents. Once we are using
the panel, put every question that needs my answer there, not only in the chat: I will be looking at the panel.

1. Write `.idetea/ui/<name>.json` (name: lowercase letters, digits, `_` and `-`). Write to a temporary file and
   rename it, so the IDE never sees a half-written file: `cat > .idetea/ui/.<name>.json.tmp`, then `mv` to the
   final name.
   ```json
   {"format": "idetea-ui", "version": 1, "revision": 1, "title": "…", "source": "<your name>",
    "data": {"option": "a", "tests": true},
    "components": [
      {"id": "q", "component": "text", "text": "…"},
      {"id": "op", "component": "select", "label": "…", "value": "/option", "options": ["a", {"value": "b", "label": "B"}]},
      {"id": "t", "component": "checkbox", "label": "Write tests", "value": "/tests"},
      {"id": "n", "component": "input", "label": "Name", "placeholder": "optional", "value": "/name"},
      {"id": "ok", "component": "button", "label": "Confirm", "action": {"name": "ok", "context": ["/option", "/tests", "/name"]}},
      {"id": "no", "component": "button", "label": "Cancel", "style": "secondary", "action": {"name": "cancel"}}
    ]}
   ```
   - Components: `text` (`style`: `muted`, `warning`, `error`), `markdown` (`text`: lists, `code`, emphasis,
     links), `select`, `checkbox` (boolean), `input` (text) and `button` (`style`: `secondary`, `danger`).
     `value` is a JSON pointer to a value in `data`; `data` holds the initial values.
   - The response only carries the paths listed in the `context` of the clicked button; a field never filled
     comes as `null`.
   - `action.name` is free text that comes back in the response so you know which button was clicked.
   - After a click, the buttons stay disabled until you write a new revision.
   - To show me a file, or ask for a longer text (commit message, PR description), use the `file` component:
     `{"id": "f", "component": "file", "path": ".idetea/ui/rascunhos/commit.md", "line": 3, "column": 1,
     "label": "Edit the message", "open": true}`. `path` is relative to the root; the link opens the file at that
     position, and `open: true` opens it by itself when the revision arrives. Create the file first (drafts in
     `.idetea/ui/rascunhos/`, outside git). When a button is clicked, the IDE saves the file before writing the
     response: read the file from disk after it.
   - Links in `markdown` open the web (`https://…`), project files (`src/main.rs:42`) or the IDE config
     (`idetea:config`, `idetea:config-project`). Use a `markdown` for step lists and commands instead of one
     `text` per step.
2. Wait for the response in `.idetea/ui/responses/<name>/`. Use bash (in zsh, `compgen` does not exist or
   behaves differently):
   ```sh
   timeout 600 bash -c 'until ls .idetea/ui/responses/<name>/*.json >/dev/null 2>&1; do sleep 1; done'
   ```
   Each file is one response:
   `{"action": "ok", "values": {"/option": "b", "/tests": true, "/name": null}, "revision": 1, "seq": 1, ...}`.
   `action` is the button's `action.name`; `revision` is the revision I was seeing when I clicked; `seq` is the
   order (there may be more than one file; read them in name order). If I wrote a comment, it comes as
   `"comment": "…"`: it may be a question back or a caveat, so answer it (new revision or chat). The IDE adds the
   comment field by itself; don't add your own "notes" input. Read the response and **delete the response file**:
   that is how the IDE knows you got it.
3. To show a result or ask something else, rewrite the same file with a higher `revision`. A document with only
   `text` and no buttons shows something without waiting for a response.
4. When you are done, delete `.idetea/ui/<name>.json`. The rest of the folder (`.gitignore`, `responses/`)
   belongs to the IDE; leave it.

Do not put secrets in the document. If no response comes within 10 minutes (the panel may be turned off), ask in
the chat.
<!-- idetea-ui:end -->
