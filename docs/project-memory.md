# Project memory

Coding agents forget everything between sessions, and each one keeps its own notes where the others can't see them
(Claude Code in `~/.claude/projects/…`, Cursor in `.cursor/rules/`…). IDE TEA proposes a plain-file convention
instead: a memory that belongs to the **project**, lives in git, and any agent that reads `AGENTS.md` or
`CLAUDE.md` can use. It is a convention, not a format: Markdown files, a short frontmatter and an index.

"Interfaces: teach this project's agents" writes the instructions for it into `AGENTS.md`/`CLAUDE.md`, next to
the ones for the Interfaces panel (the text is in [interfaces/agentes.md](interfaces/agentes.md)). You don't need
to create anything: the first agent that learns something worth keeping creates the folder.

## Layout

```
docs/memory/
  README.md              the index: one line per fact
  tests-offline.md       one fact per file
  release-checklist.md
docs/decisions/
  0001-use-sqlite.md     architecture decisions (ADRs), numbered
  0002-…
```

## Facts (`docs/memory/`)

One fact per file: something non-obvious that the next agent (or person) needs, and that the code, the git history
and `AGENTS.md` don't already say. A constraint, a convention, a gotcha, or where something lives outside the
repository.

```markdown
---
name: tests-offline
description: the tests must not use the network; CI runs offline
type: constraint
updated: 2026-11-13
---
The tests run in CI without network access. Use the fixtures in `tests/fixtures/`.

**Why:** the CI runner has no internet access.
```

- `name`: a short slug, the same as the file name.
- `description`: one line, used to decide whether to open the file.
- `type`: `constraint`, `convention`, `gotcha` or `reference`.
- `updated`: the date of the last change. A fact reflects what was true then.

The index, `docs/memory/README.md`, has one line per fact, and agents read it first:

```markdown
# Project memory

- [Tests run offline](tests-offline.md) — no network in CI; use tests/fixtures
- [Release checklist](release-checklist.md) — what to bump and check before tagging
```

The rules the agents get: read the index at the start of a task; check that a file, function or flag a fact names
still exists before relying on it; update a fact instead of writing a duplicate; delete a fact that turned out
wrong, with its index line.

## Decisions (`docs/decisions/`)

Architecture decisions (a library, a format, a structure that is hard to change) go in as
[ADRs](https://adr.github.io/) in the [MADR](https://adr.github.io/madr/) template, one per file, numbered:

```markdown
# 0003. Store the cache in SQLite

Status: accepted (2026-11-13)

## Context

The cache outgrew a JSON file: …

## Options

- SQLite: …
- A JSON file per entry: …

## Decision

SQLite, through `rusqlite` with the bundled library.

## Consequences

One more C dependency; the cache survives crashes; …
```

An accepted ADR is not rewritten to change the decision: a new one replaces it, and the old one's status becomes
"Superseded by 0007".

## Good to know

- **Language:** agents write these files in the language of the project's documentation.
- **Secrets:** the folder goes to git. The instructions forbid credentials and personal data, but review what goes
  into commits as you would any other change.
- **Out of git:** if you don't want the memory in the repository, add `docs/memory/` and `docs/decisions/` to
  `.gitignore`. It then stays on your machine, and other people and machines don't see it.
- **Not included, on purpose:** no database, embeddings or memory server. For the size of a project's memory, the
  index and a text search are enough. The IDE doesn't write facts by itself either: agents and people do.
