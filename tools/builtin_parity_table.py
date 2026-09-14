#!/usr/bin/env python3
"""What does each backend emit for a builtin SPELLING the specs rely on?

`tools/backend_parity_table.py` asks this about DECLARATION FORMS. This asks it
about CALLS, which is the other axis, and it exists because that axis has now
produced five defects of one shape -- a rule present in three backends and
missing from the fourth:

    Type.member / Type::member   gen-c had no answer      (#3467, #3469)
    `.` versus `->`              gen-c had no answer      (#3477)
    a brace literal as a value   gen-c had no answer      (#3475, #3492)
    `s.len` on a string          gen-c had no answer      (#3489)
    cast_iN(x)                   gen-c had no answer      (#3497)

The last one had been filed as UNFIXABLE for four passes -- "a name nothing in
the tree declares" -- while `gen-zig` lowered it, and the comment above that
code stated the very fact used as the blocker. This table would have printed
the disagreement on its first run.

WHY IT CANNOT BE A GATE. A blank column is not a defect by itself: `print` has
no business in synthesizable Verilog, and a float cast is refused deliberately
in two backends because choosing the conversion silently is a semantic decision.
The table proposes; a person concludes. Three of the five above needed a real
compiler to settle.

The population column is measured from the corpus, so a row says what it is
worth: `cast_i8` is 1079 uses, `sign` is 63.

Usage:
  tools/builtin_parity_table.py               the table
  tools/builtin_parity_table.py --self-check  negative control

Exit codes:
  0  the table printed
  2  COULD NOT RUN (no t27c binary), or the self-check failed
"""

import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

BACKENDS = (("C", "gen-c"), ("Rust", "gen-rust"), ("Zig", "gen"), ("Verilog", "gen-verilog"))

# Hand-chosen, like the sibling table's forms: the point is a page a person
# reads. Every row is a spelling the specs actually use; the counts beside them
# come from the corpus, not from this list.
CALLS = [
    "cast_i8(v)", "cast_i16(v)", "cast_i32(v)", "cast_f32(v)",
    "abs(v)", "abs_i16(v)", "abs_f32(v)",
    "sqrt(v)", "floor(v)", "round(v)", "min(v, v)", "max(v, v)", "sign(v)",
    "default_input()", "valid_input()", "random_input()", "finite_input()",
    "assert_eq(v, v)", "approximately_equal(v, v)", "eq(v, v)",
    "print(v)", "len(v)",
    # Added once the population column counted CODE rather than prose, which
    # reordered the list: `assert_eq` is 178 uses in 60 specs and `eq` 40 in 9,
    # while `sign` -- recommended as a language question on 63 -- is one use in
    # one spec. These rows were chosen from what the corpus actually calls.
    "from_f64(v)", "compose(v, v, v)", "all_positive(v)", "all_equal(v)",
]

# The probe carries a TEST BLOCK, because a spec without one is not
# representative -- 528 of 651 specs have tests -- and because two of the
# preambles are conditional on it: C emits `#define assert_eq(a, b)` and
# `<assert.h>` only when a module has tests. Probing without one reported
# `assert_eq` as undeclared in a backend that declares it.
HEAD = ("module P {\n"
        "    fn probe(v: i32) -> i32 { var a = {call}; return 0; }\n"
        "    test \"t\" { assert(1 == 1); }\n"
        "}\n")


def t27c() -> str:
    for rel in ("target/release/t27c", "bootstrap/target/release/t27c"):
        p = os.path.join(ROOT, rel)
        if os.path.isfile(p) and os.access(p, os.X_OK):
            return p
    env = os.environ.get("TRI_T27C", "")
    if env and os.access(env, os.X_OK):
        return env
    print("builtin_parity_table: t27c not built. Exit 2 = COULD NOT RUN.", file=sys.stderr)
    print("  cargo build --release -p t27c, or set TRI_T27C.", file=sys.stderr)
    sys.exit(2)


def emit(binary: str, sub: str, call: str, d: str) -> str:
    p = os.path.join(d, "in.t27")
    with open(p, "w", encoding="utf-8") as fh:
        fh.write(HEAD.replace("{call}", call))
    r = subprocess.run([binary, sub, p], capture_output=True, text=True)
    return r.stdout


