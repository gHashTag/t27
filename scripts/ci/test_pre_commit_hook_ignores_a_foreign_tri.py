#!/usr/bin/env python3
"""#6009: `.githooks/pre-commit` must not hand the commit to a foreign `tri`.

THE DEFECT. With no `target/*/tri` in the checkout, the hook ran whatever
`command -v tri` named. `tri` is a common name. Measured 2026-10-04 on one
machine, PATH held two of them, and both refused EVERY commit in an unbuilt
checkout (5 of 38 worktrees had a build):

    ~/.local/bin/tri   an ops script         "unknown command: hooks"   exit 1
    ~/.cargo/bin/tri   an April tri build    no `hooks` subcommand      exit 2

The refusal named no defect in the commit; the gates had not run.

THE FIX THIS HOLDS. A PATH `tri` is used only if `tri hooks pre-commit --help`
answers 0. Without a usable tri the hook still runs the conflict-marker gate --
a Python script tri only calls -- over the index (`--staged`), and says that the
census and fix( gates did not run.

The fixture is a temporary repository holding only the hook and the marker
script, run with a PATH built here: a fake-bin directory first, then links to
`git` and `python3`, then /usr/bin:/bin. The machine's own `tri` copies are not
on it, so the answer does not depend on which machine runs the test.

Negative control: `--hook PATH` runs the same cases against another copy of the
hook. Against master's hook before this change the foreign-tri and no-tri cases
FAIL by name; that is what shows these checks can fail.
"""

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAILURES = []

# Built from pieces: a literal marker in this file would make the conflict-marker
# gate refuse the test's own source.
MARKED = "<" * 7 + " HEAD\nours\n" + "=" * 7 + "\ntheirs\n" + ">" * 7 + " other\n"

FOREIGN_EXIT_1 = """#!/bin/sh
echo "$*" >> "$TRI_CALLS"
echo "unknown command: $1 (tri help)" >&2
exit 1
"""

FOREIGN_EXIT_2 = """#!/bin/sh
echo "$*" >> "$TRI_CALLS"
echo "error: unrecognized subcommand '$1'" >&2
exit 2
"""

# Answers the probe, and leaves a trace when the gate itself runs.
GENUINE = """#!/bin/sh
echo "$*" >> "$TRI_CALLS"
if [ "$1" = hooks ] && [ "$2" = pre-commit ]; then
  [ "${3:-}" = --help ] && exit 0
  echo "GENUINE TRI RAN THE GATES"
  exit 0
fi
exit 1
"""

# This repository's tri, finding something: the hook must still refuse.
GENUINE_REFUSING = GENUINE.replace(
    'echo "GENUINE TRI RAN THE GATES"\n  exit 0', 'echo "GENUINE TRI SAID NO"\n  exit 1'
)


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def tools_bin(base: Path) -> Path:
    """Links to git and python3 only, so no other `tri` rides in on their directory."""
    d = base / "toolsbin"
    d.mkdir()
    for tool in ("git", "python3"):
        found = shutil.which(tool)
        if not found:
            raise SystemExit(f"COULD NOT RUN: {tool} is not on PATH")
        (d / tool).symlink_to(found)
    return d


def fixture(base: Path, hook: Path, name: str, tri_script, staged_text, with_markers=True):
    root = base / name
    (root / ".githooks").mkdir(parents=True)
    (root / "tools").mkdir()
    shutil.copy(hook, root / ".githooks" / "pre-commit")
    if with_markers:
        shutil.copy(REPO / "tools" / "check_conflict_markers.py", root / "tools")
        baseline = REPO / "tools" / "conflict_markers_baseline.txt"
        if baseline.exists():
            shutil.copy(baseline, root / "tools")
    fakebin = base / f"{name}-fakebin"
    fakebin.mkdir()
    if tri_script is not None:
        tri = fakebin / "tri"
        tri.write_text(tri_script)
        tri.chmod(0o755)
    env = {
        "PATH": f"{fakebin}:{base / 'toolsbin'}:/usr/bin:/bin",
        "HOME": str(base / "home"),
        "GIT_CONFIG_NOSYSTEM": "1",
        "TRI_CALLS": str(base / f"{name}-calls.txt"),
        "LC_ALL": "C",
    }
    which = shutil.which("tri", path=env["PATH"])
    expected = str(fakebin / "tri") if tri_script is not None else None
    if which != expected:
        raise SystemExit(
            f"COULD NOT RUN: the fixture PATH resolves tri to {which!r}, expected {expected!r}"
        )

    def git(*args):
        subprocess.run(["git", *args], cwd=root, env=env, check=True, capture_output=True)

    git("init", "-q")
    (root / "probe.txt").write_text(staged_text)
    git("add", "probe.txt")
    out = subprocess.run(
        ["bash", str(root / ".githooks" / "pre-commit")],
        cwd=root,
        env=env,
        capture_output=True,
        text=True,
    )
    calls_file = Path(env["TRI_CALLS"])
    calls = calls_file.read_text().splitlines() if calls_file.exists() else []
    return out, out.stdout + out.stderr, calls


