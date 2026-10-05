#!/usr/bin/env python3
"""L2 GENERATION: a modified gen/ file passes only if t27c reproduces it.

Closes #6226. The L2 step in .github/workflows/l1-traceability.yml used to fail
every `M gen/...` line. That could not tell a hand edit from a file regenerated
out of a spec changed in the same PR, so a correct PR went red (#6218, #6246)
and reviewers learned to read L2 as noise.

The rule now, per modified file gen/<backend>/<path>.<ext>:

  spec      specs/<path>.t27 (the same mapping `t27c gen-<backend>` is fed)
  command   t27c gen-<backend> specs/<path>.t27   (stdout is the file)
  verdict   bytes equal -> regenerated, pass
            bytes differ, no spec, unknown backend, t27c failed -> FAIL
            no t27c at all -> "not checked", FAIL

The list is `git diff base...head` (three dots: from the merge base), so a
gen/ file master changed after the PR branched is not charged to the PR
(#6247 failed on master's c5406e6b8 with two dots).

Nothing here can pass a file it did not regenerate. The spec and the gen file
are both read from the working tree, which in CI is the PR merge checkout the
t27c was built from.

Usage:
  python3 tools/l2_regen_check.py --base origin/master --head HEAD [--t27c PATH]
  python3 tools/l2_regen_check.py --list --base B --head H   # print M gen/ paths
"""
import argparse
import os
import subprocess
import sys

BACKENDS = {"c": "gen-c", "verilog": "gen-verilog", "rust": "gen-rust", "zig": "gen-zig"}


def modified_gen(base, head):
    out = subprocess.run(
        ["git", "diff", "--no-renames", "--name-status", f"{base}...{head}"],
        capture_output=True, text=True, check=True,
    ).stdout
    paths = []
    for line in out.splitlines():
        parts = line.split("\t")
        if len(parts) == 2 and parts[0] == "M" and parts[1].startswith("gen/"):
            paths.append(parts[1])
    return paths


def spec_for(path):
    """gen/<backend>/<rel>.<ext> -> (subcommand, specs/<rel>.t27) or (None, why)."""
    parts = path.split("/")
    if len(parts) < 3:
        return None, "no backend directory"
    sub = BACKENDS.get(parts[1])
    if sub is None:
        return None, f"unknown backend gen/{parts[1]}/"
    rel = "/".join(parts[2:])
    stem, dot, _ = rel.rpartition(".")
    if not dot:
        return None, "no file extension"
    spec = f"specs/{stem}.t27"
    if not os.path.isfile(spec):
        return None, f"no spec at {spec}"
    return sub, spec


def check(path, t27c):
    sub, spec = spec_for(path)
    if sub is None:
        return False, spec
    r = subprocess.run([t27c, sub, spec], capture_output=True)
    if r.returncode != 0:
        err = r.stderr.decode("utf-8", "replace").strip().splitlines()
        return False, f"t27c {sub} {spec} exited {r.returncode}: {err[:1]}"
    with open(path, "rb") as f:
        have = f.read()
    if have == r.stdout:
        return True, f"t27c {sub} {spec} reproduces it byte for byte"
    n = next((i for i, (a, b) in enumerate(zip(have, r.stdout)) if a != b),
             min(len(have), len(r.stdout)))
    return False, (f"differs from t27c {sub} {spec} at byte {n} "
                   f"({len(have)} bytes in tree, {len(r.stdout)} generated)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True)
    ap.add_argument("--head", required=True)
    ap.add_argument("--t27c", default="")
    ap.add_argument("--list", action="store_true")
    a = ap.parse_args()

    paths = modified_gen(a.base, a.head)
    if a.list:
        for p in paths:
            print(p)
        return 0
    if not paths:
        print("  ok: no gen/ file modified (new files allowed)")
        return 0

    if not (a.t27c and os.path.isfile(a.t27c) and os.access(a.t27c, os.X_OK)):
        for p in paths:
            print(f"  not checked  {p}: no t27c to regenerate it with")
        print("::error::L2 GENERATION: modified gen/ files were not checked "
              "(no t27c). Not checked is not a pass.")
        return 1

    bad = 0
    for p in paths:
        ok, why = check(p, a.t27c)
        print(f"  {'regenerated' if ok else 'HAND EDIT  '}  {p}: {why}")
        bad += 0 if ok else 1
    if bad:
        print(f"::error::L2 GENERATION VIOLATION: {bad} of {len(paths)} modified "
              "gen/ file(s) are not what t27c generates from their spec. "
              "Edit the spec and regenerate instead.")
        return 1
    print(f"  ok: all {len(paths)} modified gen/ file(s) are t27c output")
    return 0


if __name__ == "__main__":
    sys.exit(main())
