#!/usr/bin/env python3
"""`tri damage` must report zero on lines that are not damage.

The detector's first regex called an unbalanced `[` damage, and reported 429
damaged lines over the corpus. **230 of those were latency bounds** -- `target:
< 5000ns` is valid t27 and not damage -- so the number measured the regex, not
the corpus. Six fixtures were written to hold that lesson: things that look
damaged to a careless pattern and are not.

They were written in #2161 and never merged, while the tool they guard landed
through other PRs. The control has therefore never run.

**This gate asserts two things, and the second is the point.** Zero findings is
not enough: `tri damage` over a path that does not exist also reports zero, and
until the file count was added to its output the two were the same line of
text. So the fixtures must be FOUND -- an exact count -- and then found clean.
A negative control that cannot tell "nothing is wrong" from "nothing was
looked at" is not a control.
"""
import argparse
import os
import pathlib
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _prereq import broken, skip  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "bootstrap" / "tests" / "fixtures" / "damage_negative"

# Pinned, not counted from the directory. Deriving the expectation from the
# thing under test makes the comparison tautological: delete a fixture and both
# sides fall to five, and the gate reports OK over a control set that lost a
# case. Six were written; losing one is a finding, and adding one is a
# deliberate edit here.
EXPECTED_FIXTURES = 6
TRI = ROOT / "scripts" / "tri"


def parse_report(text):
    """(files_scanned, damaged_lines) from `tri damage` output, or None."""
    scanned = damaged = None
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("files scanned:"):
            scanned = int(line.split(":", 1)[1].strip())
        elif line.startswith("damaged lines:"):
            damaged = int(line.split(":", 1)[1].strip().split()[0])
    if scanned is None or damaged is None:
        return None
    return scanned, damaged


def run_damage(path):
    r = subprocess.run(
        [str(TRI), "damage", str(path)], capture_output=True, text=True, cwd=str(ROOT)
    )
    return r.stdout + r.stderr


def run(require):
    if not TRI.is_file():
        skip("scripts/tri is not present")
    if not FIXTURES.is_dir():
        broken(f"{FIXTURES.relative_to(ROOT)} is missing -- these fixtures are tracked in git")
    on_disk = sorted(p.name for p in FIXTURES.glob("*.t27"))
    if not on_disk:
        broken(f"{FIXTURES.relative_to(ROOT)} holds no .t27 fixtures")

    out = run_damage(FIXTURES)
    parsed = parse_report(out)
    if parsed is None:
        # The output shape changed. Reporting CLEAN here would be the exact
        # failure this gate exists to prevent, one level up.
        broken(
            "could not read a file count and a damage count from `tri damage`.\n"
            "Its output shape changed; this gate cannot score it and will not "
            "guess.\n--- output ---\n" + out
        )
    scanned, damaged = parsed

    print(f"fixtures expected: {EXPECTED_FIXTURES}")
    print(f"fixtures on disk: {len(on_disk)}")
    print(f"files scanned:    {scanned}")
    print(f"damaged lines:    {damaged}")

    problems = []
    if len(on_disk) != EXPECTED_FIXTURES:
        problems.append(
            f"{len(on_disk)} fixture(s) on disk, {EXPECTED_FIXTURES} expected. "
            "Each one holds a false positive this detector used to produce; "
            "losing one silently retires that case."
        )
    if scanned != EXPECTED_FIXTURES:
        problems.append(
            f"scanned {scanned} of {EXPECTED_FIXTURES} fixtures. A zero damage "
            "count over a set that was not read is not a result."
        )
    if damaged != 0:
        problems.append(
            f"{damaged} damaged line(s) reported on fixtures that are NOT damage. "
            "The detector has regained a false positive; read the fixture "
            "headers, each says what it is."
        )

    if problems:
        print()
        for p in problems:
            print(f"FAIL: {p}")
        return 1

    print(f"\nOK: {scanned} fixture(s) read, 0 false positives.")
    if require:
        print("(--require: a skip would have been a failure)")
    return 0


def self_check():
    """The scoring, against outputs this gate must not mis-read."""
    cases, failures = [], []

    def case(name, ok):
        cases.append(name)
        if not ok:
            failures.append(name)

    clean = "corpus: x\nfiles scanned: 6\ndamaged lines: 0 in 0 files, 0 distinct shapes\n"
    empty = "corpus: x\nfiles scanned: 0\ndamaged lines: 0 in 0 files, 0 distinct shapes\n"
    dirty = "corpus: x\nfiles scanned: 6\ndamaged lines: 3 in 2 files, 1 distinct shapes\n"

    case("a clean report parses", parse_report(clean) == (6, 0))
    case("an empty scan parses, and is NOT the same as clean",
         parse_report(empty) == (0, 0) and parse_report(empty) != parse_report(clean))
    case("a dirty report parses", parse_report(dirty) == (6, 3))
    case("output with no counts is unreadable, not clean", parse_report("corpus: x\n") is None)
    # The whole point: both zeros are zero, and only the denominator separates
    # them. If this ever compares equal the gate has lost its meaning.
    case("zero-damage and zero-scanned differ only in the count",
         parse_report(empty)[1] == parse_report(clean)[1]
         and parse_report(empty)[0] != parse_report(clean)[0])

    print(f"self-check: {len(cases) - len(failures)}/{len(cases)} passed")
    for f in failures:
        print(f"  FAILED: {f}")
    return 1 if failures else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--require", action="store_true", help="a skip is a failure")
    ap.add_argument("--self-check", action="store_true", help="check this checker")
    a = ap.parse_args()
    if a.self_check:
        return self_check()
    return run(a.require)


if __name__ == "__main__":
    sys.exit(main())
