# Langages

[English](languages.md) · **Français** · [Deutsch](languages.de.md) · [Italiano](languages.it.md) · [日本語](languages.ja.md) · [中文（简体）](languages.zh-CN.md)

Ce qu'IDE TEA fait pour chaque langage, et ce qu'il ne fait pas. L'IDE intègre des outils que vous installez
vous-même (compilateur, serveur de langage, débogueur) ; il ne les télécharge jamais. Sans eux, les fichiers
s'ouvrent quand même avec des couleurs et l'édition fonctionne ; seules les parties qui ont besoin de l'outil
manquent, et l'IDE dit quoi installer.

Pour la liste complète des fonctionnalités, voir [features.fr.md](features.fr.md).

## Vue d'ensemble

| Langage | Fichiers | Serveur de langage (commande par défaut) | Tâches (`Ctrl+Shift+B`) | Débogage (`F5`) |
|---|---|---|---|---|
| Rust | `.rs` | `rust-analyzer` | cargo | gdb, compile avec cargo |
| C / C++ | `.c` `.h` `.cc` `.cpp` `.cxx` `.hpp` `.hh` | `clangd` | cibles de Makefile | gdb, avec une section `[debug]` |
| Python | `.py` `.pyi` | `pyright-langserver --stdio` | — | — |
| TypeScript / JavaScript | `.ts` `.tsx` `.js` `.jsx` | `typescript-language-server --stdio` | scripts de `package.json` | — |
| Go | `.go` | `gopls` | — | — |
| Lua | `.lua` | `lua-language-server` | — | — |
| Zig | `.zig` `.zon` | `zls` | étapes de `build.zig` | gdb, compile avec `zig build` |
| [Ziex](https://ziex.dev) | `.zx` `.mdzx` | `zx lsp` | étapes de `build.zig` | — |
| [Hare](#hare) | `.ha` | `hare-lsp -S` | cibles de Makefile | — |
| [MoonBit](#moonbit) | `.mbt` `.mbti` `.mbt.md` | `moon lsp` | `moon check`, `build`, `test`, `fmt --check` | gdb, compile avec `moon build --target native -g` (expérimental) |

Ce qu'un serveur de langage offre (complétion, renommage, mise en forme…) dépend du serveur : l'IDE utilise ce qu'il
annonce. Chaque langage a des couleurs ; Hare, MoonBit et Ziex apportent leur propre grammaire avec l'IDE.

## Hare

[Hare](https://harelang.org) est un petit langage système. Son serveur de langage est jeune, et l'IDE le dit au lieu
de le cacher.

**Installation**

- `hare` depuis votre distribution (sous Arch : `pacman -S hare`).
- `hare-lsp` n'est pas livré avec Hare. Compilez-le depuis [son dépôt](https://git.sr.ht/~whynothugo/hare-lsp)
  (il a besoin de la bibliothèque [`hare-json`](https://git.sr.ht/~sircmpwn/hare-json) ; sous Arch elle est dans
  l'AUR), puis placez l'exécutable dans votre `PATH`, par exemple `~/.local/bin` :

  ```sh
  make build PREFIX=/usr          # PREFIX : où la bibliothèque standard de Hare est installée
  install -Dm755 hare-lsp -t ~/.local/bin/
  ```

**Vérifié le 2026-11-09** avec Hare 0.26.0.1 et `hare-lsp` `e3db7b8` :

| | |
|---|---|
| Couleurs | oui |
| Survol | oui |
| Aller à la définition (`F12`), à travers les modules | oui |
| Plan et symboles dans le fichier (`Ctrl+Shift+O`) | oui |
| Erreurs de syntaxe | oui |
| Erreurs de type et autres vérifications sémantiques | non (pas encore dans le serveur) |
| Complétion, références, renommage, mise en forme | non (pas encore dans le serveur) |
| Inlay hints | le serveur les a ; l'IDE n'affiche pas encore les inlay hints |
| Tâches | vos cibles de Makefile ; un Makefile seul ne signifie pas Hare, donc l'IDE n'ajoute pas de tâches `hare`. Ajoutez les vôtres dans `[tasks]` |

**Problème connu :** l'analyseur propre à Hare (`hare::parse`, utilisé par `hare-lsp`) plante sur toute instruction
à qui il manque son `;`, par exemple `let x = 1` suivi d'une autre ligne. L'IDE vous le dit la première fois et
relance le serveur de lui-même quand vous arrêtez de taper.

**Pour essayer :** ouvrez [`examples/languages/hare`](../examples/languages/hare) et lancez `check` depuis les tâches.

## MoonBit

[MoonBit](https://www.moonbitlang.com) livre son serveur de langage avec la chaîne d'outils, donc il n'y a rien
d'autre à installer.

**Installation :** l'[installateur officiel](https://www.moonbitlang.com/download). Il place `moon` dans
`~/.moon/bin` (ou `$MOON_HOME/bin`) ; l'IDE regarde là aussi, donc cela fonctionne même ouvert depuis le menu des
applications.

**Vérifié le 2026-11-09** avec `moon 0.1.20261104` :

| | |
|---|---|
| Couleurs, y compris l'interpolation de chaînes | oui |
| Fichiers d'interface `.mbti` | oui |
| Survol | oui |
| Erreurs de type | oui |
| Complétion, aller à la définition, mise en forme via le serveur | ce que `moon lsp` offre ; pas encore vérifié un par un |
| Tâches | `moon check`, `moon build`, `moon test`, `moon fmt --check` |
| Débogage (`F5`) | expérimental : backend natif avec gdb, voir plus bas |
| `.mbt.md` (Markdown avec du code MoonBit) | couleurs dans les blocs ` ```mbt ` et ` ```moonbit `, survol à l'intérieur ; `moon test` exécute les blocs ` ```mbt check ` |

Les tâches s'exécutent dans le dossier du `moon.mod` le plus proche du fichier ouvert, donc chaque module d'un
espace de travail lance les siennes ; sans fichier ouvert, le `moon.mod` de la racine du projet est utilisé. L'ancien
`moon.mod.json` n'active pas les tâches : MoonBit demande de le migrer avec `moon fmt`.

**Débogage** (vérifié le 2026-11-10, Linux, gdb 18) : avec un `moon.mod` à la racine du dossier ouvert et sans
build `[debug]` dans la configuration, `F5` lance `moon build --target native -g` et débogue le programme
compilé. Les points d'arrêt dans les fichiers `.mbt`, le pas à pas, la pile des appels avec les bons fichiers et les
bonnes lignes, et les valeurs des nombres et des autres variables simples fonctionnent. Ce qui reste rugueux :

- Les noms de fonctions dans la pile des appels sont les noms compilés, par exemple `_M0FP26idetea7example7sum__to`
  pour `sum_to`.
- Les chaînes s'affichent en octets UTF-16 (`"M\000o\000o\000n..."`).
- Juste après un pas, une variable peut s'afficher comme `<optimized out>`.
- Seulement le backend natif : les programmes Wasm et JavaScript ne peuvent pas être débogués depuis l'IDE.

Dans les fichiers `.mbt.md`, du Markdown autour et du MoonBit à l'intérieur des blocs de code ; l'aperçu Markdown
fonctionne comme pour n'importe quel `.md`. L'IDE y utilise une petite grammaire Markdown à lui, donc le reste du
texte a moins de couleurs que dans un `.md` ordinaire.

**Pour essayer :** ouvrez [`examples/languages/moonbit`](../examples/languages/moonbit), survolez `sum_to` et lancez
`moon test` depuis les tâches ; `sum.mbt.md` a un test dans son Markdown.

## Un langage qui n'est pas dans la liste

Trois choses dans la configuration couvrent la plupart des langages, sans modifier l'IDE :

```toml
# ~/.config/idetea/config.toml, ou .idetea/config.toml dans le projet
[lsp.servers.mylang]
command = "mylang-lsp"
args = ["--stdio"]
languages = { ml = "mylang" }    # extension de fichier = languageId envoyé au serveur

[tasks]
"mylang test" = "mylang test"
```

Pour les couleurs, placez une grammaire `.sublime-syntax` dans `~/.config/idetea/syntaxes/`. Une grammaire à cet
endroit remplace aussi celle intégrée pour la même extension, et `[lsp.servers.<name>]` peut modifier ou couper
(`enabled = false`) n'importe quel serveur du tableau ci-dessus.

Si vous utilisez un langage que les autres IDE laissent de côté et que vous aimeriez le voir intégré, ouvrez un
ticket en disant sur quel projet vous l'essayeriez.
