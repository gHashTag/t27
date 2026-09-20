#!/usr/bin/env python3
r"""tri toolbelt -- the read-only commands a worker can run, each one exercised before it is printed.

WHY THIS EXISTS
---------------
`t27c --help` lists 155 subcommands. The brief the swarm is given names four of
them, and the issues it files name the same four: `spec-status`, `gen`, `parse`,
`typecheck`. Everything an agent has needed beyond those it has rebuilt out of
`grep` -- which is how eleven issues came to claim "today: 0 tests" about files
carrying nine, and how a spec that satisfies every grep in its acceptance
criteria still fails to compile.

The commands that answer those questions already exist. `coverage` says which
functions have a test. `lint` prints `WARN: fn 'diff' has no test or invariant`,
which is the review's complaint, before the review. `test-report` builds the spec
and runs its own tests, which is the oracle's question. Nobody was told.

THE LIST LIVES IN THE DOCUMENT, NOT IN THIS FILE
------------------------------------------------
`docs/BEE_TOOLBELT.md` is the single source; `tools/toolbelt.py` is the one
parser of it, shared with the two feeders that embed it. A list kept here as
well would be a second source of truth, and `check_documented_commands_exist.py`
already holds every `t27c <sub>` named under `docs/` to the binary's own --help,
so a command that is renamed out of the compiler turns that gate red rather than
misleading a worker. What that gate cannot see is whether a command that EXISTS
also WORKS on a real spec; that is what this runs.

WHAT THIS ESTABLISHES, AND WHAT IT DOES NOT
-------------------------------------------
Established: the command exists, exits as documented on the sample spec, and
prints something. NOT established: that what it prints is correct. `symbols`
reporting line 0 for every const is a real defect in `symbols`, visible in the
output below, and this tool will still call it OK -- it measures reachability,
not truth.

Usage:
    tri toolbelt                  # print the table, measured on a sample spec
    tri toolbelt --spec PATH      # measure on a spec of your choosing
    tri toolbelt --check          # exit 1 if any documented command fails to run
    tri toolbelt --brief          # the "Start here" block exactly as an issue embeds it

Exit codes: 0 nothing failed; 1 a documented command failed; 2 could not run.
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from toolbelt import DOC_RELPATH, START_HERE, commands, read, section  # noqa: E402

DOC = ROOT / DOC_RELPATH
# Mid-size, parses, holds functions AND tests, so every reader has something to
# report. A spec with no tests makes `coverage` print an empty table, which is
# indistinguishable here from a `coverage` that cannot read the file.
DEFAULT_SPEC = "specs/git/diff.t27"


def t27c() -> str | None:
    for candidate in (
        os.environ.get("TRI_T27C"),
        str(ROOT / "target" / "release" / "t27c"),
        str(ROOT / "target" / "debug" / "t27c"),
    ):
        if candidate and os.access(candidate, os.X_OK):
            return candidate
    return None


def a_function_in(spec: str) -> str:
    """A function name the sample spec really declares.

    `<function>` left unsubstituted still RUNS - `dupe_scan --name '<function>'`
    answers "no function named <function>" and exits 0 - so the measurement
    would pass without ever exercising the lookup it claims to check.
    """
    for line in (ROOT / spec).read_text(errors="replace").split("\n"):
        match = re.match(r"\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)", line)
        if match:
            return match.group(1)
    return "main"


def run(cmd: str, spec: str, binary: str) -> tuple[int, float, int, str]:
    """Run one documented command with its placeholders filled in. Never writes."""
    cmd = cmd.replace("<spec>", spec).replace("<function>", a_function_in(spec))
    argv = cmd.split()
    if argv and argv[0] == "t27c":
        argv[0] = binary
    start = time.time()
    try:
        done = subprocess.run(
            argv, cwd=ROOT, capture_output=True, text=True,
            stdin=subprocess.DEVNULL, timeout=180,
        )
        rc, out = done.returncode, (done.stdout or "") + (done.stderr or "")
    except subprocess.TimeoutExpired:
        rc, out = 124, "timed out after 180 s"
    except OSError as exc:
        rc, out = 127, str(exc)
    secs = time.time() - start
    lines = out.count("\n") + (1 if out and not out.endswith("\n") else 0)
    first = (out.split("\n")[0] if out else "")[:60]
    return rc, secs, lines, first


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--spec", default=DEFAULT_SPEC)
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--brief", action="store_true")
    args = ap.parse_args()

    if not DOC.exists():
        print(f"could not run: {DOC.relative_to(ROOT)} is missing", file=sys.stderr)
        return 2
    text = read(str(ROOT))

    if args.brief:
        block = section(text, START_HERE)
        if not block:
            print(f"could not run: no `{START_HERE}` section in {DOC.name}", file=sys.stderr)
            return 2
        print(block)
        return 0

    binary = t27c()
    if binary is None:
        print("could not run: no t27c built (cargo build --release -p t27c)", file=sys.stderr)
        return 2
    spec = args.spec
    if not (ROOT / spec).exists():
        print(f"could not run: no such spec {spec}", file=sys.stderr)
        return 2

    failed = 0
    for heading in [l.strip() for l in text.split("\n") if l.startswith("## ")]:
        cmds = commands(section(text, heading))
        if not cmds:
            continue
        print(f"\n{heading}")
        for cmd in cmds:
            rc, secs, lines, first = run(cmd, spec, binary)
            # `grep -c` exits 1 on a zero count, and a zero count is an answer.
            ok = rc == 0 or (rc == 1 and first.strip().isdigit())
            if not ok:
                failed += 1
            print(f"  {'ok ' if ok else 'FAIL'} {cmd.replace('<spec>', spec):<52} "
                  f"{secs:5.1f}s {lines:>4} lines  {first}")
    print(f"\nspec: {spec}    t27c: {os.path.relpath(binary, ROOT)}")
    if failed:
        print(f"{failed} documented command(s) did not run. Fix the command or the document.")
        return 1
    print("every documented command ran. Whether its answer is CORRECT is not "
          "established here.")
    return 0 if not args.check else 0


if __name__ == "__main__":
    sys.exit(main())
