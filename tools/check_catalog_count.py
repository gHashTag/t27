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
    python3 tools/check_catalog_count.py --self-check    negative control

Exit codes:
    0  hard invariant holds (paper mismatch is WARN only by default)
    2  hard invariant violated (SSOT != fresh regen)
    3  paper count diverges AND --strict-paper passed
    4  SSOT below the MIN_ROWS floor (rows left the catalog)
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


# The catalog has only ever grown: 83 at the paper, then 92, then 109. This is
# a ratchet floor, not a target -- see the check in main() for why equality
# alone is not enough.
MIN_ROWS = 109


# ------------------------------------------------------------- negative control
# Separately, and NOT fixed here: this gate's paper-divergence WARN fires on
# every single run (MIN_ROWS 109 vs PAPER_DECLARED_COUNT 83) and will until an
# erratum lands, so it carries no information. See #2466.
#
# T86: `tri gates sweep` found this gate had no negative control at all -- it
# had never once been shown to go RED, which is the same evidence as a gate
# that CANNOT. Each case below plants a fault in a throwaway tree and runs THE
# REAL GATE against it: this very file, in a subprocess, through main(). Nothing
# here re-implements the comparison. A control that evaluates its own copy of
# the rule certifies the copy; mutate the real logic and the copy still agrees
# with itself.
#
# ROOT, and why it cannot fall back: main() derives `repo` from __file__ ONLY to
# build the argparse DEFAULTS for --ssot/--tool. Every case passes both
# explicitly as absolute paths into the planted tree, so the default is never
# consulted -- not for the SSOT, and not for the codegen. The child also runs
# with cwd set to the planted dir, so nothing can reach the repository by
# relative path either. Belt and braces: every expected message below names a
# planted count (0, 110, 111) and the repository's own count is 109, so a leak
# back to the real SSOT fails the assertion instead of passing quietly.
#
# Each case asserts the MESSAGE, not just the exit code, and asserts that a
# NEIGHBOURING branch's marker is ABSENT. The two exit-2 branches are the reason:
# a dropped row and a malformed row both exit 2, through different code, and a
# control that reads only the status cannot tell them apart.

_PLANTED_ROW = (
    '  // CATALOG: id=synth{i:03d} name="synthetic row {i}" bits=8 s=1 e=4 '
    'm=3 bias=7 phi_distance=0.5 storage=u8 cluster=Synthetic '
    'status=Experimental standard="negative control" '
    'use_case="negative control" gf_relation=competitor source="negative control"'
)

# Counted by THIS file's CATALOG_LINE (`//\s*CATALOG:`), invisible to the
# codegen's (`//\s*CATALOG:\s*(.+)$`): with nothing after the colon and the file
# ending here, that trailing `(.+)` has no line left to match, so codegen skips
# the row in silence -- no malformed warning. Must stay the LAST line: given a
# following line, `\s*` eats the newline and `(.+)` swallows THAT line instead,
# which lands in the malformed branch and would test the wrong thing.
_DROPPED_ROW = "// CATALOG:\n"

# Counted by both regexes, but missing every field after `name`, so the codegen
# parser raises and prints `WARN: malformed CATALOG line` -- the other exit-2.
_MALFORMED_ROW = '// CATALOG: id=broken name="row missing its fields"\n'


def _planted_ssot(rows: int, trailer: str = "") -> str:
    """A well-formed SSOT carrying `rows` parseable CATALOG lines, plus trailer."""
    body = "\n".join(_PLANTED_ROW.format(i=i) for i in range(rows))
    return ("// planted SSOT (negative control)\n"
            f"module Planted {{\n{body}\n}}\n{trailer}")


def _run_planted(tool: Path, ssot_text: str, extra: list[str]) -> tuple[int, str]:
    """Run THE REAL GATE against a planted SSOT. Returns (exit code, all output)."""
    gate = Path(__file__).resolve()
    with tempfile.TemporaryDirectory() as td:
        ssot = Path(td) / "planted_catalog.t27"
        ssot.write_text(ssot_text, encoding="utf-8")
        r = subprocess.run(
            [sys.executable, str(gate),
             "--ssot", str(ssot), "--tool", str(tool)] + extra,
            capture_output=True, text=True, cwd=td,
        )
    return r.returncode, r.stdout + r.stderr


