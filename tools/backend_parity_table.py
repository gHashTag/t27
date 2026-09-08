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
  tools/backend_parity_table.py               all three positions
  tools/backend_parity_table.py return field   only those positions
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

POSITIONS = ("param", "return", "field", "local")

# One row per declaration form. Kept deliberately small and hand-chosen: the
# point is a page a person reads, not exhaustive coverage of the type surface.
FORMS = [
    "u8", "i64", "f64", "bool", "char", "GF16", "gf16::GF16", "tri",
    "[4]u8", "[N]u8", "[0]u8", "[2][3]u8", "[4]GF16",
    "[]u8", "[]const u8", "str",
    "*i32", "?u8", "Pair", "Colour", "[4]Pair",
    # Combinations. The single forms above found six defects across four
    # positions; nothing had yet asked what happens when two of them meet.
    "[]Pair", "[][]u8", "?[4]u8", "?Pair", "*Pair",
    "[4]Colour", "(u8, i32)", "([4]u8, i32)", "[]*i32", "Deep",
]

HEAD = (
    "module P {\n"
    "    const N : usize = 4;\n"
    "    struct Pair { a : i32, b : i32, }\n"
    "    enum Colour { Red, Green }\n"
    "    struct Deep { p : Pair, a : [2]u8, }\n"
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


BODY = {
    "param": "    fn probe(x: {t}) -> i32 {{ return 0; }}\n",
    "return": "    fn probe(v: i32) -> {t} {{ return 0; }}\n",
    "field": ("    struct Holder {{ f : {t}, g : i32, }}\n"
              "    fn probe(h: Holder) -> i32 {{ return h.g; }}\n"),
    # The local is initialised FROM A PARAMETER of the same type, which keeps
    # one probe for every form -- a per-type literal would be a second thing
    # that can be wrong. It also has to be initialised at all: `var x : T;`
    # parses, and C and Rust declare it, but Zig and Verilog emit NOTHING for
    # an uninitialised local, so that probe measured their dead-code removal
    # rather than their lowering.
    "local": "    fn probe(p: {t}) -> i32 {{ var x : {t} = p; return 0; }}\n",
}

# One pattern per (position, backend). Three positions, because the same
# declaration gets three different answers and the parameter table alone hid
# two defects: a fixed-array RETURN gave `uint8_t*` to a stack local
# (-Wreturn-stack-address, 18 sites in 14 files, #3445) and a fixed-array FIELD
# gave a pointer with no storage, making one struct 16 bytes in C and 8 in Rust
# (65 fields in 31 specs, #3446).
POS_PATTERNS = {
    "param": PATTERNS,
    "return": {
        "C": r"^([\w \*]+?)\s+probe\(",
        "Rust": r"^pub fn probe\([^)]*\)\s*->\s*([^{\n]+)",
        "Zig": r"^(?:pub )?fn probe\([^)]*\)\s*([^{\n]+)",
        "Verilog": r"function\s+([^;\n]*?)\s*\bprobe;",
    },
    "local": {
        # The declarator only, up to `=` -- the initialiser is the same in
        # every row and would just push the type out of the column.
        "C": r"^\s{4}([^\n;=]*\bx(?:\[[^\]]*\])*)\s*(?:=[^\n;]*)?;",
        "Rust": r"^\s{4}(let (?:mut )?x:[^=;\n]*)",
        "Zig": r"^\s{4}((?:var|const) x:[^=;\n]*)",
        "Verilog": r"^\s*(reg[^\n;]*\bx\b[^\n;]*);",
    },
    "field": {
        "C": r"struct Holder \{[^}]*?\n\s*([^\n;]*\bf(?:\[[^\]]*\])*)\s*;",
        "Rust": r"pub struct Holder \{[^}]*?\n\s*(pub f:[^,\n]*)",
        "Zig": r"(?:pub )?const Holder = (?:extern |packed )?struct \{[^}]*?\n\s*(f:[^,\n]*)",
        # A struct has TWO Verilog shapes and the first pattern only knew one.
        # When every field is lowerable the struct becomes one packed vector and
        # a comment states its width; otherwise each field gets its own `reg`,
        # under a comment saying the struct is unsupported. Matching only the
        # packed form printed "(NOT MATCHED)" for six rows -- which reads as a
        # finding and was a gap in this file.
        # `(?im)` may not appear mid-pattern -- Python raises
        # "global flags not at the start of the expression". re.M is already
        # passed at the call site, and nothing here needs case folding.
        "Verilog": (r"^\s*//\s*struct Holder (lowered[^\n]*)$"
                    r"|^\s*(reg[^\n;]*\bholder_f\b[^\n;]*);"),
    },
}


def row(binary: str, form: str, position: str = "param") -> dict:
    d = tempfile.mkdtemp()
    p = os.path.join(d, "p.t27")
    with open(p, "w", encoding="utf-8") as fh:
        fh.write(HEAD + BODY[position].format(t=form) + "}\n")
    out = {}
    for label, cmd in BACKENDS:
        text = subprocess.run([binary] + cmd + [p], capture_output=True, text=True).stdout
        if not text.strip():
            out[label] = "(no output)"
            continue
        m = re.search(POS_PATTERNS[position][label], text, re.M | re.S)
        got = next((g for g in m.groups() if g), None) if m else None
        out[label] = re.sub(r"\s+", " ", got).strip() if got else "(NOT MATCHED)"
    return out


def self_check(binary: str) -> int:
    """Every backend must answer for a plain `u8`, in every position.

    Without this a table of "(NOT MATCHED)" reads exactly like a table of real
    findings -- and it did: the first Verilog column was empty in every row
    because the declaration ends `probe; // -> i32` and the pattern demanded a
    newline right after `probe;`. A failure of this file, printed as a fact
    about the backend."""
    ok = True
    for position in POSITIONS:
        r = row(binary, "u8", position)
        for label, _ in BACKENDS:
            good = r[label] not in ("(no output)", "(NOT MATCHED)")
            print(f"  {position:7} {label:8} on u8 -> {r[label]!r} {'PASS' if good else 'FAIL'}")
            ok &= good
    return 0 if ok else 2


def main() -> int:
    binary = t27c()
    if "--self-check" in sys.argv:
        return self_check(binary)

    wanted = [a for a in sys.argv[1:] if a in POSITIONS] or list(POSITIONS)
    widths = {"C": 26, "Rust": 22, "Zig": 18, "Verilog": 30}
    w = max(len(f) for f in FORMS)
    for position in wanted:
        header = f"{position} position".ljust(w) + " | " + " | ".join(
            l.ljust(widths[l]) for l, _ in BACKENDS
        )
        print(header)
        print("-" * len(header))
        for form in FORMS:
            r = row(binary, form, position)
            cells = [r[l][: widths[l]].ljust(widths[l]) for l, _ in BACKENDS]
            print(form.ljust(w) + " | " + " | ".join(cells))
        print()
    print("A reader, not a gate. A row whose columns disagree is a QUESTION;")
    print("three of the five known divergences needed a real compiler to settle,")
    print("and one candidate was refuted by reading the compiler's own comments.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
