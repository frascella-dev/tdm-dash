"""Minimal S-expression reader/writer for KiCad files.

Atoms keep their quoting: a quoted string is stored as `Q("text")`, a bare
token as a plain `str`. That round-trips library symbols exactly enough for
KiCad to load them back.
"""

from __future__ import annotations


class Q(str):
    """A quoted string atom."""


def parse(text: str):
    pos = 0
    n = len(text)
    stack: list[list] = [[]]
    while pos < n:
        c = text[pos]
        if c in " \t\r\n":
            pos += 1
        elif c == "(":
            stack.append([])
            pos += 1
        elif c == ")":
            item = stack.pop()
            stack[-1].append(item)
            pos += 1
        elif c == '"':
            pos += 1
            buf = []
            while text[pos] != '"':
                if text[pos] == "\\":
                    pos += 1
                    buf.append({"n": "\n", "t": "\t"}.get(text[pos], text[pos]))
                else:
                    buf.append(text[pos])
                pos += 1
            pos += 1
            stack[-1].append(Q("".join(buf)))
        else:
            start = pos
            while pos < n and text[pos] not in ' \t\r\n()"':
                pos += 1
            stack[-1].append(text[start:pos])
    assert len(stack) == 1, "unbalanced parentheses"
    return stack[0]


def quote(s: str) -> str:
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n") + '"'


def fmt_num(v) -> str:
    if isinstance(v, bool):
        return "yes" if v else "no"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, float):
        s = f"{v:.4f}".rstrip("0").rstrip(".")
        return "0" if s in ("-0", "") else s
    return str(v)


def dump(node, indent: int = 0) -> str:
    """Pretty-prints like KiCad: one list per line, tabs for indentation."""
    if not isinstance(node, list):
        return quote(node) if isinstance(node, Q) else fmt_num(node)
    simple = all(not isinstance(x, list) for x in node)
    if simple:
        return "(" + " ".join(dump(x) for x in node) + ")"
    pad = "\t" * (indent + 1)
    head = []
    rest = []
    for x in node:
        if not rest and not isinstance(x, list):
            head.append(dump(x))
        else:
            rest.append(x)
    lines = ["(" + " ".join(head)]
    for x in rest:
        lines.append(pad + dump(x, indent + 1))
    lines.append("\t" * indent + ")")
    return "\n".join(lines)


def find(node: list, key: str):
    for x in node:
        if isinstance(x, list) and x and x[0] == key:
            return x
    return None


def find_all(node: list, key: str):
    return [x for x in node if isinstance(x, list) and x and x[0] == key]
