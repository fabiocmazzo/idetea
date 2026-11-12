# Das Manifest von IDE TEA

<p><a href="MANIFESTO.md">English</a> · <a href="MANIFESTO.pt-BR.md">Português</a> · <a href="MANIFESTO.es.md">Español</a> ·
<a href="MANIFESTO.fr.md">Français</a> · <b>Deutsch</b> · <a href="MANIFESTO.it.md">Italiano</a> ·
<a href="MANIFESTO.ja.md">日本語</a> · <a href="MANIFESTO.zh-CN.md">中文（简体）</a></p>

Ein Editor ist der Ort, an dem Sie den Tag verbringen. Er sollte sich wie ein stilles Zimmer anfühlen, nicht wie ein Einkaufszentrum.

## 1. Schnell ist eine Funktion

Warten unterbricht die Konzentration. IDE TEA öffnet in einem Augenblick, öffnet große Dateien, ohne einzufrieren, und erledigt langsame Arbeit (Suche,
git, Sprachserver) abseits des Bildschirms, auf dem Sie tippen. Wenn etwas langsam ist, ist das ein Fehler.

## 2. Still als Voreinstellung

Kein Konto. Keine Telemetrie. Keine Pop-ups über Funktionen, um die Sie nicht gebeten haben. Das Netz wird nur genutzt, wenn Sie
danach fragen (Pushen, Pullen, Abrufen). Unter Ihren Füßen ändert sich nichts: Die Einstellungen leben in einer schlichten TOML-Datei, die Sie lesen,
versionieren und teilen können.

## 3. Vorhersehbar geht vor clever

Jedes Tastenkürzel steht an einem Ort (`F1`) und lässt sich ändern. Ein Bereich bleibt, wo Sie ihn hinlegen. Beim Schließen mit ungespeicherten Änderungen
wird immer gefragt. Eine gelöschte Datei landet im Papierkorb. Dateien, die außerhalb der IDE geändert werden, fallen auf, und ungespeicherte
Arbeit wird nie verworfen, um Ihnen zu „helfen“.

## 4. Innen klein, außen vollständig

Der Kern bearbeitet nur Dateien. Der Baum, das Terminal, git, der Debugger und alles andere sind Erweiterungen, die Sie
abschalten können. Ein kleiner Kern ist leichter zu vertrauen, leichter zu lesen und schwerer zu zerbrechen.

## 5. Agenten sind willkommen, und Sie behalten das Sagen

Coding-Agenten gehören dazu, wie wir heute arbeiten. IDE TEA gibt ihnen einen einfachen, offenen, dateibasierten Weg, mit Ihnen zu sprechen:
Fragen als echte Formulare, Terminals, um die sie bitten können, Helfer in ihren eigenen Worktrees. Die Regeln sind immer dieselben:

- **Sie sehen es, bevor es läuft.** Jeder Befehl, um den ein Agent bittet, wird vollständig gezeigt und wartet auf Ihre Zustimmung,
  es sei denn, Sie setzen ihn auf eine Zulassungsliste in *Ihrer* Konfiguration. Ein Projekt kann sich nie selbst die Erlaubnis geben.
- **Ihre Antwort ist die Antwort.** Eine Zustimmung ist eine Entscheidung, kein Klick, um einen Dialog loszuwerden. Schaltflächen, auf die es
  ankommt, warten, bis Sie die Schritte erledigt haben.
- **Schlichte Dateien, kein Lock-in.** Das Protokoll ist JSON in einem Ordner. Jeder Agent, der Dateien lesen und schreiben kann, kann es
  nutzen, und Sie können jede Nachricht selbst lesen.

## 6. Wirklich frei

IDE TEA steht unter Apache 2.0 und ist kostenlos, und das bleibt so. Keine Bezahlstufe, die das Gute verbirgt.

## 7. Für unterschiedliche Köpfe gebaut

Der Name hat eine zweite Bedeutung: Auf Portugiesisch ist TEA die Abkürzung für Autismus. IDE TEA wird von einem autistischen
Entwickler gemacht, und das zeigt sich in dem, was sie schätzt: wenig Lärm, klarer Text, Dinge, die sich jedes Mal gleich verhalten,
und Raum zum Konzentrieren. Wenn Ihr Kopf auch so arbeitet, willkommen. Wenn nicht, wird sie Ihnen wahrscheinlich trotzdem gefallen.
