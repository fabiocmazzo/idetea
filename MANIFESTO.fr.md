# Le manifeste d'IDE TEA

<p><a href="MANIFESTO.md">English</a> · <a href="MANIFESTO.pt-BR.md">Português</a> · <a href="MANIFESTO.es.md">Español</a> ·
<b>Français</b> · <a href="MANIFESTO.de.md">Deutsch</a> · <a href="MANIFESTO.it.md">Italiano</a> ·
<a href="MANIFESTO.ja.md">日本語</a> · <a href="MANIFESTO.zh-CN.md">中文（简体）</a></p>

Un éditeur, c'est là que vous passez la journée. Il devrait ressembler à une pièce calme, pas à un centre commercial.

## 1. La vitesse est une fonctionnalité

Attendre casse la concentration. IDE TEA s'ouvre en un clin d'œil, ouvre les gros fichiers sans se figer, et fait le
travail lent (recherche, git, serveurs de langage) à l'écart de l'écran où vous tapez. Quand quelque chose est lent,
c'est un bug.

## 2. Silencieux par défaut

Sans compte. Sans télémétrie. Sans fenêtre sur des fonctions que vous n'avez pas demandées. Le réseau ne sert que
lorsque vous le demandez (pousser, tirer, récupérer). Rien ne change sous vos pieds : les paramètres vivent dans un
fichier TOML lisible, que vous pouvez versionner et partager.

## 3. Prévisible, plutôt que malin

Tous les raccourcis sont listés au même endroit (`F1`) et peuvent être changés. Un panneau reste là où vous l'avez mis.
Fermer quelque chose qui a des modifications non enregistrées demande toujours confirmation. Supprimer un fichier
l'envoie à la corbeille. Les fichiers modifiés hors de l'IDE sont signalés, et votre travail non enregistré n'est
jamais jeté pour vous « aider ».

## 4. Petit dedans, complet dehors

Le noyau ne fait qu'éditer des fichiers. L'arborescence, le terminal, git, le débogueur et tout le reste sont des
extensions que vous pouvez désactiver. Un petit noyau est plus digne de confiance, plus facile à lire, et plus
difficile à casser.

## 5. Les agents sont les bienvenus, et vous restez aux commandes

Les agents de code font partie de la façon dont nous travaillons aujourd'hui. IDE TEA leur donne un moyen simple,
ouvert et fait de fichiers de vous parler : des questions sous forme de vrais formulaires, des terminaux qu'ils peuvent
demander, des assistants dans leur propre worktree. Les règles sont toujours les mêmes :

- **Vous le voyez avant qu'il ne s'exécute.** Chaque commande demandée par un agent est montrée en entier et attend
  votre approbation, sauf si vous la placez dans une liste d'autorisation de *votre* configuration. Un projet ne peut
  jamais s'accorder une permission.
- **Votre réponse est la réponse.** Une approbation est une décision, pas un clic pour chasser une boîte de dialogue.
  Les boutons qui comptent attendent que vous ayez fait les étapes.
- **Des fichiers simples, sans enfermement.** Le protocole est du JSON dans un dossier. Tout agent qui sait lire et
  écrire des fichiers peut l'utiliser, et vous pouvez lire chaque message vous-même.

## 6. Libre, pour de vrai

IDE TEA est sous Apache 2.0 et gratuit, et il le restera. Aucun palier payant qui cache les bonnes parties.

## 7. Fait pour des esprits différents

Le nom a un second sens : en portugais, TEA est le sigle de l'autisme (*Transtorno do Espectro Autista*, le trouble du
spectre de l'autisme). IDE TEA est fait par un développeur autiste, et cela se voit dans ce qu'il valorise : peu de
bruit, un texte clair, des choses qui se comportent toujours de la même façon, et de la place pour se concentrer. Si
votre esprit fonctionne ainsi, bienvenue. Sinon, il y a des chances que cela vous plaise quand même.
