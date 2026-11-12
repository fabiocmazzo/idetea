# One-command demo:  make demo
# From the repository root, the same thing is:  just demo
# Optional:           make narration
# Swap TTS:           TTS_ENGINE=openrouter OPENROUTER_API_KEY=... make demo
#                     (the key is read from the environment; do not commit it)

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
REPO := $(abspath $(ROOT)/../..)
IDETEA_BIN ?= $(REPO)/target/release/idetea
OUT ?= $(REPO)/dist/idetea-demo.mp4
PYTHON ?= python3

export IDETEA_BIN IDETEA_DEMO_OUT=$(OUT)

.PHONY: demo narration build clean help

help:
	@echo "make demo        record dist/idetea-demo.mp4 (same as: just demo)"
	@echo "make narration   generate speech audio only"
	@echo "make build       cargo build --release the IDE TEA binary"
	@echo "make clean       remove work/ intermediates (not the voices or binary)"
	@echo "TTS: TTS_ENGINE=piper (default without a key) | openrouter | elevenlabs | openai"
	@echo "OpenRouter voice defaults to sage (OPENROUTER_VOICE)."

demo:
	$(ROOT)/demo.sh

narration:
	$(PYTHON) $(ROOT)/record_demo.py --narration-only

build $(IDETEA_BIN):
	cargo build --release --locked -p tea-app --manifest-path $(REPO)/Cargo.toml

clean:
	rm -rf $(ROOT)/work
