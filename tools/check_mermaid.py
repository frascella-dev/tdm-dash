#!/usr/bin/env python3
"""Render every ```mermaid block in the repo's Markdown files with mermaid-cli.

A block that fails to render fails the check, so broken diagrams never reach
the manual. Usage:

    python3 tools/check_mermaid.py --mmdc node_modules/.bin/mmdc [--chrome PATH]
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
BLOCK = re.compile(r"^```mermaid\s*\n(.*?)^```\s*$", re.S | re.M)
SKIP_DIRS = {"node_modules", "target", "buildroot", "output", ".git"}


def markdown_files() -> list[pathlib.Path]:
    return sorted(
        p
        for p in ROOT.rglob("*.md")
        if not SKIP_DIRS.intersection(p.relative_to(ROOT).parts)
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--mmdc", required=True, help="path to the mmdc binary")
    ap.add_argument("--chrome", help="Chromium executable (optional)")
    args = ap.parse_args()

    failures = 0
    total = 0
    with tempfile.TemporaryDirectory() as tmp:
        tmpdir = pathlib.Path(tmp)
        puppeteer_cfg = tmpdir / "puppeteer.json"
        cfg: dict = {"args": ["--no-sandbox"]}
        if args.chrome:
            cfg["executablePath"] = args.chrome
        puppeteer_cfg.write_text(json.dumps(cfg))

        for md in markdown_files():
            text = md.read_text(encoding="utf-8")
            for i, m in enumerate(BLOCK.finditer(text)):
                total += 1
                line = text.count("\n", 0, m.start()) + 1
                src = tmpdir / f"block{total}.mmd"
                src.write_text(m.group(1), encoding="utf-8")
                out = tmpdir / f"block{total}.svg"
                proc = subprocess.run(
                    [args.mmdc, "-q", "-p", str(puppeteer_cfg), "-i", str(src), "-o", str(out)],
                    capture_output=True,
                    text=True,
                )
                rel = md.relative_to(ROOT)
                if proc.returncode != 0:
                    failures += 1
                    print(f"FAIL {rel}:{line} (block {i + 1})")
                    print((proc.stderr or proc.stdout).strip()[:2000])
                else:
                    print(f"ok   {rel}:{line}")

    print(f"\n{total - failures}/{total} Mermaid blocks rendered")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
