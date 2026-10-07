#!/usr/bin/env python3
"""L2 GENERATION plumbing. The rule is specs/policy/l2_generation.t27 (#7113).

This file decides nothing. It gathers the facts the spec's plan_all() reads,
runs plan_all() from gen/c/policy/l2_generation.c (t27c gen-c of the spec,
built here with a one-line C main, as own-language.yml builds its gate), then
runs t27c on each copy the plan names and compares bytes. Which gen/ files are
checked (#6226 modified copies, #7127 added ones, #7103 copies whose spec or
an import of it changed), which t27c subcommand writes which backend
directory, how a `use` path names a spec and every failure message are the
spec's; read them there.

The facts, one buffer:
  --pr | --all
  git diff --no-renames --name-status base...head   (three dots: from the
      merge base, so a gen/ file master changed after the PR branched is not
      charged to the PR, #6247; nothing in --all mode)
  --gen    git ls-files gen/
  --specs  git ls-files specs/*.t27
  --use    git grep of the `use` lines in specs/*.t27

On a PR the plan comes from the BASE's copy of the rule (git show
<base>:gen/c/policy/l2_generation.c), so a PR cannot loosen the rule that
judges it. A base without the copy (the PR that adds it) and --all run the
tree's copy, and then that copy is the first row checked: a copy t27c does not
reproduce from the spec fails before its plan is believed. The spec and the gen
file being compared are read from the working tree, which in CI is the PR
merge checkout the t27c was built from. Nothing here can pass a file t27c did
not reproduce: no t27c, no C compiler, a plan without its "--end" line or a
row that is not five fields is a failure, never "not checked, fine".

Usage:
  python3 tools/l2_regen_check.py --base origin/master --head HEAD [--t27c PATH]
  python3 tools/l2_regen_check.py --list --base B --head H   # print the paths it checks
  python3 tools/l2_regen_check.py --all [--t27c PATH]       # every tracked gen/ file
"""
import argparse
import os
import shutil
import subprocess
import sys
import tempfile

RULE_COPY = "gen/c/policy/l2_generation.c"
RULE_SPEC = "specs/policy/l2_generation.t27"
MAIN = (
    '#include "rule.c"\n#include <stdio.h>\n'
    "int main(void){static char b[1<<24],o[1<<22],m[1<<16];"
    "size_t n=fread(b,1,sizeof b,stdin);"
    "plan_all((uint8_t*)b,n,(uint8_t*)o,sizeof o-1,(uint8_t*)m,sizeof m);"
    "fputs(o,stdout);return n==sizeof b;}\n"
)


def fail(msg):
    print(f"::error::L2 GENERATION: {msg}. Not checked is not a pass.")
    sys.exit(1)


def git(*args, ok=(0,)):
    r = subprocess.run(["git", *args], capture_output=True)
    if r.returncode not in ok:
        fail(f"git {' '.join(args)} exited {r.returncode}")
    return r.stdout


def facts(base, head):
    parts = [b"--all\n" if not base else b"--pr\n"]
    if base:
        parts.append(git("diff", "--no-renames", "--name-status", f"{base}...{head}"))
    parts += [b"--gen\n", git("ls-files", "gen/"),
              b"--specs\n", git("ls-files", "specs/*.t27"),
              b"--use\n", git("grep", "-I", "-E", "-e", "^[[:space:]]*use ",
                              "--", "specs/*.t27", ok=(0, 1))]
    return b"".join(p if p.endswith(b"\n") or not p else p + b"\n" for p in parts)


def rule_source(base):
    if base:
        r = subprocess.run(["git", "show", f"{base}:{RULE_COPY}"], capture_output=True)
        if r.returncode == 0:
            return r.stdout, f"{RULE_COPY} at {base}"
    try:
        with open(RULE_COPY, "rb") as f:
            return f.read(), f"{RULE_COPY} in the tree"
    except OSError:
        fail(f"no {RULE_COPY} to read the rule from")


