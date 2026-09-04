#!/usr/bin/env python3
r"""tri merge-losses -- merges that took ours and discarded the other side's work.

A merge can resolve a file by taking one parent's version whole. When that is a
deliberate choice it is correct; when it is used to dodge a real merge it
silently throws the other side away, and **the usual diff does not show it**:
`git show --numstat <merge>` diffs against the FIRST parent only, so a file that
lost the second parent's work does not appear in the merge's diff at all.

MEASURED HERE
-------------
Two, across 415 merges reachable from HEAD (2026-09-04):

    fc4f468f3  proofs/lean4/Trinity/TernaryFPGABoot.lean   -649 lines
    b4d8ee6d4  bootstrap/src/main.rs                       -454 lines

Neither healed. `GenXdc`, `CheckPins` and `ClockAssignment` from the second are
in ZERO files anywhere at HEAD; `EnvelopeCorner` from the first survives in five
files and all five are documentation -- the reports describing work that no
longer exists. See #3150.

The second merge discarded a whole SIDE, not one file: with `main.rs` went
`proofs/sacred/gamma_phi3.v` (32 lines -> 12 at HEAD),
`proofs/gravity/dl_bounds.v` (32 -> 15), `proofs/sacred/l5_identity.v` (27 -> 18)
and their build artefacts. None of those three is in `coq/_CoqProject`, so
nothing compiles them and nothing could have noticed.

WHY THE OBVIOUS CHECK IS USELESS
--------------------------------
"Files that shrink across a merge relative to a non-first parent" returns 1197
hits and 2,595,752 "lost" lines on this repository. That is ordinary divergence:
everything the other lineage never had reads as a deletion. A number that large
is a matcher describing its own population rather than the tree.

The discriminator below is structural and needs no size threshold at all:

    for a merge M with parents P1, P2, base = merge-base(P1, P2):
      for each file F that P2 changed since base:
        blob(M:F) == blob(P1:F) != blob(P2:F)   ->  resolved "ours", theirs dropped

Two hits, both real. `--min-lines` only RANKS the output; it decides nothing.

WHAT THIS DOES NOT ESTABLISH
----------------------------
That a listed merge is wrong. Taking one side whole is sometimes exactly right --
a generated file, a vendored copy, a deliberate revert -- and this cannot tell
that from a mistake. It reports a SHAPE that deserves a human sentence.

It also sees only two-parent merges, only files present in both the merge and P1,
and only the second parent. An octopus merge, or work lost from a third parent,
is invisible here.

    tri merge-losses                 # the report
    tri merge-losses --json
    tri merge-losses --min-lines 0   # every hit, unranked
"""
from __future__ import annotations

import json
import subprocess
import sys

CODE = ("proofs/", "specs/", "bootstrap/src/", "cli/", "tools/", ".github/workflows/")

# The hand-verified instance. If this stops being found, the walk changed and the
# count is not comparable to the one in this docstring.
CONTROL_FILE = "proofs/lean4/Trinity/TernaryFPGABoot.lean"


def sh(args: list[str]) -> str:
    return subprocess.run(args, capture_output=True, text=True).stdout


def ok(args: list[str]) -> bool:
    return subprocess.run(args, capture_output=True, text=True).returncode == 0


def blob(rev: str, path: str) -> str | None:
    out = subprocess.run(["git", "rev-parse", f"{rev}:{path}"],
                         capture_output=True, text=True)
    return out.stdout.strip() if out.returncode == 0 else None


def lines(rev: str, path: str) -> int:
    return len(sh(["git", "show", f"{rev}:{path}"]).split("\n"))


def scan() -> list[dict]:
    merges = sh(["git", "rev-list", "--merges", "HEAD"]).split()
    if not merges:
        print("tri merge-losses: no merge commits reachable from HEAD.", file=sys.stderr)
        print("  Exit 2 = could not run, not a clean history.", file=sys.stderr)
        raise SystemExit(2)
    hits = []
    for m in merges:
        parents = sh(["git", "rev-list", "--parents", "-n1", m]).split()[1:]
        if len(parents) != 2:
            continue          # octopus merges are out of scope, see the docstring
        p1, p2 = parents
        base = sh(["git", "merge-base", p1, p2]).strip()
        if not base:
            continue
        theirs = {f for f in sh(["git", "diff", "--name-only", base, p2]).split("\n")
                  if f and f.startswith(CODE)}
        if not theirs:
            continue
        # A file resolved "ours" is byte-identical to P1 in the merge, so it is
        # precisely a file the merge did NOT change relative to P1. Subtracting
        # two name-only diffs replaces three `rev-parse` calls per file; the walk
        # went from over ten minutes to seconds on 415 merges.
        touched = set(sh(["git", "diff", "--name-only", p1, m]).split("\n"))
        for f in sorted(theirs - touched):
            b1, b2 = blob(p1, f), blob(p2, f)
            if not (b1 and b2):
                continue
            if b2 != b1:
                hits.append({
                    "merge": m, "file": f, "parent2": p2,
                    "lost": lines(p2, f) - lines(p1, f),
                    "subject": sh(["git", "log", "-1", "--format=%s", m]).strip(),
                    "side": sh(["git", "log", "-1", "--format=%s", p2]).strip(),
                })
    return hits


def main() -> int:
    as_json = "--json" in sys.argv
    floor = 50
    if "--min-lines" in sys.argv:
        try:
            floor = int(sys.argv[sys.argv.index("--min-lines") + 1])
        except (IndexError, ValueError):
            print("tri merge-losses: --min-lines needs a number.", file=sys.stderr)
            return 2

    hits = scan()
    ranked = sorted((h for h in hits if h["lost"] >= floor),
                    key=lambda h: -h["lost"])
    control = any(h["file"] == CONTROL_FILE for h in hits)

    if as_json:
        print(json.dumps({"all": len(hits), "shown": len(ranked),
                          "control_found": control, "hits": ranked}, indent=1))
        return 0

    print("tri merge-losses -- merges that took ours and discarded the other side")
    print()
    print(f"  two-parent merges walked   {len(sh(['git','rev-list','--merges','HEAD']).split())}")
    print(f"  resolved-ours on a code file  {len(hits)}")
    print(f"  losing at least {floor} line(s)   {len(ranked)}")
    print()
    if not control:
        print(f"  CONTROL LOST: {CONTROL_FILE} is no longer among the hits.")
        print("  The walk changed; these numbers are not comparable to the ones in")
        print("  this tool's own docstring until that is explained.")
        print()
    else:
        print(f"  control found: {CONTROL_FILE}")
        print()

    for h in ranked:
        print(f"  {h['merge'][:9]}  -{h['lost']:>5} lines  {h['file']}")
        print(f"      merge: {h['subject'][:74]}")
        print(f"      side:  {h['parent2'][:9]}  {h['side'][:66]}")
        print(f"      recover: git show {h['parent2'][:9]}:{h['file']}")
    print()
    print("  This does NOT establish that a listed merge is wrong. Taking one side")
    print("  whole is sometimes exactly right -- a generated file, a vendored copy,")
    print("  a deliberate revert -- and this cannot tell that from a mistake. It")
    print("  reports a shape that deserves a human sentence.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
