#!/usr/bin/env python3
"""`_prereq.skip()` is the could-not-run case, so it must exit 2.

The repository's vocabulary is written down in several places and read in at
least two: 0 pass, 1 a check RAN and said no, 2 the check COULD NOT RUN.
`.githooks/pre-commit` branches on 2 to tell a contributor that nothing about
their commit was examined.

`tools/_prereq.py` is the shared vocabulary for exactly this distinction -- its
own docstring splits `skip()` (the ENVIRONMENT is incomplete) from `broken()`
(the PRODUCT failed) -- and it answered the first with 1, the same code as the
second. It is imported by 15 files with 26 live call sites, so the wrong word
was being taught to all of them.

Three states, three codes, each with its own assertion. The controls matter as
much as the assertions: without them a module that exits 2 for everything, or 0
for everything, would satisfy half this file.
"""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
PREREQ = REPO / "tools" / "_prereq.py"
FAILURES: list[str] = []


def check(name: str, ok: bool, detail: str = "") -> None:
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def run(body: str, *args: str) -> subprocess.CompletedProcess:
    with tempfile.TemporaryDirectory() as td:
        (Path(td) / "_prereq.py").write_text(
            PREREQ.read_text(encoding="utf-8"), encoding="utf-8"
        )
        script = Path(td) / "probe.py"
        script.write_text(
            "import sys\nsys.path.insert(0, '.')\nimport _prereq\n" + body,
            encoding="utf-8",
        )
        return subprocess.run(
            [sys.executable, "probe.py", *args],
            cwd=td,
            capture_output=True,
            text=True,
        )


def main() -> int:
    if not PREREQ.is_file():
        print(f"tools/_prereq.py is missing; nothing was checked.", file=sys.stderr)
        return 2

    plain = run("_prereq.skip('no compiler here')")
    check(
        "a skip with no --require is tolerated (0)",
        plain.returncode == 0,
        f"got {plain.returncode}",
    )
    check(
        "and it says SKIP rather than FAIL",
        "SKIP" in plain.stdout,
        f"stdout={plain.stdout!r}",
    )

    required = run("_prereq.skip('no compiler here')", "--require")
    check(
        "a skip under --require is COULD NOT RUN (2), not a failed check (1)",
        required.returncode == 2,
        f"got {required.returncode}; stdout={required.stdout!r}",
    )
    check(
        "and it says so, so a reader is not left to infer it",
        "could not run" in required.stdout.lower(),
        f"stdout={required.stdout!r}",
    )

    # THE CONTROL. Without it, a module that exits 2 for everything satisfies
    # the assertion above -- and `broken()` is precisely the case that must not.
    prod = run("_prereq.broken('the product failed')")
    check(
        "control: broken() is a failed check (1), not could-not-run",
        prod.returncode == 1,
        f"got {prod.returncode}; stdout={prod.stdout!r}",
    )
    prod_req = run("_prereq.broken('the product failed')", "--require")
    check(
        "control: broken() ignores --require, because the flag is not its subject",
        prod_req.returncode == 1,
        f"got {prod_req.returncode}",
    )

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("ok: skip is 0 or 2, broken is 1, and the flag moves only the first.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
