# The IDE TEA manifesto

<p><b>English</b> · <a href="MANIFESTO.pt-BR.md">Português</a> · <a href="MANIFESTO.es.md">Español</a> ·
<a href="MANIFESTO.fr.md">Français</a> · <a href="MANIFESTO.de.md">Deutsch</a> · <a href="MANIFESTO.it.md">Italiano</a> ·
<a href="MANIFESTO.ja.md">日本語</a> · <a href="MANIFESTO.zh-CN.md">中文（简体）</a></p>

An editor is where you spend your day. It should feel like a quiet room, not a shopping mall.

## 1. Fast is a feature

Waiting breaks focus. IDE TEA opens in a blink, opens big files without freezing, and does slow work (search,
git, language servers) away from the screen you are typing in. When something is slow, that is a bug.

## 2. Quiet by default

No account. No telemetry. No pop-ups about features you did not ask for. The network is used only when you ask
for it (push, pull, fetch). Nothing changes under your feet: settings live in a plain TOML file you can read,
version and share.

## 3. Predictable over clever

Every shortcut is listed in one place (`F1`) and can be changed. A panel stays where you put it. Closing something with unsaved changes
always asks. Deleting a file sends it to the trash. Files changed outside the IDE are noticed, and your unsaved
work is never thrown away to "help" you.

## 4. Small inside, complete outside

The core only edits files. The tree, the terminal, git, the debugger and everything else are extensions you can
turn off. A small core is easier to trust, easier to read and harder to break.

## 5. Agents are welcome, and you stay in charge

Coding agents are part of how we work now. IDE TEA gives them a simple, open, file-based way to talk to you:
questions as real forms, terminals they can ask for, helpers in their own worktrees. The rules are always the same:

- **You see it before it runs.** Every command an agent asks for is shown in full and waits for your approval,
  unless you put it on an allow list in *your* config. A project can never give itself permission.
- **Your answer is the answer.** An approval is a decision, not a click to get rid of a dialog. Buttons that
  matter wait until you have done the steps.
- **Plain files, no lock-in.** The protocol is JSON in a folder. Any agent that can read and write files can use
  it, and you can read every message yourself.

## 6. Free, for real

IDE TEA is Apache 2.0 and free of charge, and it will stay that way. No paid tier hiding the good parts.

## 7. Built for different minds

The name has a second meaning: in Portuguese, TEA is the acronym for autism. IDE TEA is made by an autistic
developer, and it shows in what it values: low noise, clear text, things that behave the same way every time,
and room to focus. If that is how your mind works too, welcome. If it is not, you will probably like it anyway.
