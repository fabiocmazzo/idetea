---
name: novo-projeto-python
description: Use when creating a new Python project from the TEA IDE "Python" template (AGENTS.md initial request mentions Python, and it is not a FastAPI web API). Says what the smallest runnable Python project looks like - uv, src layout, pytest, ruff - and what to ask the user first.
---

# New Python project

Use together with the `novo-projeto` skill (the common workflow). For a web API, use `novo-projeto-fastapi` instead.

## Ask first (in the same panel interface as the common questions)

- **Kind:** command-line program, library, or script/notebook-style analysis. Default: command-line program.
- **Tool:** `uv` if it is installed (check with `uv --version`), otherwise plain `venv` + `pip`. Only ask if neither
  is obvious.

## Create

- With uv: `uv init --package` in the current folder (git already exists; do not let it create another repository).
  Without uv: `pyproject.toml` by hand, `python -m venv .venv`.
- **src layout:** `src/<package>/__init__.py`, `src/<package>/__main__.py` (or a `[project.scripts]` entry point)
  doing one real, small thing from the initial request.
- Tests with **pytest** in `tests/`, at least one that calls the code and one that runs the entry point.
- **ruff** for lint and format, configured in `pyproject.toml`.
- `.gitignore` with `.venv/`, `__pycache__/`, `.pytest_cache/`, `.ruff_cache/`, `dist/`.
- The Python version: the one installed (`python3 --version`), written as `requires-python` and in the README.

## Prove it works

```sh
uv run pytest            # or: .venv/bin/python -m pytest
uv run ruff check .
uv run ruff format --check .
uv run <command> --help  # the entry point
```

All of them must pass before the manual test.

## Manual test to hand over

Steps like: run the command with an example and see the expected output; run the tests; read the README.
