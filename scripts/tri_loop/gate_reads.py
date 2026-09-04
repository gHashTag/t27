#!/usr/bin/env python3
r"""tri gate-reads -- how many files each gate actually opens, and how many it delegates.

`tri claims` lists CI step names carrying a completeness word and says plainly
what it cannot do: "That a flagged name is wrong ... only reading the tool can."
This is the other half. It runs each `tools/check_*.py` under an audit hook and
records every file the process opens and every subprocess it spawns, so the
question "how much of its subject does this gate actually read" has an answer
that is not a reading of the source.

WHAT IT FOUND, WHICH WAS NOT WHAT I EXPECTED
--------------------------------------------
Nothing. Across all 20 gates, **not one reads a slice of its own declared
subject**. Every small number has an innocent explanation, and each was checked:

    check_duplicate_agreement   64 of 64 `specs/ternary/*.t27` -- its whole declared set
    check_sync_repo_root         1 target file, by design
    check_graph_law8             1 graph JSON, by design
    check_vector_data           34 vector JSONs, a named subset
    check_specs_generate         3 files opened and 1040 SUBPROCESSES -- it delegates
    check_verilog_widths         3 files opened and  650 subprocesses -- likewise

So the shape found three times in two passes -- a gate whose population is
smaller than its subject -- **does not live in the tools**. It lives one layer
up, in the wiring: `lake build` red with its workflow run once in sixty (#3142),
the `Admitted` gate scoped to two of 58 Coq files (#3153), a `paths:` filter that
has matched nothing in months. The tools are honest about what they read; what
shrinks the population is which of them runs, and on what.

TWO WAYS THIS LIES, BOTH LEARNED THE HARD WAY
---------------------------------------------
**A skipped gate reads nothing.** On the first run eleven gates opened two files
and exited 0, which reads as a fleet of empty checks. They had skipped: `t27c`
was not at the path `tools/_prereq.py` looks for, and without `--require` a skip
exits 0. CI passes `--require`, so this never happens there. Build first, or every
number here is a measurement of your own environment.

**A gate that shells out reads nothing visibly.** The audit hook sees `open`
inside this process; `grep`, `git grep` and `t27c` do their reading in a child.
That is why the spawn column exists and why a low `opened` beside a high `spawn`
is not a finding.

WHAT THIS DOES NOT ESTABLISH
----------------------------
Coverage. "Files opened of extension X" against "files of extension X in the
tree" is the wrong ratio, and computing it produced the only wrong conclusions in
this work: `check_assertionless_spec_tests` reads 650 of 745 `.t27` and is
complete, because its subject is `specs/` and the other 95 are elsewhere. **The
denominator has to come from the gate**, and no gate here declares one
machine-readably. This reports what was read; deciding whether that is the whole
subject is still reading the tool.

    tri gate-reads
    tri gate-reads --json
"""
from __future__ import annotations

import glob
import json
import os
import subprocess
import sys
import tempfile

TRACER = r'''
import sys, os, atexit
opened, spawned = set(), []
root = os.getcwd()
def hook(event, args):
    if event == "subprocess.Popen":
        try: spawned.append(" ".join(str(x) for x in (args[1] or [])[:2]))
        except Exception: pass
    elif event == "open":
        p = args[0]
        if isinstance(p, str):
            try: ap = os.path.realpath(p)
            except Exception: return
            if ap.startswith(root): opened.add(os.path.relpath(ap, root))
sys.addaudithook(hook)
target = sys.argv[1]
sys.argv = [target] + sys.argv[2:]
out = os.environ["TRACE_OUT"]
atexit.register(lambda: open(out, "w").write("\n".join(sorted(opened))))
atexit.register(lambda: open(out + ".spawn", "w").write("\n".join(spawned)))
rc = 0
try:
    exec(compile(open(target).read(), target, "exec"),
         {"__name__": "__main__", "__file__": target})
except SystemExit as e:
    rc = e.code if isinstance(e.code, int) else (0 if e.code is None else 1)
sys.exit(rc)
'''


def repo_root() -> str:
    out = subprocess.run(["git", "rev-parse", "--show-toplevel"],
                         capture_output=True, text=True).stdout.strip()
    if not out:
        print("tri gate-reads: not inside a git repository.", file=sys.stderr)
        raise SystemExit(2)
    return out


def main() -> int:
    root = repo_root()
    os.chdir(root)
    gates = sorted(glob.glob("tools/check_*.py"))
    if not gates:
        print("tri gate-reads: no tools/check_*.py found, so this measured nothing.",
              file=sys.stderr)
        return 2

    with tempfile.TemporaryDirectory() as td:
        tracer = os.path.join(td, "tracer.py")
        open(tracer, "w").write(TRACER)
        rows = []
        for g in gates:
            out = os.path.join(td, os.path.basename(g) + ".trace")
            env = dict(os.environ, TRACE_OUT=out)
            try:
                rc = subprocess.run([sys.executable, tracer, g], capture_output=True,
                                    text=True, timeout=180, env=env).returncode
            except subprocess.TimeoutExpired:
                rc = "TIMEOUT"
            files = open(out).read().split("\n") if os.path.exists(out) else []
            spawn = open(out + ".spawn").read().split("\n") if os.path.exists(out + ".spawn") else []
            rows.append({"gate": os.path.basename(g), "exit": rc,
                         "opened": len([f for f in files if f]),
                         "spawned": len([s for s in spawn if s])})

    skipped = [r for r in rows if r["opened"] <= 2 and r["spawned"] == 0 and r["exit"] == 0]
    if "--json" in sys.argv:
        print(json.dumps({"rows": rows, "quiet": [r["gate"] for r in skipped]}, indent=1))
        return 0

    print("tri gate-reads -- files each gate opens, and subprocesses it delegates to")
    print()
    print(f"  {'gate':<44}{'exit':>6}{'opened':>8}{'spawned':>9}")
    for r in sorted(rows, key=lambda x: (x["opened"] + x["spawned"])):
        print(f"  {r['gate']:<44}{str(r['exit']):>6}{r['opened']:>8}{r['spawned']:>9}")
    print()
    if skipped:
        print(f"  {len(skipped)} gate(s) opened two files or fewer and spawned nothing.")
        print("  Before reading that as an empty check: a SKIPPED gate reads nothing, and")
        print("  without --require a skip exits 0. Build t27c first -- eleven gates looked")
        print("  empty on the first run of this tool for exactly that reason.")
        print()
    print("  This does NOT establish coverage. Files-opened against files-in-the-tree is")
    print("  the wrong ratio: a gate reading 650 of 745 `.t27` is complete when its")
    print("  subject is `specs/`. The denominator has to come from the gate, and none")
    print("  here declares one machine-readably.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
