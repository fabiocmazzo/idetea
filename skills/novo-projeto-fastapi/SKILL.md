---
name: novo-projeto-fastapi
description: Use when creating a new web API with FastAPI from the TEA IDE "FastAPI" template (AGENTS.md initial request mentions FastAPI or an HTTP API in Python). Says what the smallest runnable FastAPI project looks like - uv, app package, health route, tests with TestClient, settings from the environment - and what to ask the user first.
---

# New FastAPI project

Use together with the `novo-projeto` skill (the common workflow). It builds on the Python conventions (uv, src
layout, pytest, ruff); read `novo-projeto-python` too if it is installed.

## Ask first (in the same panel interface as the common questions)

- **First resource:** the one thing the API does first (e.g. "tasks": list and create). Default: taken from the
  initial request.
- **Storage:** in memory for now, SQLite, or PostgreSQL. Default: in memory (a database is a later, separate step).
- **Port:** default 8000.

## Create

- `uv init --package`, then add `fastapi` and `uvicorn[standard]`; `pytest` and `httpx` as dev dependencies.
- `src/<package>/main.py` with `app = FastAPI(...)`, a `GET /health` returning `{"status": "ok"}`, and the first
  resource in its own router module (`routers/<resource>.py`) with Pydantic models for input and output.
- Settings read from the environment (a small `Settings` with defaults; `pydantic-settings` only if needed), and an
  `.env.example` with empty values for anything secret. Never commit a real `.env`.
- Tests with `fastapi.testclient.TestClient`: health, and create + list of the first resource.
- A run command in the README: `uv run uvicorn <package>.main:app --reload --port 8000`.

## Prove it works

```sh
uv run pytest
uv run ruff check .
uv run uvicorn <package>.main:app --port 8000 &   # then:
curl -s localhost:8000/health
kill %1
```

Stop the server you started before handing over.

## Manual test to hand over

Steps like: start the server with the README command; open `http://localhost:8000/docs` and try the first resource
there; run the tests.
