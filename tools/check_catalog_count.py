#!/usr/bin/env python3
"""Catalog count invariant gate (CI-01 enforcement).

Kills the count-drift class of bug. Enforces, in this order:

  1. SSOT count == regenerated gen JSON count   (HARD FAIL on mismatch)
  2. SSOT count == PAPER_DECLARED_COUNT          (WARN -> errata required)

The SSOT count is the single canonical number: the count of `// CATALOG:`
lines in specs/numeric/formats_catalog.t27. Codegen run fresh against the
SSOT must reproduce it exactly (this catches the parser-bug class that
silently dropped formula-bias rows). The paper count is declared here as a
constant; when it diverges, CI emits a loud errata reminder rather than
silently shipping divergent numbers.

NOTE on gen/: per the repo constitution (L2 GENERATION), gen/ artifacts are
DERIVED and are never hand-committed in a PR. This gate therefore does NOT
compare against committed gen/ -- it regenerates into a temp dir and checks
that against the SSOT. The canonical number lives in the SSOT, full stop.

Usage:
    python3 tools/check_catalog_count.py
    python3 tools/check_catalog_count.py --ssot PATH --tool PATH

Exit codes:
    0  hard invariant holds (paper mismatch is WARN only by default)
    2  hard invariant violated (SSOT != fresh regen)
    3  paper count diverges AND --strict-paper passed
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

# The count currently declared by the published paper.
#
# Do NOT silently edit this to match the SSOT -- the whole point is to surface a
# divergence. Change it ONLY when the PAPER itself changes, and record the
# evidence here so the next reader can re-check it in one command.
#
# 84 -> 83 on 2026-08-01. The v2 replacement corrected BOTH the title and the
# abstract; v1's 84 is withdrawn. Verified by fetching the arXiv entry directly:
#
#   curl -sS "https://export.arxiv.org/api/query?id_list=2606.09686"
#
#   TITLE    : An 83-Format Numeric Catalog with Bit-Exact Conformance Vectors:
#              A Vendor-Neutral Reference for FP8, BF16, MXFP4, and Microscaling
#              Formats
#   ID       : http://arxiv.org/abs/2606.09686v2
#   UPDATED  : 2026-06-22T12:28:45Z
#   abstract : "a catalog of 83 numeric formats spanning 13 families"
#
# Before this, the constant tracked the withdrawn v1 and the gate printed
# "an erratum is required" on every run for work ERRATA_2026-06-14.md and the v2
# replacement had already done. A gate that cries wolf every run is one nobody
# reads on the day the divergence is real.
PAPER_DECLARED_COUNT = 83
PAPER_ID = "arXiv:2606.09686"

CATALOG_LINE = re.compile(r"//\s*CATALOG:")


def ssot_count(ssot: Path) -> int:
    text = ssot.read_text(encoding="utf-8")
    return sum(1 for line in text.splitlines() if CATALOG_LINE.search(line))


def regen_count(ssot: Path, tool: Path) -> int:
    """Run codegen into a temp dir and read its JSON count (independent path)."""
    with tempfile.TemporaryDirectory() as td:
        r = subprocess.run(
            [sys.executable, str(tool), str(ssot), td],
            capture_output=True, text=True,
        )
        if r.returncode != 0:
            print("codegen failed:\n" + r.stderr, file=sys.stderr)
            sys.exit(2)
        # surface any malformed-line warnings as hard signal
        if "WARN: malformed" in r.stderr:
            print("codegen dropped a CATALOG line (parser bug regressed):",
                  file=sys.stderr)
            for ln in r.stderr.splitlines():
                if "WARN: malformed" in ln:
                    print("  " + ln, file=sys.stderr)
            sys.exit(2)
        gen = json.loads((Path(td) / "formats_catalog.json").read_text())
        return int(gen["count"])


def main(argv: list[str]) -> int:
    repo = Path(__file__).resolve().parent.parent
    ap = argparse.ArgumentParser()
    ap.add_argument("--ssot",
                    default=repo / "specs/numeric/formats_catalog.t27")
    ap.add_argument("--tool",
                    default=repo / "tools/gen_formats_catalog.py")
    ap.add_argument("--strict-paper", action="store_true",
                    help="treat paper-count divergence as a hard failure")
    args = ap.parse_args(argv[1:])

    ssot = Path(args.ssot)
    tool = Path(args.tool)

    n_ssot = ssot_count(ssot)
    n_regen = regen_count(ssot, tool)

    print(f"SSOT   (// CATALOG: lines)      = {n_ssot}")
    print(f"regen  (codegen fresh)          = {n_regen}")
    print(f"paper  ({PAPER_ID} declared)    = {PAPER_DECLARED_COUNT}")

    if n_ssot != n_regen:
        print(f"FAIL: SSOT ({n_ssot}) != regen ({n_regen}) -- "
              f"codegen drops/adds rows. Parser bug or SSOT malformed.",
              file=sys.stderr)
        return 2

    if n_ssot != PAPER_DECLARED_COUNT:
        msg = (f"WARN: SSOT ({n_ssot}) != paper count "
               f"({PAPER_DECLARED_COUNT}). An erratum to {PAPER_ID} is "
               f"required (see ERRATA_2026-06-14.md). Canonical live count "
               f"is {n_ssot}.")
        print(msg, file=sys.stderr)
        if args.strict_paper:
            return 3

    print(f"OK: SSOT == fresh regen == {n_ssot} (canonical).")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
