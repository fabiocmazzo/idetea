# Comparaison honnête : IDE TEA face aux autres IDE du marché

[Português](comparativo-ides.md) · **Français** · [Deutsch](comparativo-ides.de.md) · [Italiano](comparativo-ides.it.md) · [日本語](comparativo-ides.ja.md) · [中文（简体）](comparativo-ides.zh-CN.md)

Ce document fait une évaluation technique, directe et sans fard d'**IDE TEA** (état actuel, `v0.4.0`) face aux
principaux IDE et éditeurs du marché :

- **VS Code / VSCodium** (le standard universel fondé sur Electron)
- **Cursor / Windsurf** (forks de VS Code centrés sur l'IA)
- **Zed** (IDE natif en Rust, centré sur la vitesse, la collaboration et l'IA intégrée)
- **JetBrains (RustRover, IntelliJ, PyCharm, CLion)** (IDE sémantiques lourds)
- **Neovim / Helix / Lapce** (éditeurs modaux/natifs légers)

> **Portée de cette analyse :** le focus est strictement technique — **stabilité, performance, fonctions d'édition, LSP, débogage, Git, écosystème et intégration avec les agents**. Elle ne tient pas compte de la taille de l'équipe ni de qui maintient le projet.

> Les versions ultérieures livrent aussi macOS (Apple Silicon) et Windows (x86_64). Le tableau et le texte restent l'instantané de la 0.4.0, y compris la cellule « Uniquement Linux ».

---

## 1. Vue d'ensemble (matrice directe)

| Dimension | IDE TEA | VS Code | Cursor / Windsurf | Zed | JetBrains | Helix / Neovim |
|---|---|---|---|---|---|---|
| **Temps d'ouverture (démarrage à froid)** | **Excellent (< 200 ms)** | Lent (1–3 s) | Lent (1,5–4 s) | **Excellent (< 150 ms)** | Très lent (5–20 s) | **Instantané (< 50 ms)** |
| **Consommation de RAM au repos** | **Faible (~60–150 Mo)** | Élevée (400 Mo–1,5 Go) | Très élevée (600 Mo–2 Go) | Faible (~120–300 Mo) | Très élevée (1,5–4 Go et plus) | **Minimale (15–60 Mo)** |
| **Fichiers géants (50 Mo et plus / minifiés)** | **Excellent (mode léger automatique)** | Moyen (se bloque sur les longues lignes) | Moyen | Bon | Souffre au-delà de 20 Mo | Dépend des extensions (Neovim) |
| **Vie privée / hors ligne / zéro compte** | **Totale (zéro réseau, sauf pousser/tirer avec git)** | Télémétrie par défaut (sauf VSCodium) | Exige un compte et le nuage | Demande un compte pour l'IA et la collaboration | Exige un compte / une licence | **Totale** |
| **Protocole structuré pour les agents (UI déclarative, approbation des commandes, mascotte)** | **Seul dans la catégorie (natif, en fichiers)** | N'en a pas (dépend d'extensions de conversation) | Centré sur un chat/diff propriétaire | Centré sur le chat et l'aide dans le texte | Centré sur le chat et l'aide dans le texte | N'en a pas (terminal seulement) |
| **Autocomplétion IA dans le texte (« Ghost text » / Tab)** | **N'en a pas (par philosophie)** | Via Copilot | **Excellente (en tête)** | Très bonne (Zeta) | Bonne | Via des extensions |
| **Maturité de l'éditeur de texte** | Bonne sur l'essentiel, il manque des fonctions avancées | **Référence** | **Référence** | Très haute | **Référence** | **Référence (modale)** |
| **Profondeur du LSP et de la refactorisation** | Moyenne (sans Code Actions ni Inlay Hints) | **Complète** | **Complète** | Haute | **Inégalée (AST propre)** | Haute |
| **Débogage (Debug)** | Basique (GDB 14+ pour Rust/C/C++/Zig ; le natif MoonBit est expérimental) | **Complet (n'importe quel langage)** | **Complet** | Basique/moyen (DAP récent) | **Excellent** | Basique (nvim-dap) |
| **Contrôle de version (Git)** | Basique/moyen (sans diff de marge ni fusion à 3 voies) | **Complet** | **Complet** | Moyen | **Excellent (meilleure fusion)** | Via LazyGit / Fugitive |
| **Prise en charge multiplateforme** | **Uniquement Linux (Wayland), testé aujourd'hui** | **Linux, macOS, Windows, Web** | **Linux, macOS, Windows** | **macOS, Linux, Windows** | **Linux, macOS, Windows** | **N'importe quel OS / SSH** |

---

## 2. Stabilité et fiabilité

### Là où IDE TEA est vraiment solide
1. **Architecture sans WebView / Chromium :** à la différence de VS Code, Cursor et Windsurf, il n'y a pas de processus de rendu Chromium ni de dizaines d'*extension hosts* Node.js qui fuient de la mémoire au long de la journée. Le binaire est unique, compilé en Rust sûr.
2. **Isolement des pannes d'extensions et des processus externes :**
   - Les serveurs de langage (`tea-lsp`), le débogueur (`tea-dap` / `gdb`), les appels `git` et les agents tournent dans des processus séparés. Si `rust-analyzer` ou `gdb` se bloque ou meurt, la fenêtre de l'IDE ne se fige pas.
   - Les extensions internes ne parlent au noyau que par messages et événements (`Request` / `CoreEvent`), sans partager de références mutables de l'état de l'application.
3. **Protection contre la perte de travail :**
   - Fermer un onglet ou quitter l'IDE avec des modifications non enregistrées demande toujours confirmation.
   - **Récupération automatique des plantages (`backup.rs`) :** les onglets avec des modifications non enregistrées sont écrits dans `$XDG_STATE_HOME/idetea/backups/<projet>/` par PID et proposés à la restauration si la fenêtre se ferme de façon inattendue.
   - Supprimer des fichiers dans l'arborescence les envoie à la **corbeille du système** (pas d'`unlink` irréversible).
   - Les changements externes sur le disque (`git checkout`, `cargo fmt`, des agents qui éditent des fichiers) sont détectés par le watcher : les onglets sans édition locale se rechargent seuls ; les onglets avec une édition locale non enregistrée ne sont **jamais** écrasés en silence.
   - Les fichiers binaires ou non UTF-8 sont refusés avec un avertissement clair, au lieu de corrompre le tampon à l'enregistrement.
   - Le protocole `.idetea/` utilise une écriture atomique (`.tmp` + `rename`) et possède une commande d'autoréparation (`idetea fix` / *« Interfaces : corriger ce projet »*).

### Là où la stabilité d'IDE TEA est encore en retrait face aux grandes
1. **Testé seulement sous Linux (Wayland) :** aujourd'hui IDE TEA est développé et validé sous Linux. Sous macOS et Windows, il y a de vraies différences d'empaquetage, de polices, de raccourcis (`Cmd` contre `Ctrl`), de PTY du terminal (`conpty` sous Windows) et de chemins qui n'ont pas encore passé l'épreuve du feu. VS Code, JetBrains et Zed tournent de façon mûre sur les trois systèmes.
2. **Limites de l'écosystème `iced` 0.14 pour les applications complexes :**
   - L'IDE fonctionne en fenêtre unique (`iced::application`). Il n'est pas possible de détacher un terminal, un aperçu ou un éditeur vers un second écran dans une autre fenêtre native du système.
   - Il n'y a pas d'historique local continu par fichier (*Local History* / *Timeline* sur plusieurs jours sans Git, comme dans JetBrains et VS Code).
3. **Coloration syntaxique fondée sur des expressions régulières (`syntect`) face à `tree-sitter` :**
   - IDE TEA a un excellent mode léger pour les fichiers de 50 Mo ou les lignes minifiées géantes, mais dans les fichiers de code ordinaires de plusieurs milliers de lignes, `syntect` (fondé sur des grammaires `.sublime-syntax` à expressions régulières) est structurellement moins résistant au code incomplet que le `tree-sitter` utilisé par Zed, Helix et Neovim.

---

## 3. Flux avec les agents d'IA (là où IDE TEA est unique — et là où il ne concurrence pas)

C'est la différence de philosophie de produit la plus profonde du marché aujourd'hui.

### Comment Cursor, Windsurf et Zed traitent l'IA
- Ils sont pensés pour **le modèle dans l'éditeur** :
  - *Autocomplétion prédictive (Tab / Next Edit Prediction)* qui devine les lignes suivantes pendant que vous tapez.
  - *Édition dans le texte (`Ctrl+K`)* et *Composer/Agent Chat* qui montrent des diffs colorés (vert/rouge) directement entre les lignes de l'éditeur, pour accepter ou refuser hunk par hunk.
  - En échange, ils vous attachent à des comptes dans le nuage, à des abonnements mensuels ou à des clients HTTP de fournisseurs à l'intérieur de l'IDE.

### Comment IDE TEA traite l'IA (et là où il gagne)
IDE TEA part du principe que **les meilleurs agents tournent déjà dans le terminal** (Claude Code, Codex, Gemini CLI, Aider, OpenCode) et résout le problème qu'aucun autre IDE n'a résolu de façon native et ouverte : **comment ces agents parlent à l'humain sans devenir un mur de texte dans le terminal**.

1. **Interfaces déclaratives natives (`.idetea/ui/` + `idetea ask`) :**
   - Au lieu que l'agent demande au milieu de 200 lignes de journal dans le terminal, il publie un JSON validé et l'IDE rend des formulaires natifs dans `iced` : sélecteurs, cases à cocher, listes d'étapes de test manuel (où le bouton *Approuver* ne s'active qu'une fois les étapes faites), aperçus d'image/galerie, spritesheets animés et audio WAV avec forme d'onde.
   - **Aucun autre IDE du marché (VS Code, Cursor, Zed, JetBrains) ne possède un protocole déclaratif fondé sur des fichiers pour que des agents de terminal dessinent une UI native, avec un historique vérifiable (`.idetea/history/`).**
2. **Gouvernance et sécurité d'exécution (`idetea run`) :**
   - Quand un agent veut ouvrir un terminal interactif ou déléguer une tâche, l'IDE montre la commande exacte dans un onglet d'approbation. La liste des commandes autorisées (`[agents] allow`) n'est lue que depuis la configuration globale de l'utilisateur (`~/.config/idetea/config.toml`) et **jamais** depuis le dépôt — ce qui empêche un dépôt malveillant de s'accorder tout seul le droit de s'exécuter.
3. **Délégation dans des git worktrees isolés (`--worktree` + `$IDETEA_ANSWER`) :**
   - L'IDE gère nativement la création et le nettoyage de worktrees pour que des sous-agents travaillent en parallèle sans salir l'arbre principal.
4. **Mascotte réactive native (Tea) + `idetea mascot` :**
   - Alors que des outils comme *Masko.ai* exigent macOS plus un serveur MCP dans le nuage pour avoir une mascotte qui réagit à Claude Code, IDE TEA a **Tea** intégré nativement, qui observe `.idetea/ui/`, `.idetea/agents/`, les tâches et le LSP en temps réel, réagit à la souris (`curious` / `annoyed`) et accepte des paquets `mascot.toml`.

### Là où IDE TEA perd dans le flux d'IA
- **Zéro autocomplétion prédictive (« Ghost text » / Tab) :** pour qui dépend du `Tab` de Cursor/Copilot pour compléter du boilerplate en tapant à la main, IDE TEA n'a pas d'équivalent.
- **Pas de visualisation de diff IA dans l'éditeur :** quand un agent dans le terminal édite un fichier, le fichier se recharge dans l'onglet d'IDE TEA. Pour voir ce qui a changé, il faut aller au panneau **Git** ; il n'y a pas de boutons *« Accept / Reject hunk »* flottant dans l'éditeur de code comme dans Cursor ou Windsurf.
- **Le passage de contexte de l'éditeur vers l'agent est encore manuel :** l'agent peut lancer `idetea status` pour découvrir le fichier actif et la ligne du curseur, mais il n'y a pas encore de raccourci dans l'IDE (par exemple `Ctrl+L`) qui envoie un extrait sélectionné ou une erreur du LSP directement dans l'invite de l'agent ouvert dans le terminal.

---

## 4. Éditeur de code : ce qui est là, et ce qui manque

### Ce qui est déjà solide dans IDE TEA
- Ouverture rapide de fichiers (`Ctrl+P` avec recherche floue et saut `fichier:ligne`), symboles dans le fichier (`Ctrl+Shift+O`) et dans le projet (`Ctrl+T`).
- Curseurs multiples, sélection mot par mot (`Ctrl+flèches`), `Shift+clic`, repli du code (folding), annuler/rétablir, recherche et remplacement dans le fichier et **remplacement dans tout le projet (`Ctrl+Shift+H`)** avec aperçu ligne à ligne et prise en charge des expressions régulières.
- Zoom temporaire de l'éditeur (`Ctrl+=`, `Ctrl+-`, `Ctrl+0`, `Ctrl+molette`) synchronisé entre les onglets et les divisions, sans toucher au reste de l'interface.
- Division de l'éditeur côte à côte (split), fil d'Ariane cliquable (`projet › dossier › fichier › symbole`), icônes Seti par langage et polices personnalisables avec ligatures (`JetBrains Mono` intégrée).
- Mode Vim facultatif.
- Onglets natifs pour les images (avec zoom et déplacement), le PDF (rendu en Rust pur via `hayro`) et l'aperçu côte à côte de Markdown et de SVG en temps réel.

### Ce qui manque dans l'éditeur par rapport à VS Code, Zed et JetBrains
1. **Snippets :** il n'y a pas de moteur de snippets (tab-stops `$1`, `$2`) pour développer de courts blocs de code.
2. **Sélections structurelles par AST (Expand/Shrink Selection) :** dans Zed, Helix et JetBrains, un raccourci étend la sélection du mot → l'expression → le bloc `if` → la fonction entière via `tree-sitter`/AST.
3. **Encodages anciens :** n'ouvre que l'UTF-8 (en préservant LF/CRLF). Qui travaille avec des fichiers `ISO-8859-1` / `Windows-1252` ne peut pas les éditer dans IDE TEA.
4. **Plusieurs fenêtres / plusieurs projets dans la même instance :** chaque projet est un processus `idetea` séparé ; il n'y a pas de « Multi-root Workspaces » dans la même fenêtre.

---

## 5. Intelligence de langage (LSP) et refactorisation

### Ce qu'IDE TEA livre déjà très bien
- Diagnostics en temps réel avec **soulignement ondulé dans le texte de l'éditeur**, infobulle d'erreur au survol, compteur dans la barre d'état et panneau **Problèmes** (`Ctrl+Shift+M` / `F8`).
- Autocomplétion, signature des paramètres, aller à la définition (`F12`), aller au type et aller à l'implémentation (`Ctrl+F12`).
- Rechercher les références (`Shift+F12` ou `Ctrl+clic` sur la déclaration) groupées par fichier, avec historique de navigation Retour/Avancer (`Alt+←` / `Alt+→` et boutons latéraux de la souris).
- **Hiérarchie d'appels (`Alt+Shift+H`) :** qui appelle la fonction et ce qu'elle appelle.
- **Panneau Plan / Outline (`Alt+7`) :** arbre de symboles du fichier indiquant la visibilité (`pub`, `pub(crate)`, privé) et mettant en évidence le symbole sous le curseur.
- **Renommer le symbole (`F2`)** dans tout le projet, avec une boîte de dialogue qui prévisualise combien de fichiers et de changements seront touchés avant d'appliquer.
- Mise en forme à l'enregistrement et respect de `.editorconfig`.

### Lacunes importantes de LSP face à VS Code, Zed et JetBrains
1. **Pas de Code Actions / Quick Fixes (`Ctrl+.` / ampoule du LSP) :**
   - C'est aujourd'hui le **plus grand manque pratique au quotidien avec Rust/TypeScript/Go** : quand `rust-analyzer` suggère *« Import `std::collections::HashMap` »*, *« Add missing match arms »* ou *« Fill struct fields »*, dans IDE TEA vous voyez le message de diagnostic, mais vous ne pouvez pas appliquer le *Quick Fix* (`textDocument/codeAction`) avec un raccourci.
2. **Pas d'Inlay Hints :**
   - N'affiche pas les types inférés dans le texte (`let x: Vec<String> = ...`) ni les noms de paramètres dans les appels de fonction, fonction très utilisée en Rust et en TypeScript dans VS Code, Zed et RustRover.
3. **Pas de Semantic Tokens du LSP :**
   - Les couleurs de syntaxe viennent exclusivement de la grammaire `.sublime-syntax` (`syntect`), sans enrichissement sémantique du LSP (par exemple distinguer une variable mutable d'une variable immuable, ou une macro d'une fonction, par la couleur du serveur de langage).

---

## 6. Débogage (Debug)

### Ce qu'IDE TEA fait aujourd'hui
- Intégration native avec `gdb` 14+ (via DAP) : points d'arrêt en cliquant sur le numéro de ligne (`F9`), qui suivent le texte si vous éditez le fichier, `F5` (compile et lance automatiquement dans les projets Rust, Zig et, de façon expérimentale, MoonBit), pas à pas principal (`F10`), pas à pas détaillé (`F11`), pas à pas sortant (`Shift+F11`), suspendre et arrêter.
- Panneau avec la pile des appels (un clic mène à la ligne du cadre), arbre de variables locales inspectable et sortie du programme.

### Là où il perd face à VS Code et JetBrains
- **Liée à GDB 14+ (Rust, C, C++, Zig et, de façon expérimentale, MoonBit) :** ne prend pas encore en charge les adaptateurs DAP pour Python (`debugpy`), Go (`delve`), JavaScript/TypeScript (`js-debug`) ni `codelldb` (macOS). Hare n'a pas de débogueur dans l'IDE.
- **Fonctions avancées de débogage absentes :** pas de points d'arrêt conditionnels (*« s'arrêter seulement quand `i == 42` »*), de logpoints, de fenêtre d'*expressions espion* arbitraires (REPL pendant le point d'arrêt) ni de valeurs de variables dessinées dans le texte à côté du code dans l'éditeur.

