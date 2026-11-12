#!/usr/bin/env bash
# Download the Piper binary and the en_US-lessac-medium voice when they are missing.
# Neither is committed. See README.md.
set -euo pipefail
cd "$(dirname "$0")"

arch="$(uname -m)"
case "$arch" in
  x86_64) piper_asset="piper_linux_x86_64.tar.gz" ;;
  aarch64|arm64) piper_asset="piper_linux_aarch64.tar.gz" ;;
  *)
    echo "Piper download supports x86_64 and aarch64 Linux (this machine is ${arch})." >&2
    echo "Place a piper binary at tools/piper/piper, or set TTS_ENGINE=openrouter." >&2
    exit 1
    ;;
esac

if [ ! -x tools/piper/piper ]; then
  echo "Downloading Piper (${piper_asset})"
  mkdir -p tools
  tmp="$(mktemp)"
  curl -fsSL "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/${piper_asset}" -o "$tmp"
  tar -xzf "$tmp" -C tools
  rm -f "$tmp"
  chmod +x tools/piper/piper
fi

voice_name="${PIPER_VOICE:-en_US-lessac-medium}"
if [ ! -f "voices/${voice_name}.onnx" ] || [ ! -f "voices/${voice_name}.onnx.json" ]; then
  if [ "$voice_name" != "en_US-lessac-medium" ]; then
    echo "PIPER_VOICE=${voice_name} is not downloaded automatically." >&2
    echo "Place voices/${voice_name}.onnx and voices/${voice_name}.onnx.json yourself." >&2
    exit 1
  fi
  echo "Downloading Piper voice ${voice_name}"
  mkdir -p voices
  base="https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium"
  curl -fsSL -o "voices/${voice_name}.onnx" "${base}/${voice_name}.onnx"
  curl -fsSL -o "voices/${voice_name}.onnx.json" "${base}/${voice_name}.onnx.json"
fi
