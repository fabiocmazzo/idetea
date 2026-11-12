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
  <a href="README.md">English</a> · <a href="README.pt-BR.md">Português</a> · <a href="README.es.md">Español</a> ·
  <a href="README.fr.md">Français</a> · <b>Deutsch</b> · <a href="README.it.md">Italiano</a> ·
  <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">中文（简体）</a><br>
  <a href="https://idetea.belabs.cloud/de/"><b>Website und Anleitungen</b></a>
  · <a href="https://github.com/fabiocmazzo/idetea">GitHub</a>
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

## Installation

**Linux, macOS und Windows, startklar.** Laden Sie das Paket für Ihre Plattform von den
[Releases](https://github.com/fabiocmazzo/idetea/releases): `.tar.gz` oder `.AppImage` für Linux x86_64/aarch64,
`.tar.gz` mit `IDE TEA.app` für macOS aarch64, oder den Installer `-setup.exe` / das portable `.zip` für Windows x86_64.
`SHA256SUMS` enthält die Prüfsummen.

Unter Linux legt `./install.sh` `idetea` nach `~/.local/bin` und in das Anwendungsmenü. Das AppImage läuft, wie es ist
(zuerst `chmod +x`), und `--install-desktop` fügt es dem Menü hinzu. Unter macOS legt `--install-desktop`
`~/Applications/IDE TEA.app` an.

**Aus dem Quellcode.** Sie brauchen [Rust](https://rustup.rs) (stable). Dann:

```sh
cargo install --locked --git https://github.com/fabiocmazzo/idetea tea-app
idetea --install-desktop   # optional: Eintrag und Symbol im Anwendungsmenü
```

Optional: `git` für die git-Integration, `gdb` 14+ (oder `lldb-dap` unter macOS) zum Debuggen und die Sprachserver,
die Sie nutzen (rust-analyzer, clangd, pyright, typescript-language-server, gopls, lua-language-server, zls, `zx lsp`, `hare-lsp` für Hare und `moon lsp` für MoonBit sind schon eingerichtet).

## Codesignatur und Datenschutz

Die kostenlose Codesignatur unter Windows stellt [SignPath.io](https://signpath.io), das Zertifikat die
[SignPath Foundation](https://signpath.org). Dieses Programm überträgt keine Informationen an andere vernetzte Systeme,
sofern der Benutzer das nicht ausdrücklich verlangt. Siehe [`CODE_SIGNING_POLICY.md`](CODE_SIGNING_POLICY.md).

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
und Helix, geschrieben um 0.4.0. Der Teil, der weiterhin das Produkt ist:

- Terminal-Agenten bekommen ein natives, dateibasiertes Protokoll (Formulare, genehmigte Befehle, Worktrees, das Maskottchen Tea). Das ist
  das, was die anderen Editoren in dieser Notiz nicht haben.

### Tea

Die Teetasse in der Ecke hat für jede Stimmung mehrere Sätze. Die Tasse bemerkt ein Speichern, eine lange Pause, die Tageszeit und
den Build des Debuggers und nennt ab und zu ein Tastenkürzel, das wirklich belegt ist. `[mascot] chatty` ist `quiet`, `normal` oder
`chatty`; `[mascot] tips` schaltet die Hinweise an oder aus. Die ganze Oberfläche, Tea eingeschlossen, gibt es auf Portugiesisch, Englisch,
Spanisch, Französisch, Deutsch, Italienisch, Japanisch und vereinfachtem Chinesisch (`[ui] language` oder die Systemsprache). Sie
nimmt den Fokus nicht.
- Es gibt keine Ghost-Text-Vervollständigung, mit Absicht. Wenn Sie mit Tab vervollständigen oder Inline-Diffs wollen, zeigt dieses Dokument auf
  Cursor, Windsurf oder Zed.
- Code Actions, Inlay Hints und die Abdeckung des Debuggers liegen hinter VS Code und JetBrains.

Dieselbe Notiz sagt noch „nur Linux“. Versionen seither liefern auch macOS (Apple Silicon) und Windows (x86_64).

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
