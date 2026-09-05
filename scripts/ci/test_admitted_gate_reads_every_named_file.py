#!/usr/bin/env python3
"""#3063: grep has three answers and this gate used to keep one.

    if grep -n 'Admitted' coq/Kernel/Phi.v coq/Kernel/PhiFloat.v 2>/dev/null; then

0 matched, 1 no match, 2 cannot open. `if` merges 1 with 2, so a missing operand
fell through to `echo "OK: no Admitted"`. Reproduced before the fix: with Phi.v
absent and one `Admitted.` in PhiFloat.v, grep PRINTS the match, exits 2, and the
step printed OK underneath it and exited 0.

The step body is EXTRACTED from the workflow rather than restated here, so
editing the YAML and not this file is caught. If the extractor stops matching,
that is asserted as a failure -- a test that silently exercises nothing is worse
than no test.
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
WF = REPO / ".github/workflows/coq-kernel.yml"
STEP = "Verify Kernel PHI layer has no Admitted"
FAILURES = []


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def step_body():
    y = WF.read_text()
    m = re.search(rf"- name: {re.escape(STEP)}\n        run: \|\n(.*?)(?=\n      - |\n\Z)", y, re.S)
    if not m:
        return None
    return "\n".join(l[10:] if l.startswith(" " * 10) else l for l in m.group(1).split("\n"))


def arm(body, files, named=("Phi.v", "PhiFloat.v")):
    d = tempfile.mkdtemp(prefix=f"admitted-gate-{os.getpid()}-")
    try:
        os.makedirs(os.path.join(d, "coq/Kernel"))
        for f, c in files.items():
            with open(os.path.join(d, "coq/Kernel", f), "w") as fh:
                fh.write(c)
        # The gate reads its population from `_CoqProject` rather than from two
        # literal operands, so the fixture has to have one. `named` is kept
        # separate from `files` on purpose: arming ABSENCE means the project
        # still NAMES a file that is not there, which is the shape a deleted or
        # renamed source produces and the one the gate must call could-not-run.
        with open(os.path.join(d, "coq/_CoqProject"), "w") as fh:
            fh.write("-R . T27\n")
            for n in named:
                fh.write(f"Kernel/{n}\n")
        r = subprocess.run(["bash", "-c", body], capture_output=True, text=True, cwd=d)
        return r.returncode, r.stdout + r.stderr
    finally:
        shutil.rmtree(d, ignore_errors=True)


CLEAN = "Lemma a.\nQed.\n"
DIRTY = "Lemma b.\nAdmitted.\n"


def main():
    body = step_body()
    check("the step body was found in the workflow", body is not None,
          f"no step named {STEP!r} with a literal run block -- this test would assert nothing")
    if body is None:
        print("\nFAILED:\n  - extractor")
        return 1

    rc, out = arm(body, {"Phi.v": CLEAN, "PhiFloat.v": CLEAN})
    check("both files present and clean passes", rc == 0 and "OK: no Admitted" in out, f"rc={rc} out={out!r}")

    rc, out = arm(body, {"Phi.v": CLEAN, "PhiFloat.v": DIRTY})
    check("an Admitted in either file fails", rc == 1, f"rc={rc} out={out!r}")
    check("and the failure names the line", "PhiFloat.v:2" in out, f"out={out!r}")

    # THE DEFECT. Both directions, because the old code passed either way and
    # the dangerous one is where a real Admitted is masked.
    for label, other in (("a masked Admitted", DIRTY), ("a clean sibling", CLEAN)):
        rc, out = arm(body, {"PhiFloat.v": other})
        check(f"an absent operand is could-not-run, not OK ({label})", rc == 2, f"rc={rc} out={out!r}")
        check(f"and never says OK ({label})", "OK: no Admitted" not in out, f"out={out!r}")
        check(f"and names the file it could not read ({label})", "Phi.v" in out, f"out={out!r}")

    # The contract this gate was WIDENED to: it reads every file the build
    # compiles, not the two it used to name. A third file with an `Admitted` is
    # the case the old two-operand gate printed OK for.
    rc, out = arm(
        body,
        {"Phi.v": CLEAN, "PhiFloat.v": CLEAN, "Trit.v": DIRTY},
        named=("Phi.v", "PhiFloat.v", "Trit.v"),
    )
    check("an Admitted in a THIRD file is caught", rc == 1, f"rc={rc} out={out!r}")
    check("and the failure names it", "Trit.v" in out, f"out={out!r}")

    # And the population is refused when it is empty, rather than read as clean.
    rc, out = arm(body, {"Phi.v": CLEAN}, named=())
    check("an empty _CoqProject is could-not-run", rc == 2, f"rc={rc} out={out!r}")
    check("and never says OK (empty project)", "no Admitted in any" not in out, f"out={out!r}")

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("ok: the gate reads every file _CoqProject names, or says it could not.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
