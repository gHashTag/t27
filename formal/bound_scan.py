#!/usr/bin/env python3
"""Map every self-incrementing register to whatever bounds it.

Wave 632. Prop. 83 found a 16-bit accumulator that overflows after 1214 terms
and is safe only because a *different module* walks its chunk counter over an
8-bit port. Nothing in the accumulating module knew that: no counter, no input,
no comment. The width was sufficient by accident of an unrelated port width, and
an ordinary change to another file would have deleted the accident silently.

That is a class, not an incident. A register that grows is safe only relative to
a bound, and the interesting question is never "is it wide enough" but "wide
enough for what, and where is that written". This answers the second question
mechanically for every `X <= X + k` in the emitted bundle:

  LOCAL     compared against a constant or localparam in its own module. The
            bound travels with the logic; reading the module tells you the range.
  CONTRACT  compared only against an INPUT PORT. The bound is real but lives in
            the caller. Safe until someone widens that port -- which is exactly
            the Prop. 83 shape, and is why these must be annotated rather than
            merely counted.
  FREE      nothing in the module compares it at all. It runs to its own width
            and wraps. Sometimes deliberate (a cycle counter that may wrap),
            sometimes a latent defect; the two are indistinguishable from the
            RTL, which is the whole point of requiring a note.

The gate: every CONTRACT and FREE register must carry a `// BOUND:` comment in
its module giving the reason. LOCAL registers need nothing -- their bound is
already legible. This does not prove any register is safe. It makes the absence
of an argument visible, which is the step that was missing when a 16-bit
accumulator went 600 waves without anyone asking what limited it.

Usage:  python3 formal/bound_scan.py [--self-test]
"""

import pathlib
import re
import sys

MODULE = re.compile(r"^module\s+(\w+)\s*(.*?)^endmodule", re.M | re.S)
INCR = re.compile(r"^\s*(?:.*?\b)?(\w+)\s*<=\s*([^;]*?\b\1\b\s*\+[^;]*);", re.M)
# Countdowns. Wave 633: the mirror of overflow is underflow, and it is the
# sharper risk here because two of this bundle's tightest bounds are enforced by
# a separate countdown rather than by any comparison on the thing being bounded.
# `X <= X - k` wraps to near 2^N the moment X < k, and a wrapped countdown does
# not stop -- it runs for another 2^N steps. Every drain must state its
# terminator.
DECR = re.compile(r"^\s*(?:.*?\b)?(\w+)\s*<=\s*([^;]*?\b\1\b\s*-[^;]*);", re.M)
PORT = re.compile(r"\b(?:input|output)\s+(?:wire\s+|reg\s+)?(?:signed\s*)?"
                  r"(?:\[[^\]]*\]\s*)?(\w+)")
NOTE = re.compile(r"//\s*BOUND:\s*`?(\w+)`?\s+(.+)")

# `==`, `!=`, `<` and `>` only. NOT `<=` or `>=`: in Verilog `<=` at statement
# level is the nonblocking ASSIGNMENT, and a regex cannot tell it from the
# comparison without parsing. The first draft of this file accepted it and so
# read `accumulator <= first_chunk ? ...` as "accumulator is compared against
# first_chunk" -- classifying the Prop. 83 accumulator, the register this scan
# exists because of, as bounded by a contract when it is bounded by nothing.
# Every LOCAL verdict in that draft came from a reset assignment `X <= 0` read
# as a bound. Dropping the ambiguous operators loses genuine `if (c <= limit)`
# bounds, which then read as FREE and demand a note: over-reporting, in the
# direction that asks for an argument rather than inventing one.
CMP = re.compile(r"(?:==|!=|<(?!=)|>(?!=))")


def strip_comments(text):
    return re.sub(r"//[^\n]*", "", text)


