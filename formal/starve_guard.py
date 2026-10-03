#!/usr/bin/env python3
"""Gate 31: refuse to trust any result while a destructive run's stash exists.

Prop. 206 established that in-process cleanup establishes nothing under abnormal
termination, and moved `absence_sweep`'s restore to its own startup. That closes
the window for the next run of THAT gate. Prop. 207 is the remaining half: every
OTHER gate run in between still measures a starved tree.

And it cannot tell. A gate that finds no `.sv` files under `build/rtl` reports
"found no property files" or "0 disagreements" -- and **which of those two it
reports is not a property of the tree, it is a property of how that gate was
written**. Measured when this actually happened: of 44 gates, 23 failed and 21
passed, on an identically starved tree. The 21 passes were not evidence of
anything.

WHAT THIS GATE DOES. Fails if `build/_absence_bak` exists. That directory is
created only by a destructive sweep and removed by its restore, so its presence
means a run died holding the subjects. Placed FIRST in the workflow, it converts
a suite-wide cascade of unrelated failures -- the shape that cost most of a wave
to diagnose -- into one line naming the cause.

It does not restore. Restoring is `absence_sweep`'s job and `run_all`'s;
a gate that silently repaired the tree would make the outage invisible again,
which is the failure this whole chain is about.

COVERAGE. Checks one condition: the existence of the stash directory. It does not
verify the tree is otherwise intact -- a subject deleted by anything other than
this sweep is not detected, and neither is a partially restored tree. It is a
guard against one known, previously-observed failure mode, not a health check.

ARTIFACTS. Reads the existence of `build/_absence_bak`. Writes nothing.

Prop. 207.
"""
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
STASH = ROOT / "build" / "_absence_bak"


def main():
    if not STASH.exists():
        print("starve guard: no interrupted-sweep stash; subjects are in place")
        return 0
    held = sorted(p.name for p in STASH.rglob("*") if p.is_file())
    print(f"::error::starve guard: build/_absence_bak exists, holding "
          f"{len(held)} file(s). A destructive sweep died before restoring its "
          f"subjects, so every gate after this one would measure a STARVED tree "
          f"-- and would report a pass or a failure according to how it happens "
          f"to be written, not according to the tree. Run "
          f"`python3 formal/absence_sweep.py` (it restores at startup) or "
          f"`python3 formal/run_all.py`, then re-run. Do NOT delete the stash: "
          f"it may hold the only copy (Prop. 207)")
    for h in held[:10]:
        print(f"  {h}")
    return 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::starve guard: could not check build/_absence_bak "
              f"({type(exc).__name__}: {exc}) -- nothing was checked")
        sys.exit(1)