---

## 7. Git et contrôle de version

### Ce qu'IDE TEA fait aujourd'hui
- Barre d'état avec la branche actuelle, les commits en avance/en retard (`ahead`/`behind`) et le total de fichiers modifiés.
- Panneau Git avec indexer/désindexer par fichier ou globalement, visualisation de diff coloré, zone de message de validation, abandon des modifications avec confirmation, et boutons de `push`, `pull` et `fetch` qui utilisent le `git` et les identifiants/SSH du système.
- Couleurs d'état Git dans l'arborescence.

### Là où il perd face à VS Code, Zed et JetBrains
1. **Pas de marques Git dans la marge de l'éditeur (Gutter Diff) :** pendant que vous éditez un fichier, il n'y a pas ces petites barres vertes/bleues/rouges à côté des numéros de ligne montrant quelles lignes ont été ajoutées ou changées depuis le dernier commit.
2. **Pas de Git Blame dans le texte :** ne montre pas qui a changé la ligne actuelle, ni dans quel commit.
3. **Pas de résolveur visuel de conflits de fusion (3-way merge) :** en cas de conflit, vous le résolvez en éditant les marqueurs `<<<<<<<` dans le texte, ou par le terminal.
4. **Pas de changement/création de branches ni de Stash par l'interface :** pour changer de branche, créer une branche ou faire un `git stash`, vous utilisez le terminal intégré.

