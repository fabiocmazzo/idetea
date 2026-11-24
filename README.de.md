<p align="center">
  <img src="assets/idetea.svg" width="112" alt="Symbol von IDE TEA: ein Glas Eistee">
</p>

<h1 align="center">IDE TEA</h1>

<p align="center"><b>Nur der Code. Eis darfst du selbst zugeben.</b></p>

<p align="center">
  Eine schnelle, stille IDE in reinem Rust, in der Coding-Agenten fragen, bevor sie handeln.<br>
  Frei, und das bleibt so: <a href="LICENSE">Apache 2.0</a>. Kein Konto, keine Telemetrie, kein Marketplace.
</p>

<p align="center">
  <a href="https://github.com/fabiocmazzo/idetea/actions/workflows/ci.yml"><img src="https://github.com/fabiocmazzo/idetea/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://github.com/fabiocmazzo/idetea/releases"><img src="https://img.shields.io/github/v/release/fabiocmazzo/idetea?include_prereleases&amp;label=release" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue" alt="License: Apache 2.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024-orange?logo=rust" alt="Rust 2024"></a>
  <a href="https://github.com/iced-rs/iced"><img src="https://img.shields.io/badge/GUI-iced%200.14-3645ff" alt="GUI: iced 0.14"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey" alt="Platforms: Linux, macOS, Windows">
  <a href="https://codeberg.org/fabiocmazzo/idetea"><img src="https://img.shields.io/badge/Codeberg-mirror-2185D0?logo=codeberg&amp;logoColor=white" alt="Codeberg mirror"></a>
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.pt-BR.md">Português</a> · <a href="README.es.md">Español</a> ·
  <a href="README.fr.md">Français</a> · <b>Deutsch</b> · <a href="README.it.md">Italiano</a> ·
  <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">中文（简体）</a><br>
  <a href="https://idetea.belabs.cloud/de/"><b>Website und Anleitungen</b></a>
  · <a href="https://github.com/fabiocmazzo/idetea">GitHub</a>
  · <a href="https://codeberg.org/fabiocmazzo/idetea">Codeberg</a> (Spiegel)
</p>

<p align="center">
  <a href="https://idetea.belabs.cloud/de/#demo">
    <img src="assets/demo-preview.gif" width="880" alt="Erzählte Demo: schnelles Öffnen, Rust mit Syntaxhervorhebung, ein geteilter Editor, das integrierte Terminal, dann ein Agentenformular im Bereich Interfaces, während das Maskottchen Tea zu Achtung wechselt.">
  </a>
</p>

<p align="center">
  <a href="https://idetea.belabs.cloud/de/#demo"><b>Die Demo von 52 Sekunden ansehen</b></a>
  · auf Englisch erzählt, Untertitel eingebrannt<br>
  GitHub spielt eine Videodatei aus dem Repository im README nicht ab.
  Der Clip darüber öffnet sie auf der Website, dort mit Ton.
  Die Datei selbst ist <a href="assets/idetea-demo.mp4"><code>assets/idetea-demo.mp4</code></a>.
</p>

---

> **Stand:** in aktiver Entwicklung, vor 1.0. Gebaut und getestet auf Linux, macOS (Apple Silicon) und Windows (x86_64).
> Es gibt noch raue Kanten. Bitte melden Sie sie.

## Für wen sie ist

Für Menschen, die Code mit einem Agenten im Terminal neben dem Editor schreiben: Claude Code, Codex oder jeder Agent, der
`AGENTS.md` und `CLAUDE.md` liest. IDE TEA legt kein Chat-Modell in das Fenster. Die IDE gibt diesem Agenten einen kleinen,
dateibasierten Weg, Sie zu fragen, und Sie behalten bei jedem Befehl das Sagen.

## Warum Menschen bleiben

**Schnell.** Startet deutlich unter 200 ms. Eine Datei von 50 MB öffnet in einer Zehntelsekunde, und eine minifizierte Datei mit einer riesigen
Zeile öffnet in einem Leichtmodus, statt das Fenster einzufrieren.

**Still.** Kein Konto, keine Telemetrie, kein Marketplace, keine Pop-ups, um die Sie nicht gebeten haben. Das Netz wird nur genutzt, wenn
Sie es verlangen (Pushen, Pullen, Abrufen oder einen Skill, den Sie installieren). Aussehen, Tasten und Bereiche leben in einer schlichten TOML-Datei.

