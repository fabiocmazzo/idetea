---
name: novo-projeto
description: Use when this project was just created by the TEA IDE ("Novo projeto") and AGENTS.md has an "Initial request" section. Covers the common workflow for starting any project - asking the user the open questions through the IDE panel, the first files, the first commit and handing over a manual test. Pair it with the skill for the stack (novo-projeto-rust, -python, -fastapi, -site) when there is one.
---

# Starting a project created by the TEA IDE

The folder was created by the TEA IDE. It already has `git init`, this skill (and maybe one for the stack) in
`.agents/skills/` and `.claude/skills/`, `.idetea/skills.toml`, and an `AGENTS.md` with the panel instructions and an
**Initial request** section. Nothing else exists yet: you create the project.

## Workflow

1. **Read** `AGENTS.md` (the initial request and the panel instructions) and the stack skill, if there is one.
2. **Ask before creating.** Put the open questions in one interface in the IDE panel, with a sensible default in each
   field. Usually: what the project is for (one sentence), the name shown to people, the license (MIT, Apache-2.0,
   none), and the stack questions from the stack skill. Do not ask what the initial request already answers.
3. **Create the smallest project that runs:** the stack skill says what that means. Add a `README.md` (what it is,
   how to run, how to test) and a `.gitignore` for the stack. Keep `.idetea/ui/` out of git (the IDE already puts a
   `.gitignore` inside it).
4. **Prove it works yourself:** build, run the tests, run the program once. Fix what fails before showing anything.
5. **Hand over a manual test** in the panel: what was created (a `markdown`), what to try (a `steps`), and the three
   buttons from the panel instructions. Put every command in a fenced code block.
6. **On approval, commit** everything with a short message (the skills folders and `.idetea/skills.toml` included),
   and **remove the Initial request section** from `AGENTS.md` in that same commit, so the next session does not
   start the project again. Push only if the user asks.

## Rules

- Talk to the user in the language they use with you, in the chat and in every interface.
- Prefer the stack's own tools to create files (`cargo new`, `uv init`), then adjust.
- Pin nothing to versions you have not checked: use the stack's current stable tools as installed on the machine,
  and say in the README which versions you used.
- Do not install system packages or global tools without asking in the panel first.
- Never put secrets in files. Configuration that needs a secret reads it from the environment, with an
  `.env.example` that has empty values.
