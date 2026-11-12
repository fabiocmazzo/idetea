#!/usr/bin/env bash
# One-command demo:  ./demo.sh
# From the repository root:  just demo
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
exec python3 record_demo.py --bin "$BIN" --out "${IDETEA_DEMO_OUT:-$REPO/dist/idetea-demo.mp4}" "$@"