def strip_formal(text):
    """Return the design as it ships: T27_FORMAL* branches resolved as UNDEFINED.

    Wave 636b introduced this so the gate stopped crediting formal ASSERTIONS as
    design bounds. Wave 639b found it deleted real design in two ways, both
    verified:

      `ifndef T27_FORMAL   -- the body is DESIGN code (it is what compiles when
                              the define is absent) and was being removed
      `else                -- the else branch of an `ifdef T27_FORMAL is
                              likewise design, and was being removed with it

    Deleting design can only push a register toward FREE, which demands a note
    rather than hiding a defect -- so the direction was safe. It was still
    wrong, and it hid whatever bound lived in those branches.

    This resolves the guards properly instead of deleting regions: for a
    T27_FORMAL* guard the `ifdef branch is dropped and the `else branch kept,
    and for `ifndef the reverse. Guards on any OTHER symbol are left untouched,
    since this gate has no opinion about them.
    """
    DIRECTIVE = re.compile(r"`(ifdef|ifndef|elsif|else|endif)(?:\s+(\w+))?")
    out = []
    # stack of (is_formal_guard, keeping_now)
    stack = []
    pos = 0

    def emitting():
        return all(k for _, k in stack)

    for m in DIRECTIVE.finditer(text):
        if emitting():
            out.append(text[pos:m.start()])
        kind, sym = m.group(1), m.group(2) or ""
        if kind in ("ifdef", "ifndef"):
            formal = sym.startswith("T27_FORMAL")
            if formal:
                # T27_FORMAL is never defined in the shipped design.
                stack.append((True, kind == "ifndef"))
            else:
                stack.append((False, True))
        elif kind == "elsif":
            if stack:
                formal, _ = stack[-1]
                stack[-1] = (formal, False if formal else True)
        elif kind == "else":
            if stack:
                formal, keep = stack[-1]
                stack[-1] = (formal, (not keep) if formal else True)
        elif kind == "endif":
            if stack:
                stack.pop()
        pos = m.end()
    if emitting():
        out.append(text[pos:])
    return "".join(out)


def classify(body):
    """name -> (kind, evidence) for every self-incrementing register."""
    code = strip_formal(strip_comments(body))
    ports = set(PORT.findall(code))
    out = {}
    for m in DECR.finditer(code):
        name = m.group(1)
        if name not in out:
            out[name] = ("DRAIN", "counts down; underflow wraps to near 2^N")
    for m in INCR.finditer(code):
        name = m.group(1)
        if name in out:
            continue
        # Every genuine comparison in the module that mentions this register,
        # in either operand order.
        cmps = []
        for pat in (rf"\b{name}\s*{CMP.pattern}\s*([\w'\[\]:. +-]+?)\s*(?:\)|;|&|\||\?)",
                    rf"([\w'\[\]:. +-]+?)\s*{CMP.pattern}\s*\b{name}\b"):
            cmps += [c.strip() for c in re.findall(pat, code)]
        cmps = [c for c in cmps if c and c != name]
        if not cmps:
            out[name] = ("FREE", "no comparison in the module")
            continue
        # A bound that mentions a port is a bound the caller owns.
        ext = [c for c in cmps if any(re.search(rf"\b{p}\b", c) for p in ports)]
        if ext:
            out[name] = ("CONTRACT", f"bounded by port expr: {ext[0]}")
        else:
            out[name] = ("LOCAL", f"bounded in-module: {cmps[0]}")
    return out


def scan_file(path):
    src = path.read_text()
    rows, notes = [], {}
    for m in NOTE.finditer(src):
        notes[m.group(1)] = m.group(2).strip()
    for mod in MODULE.finditer(src):
        for name, (kind, why) in classify(mod.group(2)).items():
            rows.append((path.name, mod.group(1), name, kind, why,
                         notes.get(name)))
    return rows


def scan(root):
    files = sorted((root / "build" / "rtl").glob("*.sv"))
    if not files:
        print(f"::error::bound_scan found no RTL under {root}/build/rtl -- "
              "emit the bundle before running this gate")
        return 1
    rows = []
    for f in files:
        rows += scan_file(f)

    print(f"{'module':26s} {'register':20s} {'bound':9s} why")
    print("-" * 96)
    for _, mod, name, kind, why, note in sorted(rows, key=lambda r: (r[3], r[1])):
        print(f"{mod:26s} {name:20s} {kind:9s} {note or why}")

    bad = [r for r in rows if r[3] in ("CONTRACT", "FREE", "DRAIN") and not r[5]]
    for f, mod, name, kind, why, _ in bad:
        print(f"::error::{f}: `{mod}.{name}` is {kind} ({why}) and carries no "
              f"`// BOUND: {name} <reason>` note. A register whose limit is not "
              "in its own module is safe only relative to an argument, and an "
              "argument nobody wrote down is one the next change deletes "
              "silently. See Prop. 83.")

    # A scan that parsed nothing reports zero unannotated registers and reads
    # as a pass. Prop. 82c's lesson, kept.
    # WITNESS: `accumulator` is the Prop. 83 register -- the 16-bit sum safe
    # only because a different module bounds its chunk run. It is why this gate
    # exists, so its absence must be loud rather than inferred from a total.
    if not any(r[2] == "accumulator" for r in rows):
        print("::error::bound_scan never classified `accumulator`, the register "
              "this gate was written for (Prop. 83). It classified "
              f"{len(rows)} others and would have reported clean. See Prop. 124.")
        return 1

    if not rows:
        print(f"::error::bound_scan found no self-incrementing registers across "
              f"{len(files)} files -- it checked nothing, so its silence means "
              "nothing. Either the emitters changed or the parser broke.")
        return 1

    kinds = {k: sum(1 for r in rows if r[3] == k) for k in
             ("LOCAL", "CONTRACT", "FREE", "DRAIN")}
    print(f"\nbound scan: {len(rows)} counting registers across {len(files)} "
          f"files -- {kinds['LOCAL']} local, {kinds['CONTRACT']} by contract, "
          f"{kinds['FREE']} free, {kinds['DRAIN']} draining, {len(bad)} "
          "unannotated")
    return 1 if bad else 0


