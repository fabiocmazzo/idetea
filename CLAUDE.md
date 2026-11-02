<!-- idetea-ui:begin (IDE TEA: "Interfaces" command; edit outside these markers) -->
## Questions through the IDE (Interfaces panel)

When you need me to choose between options, confirm something or fill in a few fields, you can create an
interface in the TEA IDE instead of asking in the chat. This text is enough; the full format (in Portuguese) is
in `.idetea/ui/FORMAT.md`, which the IDE writes for the version that is open, with the JSON Schema next to it
(`.idetea/ui/idetea-ui.schema.json`).

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
     comes as `null`. A path to an object (`/review`) brings the whole object, with my changes to the fields
     inside it.
   - `action.name` is free text that comes back in the response so you know which button was clicked.
   - After a click, the buttons stay disabled until you write a new revision.
   - To show me a file, or ask for a longer text (commit message, PR description), use the `file` component:
     `{"id": "f", "component": "file", "path": ".idetea/ui/rascunhos/commit.md", "line": 3, "column": 1,
     "label": "Edit the message", "open": true}`. `path` is relative to the root; the link opens the file at that
     position, and `open: true` opens it by itself when the revision arrives (several with `open`: one tab each,
     the last one in front; use it on one file). Create the file first (drafts in
     `.idetea/ui/rascunhos/`, outside git). When a button is clicked, the IDE saves the file before writing the
     response: read the file from disk after it.
   - Images in the panel: `{"id": "s", "component": "image", "path": "art/ship.png", "pixelated": true,
     "background": "checker", "height": 96}`; to compare or let me choose, `{"id": "g", "component": "gallery",
     "select": "one", "value": "/pick", "items": [{"id": "a", "path": "a.png", "label": "A"}, …]}` (`select`:
     `none`, `one` or `many`). The panel rereads an image when the file changes, so regenerate it and I see the
     new one. A click opens it in the image tab (zoom). Images inside `markdown` show only their alt text.
   - Animated spritesheet: `{"id": "fly", "component": "sprite", "path": "art/bird.png", "frames": 2, "fps": 8,
     "pixelated": true, "background": "checker", "height": 64}` (frames in a row of equal width; for a grid add
     `frame_width` and `frame_height`). I get Pause, Next frame and the frame number.
   - Sound in the panel: `{"id": "v", "component": "audio", "path": "sfx/wind.wav", "label": "Wind", "loop": true}`
     (`wav`, `ogg`, `oga`, `opus`, `flac`, `mp3`). I get Play/Stop (one sound at a time, through the system
     player); for WAV the panel also shows duration, rate, channels, bits, peak in dBFS (with a warning at 0 dBFS)
     and the waveform, and rereads them when the file changes.
   - Other media: a `file` (or a `markdown` link) to a PDF opens a PDF tab; SVG opens as text with a preview.
     Audio, video and other binary files do not open in a tab.
   - Links in `markdown` open the web (`https://…`), project files (`src/main.rs:42`), the IDE config
     (`idetea:config`, `idetea:config-project`) or a project file in the system app (`[play](idetea:open/index.html)`;
     only `html`, `htm`, `pdf`, images, audio, `mp4` and `webm`). Use a `markdown` for step lists and commands instead of one
     `text` per step. Put every command I should run in a fenced code block: the panel shows a Copy button on it.
   - When I must do several things before answering (a manual test), use `steps` instead of a numbered list, and
     make the approve button wait for them: `{"id": "s", "component": "steps", "value": "/done", "items":
     [{"id": "a", "text": "Run `x`"}, {"id": "b", "text": "Maybe y", "optional": true}]}` and
     `"requires": "/done"` on the button. The value is the list of checked ids; put `/done` in the `context` of
     the "failed" button too, so you see how far I got.
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
   comment field by itself; don't add your own "notes" input. The response also says which IDE wrote it:
   `"ide": {"pid": …, "version": …, "outdated": false}`. `outdated: true` means the IDE was rebuilt after it opened
   and the click came from old code: when testing a change, ask me to restart it (palette: "Restart the IDE") and
   repeat. Don't write claims about which IDE is open into the document; the panel shows that itself.
   `ide.instructions` is the version of these instructions in the IDE; if it differs from the "Instructions
   version" line at the end of this block, the IDE has newer ones: ask me to run "Interfaces: teach the agents"
   and reread this file. With images or sounds in the document, the response has `media`, by path:
   `{"sfx/wind.wav": {"size": 61784, "modified": "…", "seconds": 1.9, "peak_db": -0.3, "played": 2, "completed": 1}}`
   (`size` and `modified` of the file that was on screen; for WAV, what the panel showed: `seconds`, `peak_db`,
   `clips: true` at 0 dBFS; counts in this revision: `opened` (also `idetea:open` links), `played`, `completed`,
   `stopped` (I clicked Stop), `interrupted` (another sound started), `stepped`). It tells what I did, not what I
   heard or saw. Read the
   response and **delete the response file**: that is how the IDE knows you got it.
3. To show a result or ask something else, rewrite the same file with a higher `revision`. A document with only
   `text` and no buttons shows something without waiting for a response.
4. When you are done, delete `.idetea/ui/<name>.json`. The rest of the folder (`.gitignore`, `responses/`)
   belongs to the IDE; leave it. To reuse a name later, first delete any response left in
   `.idetea/ui/responses/<name>/`, and keep `revision` higher than any you wrote under that name: then a response
   whose `revision` is not the one you wrote is an old one.

Handing me a manual test, do it in one interface: what changed in a `markdown`, what to do in a `steps`, and three
buttons: "Approved, you can commit", "Approved, don't commit" and "Failed" (the first two with `requires`). An
approval is my judgement, not proof: before committing, check what the test should have left behind (files on
disk, `ide.outdated` false in the response). If something is missing, ask again in a new revision saying what you
did not find.

Do not put secrets in the document. If no response comes within 10 minutes (the panel may be turned off), ask in
the chat.

Instructions version: `dd57ade4`
<!-- idetea-ui:end -->
