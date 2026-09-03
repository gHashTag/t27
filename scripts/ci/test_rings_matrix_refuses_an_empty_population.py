#!/usr/bin/env python3
"""#3069: an empty matrix is not a clean build.

`discover` drops a directory on three conditions -- the name must start `ring-`,
end `-rust`, and hold a `Cargo.toml` -- and every one of them is a rename away.
With the matrix empty the workflow did not fail: the build job is guarded by
`if: needs.discover.outputs.count != '0'`, a SKIPPED job is green, and the run
concluded success having compiled nothing.

The trigger is not hypothetical. rings-rust.yml filters on `rings/ring-*-rust/**`,
so the very commit that renames the crates matches the filter, RUNS the workflow,
and collects a green tick for it.

Every assertion has a control: a tree that DOES carry ring crates must still emit
its matrix and exit 0, or this file cannot fail.

The step body is extracted from the workflow rather than restated, so a change to
the YAML that stops the failure propagating is caught -- GitHub runs steps under
`bash --noprofile --norc -eo pipefail`, and this runs them the same way.
"""

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
WF = REPO / ".github/workflows/rings-rust.yml"
SCRIPT = REPO / "scripts/ci/rings_matrix.py"
FAILURES = []


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def tree(dirs):
    """A scratch repo holding the script and the named ring directories."""
    d = tempfile.mkdtemp(prefix=f"rings-matrix-{os.getpid()}-")
    os.makedirs(os.path.join(d, "scripts/ci"))
    shutil.copy(SCRIPT, os.path.join(d, "scripts/ci/rings_matrix.py"))
    for name, with_cargo in dirs:
        p = os.path.join(d, "rings", name)
        os.makedirs(p)
        if with_cargo:
            with open(os.path.join(p, "Cargo.toml"), "w") as f:
                f.write("[package]\nname = \"x\"\n")
    return d


def gen_step():
    """The `Generate matrix` step body, as the workflow ships it."""
    y = WF.read_text()
    m = re.search(r"- name: Generate matrix\n.*?\n        run: \|\n(.*?)(?=\n  [a-z]|\n      - |\n\Z)", y, re.S)
    if not m:
        return None
    return "\n".join(l[10:] if l.startswith(" " * 10) else l for l in m.group(1).split("\n"))


def run_step(cwd, body):
    """Run the step the way GitHub does, with a real $GITHUB_OUTPUT.

    Emptying that variable instead of pointing it somewhere made every defect
    arm pass for the WRONG reason: the step died on line 2's redirect to "",
    not on line 1's exit 2. The control caught it, which is the whole reason a
    control that asserts SUCCESS sits beside three that assert failure.
    """
    env = dict(os.environ)
    fd, path = tempfile.mkstemp(prefix="gh-output-")
    os.close(fd)
    env["GITHUB_OUTPUT"] = path
    try:
        return subprocess.run(["bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", body],
                              capture_output=True, text=True, cwd=cwd, env=env)
    finally:
        os.unlink(path)


def main():
    body = gen_step()
    check("the Generate matrix step body was found in the workflow", body is not None,
          "the extractor no longer matches -- this test would assert nothing")

    # THE DEFECT, three ways the population empties.
    for label, dirs in (
        ("crates renamed away from -rust", [("ring-088-rs", True)]),
        ("a ring directory with no Cargo.toml", [("ring-088-rust", False)]),
        ("no rings/ directory at all", []),
    ):
        d = tree(dirs)
        try:
            r = subprocess.run([sys.executable, "scripts/ci/rings_matrix.py"],
                               capture_output=True, text=True, cwd=d)
            check(f"an empty population exits 2 ({label})", r.returncode == 2,
                  f"rc={r.returncode} out={(r.stdout + r.stderr)[:200]!r}")
            check(f"and prints no matrix to stdout ({label})", "include" not in r.stdout,
                  f"stdout={r.stdout!r}")
            if body is not None:
                s = run_step(d, body)
                out = s.stdout + s.stderr
                check(f"and the workflow step fails rather than reporting 0 ({label})",
                      s.returncode != 0, f"rc={s.returncode} out={out[:200]!r}")
                # For the RIGHT reason: the script's own refusal, not a broken
                # harness. Without this the arm passes on any failure at all.
                check(f"and it is the script's refusal that stopped it ({label})",
                      "rings_matrix: REFUSED" in out, f"out={out[:300]!r}")
        finally:
            shutil.rmtree(d, ignore_errors=True)

    # CONTROLS. Without these, a script that always exits 2 passes everything above.
    d = tree([("ring-088-rust", True), ("ring-089-rust", True), ("not-a-ring", True)])
    try:
        r = subprocess.run([sys.executable, "scripts/ci/rings_matrix.py"],
                           capture_output=True, text=True, cwd=d)
        check("control: a tree with ring crates still exits 0", r.returncode == 0,
              f"rc={r.returncode} err={r.stderr[:200]!r}")
        ok = False
        try:
            ok = [e["crate"] for e in json.loads(r.stdout)["include"]] == ["ring-088-rust", "ring-089-rust"]
        except Exception as exc:  # noqa: BLE001
            check("control: and emits a parseable matrix", False, f"{exc}: {r.stdout!r}")
        check("control: naming exactly the two ring crates, sorted", ok, f"stdout={r.stdout!r}")
        if body is not None:
            s = run_step(d, body)
            check("control: and the workflow step succeeds, reporting 2",
                  s.returncode == 0 and "Discovered 2 " in s.stdout,
                  f"rc={s.returncode} out={(s.stdout + s.stderr)[:200]!r}")
    finally:
        shutil.rmtree(d, ignore_errors=True)

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("ok: an empty matrix refuses; a real one still builds.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