---

## 8. Résumé honnête : quand choisir IDE TEA aujourd'hui (et quand ne pas le choisir)

### IDE TEA est déjà meilleur, ou unique, quand vous :
1. **Programmez sous Linux à côté d'agents de terminal (Claude Code, Codex, Gemini CLI, etc.)** et voulez que l'agent pose des questions structurées, montre des plans et des tests manuels, demande une permission explicite pour lancer des commandes, utilise des worktrees isolés et ait une mascotte réactive (**Tea**) sans dépendre d'un nuage ni d'un abonnement.
2. **Mettez la vitesse, la prévisibilité et le peu de bruit sensoriel en premier :** s'ouvre en moins de 200 ms, consomme peu de mémoire, n'a pas de compte, n'a pas de télémétrie, ne change pas l'interface toute seule et respecte de simples fichiers TOML.
3. **Travaillez sur des projets en Rust, Zig, C/C++, Python, Go, Hare, MoonBit ou Web** où le flux principal est d'éditer vite, de naviguer par les symboles et les références du LSP, de lancer des tâches et des tests dans le terminal intégré et de relire les commits.

### Un autre IDE est encore meilleur quand vous :
1. **Voulez une autocomplétion prédictive d'IA à chaque touche (`Tab` / Ghost text) ou une édition dans le texte par diff :** **Cursor**, **Windsurf** ou **Zed** servent mieux ce style.
2. **Dépendez fortement des refactorisations assistées par *Code Actions* (`Ctrl+.`), des *Inlay Hints* ou des débogueurs Python/JS/Go dans l'interface :** **VS Code** ou **JetBrains** sont beaucoup plus complets aujourd'hui.
3. **Travaillez sous macOS ou Windows, ou dépendez de SSH Remote Development / DevContainers :** **VS Code** et **Zed** ont déjà une prise en charge mûre de ces scénarios.