def plan(base, head):
    src, where = rule_source(base)
    cc = shutil.which("cc")
    if cc is None:
        fail("no C compiler to build the rule with")
    with tempfile.TemporaryDirectory() as d:
        with open(os.path.join(d, "rule.c"), "wb") as f:
            f.write(src)
        exe = os.path.join(d, "plan")
        r = subprocess.run([cc, "-w", "-I", d, "-x", "c", "-o", exe, "-"],
                           input=MAIN.encode(), capture_output=True)
        if r.returncode != 0:
            fail(f"{where} did not build: {r.stderr.decode('utf-8', 'replace')[:200]}")
        r = subprocess.run([exe], input=facts(base, head), capture_output=True)
    lines = r.stdout.decode("utf-8", "replace").splitlines()
    if r.returncode != 0 or not lines or lines[-1] != "--end":
        fail(f"the plan from {where} is cut short (exit {r.returncode})")
    rows = [line.split("\t") for line in lines[:-1]]
    for row in rows:
        if len(row) != 5:
            fail(f"a plan row is not five fields: {row!r}")
    if where.endswith("in the tree"):
        # A tree copy has not been judged by anything yet (the PR that adds it,
        # or --all): it is checked first, against its own spec whatever row it
        # wrote for itself, so a copy that plans nothing fails on this row
        # instead of passing everything (#7113 control 5).
        rows = [["rule is t27c output", "RULE NOT OUTPUT", "gen-c",
                 RULE_SPEC, RULE_COPY]] + [r for r in rows if r[4] != RULE_COPY]
    return rows, where


def check(sub, spec, path, t27c):
    if sub == "-":
        return False, spec
    r = subprocess.run([t27c, sub, spec], capture_output=True)
    if r.returncode != 0:
        err = r.stderr.decode("utf-8", "replace").strip().splitlines()
        return False, f"t27c {sub} {spec} exited {r.returncode}: {err[:1]}"
    try:
        with open(path, "rb") as f:
            have = f.read()
    except OSError as e:
        return False, f"unreadable: {e}"
    if have == r.stdout:
        return True, f"t27c {sub} {spec} reproduces it byte for byte"
    n = next((i for i, (a, b) in enumerate(zip(have, r.stdout)) if a != b),
             min(len(have), len(r.stdout)))
    return False, (f"differs from t27c {sub} {spec} at byte {n} "
                   f"({len(have)} bytes in tree, {len(r.stdout)} generated)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", default="")
    ap.add_argument("--head", default="")
    ap.add_argument("--t27c", default="")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--all", action="store_true")
    a = ap.parse_args()
    if a.all:
        a.base = a.head = ""
    elif not (a.base and a.head):
        ap.error("--base and --head are required unless --all")

    rows, where = plan(a.base, a.head)
    if a.list:
        for row in rows:
            print(row[4])
        return 0
    print(f"  rule: {where}")
    if not rows:
        print("  ok: no gen/ file added or modified, and no tracked copy's spec changed")
        return 0
    if not (a.t27c and os.path.isfile(a.t27c) and os.access(a.t27c, os.X_OK)):
        for row in rows:
            print(f"  not checked  {row[4]}: no t27c to regenerate it with")
        fail("gen/ files were not checked (no t27c)")

    bad = 0
    for good, wrong, sub, spec, path in rows:
        ok, why = check(sub, spec, path, a.t27c)
        print(f"  {good if ok else wrong}  {path}: {why}")
        bad += 0 if ok else 1
    if bad:
        what = "tracked" if a.all else "added, modified or spec-changed"
        print(f"::error::L2 GENERATION VIOLATION: {bad} of {len(rows)} {what} gen/ "
              "file(s) are not what t27c generates from their spec. "
              "Edit the spec and regenerate (t27c gen-<backend> specs/<path>.t27 "
              "> gen/<backend>/<path>.<ext>), or delete a copy nothing reads.")
        return 1
    print(f"  ok: all {len(rows)} checked gen/ file(s) are t27c output")
    return 0


if __name__ == "__main__":
    sys.exit(main())
