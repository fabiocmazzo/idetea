#!/usr/bin/env python3
"""Gera crates/tea-core/assets/languages/icons.ttf: os ícones de arquivo dos language packs (RFC 0014).

Desenhos próprios da IDE TEA (Apache-2.0), no estilo do Seti: uma cor só, quadro de 1000 unidades, desenho entre
~180 e ~820 na vertical. Cada pack aponta o seu caractere em `[icon] glyph` do `pack.toml`.

Precisa do fonttools (`python3 -m venv venv && venv/bin/pip install fonttools`):

    venv/bin/python tools/language-icons.py [--preview pasta]

`--preview` também escreve um SVG de cada ícone (para conferir o desenho sem abrir a fonte).
"""

import math
import sys
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.ttGlyphPen import TTGlyphPen

OUT = Path(__file__).resolve().parent.parent / "crates/tea-core/assets/languages/icons.ttf"
FAMILY = "idetea-languages"
UPEM = 1000
ADVANCE = 1000

K = 0.5522847498  # controle da cúbica para um quarto de círculo


def circle(pen, cx, cy, r, clockwise=True):
    """Círculo em quatro cúbicas. Horário = contorno externo no TrueType; anti-horário = furo."""
    pts = [(cx, cy + r), (cx + r, cy), (cx, cy - r), (cx - r, cy)]
    if not clockwise:
        pts = [pts[0], pts[3], pts[2], pts[1]]
    pen.moveTo(pts[0])
    for i in range(4):
        a, b = pts[i], pts[(i + 1) % 4]
        # Controles: a partir de cada ponta, na direção tangente.
        ca = (a[0] + (b[0] - cx) * K, a[1] + (b[1] - cy) * K)
        cb = (b[0] + (a[0] - cx) * K, b[1] + (a[1] - cy) * K)
        pen.curveTo(ca, cb, b)
    pen.closePath()


def arc(pen, cx, cy, r, a0, a1):
    """Arco de `a0` a `a1` (radianos), em cúbicas de no máximo 90°; a caneta já está no ponto inicial."""
    n = max(1, math.ceil(abs(a1 - a0) / (math.pi / 2)))
    step = (a1 - a0) / n
    k = 4 / 3 * math.tan(step / 4)
    for i in range(n):
        t0, t1 = a0 + i * step, a0 + (i + 1) * step
        p0 = (cx + r * math.cos(t0), cy + r * math.sin(t0))
        p1 = (cx + r * math.cos(t1), cy + r * math.sin(t1))
        c0 = (p0[0] - k * r * math.sin(t0), p0[1] + k * r * math.cos(t0))
        c1 = (p1[0] + k * r * math.sin(t1), p1[1] - k * r * math.cos(t1))
        pen.curveTo(c0, c1, p1)


def ellipse(pen, cx, cy, rx, ry, angle_deg, clockwise=True):
    """Elipse girada, em quatro cúbicas."""
    a = math.radians(angle_deg)
    ca, sa = math.cos(a), math.sin(a)

    def rot(x, y):
        return (cx + x * ca - y * sa, cy + x * sa + y * ca)

    quads = [((0, ry), (rx, 0)), ((rx, 0), (0, -ry)), ((0, -ry), (-rx, 0)), ((-rx, 0), (0, ry))]
    if not clockwise:
        quads = [(b, a_) for a_, b in reversed(quads)]
    pen.moveTo(rot(*quads[0][0]))
    for p, q in quads:
        c0 = (p[0] + q[0] * K, p[1] + q[1] * K)
        c1 = (q[0] + p[0] * K, q[1] + p[1] * K)
        pen.curveTo(rot(*c0), rot(*c1), rot(*q))
    pen.closePath()


def rect(pen, x0, y0, x1, y1):
    """Retângulo no sentido horário (contorno externo)."""
    pen.moveTo((x0, y0))
    pen.lineTo((x0, y1))
    pen.lineTo((x1, y1))
    pen.lineTo((x1, y0))
    pen.closePath()


