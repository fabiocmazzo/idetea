---
name: texto-cjk-e-o-recorte-da-fonte
description: texto novo em ja/zh-CN precisa ter glifo no recorte da fonte CJK embutida; o teste do tea-core confere
type: gotcha
updated: 2026-11-23
---
A IDE embute um recorte da Droid Sans Fallback (`crates/tea-core/assets/fonts/DroidSansFallback-Subset.ttf`) só com
os caracteres que os `.ftl` de `ja` e `zh-CN` usam. Um texto novo com um ideograma fora do recorte quebra o teste
`fonts::tests::cjk_subset_covers_ui_copy` do `tea-core`, não o da crate que ganhou o texto: rode
`cargo test -p tea-core` (ou `just check`) depois de mexer nesses `.ftl`.

Para corrigir: recortar de novo com `tools/fonts/subset_cjk.py` (precisa da fonte completa,
`/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf`, e do `fonttools`), ou trocar a palavra por outra que o
recorte já cobre. Para ver o que falta sem instalar nada:
`uv run -q --with fonttools python -c "from fontTools.ttLib import TTFont; ..."` lendo o `getBestCmap()` do recorte.

**Why:** em 2026-11-23, `响铃了` (o sino, RFC 0024) entrou num commit com o teste do `tea-core` quebrado.