def declares(out: str, name: str) -> bool:
    """Does the emitted file DEFINE this name itself?

    The distinction the first version missed. A spelling that survives into the
    target is a defect only if nothing declares it: C already emits
    `#define assert_eq(a, b)` and Zig `fn assert_eq(a: anytype, ...)`, so both
    columns read PASSTHROUGH while both were correct by design -- and the table
    reported "no backend lowers any of them" about a call that two backends
    answer with a helper. Defining the name is a lowering STRATEGY, not a gap.
    """
    # An INCLUDE declares too. `sqrt` is answered in C by `#include <math.h>`
    # and nothing else -- the spelling was never wrong -- and the second
    # version of this reader still called that column PASSTHROUGH, because it
    # looked only for definitions written in the file. Three readings of one
    # row, three different wrong answers, each from a narrower question than
    # "does the output declare this name".
    HEADERS = {
        "math.h": ("sqrt", "floor", "round", "ceil", "fabs", "pow", "log", "exp"),
        "string.h": ("strlen", "strcmp", "memcpy", "memset"),
        "stdlib.h": ("abs", "malloc", "free"),
        "stdio.h": ("printf", "puts"),
    }
    for hdr, names in HEADERS.items():
        if name in names and re.search(r"^\s*#include\s*<" + re.escape(hdr) + r">", out, re.M):
            return True
    pats = (
        r"^\s*#define\s+" + re.escape(name) + r"\s*\(",          # C macro
        r"^\s*(?:pub\s+)?fn\s+" + re.escape(name) + r"\s*\(",    # Zig / Rust
        r"^[\w \*]+\s+" + re.escape(name) + r"\s*\([^;]*\)\s*\{",  # C function
        r"^\s*function\b[^\n]*\b" + re.escape(name) + r"\b",     # Verilog
    )
    return any(re.search(p, out, re.M) for p in pats)


def cell(out: str, call: str) -> str:
    """What the backend did with this call, in one short phrase."""
    name = call.split("(")[0]
    if not out.strip():
        return "(no output)"
    # The line the probe's assignment landed on, if any.
    line = ""
    for l in out.splitlines():
        if re.search(r"\ba\b\s*(?::[^=]*)?=", l) or re.search(r"\ba\b\s*=", l):
            line = l.strip()
            break
    if not line:
        return "(no assignment)"
    body = line.split("=", 1)[1].strip().rstrip(";").strip() if "=" in line else line
    # PASSTHROUGH is the finding: the spelling survived into the target.
    # `@` belongs in the lookbehind: Zig's `@abs(v)` contains `abs(`, and the
    # first version of this reader called six lowered rows PASSTHROUGH -- a
    # failure of the instrument printed as a fact about the backend, which is
    # the sibling table's own recorded first-run mistake.
    if re.search(r"(?<![\w.@])" + re.escape(name) + r"\s*\(", body):
        return ("helper " if declares(out, name) else "PASSTHROUGH ") + body[:30]
    return body[:44]


def corpus_counts():
    """uses and distinct specs per name, COUNTED IN CODE ONLY.

    The first version read the whole file, comments included, and every number
    it printed was inflated -- `sign` read 63 uses in 48 specs where the code
    holds ONE, `max` 51/26 where the code holds 4, `sqrt` 100/25 where it holds
    33/13. A prose line that says "round the mantissa (see round(x))" is not a
    call, and a population column that counts it makes a row look worth more
    than it is. `sign` was recommended as a language-level question on the
    strength of 63; it is one use in one spec.
    """
    uses, files = {}, {}
    for r, _, fs in os.walk(os.path.join(ROOT, "specs")):
        for f in fs:
            if not f.endswith(".t27"):
                continue
            p = os.path.join(r, f)
            for line in open(p, encoding="utf-8", errors="replace"):
                t = line.lstrip()
                if t.startswith("//") or t.startswith("#"):
                    continue
                for m in re.finditer(r"(?<![\w.@])([a-z_][a-z0-9_]*)\s*\(", line):
                    n = m.group(1)
                    uses[n] = uses.get(n, 0) + 1
                    files.setdefault(n, set()).add(p)
    return uses, files


