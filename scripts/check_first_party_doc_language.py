#!/usr/bin/env python3
"""
Fail if Cyrillic appears in first-party Markdown outside docs/.legacy-non-english-docs.
See ADR-004, docs/nona-03-manifest/SOUL.md Law #1.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ALLOW_FILE = ROOT / "docs" / ".legacy-non-english-docs"
CYRILLIC = re.compile(r"[А-Яа-яЁё]")

DIRS = ["docs", "specs", "architecture", "clara-bridge", "conformance"]
ROOT_MD = ["README.md", "AGENTS.md", "CLAUDE.md", "task.md", "SOUL.md"]


def load_allowed() -> set[str]:
    if not ALLOW_FILE.is_file():
        return set()
    out: set[str] = set()
    for line in ALLOW_FILE.read_text(encoding="utf-8").splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            out.add(line)
    return out


def main() -> int:
    allowed = load_allowed()
    errors: list[str] = []

    for d in DIRS:
        base = ROOT / d
        if not base.is_dir():
            continue
        for path in base.rglob("*.md"):
            rel = path.relative_to(ROOT).as_posix()
            if rel in allowed:
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            if CYRILLIC.search(text):
                errors.append(rel)

    for name in ROOT_MD:
        path = ROOT / name
        if not path.is_file():
            continue
        if name in allowed:
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        if CYRILLIC.search(text):
            errors.append(name)

    for rel in sorted(errors):
        print(
            f"ERROR: Cyrillic in first-party Markdown (not in docs/.legacy-non-english-docs): {rel}",
            file=sys.stderr,
        )
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
