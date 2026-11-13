# IDE TEA : fonctionnalités et raccourcis

[English](features.md) · **Français** · [Deutsch](features.de.md) · [Italiano](features.it.md) · [日本語](features.ja.md) · [中文（简体）](features.zh-CN.md)

La liste complète, pour le lecteur technique. Pour la version courte, voir le [README](../README.fr.md).

> Avant la 1.0. Compilé et testé sous Linux, macOS (Apple Silicon) et Windows (x86_64).

## Éditeur

- Ouvrir un fichier par son nom (`Ctrl+P`) avec une recherche floue ; `main.rs:42` saute directement à la ligne.
- Aller à un symbole (fonction, type, méthode…) dans le fichier (`Ctrl+Shift+O`) ou dans le projet (`Ctrl+T`).
- Onglets avec un marqueur de modification ; ouvrir (`Ctrl+O`), enregistrer et enregistrer sous, avec la boîte de
  dialogue de fichiers du système.
- Coloration syntaxique pour environ 200 langages et types de fichiers (TOML, `.env`, INI, Dockerfile, TypeScript, Zig,
  [Ziex](https://ziex.dev) `.zx`…) ; thèmes clair et sombre (plus les thèmes d'iced : Nord, Dracula…). D'autres
  langages : placez des fichiers `.sublime-syntax` dans `~/.config/idetea/syntaxes/` (lus à l'ouverture de l'IDE).
- Rechercher et remplacer, aller à la ligne, annuler/rétablir, curseurs multiples, repli du code et un mode vim
  facultatif.
- Zoom temporaire de l'éditeur (`Ctrl+=` / `Ctrl++`, `Ctrl+-`, `Ctrl+0` ou `Ctrl+molette`), partagé par les onglets
  ouverts et les divisions ; le terminal et la taille de base configurée ne changent pas.
- Aperçu Markdown et SVG à côté de l'éditeur (`Ctrl+Shift+V` ou le bouton de la barre d'onglets), mis à jour pendant
  la frappe.
- Les images (png, jpg, gif, webp, bmp, ico…) s'ouvrent dans un onglet, avec zoom à la molette et déplacement.
- Le PDF s'ouvre dans un onglet (rendu par `hayro`, Rust pur), avec zoom et ajustement à la largeur.
- Conserve les CRLF. Un fichier qui n'est pas en UTF-8 n'est pas ouvert (avec un avertissement), au lieu d'être
  corrompu.
- Gros fichiers : un fichier de 50 Mo s'ouvre en moins de 0,1 s. Un fichier d'une seule ligne géante (JSON ou JS
  minifié) s'ouvre dans un mode léger, sans couleurs et avec retour à la ligne, pour que la fenêtre ne se fige pas.
- Facultatif à l'enregistrement : supprimer les espaces en fin de ligne, saut de ligne final, mettre en forme avec le
  serveur de langage. Un `.editorconfig` dans le projet l'emporte.

## Projet

- **Arborescence :** créer, renommer et supprimer (vers la corbeille), révéler le fichier courant, couleurs git
  (modifié, nouveau, conflit, ignoré).
- **Recherche dans le projet** au fil de la frappe, en respectant `.gitignore`, avec expression régulière, mot entier
  et respect de la casse.
- **Menus du clic droit** sur l'arborescence, les onglets et les fichiers du panneau git.
- **Tâches** (`Ctrl+Shift+B`) : vos propres commandes, depuis la configuration, plus ce que l'IDE trouve dans le projet
  (cargo, scripts de `package.json`, cibles de Makefile, étapes de `build.zig`), avec la sortie dans le terminal. Les
  modules MoonBit reçoivent aussi `moon check`, `build`, `test` et `fmt --check`, lancés depuis le `moon.mod` le plus
  proche du fichier actif.
- **Nouveau projet** (« Projet : nouveau… » ou `idetea new <folder>`) : dossier, git, les compétences d'agent d'un
  modèle (vide, Rust, site web, Python, FastAPI) et une demande initiale dans `AGENTS.md`. Voir
  [skills.fr.md](skills.fr.md).

## Langages (LSP)

- Erreurs et avertissements (soulignements ondulés et survol du diagnostic dans l'éditeur, panneau Problèmes et un
  compteur dans la barre d'état), complétion, aller à la définition, mettre en forme.
- Aller à l'implémentation (`Ctrl+F12`) et au type ; avec plusieurs résultats, la liste va dans le panneau.
- Rechercher les références (`Shift+F12`, ou `Ctrl+clic` sur la déclaration) groupées par fichier, et retour/avancer
  après chaque saut (`Alt+←` / `Alt+→`, ou les boutons latéraux de la souris).
- Hiérarchie d'appels (`Alt+Shift+H`) : qui appelle une fonction, et ce qu'elle appelle, sous forme d'arbre qui se
  charge à mesure que vous l'ouvrez.
- Panneau Plan (`Alt+7`) : fonctions, structs, méthodes et champs du fichier, en indiquant ce qui est public, privé ou
  restreint (`pub(crate)`, `protected`…), avec le symbole sous le curseur mis en évidence.
- Fil d'Ariane au-dessus de l'éditeur : `projet › dossier › fichier › impl App › fn update`, le chemin et les symboles
  où se trouve le curseur. Un dossier montre le fichier dans l'arborescence ; un symbole y saute.
  `[editor] breadcrumbs = false` le coupe.
- Police du code : JetBrains Mono est fournie ; toute police installée fonctionne (`[editor] font = "Fira Code"`),
  avec ses ligatures. L'épaisseur et la hauteur de ligne aussi (`font_weight`, `line_height`), également dans
  Paramètres → Apparence. L'éditeur, ses numéros de ligne et le terminal utilisent la même police.
- Icônes de fichiers par langage et par type (le jeu Seti de VS Code, intégré, plus les icônes propres à l'IDE pour
  Hare et MoonBit) dans l'arborescence, les onglets, Ctrl+P, le fil d'Ariane, les résultats de recherche et le panneau
  git. `[ui] file_icons = "none"` (ou Paramètres → Apparence) revient à une seule icône pour chaque fichier.
- Shift+clic étend la sélection depuis le dernier clic (ou depuis le curseur, après l'avoir déplacé au clavier).
- Mot par mot : `Ctrl+Left/Right` déplace (avec `Shift`, sélectionne), `Ctrl+Backspace/Delete` efface. `Home` va au
  premier caractère de la ligne, puis à la colonne 0. `Shift+Tab` retire l'indentation.
- Raccourcis de VS Code là où ils différaient : replier/déplier le bloc sous le curseur avec `Ctrl+Shift+[` / `]`,
  rouvrir le dernier onglet fermé avec `Ctrl+Shift+T`, panneau inférieur avec `Ctrl+J`, terminal avec `` Ctrl+` `` ou
  `Ctrl+'`.
- Déjà configurés : rust-analyzer, clangd, pyright, typescript-language-server, gopls, lua-language-server, zls (Zig),
  `zx lsp` ([Ziex](https://ziex.dev) ; sans la CLI `zx`, un projet Ziex construit son propre serveur avec `zig build`),
  `hare-lsp -S` et `moon lsp`. Installez seulement le serveur ; vous pouvez changer la commande, le désactiver ou en
  ajouter d'autres dans la configuration. Le serveur de Hare se limite aux diagnostics d'analyse syntaxique, à la définition, au
  survol et aux symboles du document ; la prise en charge de MoonBit couvre `.mbt`, `.mbti` et `.mbt.md` (Markdown
  avec des blocs MoonBit), selon les capacités de la chaîne d'outils installée. Le détail par langage :
  [languages.fr.md](languages.fr.md).

## Terminal

- Terminal intégré (alacritty) dans le panneau inférieur, avec les couleurs et la police du thème.
- Plusieurs shells dans des onglets (« Terminal 1 », « Terminal 2 »…), plus l'onglet propre à la tâche ; `+` en ouvre
  un, `×` le ferme, `exit` ferme l'onglet.
- Les raccourcis du shell (`Ctrl+C`, `Ctrl+W`…) vont au shell ; les raccourcis `Ctrl+Shift` de l'IDE continuent de
  fonctionner.

## Git

- Branche, commits en avance/en retard et nombre de modifications dans la barre d'état.
- Panneau avec indexer/désindexer par fichier ou pour tout, diff coloré, valider, abandonner les modifications (avec
  confirmation).
- Pousser, tirer et récupérer. L'IDE n'attend jamais un mot de passe : il utilise votre agent ssh ou votre assistant
  d'identifiants git.
- Utilise le `git` installé, donc votre configuration, vos hooks et vos identifiants s'appliquent.

## Débogage

- Points d'arrêt d'un clic sur le numéro de ligne (`F9`) ; ils suivent le texte pendant l'édition.
- `F5` compile et lance : dans les projets Rust, il utilise `cargo build` et trouve l'exécutable ; dans les projets
  Zig, `zig build` et les exécutables de `zig-out/bin` ; dans les modules MoonBit,
  `moon build --target native -g` (expérimental, voir [languages.fr.md](languages.fr.md#moonbit)) ; ailleurs, une
  section `[debug]` de la configuration indique le programme, les arguments et la commande de compilation.
- Pas à pas principal, détaillé et sortant (`F10`, `F11`, `Shift+F11`), suspendre et arrêter, avec la ligne courante
  marquée dans l'éditeur.
- Panneau avec la pile des appels (un clic va au cadre), les variables en arbre, et la sortie du programme.
- Utilise `gdb` 14 ou plus récent (Rust, C, C++, Zig et, à titre expérimental, MoonBit).

## Travailler avec les agents

Tout cela, ce sont de simples fichiers dans le dossier `.idetea/` du projet, donc tout agent qui lit et écrit des
fichiers peut s'en servir. « Interfaces : enseigner aux agents de ce projet » écrit les instructions dans `AGENTS.md`
et `CLAUDE.md`.

- **Panneau Interfaces** (`.idetea/ui/<name>.json`) : l'agent écrit un document, l'IDE l'affiche comme un formulaire
  (texte, Markdown, select, case à cocher, champ, boutons, étapes pour les tests manuels, fichiers à éditer, images,
  galeries, sprites animés, sons avec forme d'onde et niveau de crête) et écrit votre réponse dans
  `.idetea/ui/responses/`. Le panneau indique si l'agent attend encore. Format :
  [docs/interfaces/formato.md](interfaces/formato.md) (en portugais pour l'instant).
- **Terminaux d'agent** (`.idetea/agents/<name>.json`) : l'agent demande un terminal avec une commande ; l'IDE montre
  la demande entière dans un onglet « demande de terminal » et ne l'exécute qu'après votre autorisation. La
  progression va dans `<name>.status.json` (`pending`, `running`, `exited` avec le code, `refused`, `closed`,
  `invalid`, `lost`) ; supprimer la demande arrête le terminal (si la commande est déjà terminée, l'onglet reste
  jusqu'à ce que vous le fermiez).
- **Liste d'autorisation :** `[agents] allow = ["npm run dev", "claude *"]` dans votre configuration utilisateur
  exécute ces commandes sans demander : la commande exacte, ou un préfixe suivi seulement d'arguments simples (pas de
  `;`, `&`, `|`, `$`, accents graves, redirections…). Dans une configuration de projet, elle est ignorée, avec un
  avertissement.
- **Lancer un agent** (`Ctrl+Shift+A`, aussi sur l'écran d'accueil) : les agents de code trouvés dans le PATH
  (Claude Code, Codex, Gemini CLI, OpenCode, Aider, Cursor Agent) et vos propres `[agents.launchers]`, dans un onglet
  de terminal, dans le dossier du projet ou dans un nouveau worktree (`.idetea/worktrees/launch-<branche>`, retiré à
  la fermeture de l'onglet s'il est propre ; la branche reste). `[agents] default` vient en premier. Les lanceurs ne
  comptent que dans votre config utilisateur. Si `AGENTS.md`/`CLAUDE.md` n'a pas les instructions de cet IDE, il
  demande d'abord : les écrire, juste ouvrir, ou ne plus demander dans ce projet.
- **Assistants dans leur propre worktree :** `"worktree": "<branch>"` exécute la commande dans
  `.idetea/worktrees/<name>` sur cette branche. Chaque commande reçoit `IDETEA_ANSWER`, un fichier du projet principal
  pour la réponse. Supprimer la demande retire le worktree s'il n'a rien de non validé ; la branche reste toujours.
- **Commandes pour les agents :** `idetea ask` (une question dans le panneau, en attendant la réponse), `idetea run`
  (un terminal, en attendant la fin et la réponse de l'assistant), `idetea status` (ce que vous avez d'ouvert, les
  modifications non enregistrées, le curseur), `idetea open file:line` (vous montre un fichier), `idetea history` (ce
  que vous avez répondu dans le panneau) et `idetea mascot` (état de la mascotte et bulle, en temps réel). Elles font
  le travail de fichiers ci-dessus en un appel, avec les mêmes approbations.
- **Mascotte réactive (Tea) :** une carte tasse en pixel art dans le coin inférieur droit (repliable dans la barre
  d'état, ou basculée depuis la palette de commandes) qui surveille `.idetea/` en temps réel (`idle`, `working`,
  `thinking`, `attention`, `celebrating`, `error`, `sleeping`, plus les expressions interactives `curious` et
  `annoyed` au survol, au double-clic ou quand on la pousse). Chaque situation a plusieurs courtes répliques ; la même
  n'est pas montrée deux fois de suite. Elle remarque aussi un enregistrement de fichier, la fin ou l'échec de la
  compilation du débogueur, une longue inactivité (elle s'assoupit, et se réveille à votre retour) et l'heure de la
  journée, y compris une douce invitation à se reposer tard le soir. Cliquer sur la bulle ouvre le panneau Interfaces,
  Terminal ou Problèmes et ne prend jamais le focus. Avec `[mascot] tips = true` (et `chatty` différent de `quiet`),
  elle mentionne de temps en temps un vrai raccourci. `[mascot] chatty` vaut `quiet`, `normal` ou `chatty`. Les agents
  ou les hooks peuvent aussi la piloter directement via
  `idetea mascot <state> [--text "…"] [--source <name>] [--ttl <secs>]`, et des packs de sprites personnalisés
  fonctionnent via `.idetea/mascot/mascot.toml` ou `~/.config/idetea/mascots/<pack>/mascot.toml`
  (`[mascot] enabled = true`).
- **Sélectionner et copier le texte :** « Sélectionner le texte » en haut d'une interface (ouverte ou dans
  l'historique) montre ses textes dans un éditeur en lecture seule : sélection à la souris, copie avec Ctrl+C ;
  « Afficher mis en forme » revient à l'affichage formaté.
- **Historique des questions :** l'IDE conserve chaque question, votre réponse, votre commentaire et le moment où
  l'agent l'a lue, dans `.idetea/history/` (hors de git, jusqu'à ce que vous l'effaciez). Le panneau **Historique
  Interfaces** à gauche (masqué par défaut) liste d'abord les questions ouvertes, puis les questions répondues par
  jour, avec recherche ; une entrée s'ouvre en lecture seule, telle que vous y avez répondu.
- **Corriger ce projet** (« Interfaces : corriger ce projet ») : remet `.idetea/` dans un état utilisable après un
  plantage ou une mauvaise édition. **Compétences :** `idetea skills` installe des compétences d'agent depuis un dépôt
  git dans le projet ([skills.fr.md](skills.fr.md)).

Détails (en portugais pour l'instant) : [terminais-de-agente.md](terminais-de-agente.md). Compétences :
[skills.fr.md](skills.fr.md).

## Interface

- Palette de commandes avec chaque commande et son raccourci (`Ctrl+Shift+P`).
- Aide (`F1`) avec tous les raccourcis actuels et un court guide.
- Paramètres (`Ctrl+,`) : apparence, extensions, raccourcis (enregistre la touche que vous pressez) et serveurs de
  langage.
- Barre d'activités pour afficher et masquer les panneaux ; les panneaux se redimensionnent à la souris.
- Écran d'accueil quand aucun fichier n'est ouvert : nouveau fichier, ouvrir un fichier ou un dossier, fichiers et
  projets récents, et les principaux raccourcis. Taper ou `Ctrl+N` ramène un éditeur sans titre.
- Panneaux en îlots arrondis sur un fond plus sombre, avec de fines barres de défilement en pilule qui s'estompent
  quand la souris s'éloigne ; `[ui] style = "classic"` garde tout sur un seul fond, séparé par des lignes. La palette,
  les menus et les boîtes de dialogue entrent en douceur ; `[ui] animations = false` coupe cela.
- Interface en portugais, anglais, espagnol, français, allemand, italien, japonais et chinois simplifié
  (`[ui] language`, ou celle du système). Cela prend effet après le redémarrage de l'IDE.

## Ne pas perdre le travail

- Fermer un onglet ou quitter avec des modifications non enregistrées demande toujours confirmation d'abord.
- Supprimer un fichier l'envoie à la corbeille du système.
- Les fichiers modifiés hors de l'IDE (terminal, `cargo fmt`, un autre éditeur, `git checkout`) sont signalés :
  l'arborescence et git s'actualisent, les onglets sans modifications se rechargent, et les onglets avec des
  modifications sont conservés, avec un avertissement.

## Principaux raccourcis

| Raccourci | Action |
|---|---|
| `Ctrl+P` | Ouvrir un fichier par son nom |
| `Ctrl+Shift+P` | Palette de commandes |
| `F1` | Aide : tous les raccourcis et le guide |
| `Ctrl+,` | Paramètres |
| `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S` | Ouvrir / enregistrer / enregistrer sous |
| `Ctrl+B` | Afficher/masquer l'arborescence |
| `Ctrl+Shift+F` | Rechercher dans le projet |
| `` Ctrl+` `` / `Ctrl+'` | Terminal |
| `Ctrl+Shift+T` | Rouvrir le dernier onglet fermé |
| `Ctrl+Shift+G` | Git |
| `Ctrl+Shift+M` / `F8` | Problèmes / problème suivant |
| `F12` | Aller à la définition |

Tous peuvent être changés dans les paramètres ou dans `~/.config/idetea/config.toml`.

## Compiler depuis les sources

```sh
git clone https://github.com/fabiocmazzo/idetea && cd idetea
cargo build --release      # exécutable dans target/release/idetea
```

Avec [`just`](https://github.com/casey/just), `just run` installe aussi l'icône pour le panneau de votre bureau. Dans
un terminal, `idetea .` ouvre l'IDE et rend l'invite ; `idetea -f .` reste attaché (avec les journaux).

## Feuille de route

- [x] Éditeur, onglets, thèmes, palette, configuration TOML, arborescence, barre d'état.
- [x] Recherche dans le projet, LSP, panneau Problèmes, terminal intégré.
- [x] Git (y compris pousser/tirer), menus contextuels, aide, paramètres, icônes, panneaux redimensionnables.
- [x] Débogage avec gdb : points d'arrêt, pas à pas, pile des appels et variables.
- [x] Agents : panneau Interfaces, terminaux d'agent, liste d'autorisation, assistants dans des worktrees, nouveau
  projet avec des compétences.
- [ ] Extensions tierces en WASM, sans recompiler l'IDE (un prototype existe derrière la fonctionnalité `plugins`).
- [ ] Davantage d'adaptateurs de débogage (codelldb, debugpy, delve…).
- [ ] Marques git dans l'éditeur ; encodages autres que l'UTF-8.

Le plan détaillé, phase par phase, est dans [SPEC.md](../SPEC.md) (en portugais).