def moonbit(pen):
    """Lua crescente aberta para o alto à direita, com um bit (quadrado) na abertura."""
    ox, oy, orr = 460, 470, 300  # círculo de fora
    ix, iy, ir = 600, 590, 235  # círculo que recorta
    d = math.hypot(ix - ox, iy - oy)
    toward = math.atan2(iy - oy, ix - ox)  # direção do recorte, vista do centro de fora
    # Pontas da lua: interseção dos dois círculos, a ±alfa (de fora) e ±beta (do recorte) dessa direção.
    alpha = math.acos((orr**2 + d**2 - ir**2) / (2 * orr * d))
    beta = math.acos((ir**2 + d**2 - orr**2) / (2 * ir * d))
    a_tip = (ox + orr * math.cos(toward + alpha), oy + orr * math.sin(toward + alpha))
    # Por fora, o caminho longo (longe do recorte); por dentro, de volta pelo lado virado para o centro de fora.
    pen.moveTo(a_tip)
    arc(pen, ox, oy, orr, toward + alpha, toward - alpha + 2 * math.pi)
    # Vistas do centro do recorte, as pontas ficam a ±beta da direção do centro de fora; volta-se da ponta onde o
    # arco de fora terminou até a do começo, passando por essa direção.
    back = toward + math.pi
    b_tip = (ox + orr * math.cos(toward - alpha), oy + orr * math.sin(toward - alpha))
    start = math.atan2(b_tip[1] - iy, b_tip[0] - ix)
    sweep = 2 * beta if abs(math.remainder(start + beta - back, 2 * math.pi)) < 1e-6 else -2 * beta
    arc(pen, ix, iy, ir, start, start + sweep)
    pen.closePath()
    rect(pen, 660, 640, 770, 750)


def hare(pen):
    """Cabeça de lebre: duas orelhas longas, a cabeça e um olho vazado."""
    ellipse(pen, 395, 610, 70, 205, 14)
    ellipse(pen, 545, 625, 70, 205, -16)
    circle(pen, 470, 355, 175)
    circle(pen, 530, 385, 32, clockwise=False)


ICONS = [
    # (nome do glifo, caractere, desenho)
    ("hare", 0xE000, hare),
    ("moonbit", 0xE001, moonbit),
]


def build():
    names = [".notdef"] + [n for n, _, _ in ICONS]
    glyphs, metrics, paths = {}, {".notdef": (ADVANCE, 0)}, {}
    glyphs[".notdef"] = TTGlyphPen(None).glyph()
    for name, _, draw in ICONS:
        tt = TTGlyphPen(None)
        draw(Cu2QuPen(tt, max_err=1.0, reverse_direction=False))
        glyphs[name] = tt.glyph()
        metrics[name] = (ADVANCE, 0)
        svg = SVGPathPen(None)
        draw(svg)
        paths[name] = svg.getCommands()

    fb = FontBuilder(UPEM, isTTF=True)
    fb.setupGlyphOrder(names)
    fb.setupCharacterMap({cp: name for name, cp, _ in ICONS})
    fb.setupGlyf(glyphs)
    glyf = fb.font["glyf"]
    lsb = {}
    for n in names:
        glyf[n].recalcBounds(glyf)
        lsb[n] = getattr(glyf[n], "xMin", 0)
    fb.setupHorizontalMetrics({n: (adv, lsb[n]) for n, (adv, _) in metrics.items()})
    fb.setupHorizontalHeader(ascent=1000, descent=0)
    fb.setupNameTable({"familyName": FAMILY, "styleName": "Regular"})
    fb.setupOS2(sTypoAscender=1000, sTypoDescender=0, usWinAscent=1000, usWinDescent=0)
    fb.setupPost()
    fb.font["head"].created = fb.font["head"].modified = 0  # o mesmo arquivo a cada geração
    fb.save(OUT)
    return paths


def main():
    paths = build()
    print(f"{OUT} ({OUT.stat().st_size} bytes)")
    if "--preview" in sys.argv:
        out = Path(sys.argv[sys.argv.index("--preview") + 1])
        out.mkdir(parents=True, exist_ok=True)
        for name, d in paths.items():
            svg = (
                '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 1000" width="256" height="256">'
                '<rect width="1000" height="1000" fill="#1e1e1e"/>'
                f'<path transform="translate(0,1000) scale(1,-1)" fill="#ddd" d="{d}"/></svg>\n'
            )
            (out / f"{name}.svg").write_text(svg)
            print(out / f"{name}.svg")


if __name__ == "__main__":
    main()
