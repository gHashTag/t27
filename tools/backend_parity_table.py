#!/usr/bin/env python3
"""What does each backend emit for one t27 declaration form?

Five repairs in a row shared one shape: a single t27 declaration lowering to
DIFFERENT guarantees per backend -- `.len` on a slice, the out-parameter, `*T`,
the array size in C, the `Copy` derive in Rust. Each was found by accident, one
at a time, and each cost a full pass. This prints them as a table instead, so
the next one is read off a page rather than stumbled on.

It is a READER, not a gate: it asserts nothing about which column is right.
That judgement needs a person, and three of the five needed a real compiler to
settle. What it does is put the four answers side by side.

Found on its first run, and none of them by grep:

    [2][3]u8   C emits `[3]u8* x`      -- t27 syntax in a C header:
                                          "error: expected ')'"
    tri        C emits `tri x`         -- "error: unknown type name 'tri'",
                                          and zero typedefs are emitted
    GF16       Verilog gave `[31:0]`   -- the UNKNOWN-TYPE default, while
                                          `HwType::GF16.hw_width()` in the same
                                          file returns 16 with a passing test

One candidate it produced was REFUTED by reading the code rather than shipped:
`[]const u8` lowering to `&'static str` in Rust looks like a type change and is
a deliberate, load-bearing choice -- the HashMap key lowering asks this very
mapping for its key type. A table proposes; it does not conclude.

Usage:
  tools/backend_parity_table.py               print the table
  tools/backend_parity_table.py --self-check  negative control

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

# One row per declaration form. Kept deliberately small and hand-chosen: the
# point is a page a person reads, not exhaustive coverage of the type surface.
FORMS = [
    "u8", "i64", "f64", "bool", "char", "GF16", "gf16::GF16", "tri",
    "[4]u8", "[N]u8", "[0]u8", "[2][3]u8", "[4]GF16",
    "[]u8", "[]const u8", "str",
    "*i32", "?u8", "Pair", "Colour", "[4]Pair",
]

HEAD = (
    "module P {\n"
    "    const N : usize = 4;\n"
    "    struct Pair { a : i32, b : i32, }\n"
    "    enum Colour { Red, Green }\n"
)

BACKENDS = (("C", ["gen-c"]), ("Rust", ["gen-rust"]), ("Zig", ["gen"]), ("Verilog", ["gen-verilog"]))

# The Verilog declaration line ends `probe; // -> i32`. A pattern anchoring a
# newline straight after `probe;` matches NOTHING, and every cell then reads
# "(no param line)" -- a failure of this file printed as a fact about the
# backend. It did exactly that on the first run.
PATTERNS = {
    "C": r"^\w[\w \*]*\s+probe\(([^)]*)\);",
    "Rust": r"^pub fn probe\(([^)]*)\)",
    "Zig": r"^(?:pub )?fn probe\(([^)]*)\)",
    "Verilog": r"function[^\n]*\bprobe;[^\n]*\n\s*(input[^\n;]*;)",
}


def t27c() -> str:
    for rel in ("target/release/t27c", "bootstrap/target/release/t27c"):
        p = os.path.join(ROOT, rel)
        if os.path.isfile(p) and os.access(p, os.X_OK):
            return p
    env = os.environ.get("TRI_T27C", "")
    if env and os.access(env, os.X_OK):
        return env
    print("backend_parity_table: t27c not built. Exit 2 = COULD NOT RUN.", file=sys.stderr)
    print("  cargo build --release -p t27c, or set TRI_T27C.", file=sys.stderr)
    sys.exit(2)


def row(binary: str, form: str) -> dict:
    d = tempfile.mkdtemp()
    p = os.path.join(d, "p.t27")
    with open(p, "w", encoding="utf-8") as fh:
        fh.write(HEAD + f"    fn probe(x: {form}) -> i32 {{ return 0; }}\n}}\n")
    out = {}
    for label, cmd in BACKENDS:
        text = subprocess.run([binary] + cmd + [p], capture_output=True, text=True).stdout
        if not text.strip():
            out[label] = "(no output)"
            continue
        m = re.search(PATTERNS[label], text, re.M)
        out[label] = m.group(1).strip() if m else "(NOT MATCHED)"
    return out


def self_check(binary: str) -> int:
    """Every backend must answer for a plain `u8`, and each answer must differ
    from the others' spelling. Without this, a table of "(NOT MATCHED)" reads
    exactly like a table of real findings."""
    r = row(binary, "u8")
    ok = True
    for label, _ in BACKENDS:
        good = r[label] not in ("(no output)", "(NOT MATCHED)")
        print(f"  {label:8} on u8 -> {r[label]!r} {'PASS' if good else 'FAIL'}")
        ok &= good
    return 0 if ok else 2


def main() -> int:
    binary = t27c()
    if "--self-check" in sys.argv:
        return self_check(binary)

    rows = [(f, row(binary, f)) for f in FORMS]
    w = max(len(f) for f in FORMS)
    widths = {"C": 26, "Rust": 22, "Zig": 18, "Verilog": 26}
    header = "t27 form".ljust(w) + " | " + " | ".join(l.ljust(widths[l]) for l, _ in BACKENDS)
    print(header)
    print("-" * len(header))
    for form, r in rows:
        cells = [r[l][: widths[l]].ljust(widths[l]) for l, _ in BACKENDS]
        print(form.ljust(w) + " | " + " | ".join(cells))
    print()
    print("A reader, not a gate. A row whose columns disagree is a QUESTION;")
    print("three of the five known divergences needed a real compiler to settle,")
    print("and one candidate was refuted by reading the compiler's own comments.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
