#!/usr/bin/env python3
"""Does any spec declare the same top-level name twice?

`t27c check` was silent on this and every backend it compiles to is not.
Measured against the real compilers rather than assumed:

    struct A + struct A   cc: "redefinition of 'S'"      rustc: error[E0428]
                          zig: "duplicate ..."           iverilog: "already been declared"
    struct A + enum A     cc: "tag type that does not match"   rustc: error[E0428]
    struct A + fn A       rust and C ACCEPT it (types and values are separate
                          namespaces); zig rejects it, one namespace per container

So the two findings carry different messages, because one message for both
would be false for half the cases. This gate counts the strong kind.

`specs/ml/optimizer/adamw.t27` is the shape that motivated it: 1019 lines with
what looks like a revised second copy of its own declarations appended -- the
two `AdamWConfig` bodies are not even identical, one ends `use_phi_betas` and
the other `phi_variant`. The generated Rust carries both and does not compile:
error[E0428] 37 and error[E0119] 7 across the corpus.

A RATCHET, not a threshold. The baseline names the three specs and their counts;
the gate fails if a spec grows or a new one appears, and reports a shrink as
progress that should be recorded. A ceiling alone is satisfied by zero, so the
baseline is a per-spec map rather than a total.

Usage:
  tools/check_duplicate_declarations.py               gate
  tools/check_duplicate_declarations.py --list        print the findings and stop
  tools/check_duplicate_declarations.py --self-check  negative control

Exit codes:
  0  no spec exceeds its baseline
  1  a spec grew, or a spec not in the baseline carries a duplicate
  2  the gate COULD NOT RUN (no t27c binary, no specs, unreadable baseline)
"""

import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASELINE = os.path.join(ROOT, "tools", "duplicate_declarations_baseline.txt")
STRONG = "every backend rejects a redeclaration"
WEAK = "both a type and a function"


def t27c() -> str:
    for rel in ("target/release/t27c", "bootstrap/target/release/t27c"):
        p = os.path.join(ROOT, rel)
        if os.path.isfile(p) and os.access(p, os.X_OK):
            return p
    env = os.environ.get("TRI_T27C", "")
    if env and os.access(env, os.X_OK):
        return env
    # Exit 2, not 0: a check that could not run has not passed.
    print("check_duplicate_declarations: t27c not built. Exit 2 = COULD NOT RUN.",
          file=sys.stderr)
    print("  cargo build --release -p t27c, or set TRI_T27C.", file=sys.stderr)
    sys.exit(2)


def findings(binary: str, spec: str):
    """(strong, weak) counts for one spec."""
    r = subprocess.run([binary, "check", spec], capture_output=True, text=True)
    text = r.stdout + r.stderr
    return (
        sum(1 for ln in text.splitlines() if STRONG in ln),
        sum(1 for ln in text.splitlines() if WEAK in ln),
    )


def specs():
    out = []
    for root, _, files in os.walk(os.path.join(ROOT, "specs")):
        for f in files:
            if f.endswith(".t27"):
                out.append(os.path.join(root, f))
    return sorted(out)


def read_baseline():
    if not os.path.exists(BASELINE):
        print(f"check_duplicate_declarations: no baseline at {BASELINE}. Exit 2.",
              file=sys.stderr)
        sys.exit(2)
    base = {}
    for line in open(BASELINE, encoding="utf-8"):
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        path, n = line.rsplit(None, 1)
        base[path] = int(n)
    return base


