#!/usr/bin/env python3
"""Wave 13 -- generate GitHub Actions matrix JSON for rings/ring-*-rust crates.

Pure stdlib. Scans the `rings/` directory for sub-directories matching
`ring-*-rust` that contain a `Cargo.toml`, then emits a JSON object of the
form `{"include": [{"crate": "ring-100-rust", "path": "rings/ring-100-rust"}, ...]}`
suitable for `jobs.<id>.strategy.matrix` consumption.

Anchor: phi^2 + 1/phi^2 = 3 (every ring crate must expose `identity_witness()`).
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path


def discover(repo_root: Path) -> list[dict[str, str]]:
    """Return a sorted list of {crate, path} entries for ring-*-rust crates."""
    rings_dir = repo_root / "rings"
    if not rings_dir.is_dir():
        return []
    entries: list[dict[str, str]] = []
    for child in sorted(rings_dir.iterdir()):
        name = child.name
        if not child.is_dir():
            continue
        if not name.startswith("ring-") or not name.endswith("-rust"):
            continue
        if not (child / "Cargo.toml").is_file():
            continue
        entries.append({
            "crate": name,
            "path": f"rings/{name}",
        })
    return entries


def main() -> int:
    repo_root = Path(__file__).resolve().parents[2]
    include = discover(repo_root)
    matrix = {"include": include}
    out = json.dumps(matrix, separators=(",", ":"))
    # GitHub Actions consumes `matrix=...` on a single line.
    print(out)
    # Optional: write to $GITHUB_OUTPUT when invoked from a workflow.
    gh_out = os.environ.get("GITHUB_OUTPUT")
    if gh_out:
        with open(gh_out, "a", encoding="utf-8") as fh:
            fh.write(f"matrix={out}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