def self_test():
    import tempfile
    cases = [
        ("a counter bounded by a localparam", """
module m (input wire clk);
    localparam LIMIT = 8'd200;
    reg [7:0] c;
    always @(posedge clk) if (c == LIMIT) c <= 0; else c <= c + 8'd1;
endmodule
""", "LOCAL", 0),
        ("a counter bounded by an input port", """
module m (input wire clk, input wire [7:0] limit);
    reg [7:0] c;
    always @(posedge clk) if (c == limit) c <= 0; else c <= c + 8'd1;
endmodule
""", "CONTRACT", 1),
        ("the same, annotated", """
module m (input wire clk, input wire [7:0] limit);
    // BOUND: c the caller drives limit from an 8-bit port, so c <= 255.
    reg [7:0] c;
    always @(posedge clk) if (c == limit) c <= 0; else c <= c + 8'd1;
endmodule
""", "CONTRACT", 0),
        ("a countdown, which must state its terminator", """
module m (input wire clk, input wire go);
    reg [15:0] left;
    always @(posedge clk) if (go) left <= left - 16'd1;
endmodule
""", "DRAIN", 1),
        ("the same countdown, annotated", """
module m (input wire clk, input wire go);
    // BOUND: left terminates at exactly 1, so it reaches 0 and never wraps.
    reg [15:0] left;
    always @(posedge clk) if (go) left <= left - 16'd1;
endmodule
""", "DRAIN", 0),
        # Wave 639b: strip_formal must resolve guards, not delete regions. It
        # used to remove `ifndef T27_FORMAL bodies and `else branches, both of
        # which are DESIGN code, hiding whatever bound lived there.
        ("design inside `ifndef T27_FORMAL keeps its bound", """
module m (input wire clk);
    localparam LIMIT = 8'd200;
    reg [7:0] c;
`ifndef T27_FORMAL
    always @(posedge clk) if (c == LIMIT) c <= 0; else c <= c + 8'd1;
`endif
endmodule
""", "LOCAL", 0),
        ("design in the `else branch of a formal guard keeps its bound", """
module m (input wire clk);
    localparam LIMIT = 8'd200;
    reg [7:0] c;
`ifdef T27_FORMAL
    always @(posedge clk) c <= c + 8'd1;
`else
    always @(posedge clk) if (c == LIMIT) c <= 0; else c <= c + 8'd1;
`endif
endmodule
""", "LOCAL", 0),
        ("a register nothing compares -- the Prop. 83 shape", """
module m (input wire clk, input wire go, input wire signed [5:0] d);
    reg signed [15:0] acc;
    always @(posedge clk) if (go) acc <= acc + d;
endmodule
""", "FREE", 1),
    ]
    bad = []
    with tempfile.TemporaryDirectory() as td:
        d = pathlib.Path(td) / "build" / "rtl"
        d.mkdir(parents=True)
        for name, text, want_kind, want_bad in cases:
            f = d / "m.sv"
            f.write_text(text)
            rows = scan_file(f)
            got_kind = rows[0][3] if rows else "NONE"
            got_bad = sum(1 for r in rows if r[3] in ("CONTRACT", "FREE", "DRAIN")
                          and not r[5])
            ok = got_kind == want_kind and got_bad == want_bad
            print(f"  {'ok  ' if ok else 'FAIL'} {name}: {got_kind}, "
                  f"{got_bad} unannotated (want {want_kind}, {want_bad})")
            if not ok:
                bad.append(name)

        (d / "m.sv").write_text("module m; endmodule\n")
        rc = scan(pathlib.Path(td))
        print(f"  {'ok  ' if rc else 'FAIL'} a tree with no growing registers "
              f"fails rather than passing silently: exit {rc}")
        if rc == 0:
            bad.append("the zero-parse guard let an empty tree pass")

    for b in bad:
        print(f"::error::bound_scan self-test: {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else scan(r))
