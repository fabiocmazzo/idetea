# IDE TEA demo video

Records a narrated screen capture of IDE TEA on a private virtual display (Xvfb). Nothing is shown on the host desktop.

The video committed in `assets/idetea-demo.mp4` was made with this pipeline: OpenRouter `gpt-audio`, voice `sage`, English, burned-in subtitles, 1280×720, about 52 seconds.

## One command

From the repository root:

```sh
just demo
```

That builds the IDE if `target/release/idetea` is missing, then writes `dist/idetea-demo.mp4` (H.264 + AAC, `yuv420p`, `+faststart`, burned-in subtitles). `dist/` is gitignored.

From this folder, the same entry points are `./demo.sh` and `make demo`.

## Voice

`just demo` picks the engine from the environment. It never prints an API key.

| `TTS_ENGINE` | When |
|---|---|
| *(unset)* and `OPENROUTER_API_KEY` is set | OpenRouter, voice `sage` (`OPENROUTER_VOICE`, model `OPENROUTER_MODEL`, default `openai/gpt-audio`) |
| *(unset)* and no key | Piper, local, voice `en_US-lessac-medium` |
| `openrouter` | OpenRouter. Fails if the key is missing (it does not silently switch). |
| `piper` or `local` | Piper. `fetch-piper.sh` downloads the binary and the voice when they are missing. |
| `openai` | OpenAI TTS. Needs `OPENAI_API_KEY`. |
| `elevenlabs` | ElevenLabs. Needs `ELEVENLABS_API_KEY`. |

Piper and the voice models are not in git. The download is the 2023.11.14 Linux build from rhasspy/piper (`x86_64` or `aarch64`) and `en_US-lessac-medium` from Hugging Face. Another Piper voice has to be placed in `voices/` by hand (`PIPER_VOICE` is the `.onnx` name without the suffix).

OpenRouter takes that match the script are cached in `cache/openrouter/<hash>.wav`. The hash covers model, voice, prompt and text, so a re-run with unchanged lines makes no API call. A take that never matches the line is used once and not cached. Per-call usage goes to `work/openrouter-usage.json` (gitignored). Do not commit keys, and do not put them on the command line of a shared log if you can pass them as an environment variable instead.

Speech only, no screen recording:

```sh
make narration
# or:  python3 record_demo.py --narration-only
```

## What you need

- Linux with `ffmpeg`, `xdotool`, `Xvfb`, and a software OpenGL stack (`libgl1-mesa-dri`, llvmpipe). No GPU is required. The driver sets `WGPU_BACKEND=gl` and `LIBGL_ALWAYS_SOFTWARE=1`.
- Rust stable, to compile the IDE: `cargo build --release --locked -p tea-app` at the repository root.
- For Piper: network access the first time, so `fetch-piper.sh` can download the binary and the voice. The GitHub Actions workflow does this when `OPENROUTER_API_KEY` is not set.

## Changing the script

Edit `scenes.yaml`. Each scene has a `narration` line (English, spoken as-is) and a list of `actions`:

| Action | Effect |
|---|---|
| `wait: 1.2` | Pause, in seconds |
| `key: ctrl+p` | Press a key or chord (`ctrl+shift+p`, `ctrl+backslash`, `Return`, `ctrl+End`) |
| `type: "greet.rs"` | Type at 40 ms per character |
| `type_code: "pub fn …"` | Type more slowly, for code |
| `mousemove: {x: 640, y: 360}` | Move the pointer |
| `click: {x: 100, y: 200}` | Click |
| `agent_form: {from, name}` | Write `.idetea/ui/<name>.json` the way an agent would |

The driver generates speech first, measures each clip, runs the actions, then waits so the picture stays in sync with the voice.

Subtitles can be omitted with `python3 record_demo.py --no-subs`.

## Layout

```
tools/demo-video/
  demo.sh             one-command entry point (also what `just demo` runs)
  fetch-piper.sh      downloads Piper and the voice if they are missing
  assets/agent-review.json  the form the "agent" writes during the Interfaces scene
  Makefile            make demo / make narration / make build
  scenes.yaml         actions + narration
  record_demo.py      driver
  yaml_lite.py        small YAML loader (no PyYAML)
  sample-project/     tiny Rust crate opened in the IDE
  xdg/config/         isolated IDE config (the real ~/.config/idetea is not touched)
  cache/openrouter/   cached OpenRouter takes
  work/               generated audio, raw capture, stills (gitignored)
```

`sample-project/.git` is created at record time (branch `master`, commit "Initial tea-demo project") and is gitignored. Do not commit that nested repository. `sample-project/.idetea/ui/review.json` is also untracked on purpose: IDE TEA ignores agent UI documents that are already versioned in git, so the form would not show up.

Before each run the driver resets `sample-project/src/greet.rs`, removes the agent form, and clears unsaved-edit backups, so the IDE does not recover text from the previous take.

## What the picture shows

1. A Rust project (`src/main.rs`) with the file tree, tabs and syntax highlighting.
2. Quick open (`Ctrl+P`) to `src/greet.rs`.
3. Typing a small function in the editor.
4. An editor split (`Ctrl+\`).
5. The integrated terminal (`Ctrl+'`).
6. The Interfaces panel, with a form written as `.idetea/ui/review.json`. The mascot switches to Attention.
7. The closing line: "No account, no telemetry, no marketplace. Just code."

## Notes

- Rendering is software-only (Mesa llvmpipe through wgpu's GL backend). Do **not** force `MESA_GL_VERSION_OVERRIDE=3.3`: iced's shaders need GLSL 4.20+ and the window stays black.
- The recording starts about 0.6 s before the first scene. The driver measures that and shifts the narration and the subtitles to match.
- The final mix is loudness-normalized (EBU R128, −16 LUFS).
- GitHub Actions (`.github/workflows/demo.yml`) runs this on a published release and on `workflow_dispatch`, uploads `dist/idetea-demo.mp4` as an artifact, and attaches it to the release when there is one. The secret is optional.
