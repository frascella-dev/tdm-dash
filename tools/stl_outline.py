#!/usr/bin/env python3
"""Turns a 2.5D line-art relief (STL) into a flat SVG drawing.

The relief is a drawing extruded a couple of millimetres: its bottom faces all
lie on one plane and together cover exactly the black parts of the drawing.
The script keeps those faces, welds their vertices, follows the edges used by
a single triangle (the outline) into closed loops and writes them as one SVG
path with the even-odd fill rule. Loops lying entirely inside a --drop box
are left out (used to remove lettering and logos), and every loop is
simplified with Ramer-Douglas-Peucker.

    python3 tools/stl_outline.py model.stl out.svg \
        --drop 120,56,156,74 --drop 148,59,194,89 --drop 99,139,115,144

Coordinates of the drop boxes are in millimetres in the output drawing
(x to the right, y down, origin at the top-left corner of the drawing).
Needs numpy.
"""

from __future__ import annotations

import argparse
from collections import defaultdict

import numpy as np


def read_stl(path: str) -> np.ndarray:
    raw = np.fromfile(path, dtype=np.uint8)
    n = int(np.frombuffer(raw[80:84].tobytes(), "<u4")[0])
    if 84 + 50 * n != raw.size:
        raise SystemExit("only binary STL files are supported")
    rec = np.dtype([("n", "<f4", 3), ("v", "<f4", (3, 3)), ("a", "<u2")])
    return np.frombuffer(raw[84:].tobytes(), dtype=rec, count=n)["v"].astype(np.float64)


def outline_loops(tris: np.ndarray) -> tuple[np.ndarray, list[list[int]]]:
    key = np.round(tris.reshape(-1, 2) * 1000).astype(np.int64)
    uniq, idx = np.unique(key, axis=0, return_inverse=True)
    count: dict[tuple[int, int], int] = defaultdict(int)
    for a, b, c in idx.reshape(-1, 3):
        for e in ((a, b), (b, c), (c, a)):
            count[(min(e), max(e))] += 1
    adj: dict[int, list[int]] = defaultdict(list)
    for (a, b), k in count.items():
        if k == 1:
            adj[a].append(b)
            adj[b].append(a)
    seen: set[tuple[int, int]] = set()
    loops = []
    for start in list(adj):
        for first in adj[start]:
            if (min(start, first), max(start, first)) in seen:
                continue
            loop, prev, cur = [start], start, first
            seen.add((min(start, first), max(start, first)))
            while cur != start:
                loop.append(cur)
                nxt = next((x for x in adj[cur] if x != prev and (min(cur, x), max(cur, x)) not in seen), None)
                if nxt is None:
                    break
                seen.add((min(cur, nxt), max(cur, nxt)))
                prev, cur = cur, nxt
            if len(loop) > 2:
                loops.append(loop)
    return uniq / 1000.0, loops


def rdp(pts: np.ndarray, eps: float) -> np.ndarray:
    if len(pts) < 3:
        return pts
    a, b = pts[0], pts[-1]
    ab = b - a
    norm = np.hypot(*ab)
    if norm == 0:
        dist = np.hypot(*(pts - a).T)
    else:
        dist = np.abs(ab[0] * (pts[:, 1] - a[1]) - ab[1] * (pts[:, 0] - a[0])) / norm
    i = int(np.argmax(dist))
    if dist[i] <= eps:
        return np.array([a, b])
    return np.vstack([rdp(pts[: i + 1], eps)[:-1], rdp(pts[i:], eps)])


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("stl")
    ap.add_argument("svg")
    ap.add_argument("--drop", action="append", default=[], help="x0,y0,x1,y1 box in mm")
    ap.add_argument("--eps", type=float, default=0.04, help="simplification tolerance in mm")
    ap.add_argument("--title", default="")
    a = ap.parse_args()

    v = read_stl(a.stl)
    z = np.round(v[:, :, 2], 3)
    base = z.min()
    tris = v[(z.max(1) == base) & (z.min(1) == base)][:, :, :2]
    pts, loops = outline_loops(tris)
    x0, y1 = pts[:, 0].min(), pts[:, 1].max()
    width, height = pts[:, 0].max() - x0, y1 - pts[:, 1].min()
    pts = np.column_stack([pts[:, 0] - x0, y1 - pts[:, 1]])  # y down

    boxes = [tuple(map(float, d.split(","))) for d in a.drop]
    parts, dropped = [], 0
    for loop in loops:
        p = pts[loop]
        lo, hi = p.min(0), p.max(0)
        if any(lo[0] >= bx0 and lo[1] >= by0 and hi[0] <= bx1 and hi[1] <= by1 for bx0, by0, bx1, by1 in boxes):
            dropped += 1
            continue
        closed = np.vstack([p, p[:1]])
        s = rdp(closed, a.eps)[:-1]
        parts.append("M" + " ".join(f"{x:.2f} {y:.2f}" for x, y in s) + "Z")
    title = f"<title>{a.title}</title>" if a.title else ""
    with open(a.svg, "w", encoding="utf-8") as f:
        f.write(
            f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.2f} {height:.2f}">{title}'
            f'<path fill="#0f1113" fill-rule="evenodd" d="{" ".join(parts)}"/></svg>\n'
        )
    print(f"{a.svg}: {len(parts)} outlines ({dropped} dropped), {width:.1f} × {height:.1f} mm")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
