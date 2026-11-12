#!/usr/bin/env python3
"""Minimal YAML subset loader for the demo scenes file (no PyYAML required)."""
from __future__ import annotations

import ast
import re
from typing import Any


def load(path) -> Any:
    text = Path_read(path)
    lines = []
    for raw in text.splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        lines.append(raw.rstrip())
    value, _ = _parse_block(lines, 0, 0)
    return value


def Path_read(path) -> str:
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _parse_block(lines: list[str], i: int, min_indent: int) -> tuple[Any, int]:
    if i >= len(lines):
        return None, i
    if _indent(lines[i]) < min_indent:
        return None, i
    if lines[i].lstrip().startswith("- "):
        return _parse_list(lines, i, _indent(lines[i]))
    return _parse_map(lines, i, _indent(lines[i]))


def _parse_map(lines: list[str], i: int, indent: int) -> tuple[dict, int]:
    result: dict[str, Any] = {}
    while i < len(lines):
        line = lines[i]
        ind = _indent(line)
        if ind < indent:
            break
        if ind > indent:
            raise ValueError(f"unexpected indent at {line!r}")
        if line.lstrip().startswith("- "):
            break
        key, rest = _split_key(line.lstrip())
        if rest is None:
            # key:  (nested block on following lines)
            i += 1
            value, i = _parse_block(lines, i, indent + 1)
            result[key] = value
            continue
        if rest == "" or rest == "|":
            i += 1
            value, i = _parse_block(lines, i, indent + 1)
            result[key] = value
            continue
        result[key] = _parse_value(rest)
        i += 1
    return result, i


def _parse_list(lines: list[str], i: int, indent: int) -> tuple[list, int]:
    result: list[Any] = []
    while i < len(lines):
        line = lines[i]
        ind = _indent(line)
        if ind < indent:
            break
        if ind > indent:
            raise ValueError(f"unexpected indent at {line!r}")
        if not line.lstrip().startswith("- "):
            break
        body = line.lstrip()[2:]
        # "- key: value" or "- value" or "- {..}"
        if ":" in body and not body.startswith("{") and not body.startswith("[") and not body.startswith(("'", '"')):
            # Treat as a one-line or nested map item. The map's indent is the
            # first column of "key" (indent + 2).
            # Reconstruct a virtual map starting at this line without the dash.
            virtual = " " * (ind + 2) + body
            # Peek following lines that belong to this item (indent > indent)
            j = i + 1
            extra = []
            while j < len(lines) and _indent(lines[j]) > indent:
                extra.append(lines[j])
                j += 1
            sub_lines = [virtual, *extra]
            value, _ = _parse_block(sub_lines, 0, ind + 2)
            result.append(value)
            i = j
        else:
            result.append(_parse_value(body))
            i += 1
            # multiline list value not supported
    return result, i


def _split_key(line: str) -> tuple[str, str | None]:
    m = re.match(r"([^:]+):(.*)$", line)
    if not m:
        raise ValueError(f"not a key: {line!r}")
    key = m.group(1).strip()
    rest = m.group(2).strip()
    return key, (None if rest == "" else rest)


def _parse_value(s: str) -> Any:
    s = s.strip()
    if s in ("true", "True", "yes", "Yes"):
        return True
    if s in ("false", "False", "no", "No"):
        return False
    if s in ("null", "Null", "~", ""):
        return None
    if re.fullmatch(r"-?\d+", s):
        return int(s)
    if re.fullmatch(r"-?\d+\.\d+", s):
        return float(s)
    if s.startswith("{") and s.endswith("}"):
        return _parse_flow_map(s)
    if s.startswith("[") and s.endswith("]"):
        inner = s[1:-1].strip()
        if not inner:
            return []
        return [_parse_value(p.strip()) for p in _split_flow(inner)]
    if (s.startswith('"') and s.endswith('"')) or (s.startswith("'") and s.endswith("'")):
        return ast.literal_eval(s)
    return s


def _parse_flow_map(s: str) -> dict:
    inner = s[1:-1].strip()
    if not inner:
        return {}
    result = {}
    for part in _split_flow(inner):
        k, v = part.split(":", 1)
        result[k.strip()] = _parse_value(v.strip())
    return result


def _split_flow(s: str) -> list[str]:
    parts, buf, depth, quote = [], [], 0, None
    for ch in s:
        if quote:
            buf.append(ch)
            if ch == quote and (not buf or buf[-2] != "\\"):
                quote = None
            continue
        if ch in ('"', "'"):
            quote = ch
            buf.append(ch)
        elif ch in "{[":
            depth += 1
            buf.append(ch)
        elif ch in "}]":
            depth -= 1
            buf.append(ch)
        elif ch == "," and depth == 0:
            parts.append("".join(buf).strip())
            buf = []
        else:
            buf.append(ch)
    if buf:
        parts.append("".join(buf).strip())
    return parts
