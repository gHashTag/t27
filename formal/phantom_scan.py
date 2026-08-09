#!/usr/bin/env python3
"""Fail if a property references a signal that does not exist.

Wave 611. `a_addr_ahead_of_data` read `dut.word_index` -- an ordinary
hierarchical reference, and one this flow does not support. Yosys did not error:
it implicitly declared a new one-bit wire called `\\dut.word_index`, left it
undriven, and proved the property against it. It said so in two warnings nobody
was reading:

    Warning: Identifier `\\dut.word_index' is implicitly declared.
    Warning: Wire wp_props.\\dut.word_index is used but has no driver.

The property had proved for four waves without reading the design. Confirmed by
making `word_index` advance by two instead of one: still PROVED. Wave 610's
detection matrix had already measured it detecting nothing; this is why.

`identity_scan.py` cannot catch this -- it is a syntactic scan for bodies that
fold to constant true, and this body is a perfectly ordinary comparison. The
signal is what is fake, not the shape.

This gate compiles each property module and fails on those two warnings. It is
cheap (elaboration only, no proof) and it is the whole class, not one instance:
any misspelled port, any renamed signal, any hierarchical reference.

Usage:  python3 formal/phantom_scan.py
"""

import pathlib
import re
import subprocess
import sys

# props file -> (wrapper top, DUT sources it needs)
SUITES = [
    ("interrupt_controller_props.sv", "irq_props", ["interrupt_controller"]),
    ("axi_lite_slave_props.sv", "axi_props", ["axi_lite_slave"]),
    ("dma_controller_props.sv", "dma_props", ["dma_controller"]),
    ("layer_sequencer_props.sv", "ls_props", ["layer_sequencer"]),
    ("weight_prefetch_props.sv", "wp_props", ["weight_prefetch_ctrl"]),
    ("witnesses.sv", None, ["interrupt_controller", "axi_lite_slave", "dma_controller",
                            "layer_sequencer", "weight_prefetch_ctrl"]),
]

PHANTOM = re.compile(r"Identifier `\\([^']+)' is implicitly declared|"
                     r"Wire ([\w.\\]+) is used but has no driver")


def scan(root, props, top, duts):
    srcs = " ".join(f"build/rtl/{d}.sv" for d in duts)
    tops = f"prep -top {top} -flatten" if top else "hierarchy -check; proc"
    r = subprocess.run(
        ["yosys", "-p", f"read_verilog -sv -formal {srcs} formal/{props}; {tops}"],
        cwd=root, capture_output=True, text=True)
    out = r.stdout + r.stderr
    if r.returncode != 0:
        return [f"formal/{props}: did not elaborate: " +
                next((l for l in out.splitlines() if l.startswith("ERROR")), "?")[:80]]
    bad = []
    for line in out.splitlines():
        m = PHANTOM.search(line)
        if m:
            bad.append(f"formal/{props}: {line.strip()[:110]}")
    return bad


def main():
    root = pathlib.Path(__file__).resolve().parent.parent
    if not (root / "build" / "rtl").exists():
        print(f"::error::phantom_scan found no RTL under {root}/build/rtl -- "
              "emit the bundle before running this gate")
        return 1
    bad, n = [], 0
    for props, top, duts in SUITES:
        if not (root / "formal" / props).exists():
            print(f"::error::phantom_scan: formal/{props} is missing from the suite list")
            return 1
        n += 1
        bad += scan(root, props, top, duts)
    for b in bad:
        print(f"::error::{b} -- the property is asserting something about a wire "
              "that does not exist, so it proves without reading the design")
    print(f"phantom scan: {n} property modules, {len(bad)} phantom signals")
    return 1 if bad else 0


def self_test():
    """Inject each way a signal can be fake; every one must be caught.

    The gate is worth exactly what these cases are worth, so they ship with it
    rather than living in a scratch directory (Prop. 58e).
    """
    import shutil
    root = pathlib.Path(__file__).resolve().parent.parent
    victim = root / "formal" / "dma_controller_props.sv"
    backup = str(victim) + ".selftest.bak"
    shutil.copy(victim, backup)
    src = open(victim).read()
    i = src.rindex("endmodule")

    cases = [
        ("clean tree", src, 0),
        ("a hierarchical reference into the DUT",
         src[:i] + "    always @(posedge clk) if (rst_n) a_x: assert (dut.burst_count == 8'd0);\n"
         + src[i:], 1),
        ("a misspelled signal name",
         src[:i] + "    always @(posedge clk) if (rst_n) a_x: assert (arvalidd == 1'b0);\n"
         + src[i:], 1),
        ("a renamed port that no longer exists",
         src[:i] + "    always @(posedge clk) if (rst_n) a_x: assert (m_axi_arvalid == 1'b0);\n"
         + src[i:], 1),
    ]
    bad = []
    try:
        for name, text, want in cases:
            open(victim, "w").write(text)
            got = 1 if scan(root, "dma_controller_props.sv", "dma_props",
                            ["dma_controller"]) else 0
            print(f"  {'ok  ' if got == want else 'FAIL'} {name}  "
                  f"(caught={bool(got)}, want={bool(want)})")
            if got != want:
                bad.append(name)
    finally:
        shutil.move(backup, victim)
    for b in bad:
        print(f"::error::phantom_scan self-test: '{b}' was not caught")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(self_test() if "--self-test" in sys.argv else main())
