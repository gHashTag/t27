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
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import yaml

REPO = Path(__file__).resolve().parents[2]
WF = REPO / ".github/workflows/coq-kernel.yml"
STEP = "Verify Kernel PHI layer has no Admitted"
FAILURES = []


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def step_body(text=None):
    """Read the actual build step, independent of YAML formatting.

    The old regex required `name` immediately followed by `run`. Adding the
    required `shell: bash` and its explanation broke extraction. Do not loosen
    a regex across step boundaries: parse YAML and require one unambiguous step.
    """
    doc = yaml.safe_load(WF.read_text() if text is None else text)
    if not isinstance(doc, dict):
        raise ValueError("workflow must be a mapping")
    jobs = doc.get("jobs")
    build = jobs.get("build") if isinstance(jobs, dict) else None
    steps = build.get("steps") if isinstance(build, dict) else None
    if not isinstance(steps, list):
        raise ValueError("build.steps must be a list")
    matches = [s for s in steps if isinstance(s, dict) and s.get("name") == STEP]
    if len(matches) != 1:
        raise ValueError(f"expected exactly one build step named {STEP!r}, got {len(matches)}")
    step = matches[0]
    # The Coq container defaults to sh (dash), which cannot run these arrays.
    # Running the fixture in bash must not hide a missing workflow shell key.
    if step.get("shell") != "bash":
        raise ValueError("the Admitted step must explicitly declare shell: bash")
    body = step.get("run")
    if not isinstance(body, str) or not body.strip():
        raise ValueError("the Admitted step must contain a non-empty run string")
    return body


def extractor_tests():
    fixture = (
        "jobs:\n  build:\n    steps:\n"
        f"      - name: '{STEP}'\n"
        "        # An explanation between name and run must be harmless.\n"
        "        shell: bash\n"
        "        run: |\n"
        "          printf 'fixture\\n'\n"
        "      - name: Unrelated sibling\n"
        "        run: echo not-the-gate\n"
    )
    check("comments, quoted names and shell do not hide the body",
          step_body(fixture) == "printf 'fixture\\n'\n")
    # A different indentation, key order and block style denote the same step.
    reordered = {"jobs": {"build": {"steps": [
        {"run": "echo fixture\n", "shell": "bash", "name": STEP},
    ]}}}
    check("key order and indentation do not select another body",
          step_body(yaml.safe_dump(reordered, indent=4, sort_keys=False)) == "echo fixture\n")
    step = {"name": STEP, "shell": "bash", "run": "echo fixture"}
    for label, steps in (
        ("missing step", []),
        ("duplicate step", [step, step]),
        ("missing shell", [{"name": STEP, "run": "echo fixture"}]),
        ("wrong shell", [{**step, "shell": "sh"}]),
        ("empty body", [{**step, "run": ""}]),
        ("non-string body", [{**step, "run": ["echo fixture"]}]),
    ):
        rejected = False
        try:
            step_body(yaml.safe_dump({"jobs": {"build": {"steps": steps}}}))
        except ValueError:
            rejected = True
        check(f"extractor refuses {label}", rejected)
    for label, text in (
        ("malformed YAML", "jobs: ["),
        ("missing build job", yaml.safe_dump({"jobs": {"other": {"steps": [step]}}})),
    ):
        rejected = False
        try:
            step_body(text)
        except (ValueError, yaml.YAMLError):
            rejected = True
        check(f"extractor refuses {label}", rejected)


# The gate's operand list. Named here rather than taken from `files`, because the
# defect this test exists for is an operand the gate NAMES and cannot READ: the
# one-file cases below write a single file while the list still names two.
OPERANDS = ("Phi.v", "PhiFloat.v")


def arm(body, files, named=OPERANDS):
    """`named` is what `_CoqProject` LISTS; `files` is what exists on disk.

    They are separate so a case can name a file that is not there -- the shape a
    deleted or renamed source produces -- and so a case can widen the list past
    the two the gate used to hardcode.
    """
    d = tempfile.mkdtemp(prefix=f"admitted-gate-{os.getpid()}-")
    try:
        os.makedirs(os.path.join(d, "coq/Kernel"))
        # #3238 widened the gate from two hardcoded paths to the files
        # `coq/_CoqProject` names, and this fixture had no such file -- so every
        # case died at `grep: coq/_CoqProject: No such file or directory`, rc=2,
        # and the two cases that ASSERT rc=2 passed for the wrong reason while
        # the three that assert anything else failed. The tree the gate reads
        # must carry the file the gate reads it from.
        with open(os.path.join(d, "coq/_CoqProject"), "w") as fh:
            fh.write("-Q . Kernel\n" + "".join(f"Kernel/{f}\n" for f in named))
        for f, c in files.items():
            with open(os.path.join(d, "coq/Kernel", f), "w") as fh:
                fh.write(c)
        r = subprocess.run(
            ["bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", body],
            capture_output=True, text=True, cwd=d, timeout=30,
        )
        return r.returncode, r.stdout + r.stderr
    finally:
        shutil.rmtree(d, ignore_errors=True)


CLEAN = "Lemma a.\nQed.\n"
DIRTY = "Lemma b.\nAdmitted.\n"


def main():
    FAILURES.clear()
    extractor_tests()
    try:
        body = step_body()
    except (OSError, ValueError, yaml.YAMLError) as error:
        print(f"\nFAILED:\n  - extractor: {error}")
        return 1
    check("the unique bash step body was found in the workflow", True)

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

    # The contract the gate was WIDENED to in #3238: it reads every file the
    # build compiles, not the two it used to name. A third file carrying an
    # `Admitted` is precisely the case the old two-operand gate printed OK for,
    # and nothing here asserted it until now.
    rc, out = arm(
        body,
        {"Phi.v": CLEAN, "PhiFloat.v": CLEAN, "Trit.v": DIRTY},
        named=("Phi.v", "PhiFloat.v", "Trit.v"),
    )
    check("an Admitted in a THIRD file is caught", rc == 1, f"rc={rc} out={out!r}")
    check("and the failure names it", "Trit.v" in out, f"out={out!r}")

    # And an empty list is refused rather than read as clean: a gate whose
    # denominator can reach zero has to say so.
    rc, out = arm(body, {"Phi.v": CLEAN}, named=())
    check("an empty _CoqProject is could-not-run", rc == 2, f"rc={rc} out={out!r}")

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
