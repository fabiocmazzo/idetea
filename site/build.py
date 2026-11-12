#!/usr/bin/env python3
"""Monta o site em uma pasta (padrão: target/site), só com a biblioteca padrão.

- copia site/ (menos este script e os templates);
- traz de assets/ o ícone, o GIF antigo, o pôster, o GIF curto e o mp4 da demo;
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


# id, pasta na URL (vazia = raiz), rótulo curto, hreflang
LOCALES = [
    ("en", "", "EN", "en"),
    ("pt-BR", "pt-br", "PT", "pt-BR"),
    ("es", "es", "ES", "es"),
    ("fr", "fr", "FR", "fr"),
    ("de", "de", "DE", "de"),
    ("it", "it", "IT", "it"),
    ("ja", "ja", "JA", "ja"),
    ("zh-CN", "zh-cn", "中文", "zh-CN"),
]
# pt-BR e es não têm features.html próprio: a lista continua a inglesa, como antes.
FEATURES_LOCALES = {"en", "fr", "de", "it", "ja", "zh-CN"}
SITE_ORIGIN = "https://idetea.belabs.cloud/"
ARIA = {
    "en": "Language",
    "pt-BR": "Idioma",
    "es": "Idioma",
    "fr": "Langue",
    "de": "Sprache",
    "it": "Lingua",
    "ja": "言語",
    "zh-CN": "语言",
}
CHROME = {
    "en": ("Features · IDE TEA", "Everything IDE TEA does: editor, project, LSP, terminal, git, debug, agents, shortcuts.",
           "Home", "Getting started", "Working with agents", "Features", "Download", "Features and shortcuts",
           "made with", "Report a bug"),
    "fr": ("Fonctions · IDE TEA", "Tout ce que fait IDE TEA : éditeur, projet, LSP, terminal, git, débogage, agents, raccourcis.",
           "Accueil", "Premiers pas", "Travailler avec les agents", "Fonctions", "Télécharger", "Fonctions et raccourcis",
           "fait avec", "Signaler un bug"),
    "de": ("Funktionen · IDE TEA", "Alles, was IDE TEA kann: Editor, Projekt, LSP, Terminal, Git, Debug, Agenten, Tastenkürzel.",
           "Start", "Erste Schritte", "Mit Agenten arbeiten", "Funktionen", "Herunterladen", "Funktionen und Tastenkürzel",
           "gemacht mit", "Fehler melden"),
    "it": ("Funzioni · IDE TEA", "Tutto ciò che fa IDE TEA: editor, progetto, LSP, terminale, git, debug, agenti, scelte rapide.",
           "Home", "Per iniziare", "Lavorare con gli agenti", "Funzioni", "Scarica", "Funzioni e scelte rapide",
           "fatto con", "Segnala un bug"),
    "ja": ("機能 · IDE TEA", "IDE TEA ができること: エディター、プロジェクト、LSP、ターミナル、Git、デバッグ、エージェント、ショートカット。",
           "ホーム", "はじめに", "エージェントと働く", "機能", "ダウンロード", "機能とショートカット",
           "使用", "バグを報告"),
    "zh-CN": ("功能 · IDE TEA", "IDE TEA 能做的事：编辑器、项目、LSP、终端、Git、调试、智能体、快捷键。",
              "首页", "入门", "与智能体协作", "功能", "下载", "功能与快捷键",
              "构建于", "报告问题"),
}


def locale_by_id(lang_id: str) -> tuple[str, str, str, str]:
    return next(item for item in LOCALES if item[0] == lang_id)


def rel_href(current: str, target: str, page: str) -> str:
    """Link relativo de uma página de `current` para a mesma página em `target`."""
    if page == "features" and target not in FEATURES_LOCALES:
        page = "index"
        # Sem features traduzida: a entrada aponta para a página inicial daquele idioma.
    src = Path(locale_by_id(current)[1] or ".")
    folder, _, _, _ = locale_by_id(target)
    folder = locale_by_id(target)[1]
    name = "index.html" if page == "index" else f"{page}.html"
    dst = Path(folder) / name if folder else Path(name)
    link = Path(os_relpath(dst, src)).as_posix()
    if page == "index":
        if link == "index.html":
            return "./"
        if link.endswith("/index.html"):
            return link[: -len("index.html")]
        if link == "../index.html":
            return "../"
    return link


def os_relpath(dst: Path, src: Path) -> str:
    return str(Path(os_path_relpath(dst, src)))


def os_path_relpath(dst: Path, src: Path) -> str:
    import os
    return os.path.relpath(dst, src)


def absolute_url(lang_id: str, page: str) -> str:
    folder = locale_by_id(lang_id)[1]
    prefix = f"{folder}/" if folder else ""
    if page == "index":
        return SITE_ORIGIN + prefix
    return SITE_ORIGIN + prefix + page + ".html"


def apply_language_chrome(html: str, lang_id: str, page: str) -> str:
    """Troca o seletor de idioma e os hreflang. Idempotente."""
    links = []
    for target, _, label, hreflang in LOCALES:
        href = rel_href(lang_id, target, page)
        if target == lang_id:
            links.append(f'<a href="{href}" aria-current="true">{label}</a>')
        else:
            links.append(f'<a href="{href}" hreflang="{hreflang}">{label}</a>')
    nav = f'<nav class="lang" aria-label="{ARIA[lang_id]}">{"".join(links)}</nav>'
    html = re.sub(r'<nav class="lang"[^>]*>.*?</nav>', nav, html, count=1)
    html = re.sub(r'\n?<link rel="alternate" hreflang="[^"]+" href="[^"]+">', "", html)
    alternates = []
    targets = [item for item in LOCALES if page != "features" or item[0] in FEATURES_LOCALES]
    for target, _, _, hreflang in targets:
        alternates.append(
            f'<link rel="alternate" hreflang="{hreflang}" href="{absolute_url(target, page)}">'
        )
    alternates.append(f'<link rel="alternate" hreflang="x-default" href="{absolute_url("en", page)}">')
    block = "\n".join(alternates) + "\n"
    html = html.replace('<link rel="icon"', block + '<link rel="icon"', 1)
    return html


def page_kind(path: Path, root: Path) -> tuple[str, str] | None:
    rel = path.relative_to(root)
    name = path.name
    pages = {"index.html": "index", "start.html": "start", "agents.html": "agents", "features.html": "features"}
    if name not in pages:
        return None
    if len(rel.parts) == 1:
        return "en", pages[name]
    folder = rel.parts[0]
    for lang_id, url_folder, _, _ in LOCALES:
        if url_folder == folder:
            return lang_id, pages[name]
    return None


def stamp_languages(root: Path) -> None:
    for path in root.rglob("*.html"):
        kind = page_kind(path, root)
        if kind is None:
            continue
        lang_id, page = kind
        text = path.read_text(encoding="utf-8")
        path.write_text(apply_language_chrome(text, lang_id, page), encoding="utf-8")


def features_body(lang_id: str) -> str | None:
    if lang_id == "en":
        path = ROOT / "docs" / "features.md"
    else:
        path = ROOT / "docs" / f"features.{lang_id}.md"
    if not path.is_file():
        return None
    lines = [line for line in path.read_text(encoding="utf-8").splitlines() if not line.startswith("# ")]
    text = "\n".join(lines)
    folder = locale_by_id(lang_id)[1]
    home = SITE_ORIGIN + (f"{folder}/" if folder else "")
    # No site, o link de abertura vai para a página inicial deste idioma, não para o README no GitHub.
    text = re.sub(r"\]\(\.\./README[^)]*\)", f"]({home})", text, count=1)
    return markdown(text, "docs")


def features_page(lang_id: str) -> str | None:
    body = features_body(lang_id)
    if body is None:
        return None
    title, description, home, start, agents, features, download, h1, made, bug = CHROME[lang_id]
    folder = locale_by_id(lang_id)[1]
    template = (SITE / "features.template.html").read_text(encoding="utf-8")
    html = template
    repl = {
        "{{HTML_LANG}}": "zh-CN" if lang_id == "zh-CN" else lang_id,
        "{{TITLE}}": html_escape_attr(title),
        "{{DESCRIPTION}}": html_escape_attr(description),
        "{{PREFIX}}": "../" if folder else "",
        "{{NAV_HOME}}": home,
        "{{NAV_START}}": start,
        "{{NAV_AGENTS}}": agents,
        "{{NAV_FEATURES}}": features,
        "{{DOWNLOAD}}": download,
        "{{H1}}": h1,
        "{{MADE}}": made,
        "{{BUG}}": bug,
    }
    for key, value in repl.items():
        html = html.replace(key, value)
    return html.replace("<!-- FEATURES -->", body)


def html_escape_attr(text: str) -> str:
    return html.escape(text, quote=True)


def check_links(root: Path) -> list[str]:
    """Links internos (href/src relativos e âncoras) e a presença do CNAME. Sem rede."""
    problems: list[str] = []
    if not (root / "CNAME").is_file():
        problems.append("CNAME ausente")
    cname = (root / "CNAME").read_text(encoding="utf-8").strip() if (root / "CNAME").is_file() else ""
    if cname != "idetea.belabs.cloud":
        problems.append(f"CNAME inesperado: {cname!r}")
    html_pages = list(root.rglob("*.html"))
    if not html_pages:
        problems.append("nenhuma página HTML")
    required = [
        root / "index.html",
        root / "start.html",
        root / "agents.html",
        root / "features.html",
        root / "zh-cn" / "index.html",
        root / "ja" / "index.html",
        root / "fr" / "index.html",
        root / "de" / "index.html",
        root / "it" / "index.html",
    ]
    for path in required:
        if not path.is_file():
            problems.append(f"página obrigatória ausente: {path.relative_to(root)}")
    css = (root / "style.css").read_text(encoding="utf-8") if (root / "style.css").is_file() else ""
    if "Noto Sans SC" not in css or "Noto Sans JP" not in css:
        problems.append("style.css sem a pilha de fontes CJK")
    tracker = re.compile(r"google-analytics|googletagmanager|plausible|umami|mc\.yandex|facebook\.net|doubleclick", re.I)
    for page in html_pages:
        text = page.read_text(encoding="utf-8")
        if tracker.search(text):
            problems.append(f"{page.relative_to(root)}: rastreador")
        if "hreflang" not in text:
            problems.append(f"{page.relative_to(root)}: sem hreflang")
        for match in re.finditer(r'''(?:href|src)="([^"]+)"''', text):
            # HTML cru escapado no Markdown (um seletor colado como texto) não é um link.
            if "&lt;" in text[max(0, match.start() - 12):match.start()]:
                continue
            url = match.group(1)
            if url.startswith(("http://", "https://", "mailto:", "data:")):
                continue
            path_part, _, frag = url.partition("#")
            if path_part in ("",):
                target = page
            else:
                target = (page.parent / path_part).resolve()
                try:
                    target.relative_to(root.resolve())
                except ValueError:
                    problems.append(f"{page.relative_to(root)}: sai do site {url}")
                    continue
                if not target.exists():
                    problems.append(f"{page.relative_to(root)}: falta {url}")
                    continue
            if frag and target.suffix == ".html":
                body = target.read_text(encoding="utf-8", errors="replace")
                if f'id="{frag}"' not in body and f"id='{frag}'" not in body:
                    problems.append(f"{page.relative_to(root)}: âncora #{frag} ausente em {url}")
    zh = root / "zh-cn" / "index.html"
    ja = root / "ja" / "index.html"
    if zh.is_file() and not re.search(r"[\u4e00-\u9fff]", zh.read_text(encoding="utf-8")):
        problems.append("zh-cn/index.html sem caracteres CJK")
    if ja.is_file() and not re.search(r"[\u3040-\u30ff]", ja.read_text(encoding="utf-8")):
        problems.append("ja/index.html sem kana")
    return problems


def main() -> None:
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "target" / "site"
    if out.exists():
        shutil.rmtree(out)
    shutil.copytree(SITE, out, ignore=shutil.ignore_patterns("build.py", "*.template.html", "__pycache__"))
    for name in ["idetea.svg", "idetea.png", "demo.gif", "demo-poster.png", "demo-preview.gif", "idetea-demo.mp4"]:
        shutil.copy(ROOT / "assets" / name, out / "img" / name)
    for lang_id in FEATURES_LOCALES:
        page = features_page(lang_id)
        if page is None:
            continue
        folder = locale_by_id(lang_id)[1]
        dest = out / folder / "features.html" if folder else out / "features.html"
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(page, encoding="utf-8")
    stamp_languages(out)
    problems = check_links(out)
    if problems:
        print("links:\n" + "\n".join(problems), file=sys.stderr)
        sys.exit(1)
    print(f"site em {out}")


if __name__ == "__main__":
    main()
