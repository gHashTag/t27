#!/usr/bin/env python3
"""List every register whose reset value is not zero, and say why it matters.

Wave 637. Every module suite in this repository is proved with `-set-init-zero`,
which starts the run from the all-zero register vector. That is universally
described in this campaign as "starting from a reachable state" -- Prop. 8c
chose it over `-tempinduct` for exactly that reason.

It is not the reset state. It is the ZERO state, and the two coincide only when
every register's reset value happens to be zero. **Ten registers in this design
reset to something else**, including four FSM state registers that reset to
`IDLE` -- harmless today only because IDLE is encoded 0 in all four.

WHAT THIS IS NOT. It is not an unsoundness. Starting from a superset of the
reachable states can only produce spurious REFUTATIONS, never spurious proofs:
every property that proves under `-set-init-zero` proves for all reachable
states too. Nothing verified here is weakened by this.

WHAT IT IS. A fragility, and an invisible one. Renumbering an FSM so that any
DECODED state lands on code 0 -- a pure relabelling, since every reference in
these modules is by name -- makes the zero state decode as *active*, and any
property relating an output to that state refutes at once. Verified in Wave 637:
relabelling `dma_controller`'s READ_DATA to 3'd0 refutes exactly
`a_rready_implies_burst`; swapping `weight_prefetch_ctrl`'s IDLE and FETCH
refutes exactly `a_rready_implies_active`. Two properties, in two suites, from a
change that alters nothing in silicon.

The failure would read as a design defect. It is a modelling artifact, and
`double_buffer_props` already discovered a local instance of it -- its
`fv_started` register exists because `use_buffer_a` resets to 1 while
`-set-init-zero` starts it at 0 -- without anyone recognising the general case.

This gate does not forbid non-zero resets. It requires them to be LISTED, so
that the gap between "the zero state" and "the reset state" is written down
rather than rediscovered by a refutation.

Usage:  python3 formal/init_zero_scan.py [--self-test]
"""

import pathlib
import re
import sys

RESET = re.compile(r"if\s*\(\s*!\s*rst_n\s*\)(.*?)(?=\bend\b|\belse\b)", re.S)
ASSIGN = re.compile(r"(\w+)\s*<=\s*([^;]+);")
ZERO = re.compile(r"^(?:\d*'?s?[bdhBDH]?0+|0|\{[\w'\s:+*-]*\{?1'b0\}?\})$")
NOTE = re.compile(r"//\s*INIT-ZERO:\s*(\w+)\s+(.+)")


def strip_comments(text):
    return re.sub(r"//[^\n]*", "", text)


def nonzero_resets(text):
    """(register, value) for every reset assignment that is not zero."""
    code = strip_comments(text)
    out = []
    for m in RESET.finditer(code):
        for a in ASSIGN.finditer(m.group(1)):
            val = re.sub(r"\s+", "", a.group(2))
            if not ZERO.match(val):
                out.append((a.group(1), val))
    return out


def scan(root):
    files = sorted((root / "build" / "rtl").glob("*.sv"))
    if not files:
        print(f"::error::init_zero_scan found no RTL under {root}/build/rtl -- "
              "emit the bundle before running this gate")
        return 1

    rows, notes, resets_seen = [], {}, 0
    for f in files:
        src = f.read_text()
        for m in NOTE.finditer(src):
            notes[m.group(1)] = m.group(2).strip()
        code = strip_comments(src)
        resets_seen += len(RESET.findall(code))
        for reg, val in nonzero_resets(src):
            rows.append((f.name, reg, val, notes.get(reg)))

    print(f"{'file':28s} {'register':22s} {'resets to':14s} note")
    print("-" * 96)
    for f, reg, val, note in sorted(rows):
        print(f"{f:28s} {reg:22s} {val:14s} {note or '-- UNDOCUMENTED --'}")

    bad = [r for r in rows if not r[3]]
    for f, reg, val, _ in bad:
        print(f"::error::{f}: `{reg}` resets to {val}, not zero, and carries no "
              f"`// INIT-ZERO: {reg} <reason>` note. Every module suite is "
              "proved with -set-init-zero, so the run starts from a state this "
              "register never reaches by reset. That cannot make a proof "
              "unsound -- only spurious refutations follow from extra states -- "
              "but it makes the suite fragile to a pure relabelling, and the "
              "resulting failure reads as a design defect. See Prop. 96.")

    # A scan that found no reset blocks at all read nothing.
    if resets_seen == 0:
        print(f"::error::init_zero_scan found no `if (!rst_n)` blocks across "
              f"{len(files)} files -- it examined nothing, so its silence means "
              "nothing. Either the emitters changed or the parser broke.")
        return 1

    print(f"\ninit-zero scan: {len(files)} files, {resets_seen} reset blocks, "
          f"{len(rows)} registers resetting non-zero, {len(bad)} undocumented")
    return 1 if bad else 0


def self_test():
    import tempfile
    cases = [
        ("a register resetting to zero is not listed", """
module m (input wire clk, input wire rst_n);
    reg [3:0] c;
    always @(posedge clk or negedge rst_n)
        if (!rst_n) c <= 4'd0;
        else c <= c + 4'd1;
endmodule
""", 0, 0),
        ("a register resetting non-zero must be documented", """
module m (input wire clk, input wire rst_n);
    reg [3:0] c;
    always @(posedge clk or negedge rst_n)
        if (!rst_n) c <= 4'd7;
        else c <= c + 4'd1;
endmodule
""", 1, 1),
        ("the same, documented", """
module m (input wire clk, input wire rst_n);
    // INIT-ZERO: c starts at 7 because the counter is offset; suites that
    // assert about c must guard the zero state.
    reg [3:0] c;
    always @(posedge clk or negedge rst_n)
        if (!rst_n) c <= 4'd7;
        else c <= c + 4'd1;
endmodule
""", 1, 0),
        ("a named constant that happens to be zero is still non-zero TEXT", """
module m (input wire clk, input wire rst_n);
    localparam IDLE = 2'd0;
    reg [1:0] state;
    always @(posedge clk or negedge rst_n)
        if (!rst_n) state <= IDLE;
        else state <= state;
endmodule
""", 1, 1),
    ]
    bad = []
    with tempfile.TemporaryDirectory() as td:
        d = pathlib.Path(td) / "build" / "rtl"
        d.mkdir(parents=True)
        for name, text, want_rows, want_bad in cases:
            f = d / "m.sv"
            f.write_text(text)
            rows = nonzero_resets(text)
            notes = {m.group(1) for m in NOTE.finditer(text)}
            got_bad = sum(1 for r, _ in rows if r not in notes)
            ok = len(rows) == want_rows and got_bad == want_bad
            print(f"  {'ok  ' if ok else 'FAIL'} {name}: {len(rows)} row(s), "
                  f"{got_bad} undocumented (want {want_rows}, {want_bad})")
            if not ok:
                bad.append(name)

        (d / "m.sv").write_text("module m; endmodule\n")
        rc = scan(pathlib.Path(td).parent.parent
                  if False else pathlib.Path(td).parent.parent)
        # scan() takes the root containing build/rtl
        rc = scan(pathlib.Path(td))
        print(f"  {'ok  ' if rc else 'FAIL'} a tree with no reset blocks fails "
              f"rather than passing silently: exit {rc}")
        if rc == 0:
            bad.append("a tree with nothing to examine passed as clean")

    for b in bad:
        print(f"::error::init_zero_scan self-test: {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else scan(r))