**Sie genehmigen, was läuft.** Ein Agent schreibt eine kleine JSON-Datei, und die IDE zeigt ein Formular: Optionen, Kontrollkästchen, Text,
Bilder, Klänge, manuelle Tests Schritt für Schritt. Eine Terminalanfrage zeigt zuerst den ganzen Befehl. Ein Helfer kann in seinem
eigenen git-Worktree laufen. Ein Projekt kann sich nie selbst die Erlaubnis geben. Ihre Zulassungsliste liegt in *Ihrer* Konfiguration.

**Vollständig, wo es zählt.** Mehrfachcursor, vorübergehender Zoom, ein optionaler Vim-Modus, Dateibaum, Projektsuche,
integriertes Terminal, git (Stagen, Diff, Commit, Pushen und Pullen), LSP, Debuggen mit gdb und Vorschauen für Markdown,
SVG, Bilder und PDF.

**Innen klein.** Der Kern bearbeitet nur Dateien. Baum, Terminal, git und der Debugger sind Erweiterungen, die Sie
abschalten können. Tea, eine Teetasse in Pixel-Art, achtet auf eine wartende Frage (ein Klick öffnet den richtigen Bereich) und klappt
in die Statusleiste ein, wenn Sie die Ecke zurückhaben wollen.

Ein Befehl bringt den Agenten des Projekts die Sache bei: „Interfaces: den Agenten dieses Projekts beibringen“ schreibt das Protokoll in
`AGENTS.md` und `CLAUDE.md`.

## Funktionen in Aktion

Fünf kurze Clips, auf Englisch erzählt, Untertitel eingebrannt. GitHub spielt eine Videodatei aus dem
Repository im README nicht ab. Die Poster öffnen den Clip auf der Website, dort mit Ton. Die Dateien liegen in
`assets/`.

<table>
<tr>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/de/#feature-agents"><img src="assets/agents-poster.png" width="440" alt="Agenten: Agenten in Ihrem Terminal. Gedächtnis in Ihrem Repository."></a><br>
    <a href="https://idetea.belabs.cloud/de/#feature-agents"><b>Agenten</b></a> · 57 Sekunden<br>
    Agenten in Ihrem Terminal. Gedächtnis in Ihrem Repository.<br>
    <a href="assets/idetea-agents.mp4"><code>assets/idetea-agents.mp4</code></a>
  </td>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/de/#feature-tables"><img src="assets/tables-poster.png" width="440" alt="Tabellen: Ihre Daten als Tabelle. Immer noch nur Text."></a><br>
    <a href="https://idetea.belabs.cloud/de/#feature-tables"><b>Tabellen</b></a> · 45 Sekunden<br>
    Ihre Daten als Tabelle. Immer noch nur Text.<br>
    <a href="assets/idetea-tables.mp4"><code>assets/idetea-tables.mp4</code></a>
  </td>
</tr>
<tr>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/de/#feature-lsp"><img src="assets/lsp-poster.png" width="440" alt="LSP: Tiefe Sprachenunterstützung, fest eingebaut."></a><br>
    <a href="https://idetea.belabs.cloud/de/#feature-lsp"><b>LSP</b></a> · 41 Sekunden<br>
    Tiefe Sprachenunterstützung, fest eingebaut.<br>
    <a href="assets/idetea-lsp.mp4"><code>assets/idetea-lsp.mp4</code></a>
  </td>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/de/#feature-encodings"><img src="assets/encodings-poster.png" width="440" alt="Kodierungen: Alte Dateien, neuer Editor, nichts geht verloren."></a><br>
    <a href="https://idetea.belabs.cloud/de/#feature-encodings"><b>Kodierungen</b></a> · 47 Sekunden<br>
    Alte Dateien, neuer Editor, nichts geht verloren.<br>
    <a href="assets/idetea-encodings.mp4"><code>assets/idetea-encodings.mp4</code></a>
  </td>
</tr>
<tr>
  <td width="50%" valign="top">
    <a href="https://idetea.belabs.cloud/de/#feature-mascot"><img src="assets/mascot-poster.png" width="440" alt="Maskottchen: Eine ruhige Tasse Tee, in Ihrer Sprache."></a><br>
    <a href="https://idetea.belabs.cloud/de/#feature-mascot"><b>Maskottchen</b></a> · 52 Sekunden<br>
    Eine ruhige Tasse Tee, in Ihrer Sprache.<br>
    <a href="assets/idetea-mascot.mp4"><code>assets/idetea-mascot.mp4</code></a>
  </td>
  <td width="50%" valign="top"></td>
</tr>
</table>

## Installation

