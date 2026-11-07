#!/usr/bin/env python3
"""Converte uma gramática TextMate (.tmLanguage.json, a do VS Code) para .sublime-syntax, o formato do syntect.

Uso:
    python3 tools/tm2sublime.py zx.tmLanguage.json > crates/tea-core/assets/syntaxes/zx.sublime-syntax

Regras da conversão:
- `match` vira `match` com `scope` (o `name`) e `captures`; a captura 0 vira o `scope` da regra.
- `begin`/`end` vira um `match` que empurra um contexto: `meta_scope` (o `name`), `meta_content_scope`
  (o `contentName`), o `end` primeiro (no TextMate ele vence os padrões na mesma posição) e depois os padrões.
  Referências a capturas do `begin` no `end` (`\\2`) valem igual no sublime-syntax.
- `include`: `#regra` vira `regra`, `$self` vira `main`, `source.x` vira `scope:source.x` e
  `source.x#regra` vira `scope:source.x#<contexto>` pela tabela `EXTERNAL`.
- A gramática do ZX entra no Zig por injeção, que o syntect não tem. `--inject-into source.zig` faz o `main`
  empurrar o Zig com as regras de `--inject` como `with_prototype`: elas valem em todo contexto do Zig.
  O VS Code não injeta dentro de strings e comentários; aqui, `--guard regra` põe essa regra (a que reconhece
  strings e comentários) antes das injetadas, e os contextos que ela empurra ficam sem o protótipo
  (`meta_include_prototype: false`), para a marcação não começar dentro de uma string.
"""

import argparse
import json
import sys

# Regras de outras gramáticas que o VS Code cita pelo nome e que têm outro nome no sublime-syntax.
EXTERNAL = {
    "source.zig#stringcontent": "scope:source.zig#character_escapes",
}


def include_target(ref):
    if ref == "$self":
        return "main"
    if ref.startswith("#"):
        return ref[1:]
    if ref in EXTERNAL:
        return EXTERNAL[ref]
    if "#" in ref:
        raise SystemExit(f"include sem equivalente: {ref} (acrescente em EXTERNAL)")
    return f"scope:{ref}"


def captures(caps):
    """(scope da captura 0, mapa das outras)."""
    whole, rest = None, {}
    for k, v in (caps or {}).items():
        name = v.get("name")
        if v.get("patterns"):
            raise SystemExit(f"captura com padrões não é suportada: {k}")
        if not name:
            continue
        if k == "0":
            whole = name
        else:
            rest[int(k)] = name
    return whole, rest


def rule(r, closed=False):
    """`closed`: os contextos empurrados não recebem o protótipo (ver `--guard`)."""
    if "include" in r:
        return [{"include": include_target(r["include"])}]
    if "match" in r:
        out = {"match": r["match"]}
        whole, caps = captures(r.get("captures"))
        scope = " ".join(s for s in [r.get("name"), whole] if s)
        if scope:
            out["scope"] = scope
        if caps:
            out["captures"] = caps
        return [out]
    if "begin" in r:
        if "while" in r:
            raise SystemExit("begin/while não existe no sublime-syntax")
        out = {"match": r["begin"]}
        whole, caps = captures(r.get("beginCaptures") or r.get("captures"))
        if whole:
            out["scope"] = whole
        if caps:
            out["captures"] = caps
        ctx = [{"meta_include_prototype": False}] if closed else []
        if r.get("name"):
            ctx.append({"meta_scope": r["name"]})
        if r.get("contentName"):
            ctx.append({"meta_content_scope": r["contentName"]})
        end = {"match": r["end"]}
        whole, caps = captures(r.get("endCaptures") or r.get("captures"))
        if whole:
            end["scope"] = whole
        if caps:
            end["captures"] = caps
        end["pop"] = True
        body = patterns(r.get("patterns", []), closed)
        if r.get("applyEndPatternLast"):
            ctx += body + [end]
        else:
            ctx += [end] + body
        out["push"] = ctx
        return [out]
    if "patterns" in r:
        return patterns(r["patterns"], closed)
    return []


def patterns(ps, closed=False):
    out = []
    for p in ps:
        out += rule(p, closed)
    return out


def convert(grammar, extensions, inject_into, inject, guards):
    contexts = {}
    for name, r in grammar.get("repository", {}).items():
        contexts[name] = rule(r, closed=name in guards)
    if inject_into:
        contexts["main"] = [
            {
                "match": "",
                "push": f"scope:{inject_into}",
                "with_prototype": [{"include": i} for i in guards + inject],
            }
        ]
    else:
        contexts["main"] = patterns(grammar.get("patterns", []))
    return {
        "name": grammar.get("name", "?").upper() if grammar.get("name") else "?",
        "file_extensions": extensions or grammar.get("fileTypes", []),
        "scope": grammar["scopeName"],
        "contexts": contexts,
    }


def dump(value, indent=0):
    """YAML em bloco, com os textos em aspas duplas (JSON é YAML válido nos escapes que usamos)."""
    pad = "  " * indent
    lines = []
    if isinstance(value, dict):
        for k, v in value.items():
            key = k if isinstance(k, str) else str(k)
            if isinstance(v, (dict, list)) and v:
                lines.append(f"{pad}{key}:")
                lines.append(dump(v, indent + 1))
            else:
                lines.append(f"{pad}{key}: {scalar(v)}")
    else:
        for item in value:
            if isinstance(item, dict) and item:
                inner = dump(item, indent + 1).split("\n")
                first = inner[0].lstrip()
                lines.append(f"{pad}- {first}")
                lines.extend(inner[1:])
            else:
                lines.append(f"{pad}- {scalar(item)}")
    return "\n".join(lines)


def scalar(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, float)):
        return str(v)
    if isinstance(v, list) and not v:
        return "[]"
    return json.dumps(v, ensure_ascii=False)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("grammar")
    ap.add_argument("--name", help="nome da sintaxe (padrão: o da gramática)")
    ap.add_argument("--ext", action="append", default=[], help="extensão de arquivo (repetível)")
    ap.add_argument("--inject-into", help="escopo que a gramática injeta (ex.: source.zig)")
    ap.add_argument("--inject", action="append", default=[], help="regra que vale dentro do injetado (repetível)")
    ap.add_argument("--guard", action="append", default=[], help="regra de strings/comentários, sem injeção dentro")
    ap.add_argument("--header", default="", help="comentário no topo (origem e licença)")
    args = ap.parse_args()
    with open(args.grammar, encoding="utf-8") as f:
        grammar = json.load(f)
    syntax = convert(grammar, args.ext, args.inject_into, args.inject, args.guard)
    if args.name:
        syntax["name"] = args.name
    out = ["%YAML 1.2", "---"]
    out += [f"# {line}" if line else "#" for line in args.header.split("\n") if args.header]
    out.append(dump(syntax))
    sys.stdout.write("\n".join(out) + "\n")


if __name__ == "__main__":
    main()