def main(argv):
    hook = REPO / ".githooks" / "pre-commit"
    if "--hook" in argv:
        hook = Path(argv[argv.index("--hook") + 1]).resolve()
    print(f"hook under test: {hook}")

    with tempfile.TemporaryDirectory() as td:
        base = Path(td)
        (base / "home").mkdir()
        tools_bin(base)

        for label, script, code in (
            ("an ops script named tri (exit 1)", FOREIGN_EXIT_1, 1),
            ("an old tri without `hooks` (exit 2)", FOREIGN_EXIT_2, 2),
        ):
            tag = f"foreign{code}"
            out, text, calls = fixture(base, hook, tag, script, "clean\n")
            check(f"{label}: the commit is not refused", out.returncode == 0,
                  f"exit {out.returncode}: {text.strip()[-300:]}")
            check(f"{label}: the hook says it was not used",
                  "not this repository's tri" in text, text.strip()[-300:])
            check(f"{label}: no gate was run through it, only the help probe",
                  all(c.endswith("--help") for c in calls), f"calls: {calls}")
            check(f"{label}: conflict markers were still read",
                  "staged paths read" in text, text.strip()[-300:])

            out, text, _ = fixture(base, hook, f"{tag}-marked", script, MARKED)
            check(f"{label}, staged conflict marker: refused with exit 1",
                  out.returncode == 1 and "staged paths read" in text,
                  f"exit {out.returncode}: {text.strip()[-300:]}")

        out, text, _ = fixture(base, hook, "none", None, "clean\n")
        check("no tri at all, clean index: the commit passes", out.returncode == 0,
              f"exit {out.returncode}: {text.strip()[-300:]}")
        check("no tri at all: the note names the gates that did not run",
              "census and fix( gates did not run" in text, text.strip()[-300:])
        check("no tri at all: conflict markers were still read",
              "staged paths read" in text, text.strip()[-300:])

        out, text, _ = fixture(base, hook, "none-marked", None, MARKED)
        check("no tri at all, staged conflict marker: refused with exit 1",
              out.returncode == 1, f"exit {out.returncode}: {text.strip()[-300:]}")

        out, text, _ = fixture(base, hook, "none-noscript", None, MARKED, with_markers=False)
        check("no tri and no marker script: passes and says markers were not checked",
              out.returncode == 0 and "conflict markers were not checked" in text,
              f"exit {out.returncode}: {text.strip()[-300:]}")

        # Controls: this repository's tri on PATH must still be used, and still block.
        out, text, calls = fixture(base, hook, "genuine", GENUINE, "clean\n")
        check("this repository's tri on PATH is used",
              out.returncode == 0 and "GENUINE TRI RAN THE GATES" in text,
              f"exit {out.returncode}, calls {calls}: {text.strip()[-300:]}")
        out, text, _ = fixture(base, hook, "genuine-no", GENUINE_REFUSING, "clean\n")
        check("this repository's tri saying no still refuses the commit",
              out.returncode == 1 and "GENUINE TRI SAID NO" in text,
              f"exit {out.returncode}: {text.strip()[-300:]}")

    print()
    if FAILURES:
        print(f"FAILED: {len(FAILURES)} check(s)")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("PASSED: a foreign tri is not trusted with the commit (#6009)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