**Linux, macOS und Windows, startklar.** Die aktuelle Version ist
[v0.17.0](https://github.com/fabiocmazzo/idetea/releases/tag/v0.17.0).
`SHA256SUMS` auf dieser Seite enthält die Prüfsummen. Ändern Sie `VERSION`, wenn eine neuere erscheint, oder wählen Sie
die Datei unter [Releases](https://github.com/fabiocmazzo/idetea/releases).

**Linux (x86_64).** Entpacken und `install.sh` ausführen. Das Skript legt `idetea` nach `~/.local/bin` und ins Anwendungsmenü:

```sh
VERSION=0.17.0
curl -fsSL -O "https://github.com/fabiocmazzo/idetea/releases/download/v${VERSION}/idetea-${VERSION}-x86_64-linux.tar.gz"
tar -xzf idetea-${VERSION}-x86_64-linux.tar.gz
cd idetea-${VERSION}-x86_64-linux
./install.sh
```

Für ARM verwenden Sie `idetea-${VERSION}-aarch64-linux.tar.gz`. Wenn Sie eine einzelne Datei bevorzugen, laden Sie
`IDE_TEA-${VERSION}-x86_64.AppImage` (oder `IDE_TEA-${VERSION}-aarch64.AppImage`) herunter, dann `chmod +x` und starten.
`--install-desktop` fügt sie dem Menü hinzu.

**macOS (Apple Silicon).** Das Archiv enthält `IDE TEA.app`:

```sh
VERSION=0.17.0
curl -fsSL -O "https://github.com/fabiocmazzo/idetea/releases/download/v${VERSION}/idetea-${VERSION}-aarch64-macos.tar.gz"
tar -xzf idetea-${VERSION}-aarch64-macos.tar.gz
cd idetea-${VERSION}-aarch64-macos
./idetea --install-desktop   # legt ~/Applications/IDE TEA.app an
```

Es gibt noch kein Paket für Intel-Macs.

**Windows (x86_64).** Laden Sie
[IDE_TEA-0.17.0-x86_64-setup.exe](https://github.com/fabiocmazzo/idetea/releases/download/v0.17.0/IDE_TEA-0.17.0-x86_64-setup.exe)
herunter und führen Sie es aus, oder entpacken Sie das portable
[idetea-0.17.0-x86_64-windows.zip](https://github.com/fabiocmazzo/idetea/releases/download/v0.17.0/idetea-0.17.0-x86_64-windows.zip)
(`idetea.exe` liegt darin). In PowerShell:

```powershell
$Version = "0.17.0"
Invoke-WebRequest -Uri "https://github.com/fabiocmazzo/idetea/releases/download/v$Version/IDE_TEA-$Version-x86_64-setup.exe" -OutFile "IDE_TEA-$Version-x86_64-setup.exe"
Start-Process ".\IDE_TEA-$Version-x86_64-setup.exe"
```

**Aus dem Quellcode.** Sie brauchen [Rust](https://rustup.rs) (stable). Das Paket heißt `tea-app`; die Binärdatei heißt `idetea`:

```sh
cargo install --locked --git https://github.com/fabiocmazzo/idetea tea-app
idetea --install-desktop   # optional: Eintrag und Symbol im Anwendungsmenü
```

Optional: `git` für die git-Integration, `gdb` 14+ (oder `lldb-dap` unter macOS) zum Debuggen und die Sprachserver,
die Sie nutzen (rust-analyzer, clangd, pyright, typescript-language-server, gopls, lua-language-server, zls, `zx lsp`, `hare-lsp` für Hare und `moon lsp` für MoonBit sind schon eingerichtet).

## Codesignatur und Datenschutz

Der Windows-Installer ist noch nicht codesigniert, daher kann SmartScreen beim ersten Start warnen. Dieses Programm
überträgt keine Informationen an andere vernetzte Systeme, sofern der Benutzer das nicht ausdrücklich verlangt. Siehe
[`CODE_SIGNING_POLICY.md`](CODE_SIGNING_POLICY.md).

## Schnellstart

```sh
idetea                # den aktuellen Ordner als Projekt öffnen
idetea src/main.rs    # aktueller Ordner und diese Datei
idetea ~/projekt      # ein anderer Ordner
idetea new meine-app --model rust --agent claude     # ein neues Projekt, mit einem Agenten, der es anfangen kann
```

In der IDE: `Ctrl+Shift+P` öffnet die Befehlspalette, `Ctrl+P` öffnet eine Datei nach Namen, `F1` zeigt jedes Tastenkürzel.
Die Anleitungen stehen auf der [Website](https://idetea.belabs.cloud/de/).

## Warum IDE TEA, und wann nicht

[`docs/comparativo-ides.de.md`](docs/comparativo-ides.de.md) ist ein ehrlicher Vergleich mit VS Code, Cursor, Zed, JetBrains
und Helix, aktualisiert für 0.15.0. Kurz gesagt:

- Terminal-Agenten bekommen ein natives, dateibasiertes Protokoll (Formulare, genehmigte Befehle, Worktrees, das Maskottchen Tea). Das ist
  das, was die anderen Editoren in dieser Notiz nicht haben.
- Es gibt keine Ghost-Text-Vervollständigung, mit Absicht. Wenn Sie mit Tab vervollständigen oder Inline-Diffs wollen, zeigt dieses Dokument auf
  Cursor, Windsurf oder Zed.
- Die Abdeckung des Debuggers liegt hinter VS Code und JetBrains (nur gdb und lldb-dap).

### Tea

Die Teetasse in der Ecke hat für jede Stimmung mehrere Sätze. Die Tasse bemerkt ein Speichern, eine lange Pause, die Tageszeit und
den Build des Debuggers und nennt ab und zu ein Tastenkürzel, das wirklich belegt ist. `[mascot] chatty` ist `quiet`, `normal` oder
`chatty`; `[mascot] tips` schaltet die Hinweise an oder aus. Die ganze Oberfläche, Tea eingeschlossen, gibt es auf Portugiesisch, Englisch,
Spanisch, Französisch, Deutsch, Italienisch, Japanisch und vereinfachtem Chinesisch (`[ui] language` oder die Systemsprache). Sie
spricht außerdem einmal über einen Plattenkonflikt bei ungespeicherten Änderungen, einen Git-Konflikt, das Ende
einer Aufgabe oder Debug-Sitzung, ein Zeichen, das die Kodierung nicht hält, und ein Ersetzen in mehreren
Dateien. Warnungen und Fehler, die auf null gehen, bleiben im Modus chatty. Dieselben Schalter liegen in den
Einstellungen. `idetea mascot setup-hooks` ergänzt Claude-Code-Hooks und lässt die stehen, die schon in der
Datei sind. Sie nimmt den Fokus nicht.

## Dem Repository einen Stern geben

Wenn das der Editor ist, den Sie wollten, [geben Sie IDE TEA einen Stern](https://github.com/fabiocmazzo/idetea). Es ist das Projekt
einer Person, und ein Stern ist der Weg, auf dem die nächste Person es findet.

## Zum Namen

Es begann als Scherz. Die IDE ist mit **iced** gebaut, also musste sie **Iced Tea** heißen.

Dann zeigte sich die andere Bedeutung. Auf Portugiesisch steht **TEA** für *Transtorno do Espectro Autista*, die
Autismus-Spektrum-Störung. Der Autor ist Autist, die Doppelbedeutung passte, also blieb sie. Vielleicht erklärt das auch
einiges an der IDE: Sie mag Vorhersehbarkeit, Klartext, den Sie lesen können, und so wenig Lärm wie möglich.

Mehr dazu im [Manifest](MANIFESTO.de.md).

## Dokumentation

- [Manifest](MANIFESTO.de.md): woran IDE TEA glaubt und was sie nicht tun wird.
- [Website](https://idetea.belabs.cloud/de/): die Demo, die Installation und die Anleitungen.
- [Contributing](CONTRIBUTING.md): wie Sie bauen, testen und einen Pull Request öffnen (auf Englisch).
- [Funktionen und Tastenkürzel](docs/features.de.md): die vollständige Liste.
- [Sprachen](docs/languages.de.md): was die IDE für jede Sprache tut, einschließlich Hare und MoonBit, mit
  [Beispielen](examples/languages) zum Ausprobieren.
- [Interfaces für Agenten](docs/interfaces/formato.md) und [Agenten-Terminals](docs/terminais-de-agente.md) (vorerst
  auf Portugiesisch).
- [SPEC.md](SPEC.md) und [DESENVOLVIMENTO.md](DESENVOLVIMENTO.md): Spezifikation, Architektur und wie Sie bauen,
  testen und eine Erweiterung schreiben (vorerst auf Portugiesisch).

## Lizenz

IDE TEA ist freie Software unter der [Apache License 2.0](LICENSE): nutzen Sie sie, studieren Sie sie, ändern Sie sie und geben Sie sie weiter,
auch kommerziell. Kostenlos, und das bleibt so.

Die Symbole der Oberfläche stammen aus [Codicons](https://github.com/microsoft/vscode-codicons) © Microsoft, unter
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) (siehe
[`crates/tea-core/assets/NOTICE.md`](crates/tea-core/assets/NOTICE.md)).
