---
name: novo-projeto-site
description: Use when creating a new website from the TEA IDE "Site" template (AGENTS.md initial request mentions a site, landing page, static pages or a small web app without a backend). Says what the smallest working site looks like - plain HTML, CSS and JavaScript with no build step by default - and what to ask the user first.
---

# New site

Use together with the `novo-projeto` skill (the common workflow). If the `frontend-design` skill is installed, follow
it for the look.

## Ask first (in the same panel interface as the common questions)

- **Kind:** one page, several static pages, or a small interactive app in the browser. Default: one page.
- **Build step:** none (plain files, recommended to start), or a tool the user names (Vite, Astro…). Only add a
  tool when the user asks for one.
- **Language of the content** (often not the one of the code).

## Create

- `index.html` (valid HTML5, `lang` set, `<meta name="viewport">`, a real `<title>`), `style.css`, and `script.js`
  only if something is interactive.
- Content from the initial request, not lorem ipsum. Images in `assets/`, with `alt` text.
- Works at phone width (no horizontal scroll at 360 px) and in light and dark (`prefers-color-scheme`).
- No external scripts or fonts unless the user asked; then pin exact versions.
- `.gitignore` only if there is a build step (`node_modules/`, `dist/`).

## Prove it works

Open it the way the user will: in the IDE panel, a markdown link `[open](idetea:open/index.html)` opens it in the
system browser. Check the browser console has no errors. If there is a build step, run the build and open the
output.

## Manual test to hand over

Steps like: open the site with the `idetea:open/index.html` link; resize the window to phone width; switch the system
to dark mode; click what is interactive.
