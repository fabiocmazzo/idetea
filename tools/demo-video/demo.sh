#!/usr/bin/env bash
# One-command demo:  ./demo.sh            (the overview, scenes.yaml)
# One feature video:  ./demo.sh tables     (scenes/tables.yaml -> dist/idetea-tables.mp4)
# Every video:        ./demo.sh all
# From the repository root:  just demo [name|all]
set -euo pipefail
cd "$(dirname "$0")"
REPO="$(cd ../.. && pwd)"
BIN="${IDETEA_BIN:-$REPO/target/release/idetea}"

# OpenRouter (voice sage) when a key is already in the environment and the
# engine was not chosen. Otherwise Piper, which is local and needs no secret.
# The key is never printed.
if [ -z "${TTS_ENGINE:-}" ]; then
  if [ -n "${OPENROUTER_API_KEY:-}" ]; then
    TTS_ENGINE=openrouter
  else
    TTS_ENGINE=piper
  fi
  export TTS_ENGINE
fi

case "$TTS_ENGINE" in
  piper|local) ./fetch-piper.sh ;;
esac

if [ ! -x "$BIN" ]; then
  if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
  fi
  cargo build --release --locked -p tea-app --manifest-path "$REPO/Cargo.toml"
fi

echo "TTS engine: $TTS_ENGINE"
if [ "$TTS_ENGINE" = "openrouter" ]; then
  echo "OpenRouter voice: ${OPENROUTER_VOICE:-sage}"
fi

mkdir -p "$REPO/dist"

# First argument: a scene file name from scenes/ (without .yaml), "all", or nothing
# (the overview). Anything else is passed to record_demo.py.
NAME=""
if [ $# -gt 0 ] && [ "${1#-}" = "$1" ]; then
  NAME="$1"
  shift
fi

record() {  # record <name> [record_demo.py options]
  local name="$1"
  shift
  local scenes="scenes/$name.yaml"
  if [ ! -f "$scenes" ]; then
    echo "No such video: $name (see tools/demo-video/scenes/)" >&2
    exit 2
  fi
  python3 record_demo.py --bin "$BIN" --scenes "$scenes" --out "$REPO/dist/idetea-$name.mp4" "$@"
}

case "$NAME" in
  "")
    exec python3 record_demo.py --bin "$BIN" --out "${IDETEA_DEMO_OUT:-$REPO/dist/idetea-demo.mp4}" "$@"
    ;;
  all)
    python3 record_demo.py --bin "$BIN" --out "$REPO/dist/idetea-demo.mp4" "$@"
    for f in scenes/*.yaml; do
      n="$(basename "$f" .yaml)"
      record "$n" "$@"
    done
    ;;
  *)
    record "$NAME" "$@"
    ;;
esac
