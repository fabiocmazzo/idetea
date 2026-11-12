#!/usr/bin/env python3
"""Recorta a Droid Sans Fallback para o texto ja/zh-CN da interface.

A fonte completa fica no sistema (pacote fonts-droid-fallback) e não entra no git.
O recorte é o que o binário embute quando a máquina não tem uma fonte CJK.
"""

import sys
from pathlib import Path

from fontTools.subset import Subsetter
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_SRC = Path("/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf")
OUT = ROOT / "crates/tea-core/assets/fonts/DroidSansFallback-Subset.ttf"

# Pontuação que uma frase em chinês ou japonês usa mesmo quando o .ftl ainda não citou.
EXTRA = (
    "、。・「」『』【】〔〕〖〗〈〉《》！？：；，．〜…—―（）々〆〇"
    "・｡｢｣､･"
    "　"
    "日本語中文（简体）、。！？"
)


def chars() -> str:
    found = set(EXTRA)
    for path in (ROOT / "crates").rglob("*.ftl"):
        if path.parent.name not in ("ja", "zh-CN"):
            continue
        found.update(path.read_text(encoding="utf-8"))
    # ASCII e o latim básico a UI já desenha com a fonte da interface.
    return "".join(sorted(ch for ch in found if ord(ch) > 0x7F))


def main() -> None:
    src = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_SRC
    text = chars()
    font = TTFont(src)
    subsetter = Subsetter()
    subsetter.populate(text=text)
    subsetter.subset(font)
    # Sem hinting o arquivo fica menor e o iced pede só o contorno.
    font["glyf"].glyphOrder  # garante a tabela carregada
    for table in ("prep", "cvt ", "fpgm"):
        if table in font:
            del font[table]
    OUT.parent.mkdir(parents=True, exist_ok=True)
    font.save(OUT)
    print(f"{OUT.relative_to(ROOT)}: {OUT.stat().st_size} bytes, {len(text)} caracteres fora do ASCII")


if __name__ == "__main__":
    main()