def self_check(binary: str) -> int:
    """A name known to be lowered must not read PASSTHROUGH, and a name nothing
    can know must read PASSTHROUGH everywhere. Without both, a table of
    PASSTHROUGH and a table of nothing look alike."""
    ok = True
    with tempfile.TemporaryDirectory() as d:
        c = cell(emit(binary, "gen-c", "cast_i8(v)", d), "cast_i8(v)")
        z = cell(emit(binary, "gen", "cast_i8(v)", d), "cast_i8(v)")
        # `not PASSTHROUGH` is true of every error string too -- the first
        # version of this control passed on "(no output)", which is the shape
        # a broken probe produces. The cell has to show the LOWERING.
        good = "int8_t" in c and "@intCast" in z
        print(f"  cast_i8, C and Zig  -> {'lowered in both' if good else 'FAIL'} "
              f"({'PASS' if good else 'FAIL'})\n      C: {c}\n      Zig: {z}")
        ok &= good
        # The population column must count CODE, not prose. `sign` appears 63
        # times counting comments and ONCE in code, and the first version of
        # this reader printed the 63 -- a row made to look 63x its worth.
        uses, files = corpus_counts()
        code_only = uses.get("sign", 0) <= 5
        print(f"  `sign` counted in code only -> {uses.get('sign', 0)} uses "
              f"({'PASS' if code_only else 'FAIL: comments are being counted'})")
        ok &= code_only

        # A name the OUTPUT declares is not a gap. C emits
        # `#define assert_eq(a, b)` and Zig `fn assert_eq(...)`; the first
        # version called both PASSTHROUGH and the table said "no backend
        # lowers any of them" about a call two backends answer.
        ce = cell(emit(binary, "gen-c", "assert_eq(v, v)", d), "assert_eq(v, v)")
        ze = cell(emit(binary, "gen", "assert_eq(v, v)", d), "assert_eq(v, v)")
        helped = ce.startswith("helper") and ze.startswith("helper")
        print(f"  assert_eq is a HELPER, not a gap ({'PASS' if helped else 'FAIL'})"
              f"\n      C: {ce}\n      Zig: {ze}")
        ok &= helped

        # And an INCLUDE is a declaration. `sqrt` is answered in C by
        # `#include <math.h>` alone; a reader that looks only for definitions
        # written in the file calls that a gap.
        cs = cell(emit(binary, "gen-c", "sqrt(v)", d), "sqrt(v)")
        inc_ok = cs.startswith("helper")
        print(f"  sqrt in C is answered by its include ({'PASS' if inc_ok else 'FAIL'})"
              f"\n      C: {cs}")
        ok &= inc_ok

        n = "zzz_not_a_builtin(v)"
        seen = [cell(emit(binary, sub, n, d), n) for _, sub in BACKENDS[:3]]
        bad = all(s.startswith("PASSTHROUGH") for s in seen)
        # ...and the `@` case, because the lookbehind above is the one thing in
        # this file that has already been wrong once.
        zabs = cell(emit(binary, "gen", "abs(v)", d), "abs(v)")
        at_ok = not zabs.startswith("PASSTHROUGH")
        print(f"  Zig `@abs(v)` is not read as PASSTHROUGH ({'PASS' if at_ok else 'FAIL'})"
              f"\n      Zig: {zabs}")
        ok &= at_ok
        print(f"  a name nothing knows -> {'PASSTHROUGH everywhere' if bad else 'FAIL'} "
              f"({'PASS' if bad else 'FAIL'})")
        ok &= bad
    return 0 if ok else 2


def main() -> int:
    binary = t27c()
    if "--self-check" in sys.argv:
        return self_check(binary)
    uses, files = corpus_counts()
    print(f"{'call':22} {'uses':>5} {'specs':>5}  " + "  ".join(f"{n:<26}" for n, _ in BACKENDS))
    with tempfile.TemporaryDirectory() as d:
        for call in CALLS:
            name = call.split("(")[0]
            cells = [cell(emit(binary, sub, call, d), call) for _, sub in BACKENDS]
            print(f"{call:22} {uses.get(name, 0):5} {len(files.get(name, ())):5}  "
                  + "  ".join(f"{c:<26}" for c in cells))
    print("\nPASSTHROUGH means the spelling survived AND nothing in the output")
    print("declares it -- neither a definition in the file nor an include that")
    print("brings it in. `helper` means it survived and the output answers it --")
    print("a lowering strategy, not a gap: C emits `#define assert_eq(a, b)` and")
    print("Zig `fn assert_eq(...)`. Only PASSTHROUGH is a candidate, and even")
    print("then not a verdict: `print` in synthesizable Verilog and a float cast")
    print("in C are refusals somebody chose. Read the row, then decide.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
