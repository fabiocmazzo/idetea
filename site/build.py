#!/usr/bin/env python3
"""Monta o site em uma pasta (padrão: target/site), só com a biblioteca padrão.

- copia site/ (menos este script);
- traz de assets/ o ícone e o GIF (para não duplicar binários no git);
- gera features.html a partir de docs/features.md (a lista fica num lugar só).

Uso: python3 site/build.py [pasta]
"""

import html
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SITE = ROOT / "site"
REPO = "https://github.com/fabiocmazzo/idetea/blob/main/"


def inline(text: str, base: str) -> str:
    """Markdown de uma linha: código, links, negrito, itálico."""
    out = []
    # Separa o código (`...`) para não mexer no que está dentro.
    for i, part in enumerate(re.split(r"(`[^`]*`)", text)):
        if i % 2:
            out.append("<code>" + html.escape(part[1:-1]) + "</code>")
            continue
        part = html.escape(part, quote=False)

        def link(m: re.Match) -> str:
            label, href = m.group(1), m.group(2)
            if not re.match(r"^[a-z]+:", href) and not href.startswith("#"):
                href = REPO + str(Path(base, href)).replace("\\", "/")
                href = re.sub(r"[^/]+/\.\./", "", href)
            return f'<a href="{html.escape(href)}">{label}</a>'

        part = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", link, part)
        part = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", part)
        part = re.sub(r"(?<![*\w])\*([^*]+)\*(?![*\w])", r"<em>\1</em>", part)
        out.append(part)
    return "".join(out)


def markdown(text: str, base: str) -> str:
    """O pouco de Markdown que o docs/features.md usa: títulos, listas, tabelas, código, citações, parágrafos."""
    lines = text.splitlines()
    out: list[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if not line.strip():
            i += 1
        elif line.startswith("```"):
            body = []
            i += 1
            while i < len(lines) and not lines[i].startswith("```"):
                body.append(lines[i])
                i += 1
            i += 1
            out.append("<pre><code>" + html.escape("\n".join(body)) + "</code></pre>")
        elif m := re.match(r"^(#{1,4}) (.*)", line):
            level = len(m.group(1))
            title = m.group(2)
            anchor = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")
            out.append(f'<h{level} id="{anchor}">{inline(title, base)}</h{level}>')
            i += 1
        elif line.startswith(">"):
            body = []
            while i < len(lines) and lines[i].startswith(">"):
                body.append(lines[i].lstrip("> ").rstrip())
                i += 1
            out.append('<div class="callout">' + inline(" ".join(body), base) + "</div>")
        elif line.startswith("|"):
            rows = []
            while i < len(lines) and lines[i].startswith("|"):
                cells = [c.strip() for c in lines[i].strip().strip("|").split("|")]
                if not all(re.fullmatch(r":?-+:?", c) for c in cells):
                    rows.append(cells)
                i += 1
            head, *body = rows
            t = ["<tr>" + "".join(f"<th>{inline(c, base)}</th>" for c in head) + "</tr>"]
            t += ["<tr>" + "".join(f"<td>{inline(c, base)}</td>" for c in r) + "</tr>" for r in body]
            out.append('<div class="table-scroll"><table>' + "".join(t) + "</table></div>")
        elif re.match(r"^- ", line):
            items = []
            while i < len(lines) and (lines[i].startswith("- ") or lines[i].startswith("  ")):
                if lines[i].startswith("- "):
                    items.append(lines[i][2:].strip())
                else:
                    items[-1] += " " + lines[i].strip()
                i += 1
            lis = []
            for item in items:
                if m := re.match(r"^\[( |x)\] (.*)", item):
                    mark = "✓" if m.group(1) == "x" else "○"
                    lis.append(f"<li>{mark} {inline(m.group(2), base)}</li>")
                else:
                    lis.append(f"<li>{inline(item, base)}</li>")
            out.append("<ul>" + "".join(lis) + "</ul>")
        else:
            body = []
            while i < len(lines) and lines[i].strip() and not re.match(r"^(#|- |\||>|```)", lines[i]):
                body.append(lines[i].strip())
                i += 1
            out.append("<p>" + inline(" ".join(body), base) + "</p>")
    return "\n".join(out)


def features_page() -> str:
    text = (ROOT / "docs" / "features.md").read_text(encoding="utf-8")
    # O título vira o cabeçalho da página; a linha de abertura aponta para o README, que aqui é a página inicial.
    lines = text.splitlines()
    lines = [l for l in lines if not l.startswith("# ")]
    text = "\n".join(lines).replace("For the short version, see the [README](../README.md).",
                                    "For the short version, see the [home page](https://idetea.belabs.cloud/).")
    body = markdown(text, "docs")
    template = (SITE / "features.template.html").read_text(encoding="utf-8")
    return template.replace("<!-- FEATURES -->", body)


def main() -> None:
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "target" / "site"
    if out.exists():
        shutil.rmtree(out)
    shutil.copytree(SITE, out, ignore=shutil.ignore_patterns("build.py", "*.template.html", "__pycache__"))
    for name in ["idetea.svg", "idetea.png", "demo.gif"]:
        shutil.copy(ROOT / "assets" / name, out / "img" / name)
    (out / "features.html").write_text(features_page(), encoding="utf-8")
    print(f"site em {out}")


if __name__ == "__main__":
    main()