---

## 9. Les 5 prochaines étapes au plus fort impact (pour se rapprocher des têtes de liste sans perdre l'âme d'IDE TEA)

Si le but est de réduire la distance fonctionnelle avec VS Code et Zed tout en gardant le binaire léger et la philosophie du manifeste :

1. **Code Actions / Quick Fixes du LSP (`Ctrl+.`) :** permettre d'appliquer les *quick fixes* et les *imports* automatiques que le serveur LSP sait déjà calculer. C'est le point n° 1 qui manque le plus en programmant en Rust/TS.
2. **Marques Git dans l'éditeur (Gutter Diff) + changement de branche dans la barre d'état :** montrer les lignes ajoutées/modifiées dans la marge de l'éditeur et permettre de cliquer sur le nom de la branche dans la barre d'état pour changer de branche dans la palette.
3. **Raccourci « Envoyer la sélection / le diagnostic à l'agent » (`Ctrl+L`) :** boucler le cycle entre l'éditeur/LSP et l'agent qui tourne dans le terminal intégré.
4. **Inlay Hints du LSP (types inférés et noms de paramètres facultatifs) :** afficher des indications de type dans le texte, avec un raccourci rapide pour les activer ou les couper.
5. **Portabilité vers macOS et Windows + davantage d'adaptateurs DAP (`lldb-dap`, `debugpy`, `delve`) :** ouvrir l'IDE aux développeurs hors de Linux et prendre en charge le débogage au-delà de GDB.
