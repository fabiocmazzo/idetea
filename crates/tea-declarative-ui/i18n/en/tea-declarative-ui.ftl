## Erros do documento (aparecem no painel, para o usuário e para o agente corrigir)
ui-too-big = The document has { $size } bytes; the maximum is { $max }.
ui-not-json = The file is not valid JSON: { $error }
ui-not-object = The document must be a JSON object.
ui-wrong-format = "format" must be "idetea-ui" (got { $found }).
ui-unsupported-version = Format version { $version } is not supported (this IDE understands { $supported }).
ui-missing = Missing field "{ $field }".
ui-wrong-type = "{ $field }" must be of type { $expected }.
ui-too-many = "{ $field }" has more than { $max } items.
ui-too-long = "{ $field }" is longer than { $max } bytes.
ui-bad-id = "{ $field }": invalid id ("{ $id }"). Use letters, digits, "_" and "-" (up to 64).
ui-duplicate-id = Duplicate id: "{ $id }".
ui-bad-path = "{ $field }": "{ $path }" is not a valid path (e.g. "/name").
ui-bad-file-path = "{ $field }": "{ $path }" must be a path relative to the project root, without "..".
ui-data-too-big = "data" is larger than { $max } bytes.
ui-data-too-deep = "data" is more than { $max } levels deep.

## Avisos (o documento abre mesmo assim)
ui-unknown-field = Unknown field ignored: "{ $field }".
ui-unknown-value = "{ $field }": unknown value "{ $value }", using the default.
ui-unknown-component = Component "{ $component }" is not supported in this version.
ui-link-error = Could not open { $url }: { $error }
ui-link-ignored = Link ignored (only web, project files and idetea:config): { $url }
ui-comment-label = Comment for the agent (optional)
ui-comment-placeholder = sent along with the button you click
ui-copy = Copy
ui-copied = Copied.
ui-steps-missing = { $count ->
    [one] 1 step left to check.
   *[other] { $count } steps left to check.
}
ui-step-optional = (optional)
ui-requires-unbound = "{ $field }": "{ $path }" is not the "value" of any component; ignored.
ui-fields-missing = Still to fill in: { $fields }
ui-too-many-images = The document has more than { $max } images.
ui-image-loading = { $path }: loading…
ui-image-error = could not read it: { $error }
ui-image-not-file = not a file (symlinks are not followed)
ui-image-too-big = larger than { $max } MB
ui-image-too-large = { $width }×{ $height } px; the maximum is { $max } per side
ui-image-total = the document's images exceed { $max } million pixels
ui-too-many-sounds = The document has more than { $max } sounds.
ui-audio-play = Play
ui-audio-stop = Stop
ui-audio-info = { $seconds } s · { $rate } Hz · { $channels ->
        [one] mono
        [2] stereo
       *[other] { $channels } channels
    } · { $bits } bits · { $peak }
ui-audio-peak = peak { $db } dBFS
ui-audio-silent = silence
ui-audio-loop = looping
ui-audio-clips = Reaches 0 dBFS: it may be clipping.
ui-audio-extension = Not a sound (accepted: { $list }).
ui-audio-too-big = Larger than { $max } MB: plays, without duration or waveform.
ui-audio-unreadable = No duration or waveform ({ $error }); it still plays.
ui-audio-no-player = No sound player found (pw-play, paplay, aplay or afplay).
ui-audio-plugin = Plugin panels do not play sound.
ui-sprite-pause = Pause
ui-sprite-play = Animate
ui-sprite-step = Next frame
ui-sprite-info = frame { $frame } of { $count } · { $fps } fps
ui-sprite-frame = { $field }: frame_width and frame_height go together.
ui-sprite-too-small = The sheet ({ $width }×{ $height }) is smaller than one frame.
ui-open-blocked = { $path }: idetea:open links only open { $list }.
ui-open-not-file = { $path } is not a regular file (missing, a folder or a symlink).
ui-select-text = Select text
ui-select-done = Show formatted