def self_check(tool: Path) -> int:
    """Plant each fault this gate claims to catch; assert it catches THAT one."""
    cases: list[tuple[str, str, list[str], int, str, tuple[str, ...]]] = [
        ("clean planted tree passes",
         _planted_ssot(110), [], 0,
         "OK: SSOT == fresh regen == 110",
         ("FAIL:", "codegen dropped a CATALOG line")),
        ("floor: every row stripped (T68, where 0 == 0 used to pass)",
         _planted_ssot(0), [], 4,
         f"floor is {MIN_ROWS}",
         ("OK: SSOT ==", "!= regen")),
        ("SSOT counts a row the codegen silently drops",
         _planted_ssot(110, _DROPPED_ROW), [], 2,
         "SSOT (111) != regen (110)",
         ("OK: SSOT ==", "codegen dropped a CATALOG line")),
        ("codegen reports a malformed row (the OTHER exit 2)",
         _planted_ssot(110, _MALFORMED_ROW), [], 2,
         "codegen dropped a CATALOG line (parser bug regressed)",
         ("OK: SSOT ==", "!= regen")),
        ("--strict-paper turns the paper WARN into a failure",
         _planted_ssot(110), ["--strict-paper"], 3,
         f"An erratum to {PAPER_ID} is required",
         ("OK: SSOT ==",)),
    ]

    red = 0
    for name, ssot_text, extra, want_rc, want, forbid in cases:
        rc, out = _run_planted(tool, ssot_text, extra)
        said = want in out
        quiet = [f for f in forbid if f in out]
        ok = rc == want_rc and said and not quiet
        red += not ok
        print(f"  [{'ok ' if ok else 'RED'}] {name}")
        print(f"        exit {rc} (want {want_rc}); says {want!r}: {said}")
        if quiet:
            print(f"        WRONG BRANCH -- also said: {quiet}")
        if not ok:
            print("        ---- gate output ----")
            for ln in out.strip().splitlines():
                print(f"        | {ln}")

    if red:
        print(f"self-check FAILED: {red}/{len(cases)} planted fault(s) not caught "
              f"as specified. The gate does not do what its exit codes claim.",
              file=sys.stderr)
        return 1
    print(f"self-check: {len(cases)}/{len(cases)} planted faults caught through "
          f"the intended branch (exit 0/2/2/3/4, each by its own message).")
    return 0


def main(argv: list[str]) -> int:
    repo = Path(__file__).resolve().parent.parent
    ap = argparse.ArgumentParser()
    ap.add_argument("--ssot",
                    default=repo / "specs/numeric/formats_catalog.t27")
    ap.add_argument("--tool",
                    default=repo / "tools/gen_formats_catalog.py")
    ap.add_argument("--strict-paper", action="store_true",
                    help="treat paper-count divergence as a hard failure")
    ap.add_argument("--self-check", action="store_true",
                    help="negative control: plant each fault, assert the gate reds")
    args = ap.parse_args(argv[1:])

    ssot = Path(args.ssot)
    tool = Path(args.tool)

    if args.self_check:
        # --tool comes through the SAME argparse default the gate itself uses,
        # so the control certifies the real codegen, not a stand-in.
        return self_check(tool)

    n_ssot = ssot_count(ssot)
    n_regen = regen_count(ssot, tool)

    print(f"SSOT   (// CATALOG: lines)      = {n_ssot}")
    print(f"regen  (codegen fresh)          = {n_regen}")
    print(f"paper  ({PAPER_ID} declared)    = {PAPER_DECLARED_COUNT}")

    # T68: a FLOOR, because equality alone is satisfied at zero. Stripping
    # every `// CATALOG:` line from the SSOT made both counters read 0 and this
    # gate printed "OK: SSOT == fresh regen == 0 (canonical)." and exited 0 --
    # both counters read the same file, so at zero the "independent path" they
    # rely on collapses onto nothing. The ladder has only ever grown, 83 -> 92
    # -> 109, so a drop is a deliberate act: lower this number in the same
    # commit that removes rows, and say in the message why.
    if n_ssot < MIN_ROWS:
        print(f"FAIL: SSOT has {n_ssot} rows, floor is {MIN_ROWS}. Rows left the "
              f"catalog. Equality with the regen does not prove a catalog EXISTS "
              f"-- 0 == 0 passes it. If the removal is deliberate, lower MIN_ROWS "
              f"in the same commit.", file=sys.stderr)
        return 4

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
