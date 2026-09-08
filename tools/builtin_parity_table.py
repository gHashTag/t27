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
    "default_input()", "valid_input()", "random_input()",
    "assert_eq(v, v)", "approximately_equal(v, v)",
    "print(v)", "len(v)",
]

HEAD = "module P {\n    fn probe(v: i32) -> i32 { var a = {call}; return 0; }\n}\n"


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
        return "PASSTHROUGH " + body[:34]
    return body[:44]


def corpus_counts():
    """uses and distinct specs per name, so a row says what it is worth."""
    uses, files = {}, {}
    for r, _, fs in os.walk(os.path.join(ROOT, "specs")):
        for f in fs:
            if not f.endswith(".t27"):
                continue
            p = os.path.join(r, f)
            src = open(p, encoding="utf-8", errors="replace").read()
            for m in re.finditer(r"(?<![\w.@])([a-z_][a-z0-9_]*)\s*\(", src):
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
    print("\nPASSTHROUGH means the spelling survived into the target unchanged.")
    print("It is a CANDIDATE, not a verdict: `print` in synthesizable Verilog and")
    print("a float cast in C are refusals somebody chose. Read the row, then decide.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
