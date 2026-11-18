#!/usr/bin/env python3
"""Record an AI-narrated IDE TEA demo on a private Xvfb display.

Usage (from tools/demo-video, or `just demo` at the repository root):
    python3 record_demo.py                  # full pipeline -> dist/idetea-demo.mp4
    python3 record_demo.py --narration-only
    python3 record_demo.py --bin /path/to/idetea --out /path/to/out.mp4

TTS backends (TTS_ENGINE=piper|openrouter|elevenlabs|openai, default piper):
    OpenRouter: set OPENROUTER_API_KEY (optional OPENROUTER_VOICE=sage, OPENROUTER_MODEL=openai/gpt-audio)
    Piper (default, local, free): uses tools/piper and voices/<voice>.onnx
    ElevenLabs: set ELEVENLABS_API_KEY (optional ELEVENLABS_VOICE_ID)
    OpenAI:     set OPENAI_API_KEY (optional OPENAI_TTS_MODEL, OPENAI_TTS_VOICE)
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import tempfile
import signal
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

import yaml_lite

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
DEFAULT_OUT = REPO / "dist" / "idetea-demo.mp4"
DEFAULT_BIN = REPO / "target" / "release" / "idetea"
WORK = ROOT / "work"
AUDIO_DIR = WORK / "audio"
FRAMES_DIR = WORK / "frames"
DISPLAY_NUM = int(os.environ.get("IDETEA_DEMO_DISPLAY", "99"))
DISPLAY = f":{DISPLAY_NUM}"


def log(msg: str) -> None:
    print(msg, flush=True)


def public_error(body: bytes) -> str:
    """Shorten an HTTP error body and strip anything that looks like a credential."""
    import re
    text = body[:300].decode("utf-8", "replace")
    text = re.sub(r"sk-[A-Za-z0-9_\-]+", "sk-…", text)
    text = re.sub(r"Bearer\s+\S+", "Bearer …", text)
    return text


def run(cmd, **kw):
    kw.setdefault("check", True)
    return subprocess.run(cmd, **kw)


def duration_of(path: Path) -> float:
    out = subprocess.check_output(
        [
            "ffprobe", "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=nw=1:nk=1",
            str(path),
        ],
        text=True,
    ).strip()
    return float(out)


def x_env(extra: dict | None = None) -> dict:
    env = os.environ.copy()
    env["DISPLAY"] = DISPLAY
    env.pop("WAYLAND_DISPLAY", None)
    env.pop("WAYLAND_SOCKET", None)
    env["XDG_SESSION_TYPE"] = "x11"
    env["WINIT_UNIX_BACKEND"] = "x11"
    # Software renderer: no GPU on this machine. iced/wgpu via OpenGL + llvmpipe.
    env["LIBGL_ALWAYS_SOFTWARE"] = "1"
    env["GALLIUM_DRIVER"] = "llvmpipe"
    env["WGPU_BACKEND"] = os.environ.get("WGPU_BACKEND", "gl")
    env["WGPU_POWER_PREF"] = "low"
    # iced/wgpu shaders need unpackHalf2x16 (GLSL 4.20+). Forcing 3.3 makes
    # llvmpipe reject the pipeline and the window stays black. Leave unset to
    # use native llvmpipe (GL 4.5+), or set MESA_GL_VERSION_OVERRIDE=4.5.
    if os.environ.get("MESA_GL_VERSION_OVERRIDE"):
        env["MESA_GL_VERSION_OVERRIDE"] = os.environ["MESA_GL_VERSION_OVERRIDE"]
        env["MESA_GLSL_VERSION_OVERRIDE"] = os.environ.get("MESA_GLSL_VERSION_OVERRIDE", "450")
    env["LIBGL_DRIVERS_PATH"] = "/usr/lib/x86_64-linux-gnu/dri"
    # Isolated IDE config / state (do not touch the real home config).
    env["XDG_CONFIG_HOME"] = os.environ.get("IDETEA_DEMO_CONFIG_HOME", str(ROOT / "xdg" / "config"))
    env["XDG_STATE_HOME"] = str(ROOT / "xdg" / "state")
    env["XDG_DATA_HOME"] = str(ROOT / "xdg" / "data")
    env["XDG_CACHE_HOME"] = str(ROOT / "xdg" / "cache")
    env["IDETEA_NO_SPLASH"] = "1"
    env["LC_ALL"] = "C.UTF-8"
    env["LANG"] = "C.UTF-8"
    # `idetea` (for `idetea agent`, `idetea mascot` typed in the IDE terminal) and the
    # demo's stand-in agent script are on the PATH of the IDE and its terminals.
    extra_path = [p for p in os.environ.get("IDETEA_DEMO_PATH", "").split(os.pathsep) if p]
    if extra_path:
        env["PATH"] = os.pathsep.join(extra_path + [env.get("PATH", "")])
    if extra:
        env.update(extra)
    return env


# ---------------------------------------------------------------------------
# Narration / TTS
# ---------------------------------------------------------------------------

def voice_path(cfg: dict) -> Path:
    name = os.environ.get("PIPER_VOICE", cfg.get("voice", "en_US-lessac-medium"))
    model = ROOT / "voices" / f"{name}.onnx"
    if not model.is_file():
        sys.exit(f"Piper voice not found: {model}\nSee README.md for how to download it.")
    return model


def tts_piper(text: str, out_wav: Path, cfg: dict) -> None:
    piper = ROOT / "tools" / "piper" / "piper"
    if not piper.is_file():
        sys.exit(f"Piper binary not found: {piper}\nSee README.md for how to install it.")
    env = os.environ.copy()
    libdir = str(ROOT / "tools" / "piper")
    env["LD_LIBRARY_PATH"] = libdir + (os.pathsep + env["LD_LIBRARY_PATH"] if env.get("LD_LIBRARY_PATH") else "")
    run(
        [str(piper), "--model", str(voice_path(cfg)), "--output_file", str(out_wav)],
        input=text.encode("utf-8"),
        env=env,
    )


def tts_elevenlabs(text: str, out_wav: Path) -> None:
    key = os.environ.get("ELEVENLABS_API_KEY")
    if not key:
        sys.exit("TTS_ENGINE=elevenlabs requires ELEVENLABS_API_KEY")
    voice = os.environ.get("ELEVENLABS_VOICE_ID", "21m00Tcm4TlvDq8ikWAM")  # Rachel
    url = f"https://api.elevenlabs.io/v1/text-to-speech/{voice}?output_format=pcm_22050"
    body = json.dumps({"text": text, "model_id": os.environ.get("ELEVENLABS_MODEL", "eleven_multilingual_v2")}).encode()
    req = urllib.request.Request(url, data=body, method="POST", headers={
        "xi-api-key": key,
        "Content-Type": "application/json",
        "Accept": "application/octet-stream",
    })
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            pcm = resp.read()
    except urllib.error.HTTPError as e:
        sys.exit(f"ElevenLabs TTS failed: {e.code} {public_error(e.read())}")
    # raw pcm s16le 22050 mono -> wav
    run([
        "ffmpeg", "-y", "-f", "s16le", "-ar", "22050", "-ac", "1", "-i", "pipe:0",
        str(out_wav),
    ], input=pcm, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def tts_openai(text: str, out_wav: Path) -> None:
    key = os.environ.get("OPENAI_API_KEY")
    if not key:
        sys.exit("TTS_ENGINE=openai requires OPENAI_API_KEY")
    model = os.environ.get("OPENAI_TTS_MODEL", "gpt-4o-mini-tts")
    voice = os.environ.get("OPENAI_TTS_VOICE", "alloy")
    body = json.dumps({"model": model, "voice": voice, "input": text, "format": "wav"}).encode()
    req = urllib.request.Request("https://api.openai.com/v1/audio/speech", data=body, method="POST", headers={
        "Authorization": f"Bearer {key}",
        "Content-Type": "application/json",
    })
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            out_wav.write_bytes(resp.read())
    except urllib.error.HTTPError as e:
        sys.exit(f"OpenAI TTS failed: {e.code} {public_error(e.read())}")


OPENROUTER_SYSTEM = (
    "You are a narrator. Read the user's text aloud exactly, word for word, nothing else. "
    "Calm, friendly, professional product-demo tone."
)
USAGE_LOG: list[dict] = []
# Trim leading/trailing silence, then keep a short 0.25 s tail so scenes breathe.
TRIM_AF = ("silenceremove=start_periods=1:start_threshold=-45dB,"
           "areverse,silenceremove=start_periods=1:start_threshold=-45dB,areverse,apad=pad_dur=0.25")


def _norm(s: str) -> str:
    import re as _re
    s = s.lower().replace("’", "'")
    s = _re.sub(r"[^a-z0-9' ]+", " ", s)
    return " ".join(s.split())


def openrouter_request(text: str, voice: str, model: str, attempt: int = 1) -> tuple[bytes, str, dict]:
    import base64
    key = os.environ.get("OPENROUTER_API_KEY")
    if not key:
        sys.exit("TTS_ENGINE=openrouter requires OPENROUTER_API_KEY")
    body = json.dumps({
        "model": model,
        "modalities": ["text", "audio"],
        "audio": {"voice": voice, "format": "pcm16"},
        "stream": True,
        "usage": {"include": True},
        "messages": [
            {"role": "system", "content": OPENROUTER_SYSTEM},
            {"role": "user", "content": text if attempt == 1 else
                f"Say exactly the following text and nothing before or after it. Do not acknowledge.\n\n{text}"},
        ],
    }).encode()
    req = urllib.request.Request("https://openrouter.ai/api/v1/chat/completions", data=body, method="POST", headers={
        "Authorization": f"Bearer {key}",
        "Content-Type": "application/json",
        "Accept": "text/event-stream",
        "X-Title": "IDE TEA demo narration",
    })
    pcm = bytearray()
    transcript = []
    usage: dict = {}
    try:
        with urllib.request.urlopen(req, timeout=180) as resp:
            for raw in resp:
                line = raw.decode("utf-8", "replace").strip()
                if not line.startswith("data:"):
                    continue
                data = line[5:].strip()
                if data == "[DONE]":
                    break
                try:
                    ev = json.loads(data)
                except json.JSONDecodeError:
                    continue
                if ev.get("usage"):
                    usage = ev["usage"]
                for ch in ev.get("choices") or []:
                    audio = (ch.get("delta") or {}).get("audio") or {}
                    if audio.get("data"):
                        pcm += base64.b64decode(audio["data"])
                    if audio.get("transcript"):
                        transcript.append(audio["transcript"])
    except urllib.error.HTTPError as e:
        # never echo headers (they contain the key)
        sys.exit(f"OpenRouter TTS failed: HTTP {e.code} {public_error(e.read())}")
    return bytes(pcm), "".join(transcript), usage


def tts_openrouter(text: str, out_wav: Path) -> None:
    import hashlib
    voice = os.environ.get("OPENROUTER_VOICE", "sage")
    model = os.environ.get("OPENROUTER_MODEL", "openai/gpt-audio")
    cache_dir = ROOT / "cache" / "openrouter"
    cache_dir.mkdir(parents=True, exist_ok=True)
    h = hashlib.sha256(f"{model}\n{voice}\n{OPENROUTER_SYSTEM}\n{text}".encode()).hexdigest()[:20]
    cached = cache_dir / f"{h}.wav"
    meta = cache_dir / f"{h}.json"
    if not cached.is_file():
        best = None
        for attempt in range(1, 4):
            pcm, transcript, usage = openrouter_request(text, voice, model, attempt)
            ok = _norm(transcript) == _norm(text) and len(pcm) > 0
            USAGE_LOG.append({"text": text, "attempt": attempt, "match": ok, "usage": usage})
            log(f"    openrouter attempt {attempt}: {'match' if ok else 'MISMATCH'} transcript={transcript!r}")
            best = (pcm, transcript, ok, attempt)
            if ok:
                break
        pcm, transcript, ok, attempt = best
        if not pcm:
            sys.exit("OpenRouter returned no audio")
        tmp = cached.with_suffix(".tmp.wav")
        run(["ffmpeg", "-y", "-f", "s16le", "-ar", "24000", "-ac", "1", "-i", "pipe:0", str(tmp)],
            input=pcm, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if not ok:
            log("    WARNING: no attempt matched the script; using the last take (not cached)")
            run(["ffmpeg", "-y", "-i", str(tmp), "-af", TRIM_AF, str(out_wav)],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            tmp.unlink(missing_ok=True)
            return
        trimmed = cached.with_suffix(".trim.wav")
        run(["ffmpeg", "-y", "-i", str(tmp), "-af", TRIM_AF, str(trimmed)],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        tmp.unlink(missing_ok=True)
        os.replace(trimmed, cached)
        meta.write_text(json.dumps({"text": text, "voice": voice, "model": model, "transcript": transcript,
                                    "match": ok, "attempts": attempt}, indent=2))
    else:
        log(f"    (cached {cached.name})")
    shutil.copyfile(cached, out_wav)


def synthesize(text: str, out_wav: Path, cfg: dict) -> None:
    engine = os.environ.get("TTS_ENGINE", "piper").lower()
    out_wav.parent.mkdir(parents=True, exist_ok=True)
    if engine in ("piper", "local"):
        tts_piper(text, out_wav, cfg)
    elif engine in ("elevenlabs", "11labs"):
        tts_elevenlabs(text, out_wav)
    elif engine == "openai":
        tts_openai(text, out_wav)
    elif engine == "openrouter":
        tts_openrouter(text, out_wav)
    else:
        sys.exit(f"Unknown TTS_ENGINE={engine!r} (use piper, openrouter, elevenlabs or openai)")


def generate_narration(cfg: dict) -> list[Path]:
    AUDIO_DIR.mkdir(parents=True, exist_ok=True)
    paths = []
    for i, scene in enumerate(cfg["scenes"], 1):
        wav = AUDIO_DIR / f"{i:02d}-{scene['id']}.wav"
        log(f"  TTS [{scene['id']}] {scene['narration']}")
        synthesize(scene["narration"], wav, cfg)
        dur = duration_of(wav)
        scene["_audio"] = wav
        scene["_duration"] = dur
        log(f"    -> {dur:.2f}s  {wav.name}")
        paths.append(wav)
    return paths


def concat_audio(wavs: list[Path], out: Path) -> float:
    n = len(wavs)
    inputs = []
    for w in wavs:
        inputs += ["-i", str(w)]
    filt = "".join(f"[{i}:a]" for i in range(n)) + f"concat=n={n}:v=0:a=1[a]"
    run([
        "ffmpeg", "-y", *inputs, "-filter_complex", filt, "-map", "[a]",
        "-c:a", "pcm_s16le", str(out),
    ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return duration_of(out)


def place_audio(cfg: dict, out: Path) -> float:
    """Narration with each scene's clip at the time its scene really started.

    When a scene's actions outlast its narration (an IDE restart, say), the next
    clip waits for its scene instead of drifting ahead of the picture.
    """
    scenes = cfg["scenes"]
    inputs, parts = [], []
    for i, scene in enumerate(scenes):
        inputs += ["-i", str(scene["_audio"])]
        ms = int(round(scene["_start"] * 1000))
        parts.append(f"[{i}:a]adelay={ms}|{ms}[d{i}]")
    filt = ";".join(parts) + ";" + "".join(f"[d{i}]" for i in range(len(scenes)))
    filt += f"amix=inputs={len(scenes)}:duration=longest:normalize=0[a]"
    run([
        "ffmpeg", "-y", *inputs, "-filter_complex", filt, "-map", "[a]",
        "-c:a", "pcm_s16le", str(out),
    ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return duration_of(out)


def write_srt(cfg: dict, out: Path) -> None:
    t = 0.0
    lines = []
    def ts(sec: float) -> str:
        h = int(sec // 3600)
        m = int((sec % 3600) // 60)
        s = int(sec % 60)
        ms = int(round((sec - int(sec)) * 1000))
        if ms == 1000:
            s += 1
            ms = 0
        return f"{h:02d}:{m:02d}:{s:02d},{ms:03d}"
    for i, scene in enumerate(cfg["scenes"], 1):
        dur = scene["_duration"]
        t = scene.get("_start", t)  # where the scene really began, when known
        # wrap narration ~60 chars
        text = scene["narration"]
        lines.append(f"{i}\n{ts(t)} --> {ts(t + dur)}\n{text}\n")
        t += dur
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")


# ---------------------------------------------------------------------------
# Xvfb + window + input
# ---------------------------------------------------------------------------

class Session:
    def __init__(self, width: int, height: int, layout: dict | None = None):
        self.layout = layout or {}
        self.width = width
        self.height = height
        self.procs: list[subprocess.Popen] = []
        self.wid: str | None = None
        self.env = x_env()
        self.project: Path = ROOT
        self._waiting: list[Path] = []
        self._stop = False

    def _start_waiting(self, wait_file: Path) -> None:
        import threading
        wait_file.touch()
        self._waiting.append(wait_file)
        if len(self._waiting) == 1:
            def loop():
                while not self._stop:
                    for w in list(self._waiting):
                        try:
                            w.touch()
                        except OSError:
                            pass
                    time.sleep(1.0)
            threading.Thread(target=loop, daemon=True).start()

    def start_xvfb(self) -> None:
        # If a previous run left a leftover server, drop it.
        lock = Path(f"/tmp/.X{DISPLAY_NUM}-lock")
        if lock.exists():
            try:
                pid = int(lock.read_text().strip())
                os.kill(pid, signal.SIGTERM)
                time.sleep(0.4)
            except (ProcessLookupError, ValueError, OSError):
                pass
            try:
                lock.unlink(missing_ok=True)
            except OSError:
                pass
        cmd = [
            "Xvfb", DISPLAY,
            "-screen", "0", f"{self.width}x{self.height}x24",
            "-ac", "+extension", "GLX", "+render", "-noreset",
            "-nolisten", "tcp",
        ]
        log("Starting " + " ".join(cmd))
        p = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
        self.procs.append(p)
        # wait until the display is up
        for _ in range(50):
            if p.poll() is not None:
                raise RuntimeError(f"Xvfb exited early with {p.returncode}")
            if Path(f"/tmp/.X11-unix/X{DISPLAY_NUM}").exists() or lock.exists():
                time.sleep(0.3)
                return
            time.sleep(0.1)
        raise RuntimeError("Xvfb did not create a display")

    def xdo(self, *args, check: bool = True) -> subprocess.CompletedProcess:
        return run(["xdotool", *args], env=self.env, check=check, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def start_ide(self, binary: Path, project: Path, open_file: str) -> None:
        log(f"Launching {binary} in {project}")
        self.project = project
        # Restore a known window size / panel layout before each run.
        state = ROOT / "xdg" / "state" / "idetea" / "state.toml"
        state.parent.mkdir(parents=True, exist_ok=True)
        lay = {"left_width": 260.0, "right_width": 360.0, "bottom_height": 220.0, "split": 0.5}
        lay.update({k: float(v) for k, v in self.layout.items()})
        state.write_text(
            "".join(f"{k} = {float(v)}\n" for k, v in lay.items())
            + f"window_width = {float(self.width)}\n"
            f"window_height = {float(self.height)}\n"
            "mascot_collapsed = false\n"
        )
        self._binary, self._open_file = binary, open_file
        logf = open(WORK / "idetea.log", "a")
        p = subprocess.Popen(
            [str(binary), "--no-splash", open_file],
            cwd=str(project),
            env=self.env,
            stdout=logf,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        p._is_ide = True
        self.procs.append(p)
        self._ide_log = logf
        self.wid = self._wait_window(timeout=40)
        log(f"IDE window id={self.wid}")
        self._fit_window()

    def restart_ide(self) -> None:
        """Start a second IDE on top of the first, then close the first.

        Settings such as the interface language take effect after a restart. With no
        window manager, the new window maps over the old one, so the picture goes from
        one IDE to the other without an empty frame.
        """
        old_wid, old_procs = self.wid, [p for p in self.procs if getattr(p, "_is_ide", False)]
        before = set(self._all_windows())
        p = subprocess.Popen(
            [str(self._binary), "--no-splash", self._open_file],
            cwd=str(self.project), env=self.env, stdout=self._ide_log, stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        p._is_ide = True
        self.procs.append(p)
        deadline = time.time() + 30
        new = None
        while time.time() < deadline and not new:
            fresh = [w for w in self._all_windows() if w not in before]
            new = fresh[-1] if fresh else None
            time.sleep(0.1)
        if not new:
            raise RuntimeError("the restarted IDE window did not appear")
        self.wid = new
        self._fit_window()
        time.sleep(1.2)  # first frame drawn
        for op in old_procs:
            try:
                os.killpg(op.pid, signal.SIGTERM)
            except OSError:
                pass
        time.sleep(0.3)
        self.focus()

    def _all_windows(self) -> list[str]:
        r = self.xdo("search", "--name", "IDE TEA", check=False)
        return r.stdout.decode().split() if r.returncode == 0 else []

    def _wait_window(self, timeout: float) -> str:
        deadline = time.time() + timeout
        last_err = b""
        while time.time() < deadline:
            for args in (
                ["search", "--onlyvisible", "--name", "IDE TEA"],
                ["search", "--name", "IDE TEA"],
                ["search", "--class", "idetea"],
                ["search", "--name", "idetea"],
            ):
                r = self.xdo(*args, check=False)
                if r.returncode == 0 and r.stdout.strip():
                    # last id if several
                    return r.stdout.decode().strip().split()[-1]
                last_err = r.stderr
            time.sleep(0.25)
        # dump log tail to help debug
        try:
            tail = (WORK / "idetea.log").read_text()[-2000:]
        except OSError:
            tail = ""
        raise RuntimeError(f"IDE window did not appear on {DISPLAY} within {timeout}s\n{last_err!r}\n{tail}")

    def _fit_window(self) -> None:
        assert self.wid
        # No window manager: move/resize ourselves to fill the framebuffer.
        for args in (
            ["windowmove", self.wid, "0", "0"],
            ["windowsize", self.wid, str(self.width), str(self.height)],
            ["windowactivate", "--sync", self.wid],
            ["windowfocus", "--sync", self.wid],
        ):
            self.xdo(*args, check=False)
        time.sleep(0.4)

    def focus(self) -> None:
        if self.wid:
            self.xdo("windowactivate", "--sync", self.wid, check=False)
            self.xdo("windowfocus", "--sync", self.wid, check=False)

    def do_action(self, action: dict) -> None:
        self.focus()
        if "wait" in action:
            time.sleep(float(action["wait"]))
        elif "key" in action:
            # xdotool uses "Return", "ctrl+p", "ctrl+shift+p", "ctrl+backslash", "ctrl+End"
            self.xdo("key", "--window", self.wid, "--clearmodifiers", str(action["key"]))
            time.sleep(0.15)
        elif "type" in action:
            self.xdo(
                "type", "--window", self.wid, "--clearmodifiers", "--delay", "40",
                str(action["type"]),
            )
            time.sleep(0.1)
        elif "type_code" in action:
            self.xdo(
                "type", "--window", self.wid, "--clearmodifiers", "--delay", "70",
                str(action["type_code"]),
            )
            time.sleep(0.12)
        elif "click" in action:
            c = action["click"]
            self.xdo("mousemove", "--sync", str(c["x"]), str(c["y"]))
            self.xdo("click", "1")
            time.sleep(0.1)
        elif "dblclick" in action:
            c = action["dblclick"]
            self.xdo("mousemove", "--sync", str(c["x"]), str(c["y"]))
            self.xdo("click", "--repeat", "2", "--delay", "90", "1")
            time.sleep(0.1)
        elif "shift_click" in action:
            c = action["shift_click"]
            self.xdo("mousemove", "--sync", str(c["x"]), str(c["y"]))
            self.xdo("keydown", "shift")
            self.xdo("click", "1")
            self.xdo("keyup", "shift")
            time.sleep(0.1)
        elif "drag" in action:
            # Press, glide in small steps (so the IDE sees a drag), release.
            a, b = action["drag"]["from"], action["drag"]["to"]
            steps = int(action["drag"].get("steps", 16))
            self.xdo("mousemove", "--sync", str(a["x"]), str(a["y"]))
            self.xdo("mousedown", "1")
            time.sleep(0.15)
            for i in range(1, steps + 1):
                x = a["x"] + (b["x"] - a["x"]) * i / steps
                y = a["y"] + (b["y"] - a["y"]) * i / steps
                self.xdo("mousemove", str(int(x)), str(int(y)))
                time.sleep(0.03)
            self.xdo("mouseup", "1")
            time.sleep(0.1)
        elif "glide" in action:
            # Move the pointer smoothly to a point (looks better than a jump).
            c = action["glide"]
            r = self.xdo("getmouselocation", "--shell", check=False)
            cur = dict(l.split("=", 1) for l in r.stdout.decode().split() if "=" in l)
            x0, y0 = int(cur.get("X", c["x"])), int(cur.get("Y", c["y"]))
            steps = int(c.get("steps", 14))
            for i in range(1, steps + 1):
                self.xdo("mousemove", str(int(x0 + (c["x"] - x0) * i / steps)), str(int(y0 + (c["y"] - y0) * i / steps)))
                time.sleep(0.02)
        elif "restart_ide" in action:
            self.restart_ide()
        elif "agent_form" in action:
            spec = action["agent_form"]
            src = ROOT / spec["from"]
            name = spec.get("name", src.stem)
            ui = self.project / ".idetea" / "ui"
            ui.mkdir(parents=True, exist_ok=True)
            tmp = ui / f".{name}.json.tmp"
            tmp.write_bytes(src.read_bytes())
            os.replace(tmp, ui / f"{name}.json")
            self._start_waiting(ui / f"{name}.wait")
            time.sleep(0.2)
        elif "mousemove" in action:
            c = action["mousemove"]
            self.xdo("mousemove", "--sync", str(int(c["x"])), str(int(c["y"])))
            time.sleep(0.15)
        else:
            log(f"  (unknown action {action!r}, skipped)")

    def screenshot(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        run([
            "ffmpeg", "-y", "-f", "x11grab", "-video_size", f"{self.width}x{self.height}",
            "-i", DISPLAY, "-frames:v", "1", str(path),
        ], env=self.env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    def start_record(self, path: Path, fps: int) -> subprocess.Popen:
        path.parent.mkdir(parents=True, exist_ok=True)
        cmd = [
            "ffmpeg", "-y",
            "-f", "x11grab",
            "-video_size", f"{self.width}x{self.height}",
            "-framerate", str(fps),
            "-i", DISPLAY,
            "-draw_mouse", "1",
            "-c:v", "libx264", "-preset", "veryfast", "-crf", "18",
            "-pix_fmt", "yuv420p",
            str(path),
        ]
        log("Recording " + " ".join(cmd))
        self.rec_started = time.time()
        p = subprocess.Popen(cmd, env=self.env, stdout=subprocess.DEVNULL, stderr=open(WORK / "ffmpeg-record.log", "w"))
        self.procs.append(p)
        time.sleep(0.6)  # let ffmpeg attach
        return p

    def stop_record(self, p: subprocess.Popen) -> None:
        if p.poll() is None:
            p.send_signal(signal.SIGINT)
            try:
                p.wait(timeout=15)
            except subprocess.TimeoutExpired:
                p.kill()
                p.wait(timeout=5)

    def close(self) -> None:
        self._stop = True
        for w in self._waiting:
            w.unlink(missing_ok=True)
        # Quit the IDE cleanly first (avoids "unsaved changes" hang if we typed).
        try:
            if self.wid:
                # Don't save: Ctrl+Q may prompt. Kill the process group instead
                # after a short polite attempt.
                self.xdo("key", "--window", self.wid, "--clearmodifiers", "Escape", check=False)
                time.sleep(0.2)
        except Exception:
            pass
        for p in reversed(self.procs):
            if p.poll() is None:
                try:
                    os.killpg(p.pid, signal.SIGTERM)
                except (ProcessLookupError, PermissionError, OSError):
                    try:
                        p.terminate()
                    except OSError:
                        pass
        deadline = time.time() + 4
        for p in self.procs:
            while p.poll() is None and time.time() < deadline:
                time.sleep(0.1)
            if p.poll() is None:
                try:
                    os.killpg(p.pid, signal.SIGKILL)
                except (ProcessLookupError, PermissionError, OSError):
                    try:
                        p.kill()
                    except OSError:
                        pass
        if hasattr(self, "_ide_log"):
            try:
                self._ide_log.close()
            except OSError:
                pass


def ensure_sample_git(project: Path, date: str | None = None) -> None:
    """Give the status bar a branch without committing a nested .git in this repo.

    The sample project's .git is gitignored. A fresh recording creates it here,
    with a throwaway author, and never reads or writes the user's git config.
    """
    if (project / ".git").exists():
        return
    if shutil.which("git") is None:
        log("git not found; the sample project stays without a branch")
        return
    env = os.environ.copy()
    env.update({
        "GIT_AUTHOR_NAME": "IDE TEA demo",
        "GIT_AUTHOR_EMAIL": "demo@idetea.local",
        "GIT_COMMITTER_NAME": "IDE TEA demo",
        "GIT_COMMITTER_EMAIL": "demo@idetea.local",
    })
    if date:
        # A fixed age keeps the status bar's blame text ("3 days ago") the same width all take long.
        env["GIT_AUTHOR_DATE"] = env["GIT_COMMITTER_DATE"] = date
    run(["git", "init", "-q", "-b", "master"], cwd=project, env=env, stdout=subprocess.DEVNULL)
    run(["git", "checkout", "-B", "master"], cwd=project, env=env, stdout=subprocess.DEVNULL)
    run(["git", "add", "-A"], cwd=project, env=env, stdout=subprocess.DEVNULL)
    run(
        ["git", "commit", "-m", "Initial tea-demo project"],
        cwd=project, env=env, stdout=subprocess.DEVNULL,
    )


def reset_sample_project(project: Path) -> None:
    """Drop unsaved-looking leftovers from a previous recording attempt."""
    greet = project / "src" / "greet.rs"
    original = (
        '/// Build a short greeting for the given name.\n'
        "pub fn hello(name: &str) -> String {\n"
        '    format!("Just code. Add ice, {name}.")\n'
        "}\n"
    )
    greet.write_text(original)
    # The agent form is written live during its scene (action agent_form).
    ui = project / ".idetea" / "ui"
    ui.mkdir(parents=True, exist_ok=True)
    for name in ("review.json", "review.wait"):
        (ui / name).unlink(missing_ok=True)
    shutil.rmtree(ui / "responses", ignore_errors=True)
    shutil.rmtree(project / ".idetea" / "history", ignore_errors=True)
    # Unsaved-edit backups and sessions from an earlier run would be "recovered"
    # into the editor; start each recording from a clean IDE state.
    for sub in ("backups", "sessions", "session"):
        shutil.rmtree(ROOT / "xdg" / "state" / "idetea" / sub, ignore_errors=True)
    ensure_sample_git(project)


def prepare_project_copy(src: Path, cfg: dict) -> Path:
    """Copy a sample project to work/projects/<name>, add fixtures, give it a git repo."""
    dest = WORK / "projects" / src.name
    shutil.rmtree(dest, ignore_errors=True)
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(src, dest, ignore=shutil.ignore_patterns(".git", "target", "Cargo.lock", "state.json"))
    for sub in ("backups", "sessions", "session"):
        shutil.rmtree(ROOT / "xdg" / "state" / "idetea" / sub, ignore_errors=True)
    # The IDE keeps its history out of git (and ignores entries that are tracked).
    hist = dest / ".idetea" / "history"
    hist.mkdir(parents=True, exist_ok=True)
    (hist / ".gitignore").write_text("*\n")
    three_days_ago = time.strftime("%Y-%m-%dT%H:%M:%S", time.localtime(time.time() - 3 * 86400))
    ensure_sample_git(dest, date=three_days_ago)
    # Fixtures come after the commit, as untracked files, the way the IDE would have left them.
    for fx in cfg.get("fixtures") or []:
        target = dest / fx["to"]
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / fx["from"], target)
    return dest


def prepare_user_config(cfg: dict, binary: Path) -> None:
    """Point the IDE at a per-video user config and put `idetea` and assets/ on the PATH.

    `user_config` names a TOML template in xdg/; `@DEMO_ROOT@` becomes this folder.
    The real ~/.config/idetea is never read or written.
    """
    os.environ["IDETEA_DEMO_PATH"] = os.pathsep.join([str(binary.parent), str(ROOT / "assets")])
    name = cfg.get("user_config")
    if not name:
        os.environ.pop("IDETEA_DEMO_CONFIG_HOME", None)
        return
    # A short path: Settings prints it in its header, and a long one wraps in some languages.
    home = Path(tempfile.gettempdir()) / "idetea-demo-config"
    shutil.rmtree(home, ignore_errors=True)
    (home / "idetea").mkdir(parents=True)
    text = (ROOT / name).read_text(encoding="utf-8").replace("@DEMO_ROOT@", str(ROOT))
    (home / "idetea" / "config.toml").write_text(text, encoding="utf-8")
    os.environ["IDETEA_DEMO_CONFIG_HOME"] = str(home)


def frame_is_mostly_black(path: Path, threshold: float = 0.04) -> tuple[bool, float]:
    """Return (is_black, mean_luma 0..1) using ffmpeg signalstats."""
    # Scale down and compute average luma via a one-liner python on raw pixels.
    raw = path.with_suffix(".rgb")
    run([
        "ffmpeg", "-y", "-i", str(path), "-vf", "scale=160:90,format=rgb24",
        "-f", "rawvideo", str(raw),
    ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    data = raw.read_bytes()
    raw.unlink(missing_ok=True)
    if not data:
        return True, 0.0
    mean = sum(data) / (len(data) * 255.0)
    return mean < threshold, mean


# ---------------------------------------------------------------------------
# Mux
# ---------------------------------------------------------------------------

def mux(video: Path, audio: Path, srt: Path | None, out: Path, burn_subs: bool, offset: float = 0.0,
        extra_style: str = "") -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    if offset > 0.01 and srt and srt.is_file():
        shifted = srt.with_name("narration-shifted.srt")
        run(["ffmpeg", "-y", "-itsoffset", f"{offset:.3f}", "-i", str(srt), "-c", "copy", str(shifted)],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        srt = shifted
    cmd = ["ffmpeg", "-y", "-i", str(video), "-itsoffset", f"{offset:.3f}", "-i", str(audio)]
    filt = []
    maps = ["-map", "0:v:0", "-map", "1:a:0"]
    if burn_subs and srt and srt.is_file():
        # Escape path for the subtitles filter.
        srt_esc = str(srt).replace("\\", "\\\\").replace(":", "\\:").replace("'", "\\'")
        style = "FontName=DejaVu Sans,FontSize=11,PrimaryColour=&H00FFFFFF,OutlineColour=&H80000000,BorderStyle=3,Outline=1,Shadow=0,MarginV=12"
        if extra_style:  # e.g. "MarginR=90" keeps the captions off the mascot bubble
            style += "," + extra_style
        filt.append(f"subtitles='{srt_esc}':force_style='{style}'")
    if filt:
        cmd += ["-vf", ",".join(filt)]
    cmd += maps + [
        "-c:v", "libx264", "-preset", "medium", "-crf", "20",
        "-pix_fmt", "yuv420p",
        "-af", "loudnorm=I=-16:TP=-1.5:LRA=11", "-ar", "48000",
        "-c:a", "aac", "-b:a", "128k",
        "-movflags", "+faststart",
                str(out),
    ]
    log("Mux " + " ".join(cmd))
    run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Record an AI-narrated IDE TEA demo")
    p.add_argument("--scenes", type=Path, default=ROOT / "scenes.yaml")
    p.add_argument("--bin", type=Path, default=Path(os.environ.get("IDETEA_BIN", DEFAULT_BIN)))
    p.add_argument("--out", type=Path, default=Path(os.environ.get("IDETEA_DEMO_OUT", DEFAULT_OUT)))
    p.add_argument("--narration-only", action="store_true")
    p.add_argument("--no-subs", action="store_true", help="do not burn subtitles")
    p.add_argument("--keep-work", action="store_true")
    return p.parse_args()


def main() -> int:
    args = parse_args()
    WORK.mkdir(parents=True, exist_ok=True)
    cfg = yaml_lite.load(args.scenes)
    width, height, fps = int(cfg["width"]), int(cfg["height"]), int(cfg["fps"])
    project = (ROOT / cfg.get("project", "sample-project")).resolve()
    open_file = cfg.get("open", "src/main.rs")
    (WORK / "idetea.log").write_text("")

    log("Generating narration")
    wavs = generate_narration(cfg)
    narration_wav = WORK / "narration.wav"
    total = concat_audio(wavs, narration_wav)
    srt = WORK / "narration.srt"
    write_srt(cfg, srt)
    log(f"Narration total: {total:.2f}s")
    if USAGE_LOG:
        (WORK / "openrouter-usage.json").write_text(json.dumps(USAGE_LOG, indent=2))
        cost = sum(float((u.get("usage") or {}).get("cost") or 0) for u in USAGE_LOG)
        up = sum(float(((u.get("usage") or {}).get("cost_details") or {}).get("upstream_inference_cost") or 0) for u in USAGE_LOG)
        log(f"OpenRouter calls: {len(USAGE_LOG)}, OpenRouter cost ${cost:.4f}, upstream (BYOK) cost ${up:.4f}")
    if args.narration_only:
        log(f"Wrote {narration_wav} and {srt}")
        return 0

    binary = args.bin.resolve()
    if not binary.is_file():
        sys.exit(
            f"IDE binary not found: {binary}\n"
            "Build it from the repository root:\n"
            "  cargo build --release --locked -p tea-app\n"
            "or pass --bin /path/to/idetea"
        )
    if not project.is_dir():
        sys.exit(f"Sample project not found: {project}")

    if cfg.get("project_copy"):
        # Feature videos record in a fresh copy of their sample project, so edits,
        # saves and files the IDE writes (AGENTS.md, ADRs, history) never touch the
        # tracked originals and every take starts from the same state.
        project = prepare_project_copy(project, cfg)
    else:
        reset_sample_project(project)
    prepare_user_config(cfg, binary)
    session = Session(width, height, layout=cfg.get("layout"))
    raw_video = WORK / "capture.mp4"
    rec = None
    audio_offset = 0.0
    try:
        session.start_xvfb()
        # Smoke-test GL on this display before launching the IDE.
        session.start_ide(binary, project, open_file)
        # Give wgpu a moment to present the first frame.
        time.sleep(1.5)
        session._fit_window()
        time.sleep(0.4)
        # Actions that run before the capture starts (wait for a language server, open a panel).
        for action in cfg.get("setup") or []:
            session.do_action(action)
        first = FRAMES_DIR / "00-startup.png"
        session.screenshot(first)
        black, luma = frame_is_mostly_black(first)
        log(f"Startup frame luma={luma:.3f}  ({first})")
        if black:
            # dump log and fail — caller can inspect
            log((WORK / "idetea.log").read_text()[-3000:])
            raise RuntimeError(
                f"IDE window appears black or empty (mean luma {luma:.3f}). "
                "Software rendering likely failed. See work/idetea.log and work/frames/00-startup.png."
            )

        rec = session.start_record(raw_video, fps)
        rec_t0 = session.rec_started
        audio_offset = time.time() - rec_t0
        log(f"Audio offset {audio_offset:.2f}s")
        for scene in cfg["scenes"]:
            log(f"Scene {scene['id']} ({scene['_duration']:.2f}s)")
            t0 = time.time()
            scene["_start"] = t0 - rec_t0 - audio_offset
            for action in scene["actions"]:
                session.do_action(action)
            # Mid-scene snapshot for the last action visibility check later
            session.screenshot(FRAMES_DIR / f"{scene['id']}.png")
            remaining = scene["_duration"] - (time.time() - t0)
            if remaining > 0:
                time.sleep(remaining)
            elif remaining < -0.4:
                log(f"  (actions ran {-remaining:.1f}s longer than narration)")
        # A short hold so the last frame isn't cut by ffmpeg shutdown.
        time.sleep(0.4)
    finally:
        if rec is not None:
            session.stop_record(rec)
        session.close()

    if not raw_video.is_file() or raw_video.stat().st_size < 1000:
        sys.exit(f"Recording missing or empty: {raw_video}")

    # Scenes whose actions ran past their narration push the later clips back.
    planned = 0.0
    drift = 0.0
    for scene in cfg["scenes"]:
        drift = max(drift, abs(scene.get("_start", planned) - planned))
        planned += scene["_duration"]
    if drift > 0.15:
        log(f"Re-timing narration to the scenes (drift up to {drift:.1f}s)")
        total = place_audio(cfg, narration_wav)
        write_srt(cfg, srt)
    video_dur = duration_of(raw_video)
    log(f"Raw video {video_dur:.2f}s, audio {total:.2f}s")
    mux(raw_video, narration_wav, srt, args.out.resolve(), burn_subs=not args.no_subs, offset=audio_offset,
        extra_style=str(cfg.get("subtitle_style") or ""))
    out_dur = duration_of(args.out)
    out_size = args.out.stat().st_size
    log(f"Wrote {args.out}  ({out_dur:.2f}s, {out_size/1e6:.1f} MB)")
    if out_size > 50 * 1000 * 1000:
        log("WARNING: output is larger than 50 MB")
    # Extract a few stills from the final muxed file too.
    for ts, name in ((1.0, "final-01s"), (min(8, out_dur - 0.2), "final-mid"), (max(0.2, out_dur - 1.5), "final-end")):
        dest = FRAMES_DIR / f"{name}.png"
        run([
            "ffmpeg", "-y", "-ss", f"{ts:.2f}", "-i", str(args.out), "-frames:v", "1", str(dest),
        ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        sys.exit(130)
