#!/usr/bin/env python3
"""Does every .t27 spec still compile to at least one target?

The README says `.t27` specs in → Zig, Verilog, C out, and the constitution makes specs
the single source of truth. Measured on 2026-08-18:

    1114  specs tracked
     766  generate            68.8%
     348  do NOT generate     31.2%

Not one of the 348 is a backend mismatch. On a 25-spec random sample, **zero** generated
with any of gen-c / gen-rust / gen-verilog / gen-zig -- they fail in the parser, before a
backend is reached. That alternative was checked because "31% of the source of truth does
not compile" is an alarming claim, and an alarming claim is usually a fault in the
instrument (see .claude/skills/ci-gates/SKILL.md §7).

Where they are, and how they fail:

    specs/tri/     70     parse error at module level  120
    specs/scratch/ 58     parse error in fn            114
    specs/fpga/    35     Expected RBrace               38
    specs/igla/    15     Expected LBrace               36
    specs/numeric/ 15     unknown cast target           34
    ...                   Unexpected top-level token     4

How this was found, which matters for what to do next. `t27c seal <spec> --save`
re-seals a spec **that does not generate**, writing `gen_hash_rust=none`. So a stale seal
can be "fixed" into a seal that records reproducibility for output which does not exist.
Batch re-sealing the 113 stale seals would have blessed 46 non-generating specs that way,
and written them under new filenames besides, leaving the originals stale. Testing one
instead of the batch is what surfaced this.

The 348 are recorded in tools/specs_generate_baseline.txt as debt, one per line with its
first compiler message, so this gate holds the line without demanding they all be fixed.
The number can only go down.

Usage:
  tools/check_specs_generate.py                  gate
  tools/check_specs_generate.py --self-check     negative control
  tools/check_specs_generate.py --update-baseline
  tools/check_specs_generate.py --summary        counts by directory and error class

Exits non-zero if a spec that used to generate stops generating.
"""
import collections
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "tools/specs_generate_baseline.txt"
BACKENDS = ("c", "rust", "verilog", "zig")


def t27c():
    for p in ("target/release/t27c", "target/debug/t27c"):
        c = ROOT / p
        if c.exists():
            return str(c)
    sys.exit("FAIL: t27c not built. Run: cargo build --release -p t27c")


def specs():
    r = subprocess.run(["git", "ls-files", "*.t27"], cwd=ROOT, capture_output=True, text=True)
    return sorted(x for x in r.stdout.split() if x)


def generates(t, sp):
    """(ok, first message). ok if ANY backend accepts it -- a spec written for one
    target should not be reported as broken because another target rejects it."""
    first = ""
    for m in BACKENDS:
        r = subprocess.run([t, "gen-" + m, sp], capture_output=True, text=True, cwd=ROOT)
        if r.returncode == 0:
            return True, ""
        if not first:
            first = (r.stderr or r.stdout or "").strip().split("\n")[0][:150]
    return False, first


def baseline():
    if not BASELINE.exists():
        return set()
    return {l.split("|")[0].strip() for l in BASELINE.read_text().splitlines()
            if l.strip() and not l.startswith("#")}


def self_check():
    """A spec with a deliberate syntax error must be reported; a good one must not."""
    import tempfile
    t = t27c()
    with tempfile.TemporaryDirectory() as td:
        good = os.path.join(td, "good.t27")
        bad = os.path.join(td, "bad.t27")
        open(good, "w").write("module G;\nfn f(a: u8) -> u8 { return a; }\n")
        open(bad, "w").write("module B;\nfn f(a: u8) -> u8 { return a\n")   # missing ; and }
        g_ok, _ = generates(t, good)
        b_ok, msg = generates(t, bad)
    ok = g_ok and not b_ok
    print(f"  self-check: valid spec generates = {g_ok}, broken spec reported = {not b_ok}")
    if not b_ok and msg:
        print(f"              reported: {msg[:90]}")
    return 0 if ok else 1


def main():
    t = t27c()
    if "--self-check" in sys.argv:
        return self_check()

    all_specs = specs()
    if not all_specs:
        print("FAIL: git ls-files found no .t27 at all -- the scan is broken, not the tree")
        return 1
    bad = []
    for sp in all_specs:
        ok, msg = generates(t, sp)
        if not ok:
            bad.append((sp, msg))

    if "--summary" in sys.argv:
        print(f"  {len(all_specs)} specs, {len(all_specs)-len(bad)} generate "
              f"({100*(len(all_specs)-len(bad))/len(all_specs):.1f}%), {len(bad)} do not\n")
        print("  by directory:")
        for d, c in collections.Counter(
                sp.split("/")[1] if sp.count("/") > 1 else "." for sp, _ in bad).most_common(12):
            print(f"    {c:>4}  specs/{d}/")
        print("\n  by error class:")
        def cls(m):
            for k in ("unknown cast target", "parse error at module level",
                      "Unexpected top-level token", "Expected LBrace", "Expected RBrace",
                      "parse error in fn"):
                if k in m:
                    return k
            return m[:44]
        for k, c in collections.Counter(cls(m) for _, m in bad).most_common(10):
            print(f"    {c:>4}  {k}")
        return 0

    if "--update-baseline" in sys.argv:
        BASELINE.write_text(
            "# Specs that do not generate with ANY backend. Each line is a debt.\n"
            "# Remove the line when the spec compiles; the gate then holds it compiling.\n"
            + "".join(f"{sp} | {msg}\n" for sp, msg in bad))
        print(f"  baseline written: {len(bad)} entries")
        return 0

    known = baseline()
    new = [(sp, m) for sp, m in bad if sp not in known]
    fixed = sorted(known - {sp for sp, _ in bad})
    if fixed:
        print(f"NOTE {len(fixed)} spec(s) in the baseline now generate. Remove them from "
              f"{BASELINE.name} so the gate holds them:")
        for sp in fixed[:10]:
            print(f"  {sp}")
        print()
    if not new:
        print(f"OK: {len(all_specs)} specs, {len(all_specs)-len(bad)} generate, "
              f"{len(bad)} known-broken in {BASELINE.name}")
        return 0
    print(f"FAIL: {len(new)} spec(s) newly do not generate with any backend\n")
    for sp, m in new:
        print(f"  {sp}\n      {m}")
    print("\n  The message is the compiler's own. A spec that does not generate is not a")
    print("  source of truth for anything, and t27c seal --save will still seal it with")
    print("  gen_hash=none -- so this must fail rather than be sealed over.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
