#!/usr/bin/env python3
"""#3079: the published catalog table is 26 records stale in every row.

`docs/theory/IGLA-FORMAL-RESULTS.md` proposition P17 publishes a seven-row table
of what `t27c catalog-gate` checks. Today the catalog holds 109 records, not 83,
one listed check no longer exists, three unlisted checks run, and the findings
column says six zeros and a 5 where the gate reports three findings and exits
non-zero.

This test does not re-run the gate -- that needs a build. It pins the ONE cell
that can be re-taken with a grep and no compiler: `mandatory-field` is bumped
once per parsed record with no predicate, so its population is exactly
`grep -c 'CATALOG:'` on the catalog file.

The point is not the number. It is that the document must not carry a figure the
repository can cheaply contradict: the command's own help string already said
109 while the table said 83, in the same build.
"""

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
CATALOG = REPO / "specs/numeric/formats_catalog.t27"
DOC = REPO / "docs/theory/IGLA-FORMAL-RESULTS.md"
MAIN = REPO / "bootstrap/src/main.rs"
FAILURES = []


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def main():
    records = len([l for l in CATALOG.read_text().splitlines() if "CATALOG:" in l])
    check("the catalog has records at all", records > 0,
          "a zero here means the matcher moved, not that the catalog emptied")
    print(f"      records today: {records}")

    doc = DOC.read_text()

    # The help string and the document must not disagree about the same number.
    m = re.search(r"whose (\d+) records live in", MAIN.read_text())
    check("the command's help names a record count", m is not None,
          "the help string no longer states one -- this test would assert nothing")
    if m:
        helped = int(m.group(1))
        check(f"the help string ({helped}) matches the catalog ({records})",
              helped == records, "one build, two numbers")

    # P17 must carry a re-take anchored to a commit, not a bare stale table.
    check("P17 carries a re-take with an anchor",
          bool(re.search(r"RE-TAKEN AT `[0-9a-f]{7,}`", doc)),
          "the published table has no anchored correction beside it")
    check("and the re-take states today's record count",
          f"| **{records}** |" in doc or f"**{records}**" in doc,
          f"{records} appears nowhere in the correction")

    # The check that no longer exists must not be presented as current.
    src_has = subprocess.run(
        ["grep", "-rl", "no-spurious-layout", str(REPO / "bootstrap/src")],
        capture_output=True, text=True).stdout.strip()
    check("no-spurious-layout is gone from the source", src_has == "",
          f"it is still in {src_has}; then the table is not stale and this test is wrong")
    check("and the document says so", "the check no longer exists" in doc,
          "a row naming a check the command cannot emit is presented as current")

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("ok: the published table carries an anchored re-take that matches the tree.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