def self_check(binary: str) -> int:
    """The gate must fire on a planted duplicate and stay quiet on a clean spec.

    Both halves, because a detector that fires on everything passes the first.
    """
    ok = True
    with tempfile.TemporaryDirectory() as d:
        dup = os.path.join(d, "dup.t27")
        with open(dup, "w", encoding="utf-8") as fh:
            fh.write("module SC1 {\n  struct A { x : i32, }\n  struct A { y : i32, }\n"
                     "  fn f(a: A) -> i32 { return a.x; }\n}\n")
        s, _ = findings(binary, dup)
        print(f"  planted duplicate      -> strong findings {s} (want >= 1) "
              f"{'PASS' if s >= 1 else 'FAIL'}")
        ok &= s >= 1

        clean = os.path.join(d, "clean.t27")
        with open(clean, "w", encoding="utf-8") as fh:
            fh.write("module SC2 {\n  struct C { x : i32, }\n"
                     "  fn g(c: C) -> i32 { return c.x; }\n}\n")
        s, w = findings(binary, clean)
        print(f"  clean spec             -> strong {s}, weak {w} (want 0, 0) "
              f"{'PASS' if s == 0 and w == 0 else 'FAIL'}")
        ok &= s == 0 and w == 0

        cross = os.path.join(d, "cross.t27")
        with open(cross, "w", encoding="utf-8") as fh:
            fh.write("module SC3 {\n  struct B { x : i32, }\n"
                     "  fn B(v: i32) -> i32 { return v; }\n}\n")
        s, w = findings(binary, cross)
        print(f"  type and function      -> strong {s}, weak {w} (want 0, 1) "
              f"{'PASS' if s == 0 and w == 1 else 'FAIL'}")
        ok &= s == 0 and w == 1

        # A test block is a declaration in every backend, and 314 duplicated
        # test names were invisible to this gate until they were counted.
        duptest = os.path.join(d, "duptest.t27")
        with open(duptest, "w", encoding="utf-8") as fh:
            fh.write('module SC4 {\n  test "same" { assert(1 == 1); }\n'
                     '  test "same" { assert(2 == 2); }\n}\n')
        s, w = findings(binary, duptest)
        print(f"  duplicated test name   -> strong {s}, weak {w} (want >= 1, 0) "
              f"{'PASS' if s >= 1 and w == 0 else 'FAIL'}")
        ok &= s >= 1 and w == 0

        # And the case that keeps the population honest: a test name equal to a
        # struct name is NOT a conflict in any backend. 138 names across 54
        # specs are that shape, and counting them would be 138 false findings.
        shared = os.path.join(d, "shared.t27")
        with open(shared, "w", encoding="utf-8") as fh:
            fh.write('module SC5 {\n  struct deque_clear { x : i32, }\n'
                     '  fn f(a: deque_clear) -> i32 { return a.x; }\n'
                     '  test "deque_clear" { assert(1 == 1); }\n}\n')
        s, w = findings(binary, shared)
        print(f"  test name = struct name-> strong {s}, weak {w} (want 0, 0) "
              f"{'PASS' if s == 0 and w == 0 else 'FAIL'}")
        ok &= s == 0 and w == 0
    return 0 if ok else 1


def main() -> int:
    binary = t27c()
    if "--self-check" in sys.argv:
        return self_check(binary)

    all_specs = specs()
    if not all_specs:
        print("check_duplicate_declarations: no specs found. Exit 2.", file=sys.stderr)
        return 2

    strong, weak = {}, {}
    for p in all_specs:
        s, w = findings(binary, p)
        rel = os.path.relpath(p, ROOT)
        if s:
            strong[rel] = s
        if w:
            weak[rel] = w

    print(f"specs scanned: {len(all_specs)}")
    print(f"specs declaring a name twice in one namespace: {len(strong)} "
          f"({sum(strong.values())} names)")
    for p in sorted(strong):
        print(f"   {strong[p]:3}  {p}")
    print(f"specs sharing a name between a type and a function: {len(weak)} "
          f"({sum(weak.values())} names) -- zig rejects these; rust and C do not")
    for p in sorted(weak):
        print(f"   {weak[p]:3}  {p}")

    if "--list" in sys.argv:
        return 0

    base = read_baseline()
    bad = False
    for p, n in sorted(strong.items()):
        allowed = base.get(p)
        if allowed is None:
            print(f"NEW: {p} declares {n} name(s) twice and is not in the baseline")
            bad = True
        elif n > allowed:
            print(f"GREW: {p} {allowed} -> {n}")
            bad = True
    for p, allowed in sorted(base.items()):
        now = strong.get(p, 0)
        if now < allowed:
            print(f"progress: {p} {allowed} -> {now} -- lower the baseline")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
