#!/usr/bin/env python3
"""`tri damage-repair` has no default snapshot, and refuses instead of guessing (#2327).

The defect this pins down.

`damage_repair.py` used to carry `DEFAULT_SNAPSHOT =
"docs/corpus/damage_snapshot_2026-08-15.json"`. No default workflow produces that
file. The companion writer `tri damage-freeze` defaults its `--out` to
`docs/corpus/damage_snapshot.json` -- a DIFFERENT name -- so running the pair back
to back with no arguments never connected even once. The reader printed "no
snapshot" while a freeze it could have used sat beside it in the same directory.

That is the scenario this test reconstructs, and it is why the fixture runs
`damage-freeze` with no `--out` first: the snapshot at the writer's own default
name is present on disk for every case below. A reader that quietly adopts it is
guessing which freeze the caller meant, and a reader that reports the old dated
path is back to naming a file nothing creates. Both are failures here.

Four independent properties, each checked separately and each reported by name,
because "it exited non-zero" is satisfied by a tool that is simply broken:

  G1 refuses          no `--snapshot` exits non-zero, even with a default-named
                      snapshot sitting in docs/corpus/
  G2 for the right    the refusal names `--snapshot` and says it is required, so
     reason           it is distinguishable from an import error or a missing
                      corpus, which also exit non-zero
  G3 resolves no      the refusal names no concrete snapshot path -- no
     path             `docs/corpus`, no `.json`. Reintroducing a default in any
                      form, including as a "helpful" suggested path, trips this
  G4 still works      given an explicit `--snapshot`, the tool runs to completion
                      and reports that snapshot. Without this, "always exit 2"
                      would satisfy G1-G3

Failures are collected rather than raised, so one broken property does not hide
the state of the other three.

No compiler: `--binary` is pointed at a path that does not exist, so validation
is skipped and the run is pure Python. Nothing outside the temporary directory is
read or written.
"""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

TRI_LOOP = Path(__file__).resolve().parent.parent / "tri_loop"
FREEZE = TRI_LOOP / "damage_freeze.py"
REPAIR = TRI_LOOP / "damage_repair.py"

# One damaged field, of the restorable shape the repair tool exists to handle:
# the opening quote of the type string was replaced by `[`, which trips both of
# damage.py's signals (doubled-bracket, odd-quote).
DAMAGED_SPEC = '''Struct QuadNode {
  name : "TypeText",
  children : [[]QuadNode",
}
'''

# The writer's own default output path. Named here so the fixture can assert the
# trap condition really is set up; the reader under test must not know it.
FREEZE_DEFAULT_OUT = Path("docs") / "corpus" / "damage_snapshot.json"


def run(args, cwd):
    return subprocess.run([sys.executable, *args], cwd=cwd,
                          capture_output=True, text=True)


def build_fixture(tmp: Path) -> None:
    """A corpus with one damaged line, frozen at the WRITER's default path."""
    specs = tmp / "specs"
    specs.mkdir(parents=True)
    (specs / "quadnode.t27").write_text(DAMAGED_SPEC)

    r = run([str(FREEZE), "specs"], cwd=tmp)
    if r.returncode != 0:
        raise SystemExit(f"fixture: damage-freeze failed ({r.returncode})\n"
                         f"{r.stdout}\n{r.stderr}")
    made = tmp / FREEZE_DEFAULT_OUT
    if not made.is_file():
        raise SystemExit(
            "fixture: damage-freeze with no --out did not write "
            f"{FREEZE_DEFAULT_OUT}. This test's premise is that the writer's "
            "default output exists on disk while the reader still refuses; "
            "without it G1 would pass for the wrong reason.")


def main() -> int:
    failures = []

    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        build_fixture(tmp)
        print(f"fixture: 1 damaged spec, frozen at {FREEZE_DEFAULT_OUT} "
              "(the writer's default)\n")

        # ---- no --snapshot -------------------------------------------------
        bare = run([str(REPAIR)], cwd=tmp)
        err = bare.stderr
        print("--- tri damage-repair (no --snapshot) ---")
        print(f"exit={bare.returncode}")
        for line in err.splitlines():
            print(f"  | {line}")
        print()

        # G1
        if bare.returncode == 0:
            failures.append(
                "G1 refuses: exited 0 with no --snapshot. A snapshot named "
                f"{FREEZE_DEFAULT_OUT} was present, so the tool adopted a "
                "freeze the caller never named.")

        # G2 -- the refusal is about the argument, not about something else
        low = err.lower()
        if "--snapshot" not in err or "required" not in low:
            failures.append(
                "G2 right reason: the refusal does not both name `--snapshot` "
                "and say it is required. Exiting non-zero for an unrelated "
                f"reason would look identical. stderr was: {err!r}")

        # G3 -- no default path may be resolved or suggested, in any form
        for needle in ("docs/corpus", ".json"):
            if needle in err:
                failures.append(
                    f"G3 resolves no path: the refusal names {needle!r}. The "
                    "reader must not point at a concrete snapshot file it was "
                    "not given -- a hardcoded path here is exactly the defect "
                    f"#2327 reported. stderr was: {err!r}")

        # ---- explicit --snapshot (anti-vacuity) ----------------------------
        snap = str(FREEZE_DEFAULT_OUT)
        good = run([str(REPAIR), "--snapshot", snap,
                    "--binary", str(tmp / "no-such-binary")], cwd=tmp)
        print("--- tri damage-repair --snapshot <explicit> ---")
        print(f"exit={good.returncode}")
        for line in good.stdout.splitlines()[:4]:
            print(f"  | {line}")
        print()

        # G4
        if good.returncode != 0:
            failures.append(
                "G4 still works: an explicit --snapshot to a real freeze "
                f"exited {good.returncode}. The refusal in G1 is then not a "
                "required-argument check, it is a tool that never runs.\n"
                f"      stdout: {good.stdout[-400:]!r}\n"
                f"      stderr: {good.stderr[-400:]!r}")
        elif snap not in good.stdout:
            failures.append(
                "G4 still works: the run did not report the snapshot it was "
                f"given ({snap!r}), so there is no evidence it read that file "
                "rather than some other one.")

    if failures:
        print(f"FAIL ({len(failures)}):")
        for f in failures:
            print("  - " + f)
        return 1

    print("OK: 4/4 -- damage-repair refuses without --snapshot (G1), says so in "
          "those\nwords (G2), names no snapshot path of its own (G3), and still "
          "runs when given\none (G4). The writer's default output was present "
          "throughout and was not adopted.")
    print("Scope: this covers damage-repair's snapshot argument only. It says "
          "nothing\nabout whether the repairs it proposes are correct -- that is "
          "the double\nvalidation inside the tool.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
