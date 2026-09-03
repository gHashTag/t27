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

    # AN EMPTY MATRIX IS NOT A CLEAN BUILD (#3069).
    #
    # `discover` drops a directory on three conditions -- the name must start
    # `ring-`, end `-rust`, and hold a `Cargo.toml` -- and every one of them is a
    # rename away. With the matrix empty the workflow does not fail: the build
    # job is guarded by `if: needs.discover.outputs.count != '0'`, a SKIPPED job
    # is green, and the whole run concludes success having compiled nothing.
    #
    # The trigger is not hypothetical. `.github/workflows/rings-rust.yml` filters
    # on `rings/ring-*-rust/**`, so the very commit that renames the crates
    # matches the filter, RUNS this workflow, and gets a green tick for it.
    #
    # This file has been here before. Its workflow's own header records seven
    # master runs between 2026-05-23 and 2026-08-20 in which all 17 crate jobs
    # failed and every run concluded `success`, because the summary printed a
    # COUNT and never read what the matrix had measured. That door was closed
    # per job; this is the same door one step earlier, at the matrix itself.
    #
    # Exit 2, not 1: nothing failed to compile, the population was never built.
    # `t27c corpus` refuses a spec tree with no specs the same way, `scripts/tri`
    # uses 2 for an unbuilt compiler, and pytest reserves a code of its own --
    # 5, "No tests were collected" -- for exactly this outcome.
    if not include:
        rings_dir = repo_root / "rings"
        print(
            f"rings_matrix: REFUSED -- no ring-*-rust crate with a Cargo.toml under "
            f"{rings_dir}.\n"
            "  Nothing was compiled and nothing failed to compile: the matrix is empty,\n"
            "  the build job would be SKIPPED, and a skipped job reads as green.\n"
            "  If the crates were renamed or removed on purpose, update this script and\n"
            "  the `paths:` filter in .github/workflows/rings-rust.yml in the same commit.\n"
            "  Exit code 2 = could not take a reading, not a failed build.",
            file=sys.stderr,
        )
        return 2

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
