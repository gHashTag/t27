#!/usr/bin/env python3
"""Fail if any property is gated as an expected refutation.

Wave 608. This replaces an inline step that read:

    if grep -q "ifdef T27_FORMAL_OPEN" build/rtl/bitnet_engine_top.sv; then
      echo "::error::..."; exit 1
    fi
    echo "ok       no expected-refutation guards remain"

which had two absences that scored green. `grep` exits nonzero when the file
does not exist, and a nonzero exit in an `if` condition is not caught by
`set -e`, so a **missing subject** printed "ok" and returned 0 -- verified by
moving the file aside and running the step verbatim. And it read one file out of
the twenty-three that can carry the guard: the ten property sources in `formal/`
and thirteen emitted modules in `build/rtl/` were never looked at.

The guard itself is real and load-bearing: `T27_FORMAL_OPEN` marks a property
known to refute, which is legitimate while a defect is being worked but must
never survive into a green build claiming nothing is knowingly broken.

Usage:  python3 formal/guard_scan.py
"""

import pathlib
import re
import sys

GUARD = "T27_FORMAL_OPEN"
# `ifdef, `ifndef, `elsif and plain textual mentions all count -- the point is
# that the token is absent, not that one spelling of it is.
PATTERN = re.compile(rf"`(?:ifdef|ifndef|elsif)\s+{GUARD}\b")


def main():
    root = pathlib.Path(__file__).resolve().parent.parent
    files = sorted(root.glob("formal/*.sv")) + sorted(root.glob("build/rtl/*.sv"))
    if not files:
        # The failure this whole module exists to prevent: no subject, green.
        print(f"::error::guard_scan matched no .sv files under {root} -- "
              "emit the RTL bundle before running this gate")
        return 1

    bad = []
    for f in files:
        # Prop. 118: this matched inside `//` comments, so a retrospective
        # note saying an open-guard "has been removed" -- the exact genre of
        # comment this repository writes -- reported the guard as present.
        for n, line in enumerate(open(f).read().split("\n"), 1):
            if PATTERN.search(re.sub(r"//.*$", "", line)):
                bad.append(f"{f.relative_to(root)}:{n}: {line.strip()[:70]}")

    for b in bad:
        print(f"::error::{b} -- a property is gated as an expected refutation; "
              "fix the defect or document it in FORMAL_FOUNDATIONS.md")
    print(f"guard scan: {len(files)} files, {len(bad)} expected-refutation guards")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
