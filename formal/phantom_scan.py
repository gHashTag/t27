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

# comment-scan: matches yosys WARNING OUTPUT, not Verilog source. A `//`
# comment cannot appear in the warning stream it reads.

# props file -> (wrapper top, DUT sources it needs)
SUITES = [
    ("interrupt_controller_props.sv", "irq_props", ["interrupt_controller"]),
    ("axi_lite_slave_props.sv", "axi_props", ["axi_lite_slave"]),
    # The 4th field is extra sources the wrapper instantiates. dma_props holds
    # the AXI4 read-slave model as of Wave 618 (Prop. 70); this scan was the
    # third call site to break on that, and the third to break correctly -- it
    # reported an elaboration failure rather than a clean bill of health.
    ("dma_controller_props.sv", "dma_props", ["dma_controller"],
     ["formal/axi4_read_slave_model.sv"]),
    ("layer_sequencer_props.sv", "ls_props", ["layer_sequencer"]),
    ("double_buffer_props.sv", "db_props", ["double_buffer_ctrl"]),
    ("weight_bram_props.sv", "wb_props", ["weight_bram"]),
    ("pipeline_stage2_props.sv", "ps2_props",
     ["pipeline_stage2_compute", "trit_stdlib"]),
    ("trit_stdlib_props.sv", "dot_props", ["trit_stdlib"]),
    # Wave 634: the algebra. Each theorem elaborated so a property referencing a
    # signal that does not exist fails the build rather than proving against an
    # undriven phantom wire (Prop. 62).
    ("trit_algebra_props.sv", "not_props", ["trit_stdlib"]),
    ("trit_algebra_props.sv", "and_props", ["trit_stdlib"]),
    ("trit_algebra_props.sv", "or_props", ["trit_stdlib"]),
    ("trit_algebra_props.sv", "lattice_props", ["trit_stdlib"]),
    ("trit_algebra_props.sv", "mul_props", ["trit_stdlib"]),
    ("trit_algebra_props.sv", "cmp_props", ["trit_stdlib"]),
    ("trit_algebra_props.sv", "add3_props", ["trit_stdlib"]),
    ("trit_algebra_props.sv", "half_adder_props", ["trit_stdlib"]),
    ("trit_algebra_props.sv", "full_adder_props", ["trit_stdlib"]),
    # Wave 636: the composition proof. Its adder is an abstraction defined in
    # the property file itself rather than a module from the bundle, so a
    # mistyped port here would wire nothing and prove nothing.
    ("trit_algebra_props.sv", "add3_abstract", ["trit_stdlib"]),
    ("weight_prefetch_props.sv", "wp_props", ["weight_prefetch_ctrl"]),
    ("witnesses.sv", None, ["interrupt_controller", "axi_lite_slave", "dma_controller",
                            "layer_sequencer", "weight_prefetch_ctrl"]),
    # Prop. 176: max_size_props and zero_size_props were the only property
    # files in formal/ that no phantom scan reached. They are also the suites
    # Prop. 69 found ungated for many waves -- the same two files have now been
    # missed by two different kinds of coverage check, which is what a file
    # nobody thinks of looks like from the outside.
    ("max_size_props.sv", "ms_prefetch", ["weight_prefetch_ctrl"]),
    ("max_size_props.sv", "ms_dma", ["dma_controller"]),
    ("zero_size_props.sv", "zs_multilayer", ["multilayer_sequencer"]),
    ("zero_size_props.sv", "zs_dma", ["dma_controller"]),
    ("zero_size_props.sv", "zs_prefetch", ["weight_prefetch_ctrl"]),
    ("zero_size_props.sv", "zs_layer", ["layer_sequencer"]),
]

# The bit index is optional, and omitting it was a real hole. Wave 637c:
# yosys words the undriven-wire warning differently by WIDTH --
#
#   1-bit    Warning: Wire zz.\fv_ghost is used but has no driver.
#   n-bit    Warning: Wire zz.\fv_ghost [3] is used but has no driver.
#
# The original pattern ran `([\w.\\]+) is used`, and that character class cannot
# cross the space or the brackets, so EVERY multi-bit undriven wire went
# unmatched. This gate exists for exactly one defect -- Prop. 62, where a
# property proved against an undriven wire for four waves -- and it was catching
# that defect only at width 1. The Prop. 62 case happened to be one bit, because
# yosys implicitly declares a hierarchical reference as a single bit, which is
# why the gate looked like it worked.
PHANTOM = re.compile(r"Identifier `\\([^']+)' is implicitly declared|"
                     r"Wire ([\w.\\]+)(?:\s*\[\d+\])? is used but has no driver")


def scan(root, props, top, duts, extra=()):
    srcs = " ".join([f"build/rtl/{d}.sv" for d in duts] + list(extra))
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
    # Prop. 179: SUITES is hand-maintained, and Prop. 176 found two property
    # files that had been absent from it -- the same two that were absent from a
    # CI job for many waves. A justified omission and a forgotten one look
    # identical from inside the list, so every formal/*.sv must now be either
    # covered or explicitly excused.
    EXCUSED = {
        "assume_liveness_check.sv":
            "a self-contained canary with no DUT: it asserts something false "
            "under an unsatisfiable assumption, so it has no design signals to "
            "be phantoms of.",
        "axi4_read_slave_model.sv":
            "an environment model, not a property file. It is instantiated BY "
            "dma_props, and is scanned there as an extra source.",
        "witnesses.sv":
            "scanned via its own entries below rather than as a standalone "
            "wrapper; listed here so the coverage check can see it accounted for.",
    }
    listed = {props for props, *_ in SUITES}
    unaccounted = sorted(
        f.name for f in (root / "formal").glob("*.sv")
        if f.name not in listed and f.name not in EXCUSED
    )
    if unaccounted:
        for f in unaccounted:
            print(f"::error::formal/{f} is in neither SUITES nor EXCUSED -- a "
                  f"property file nothing phantom-scans can assert about a wire "
                  f"that does not exist and prove without reading the design")
        return 1

    bad, n = [], 0
    for props, top, duts, *rest in SUITES:
        extra = rest[0] if rest else ()
        if not (root / "formal" / props).exists():
            print(f"::error::phantom_scan: formal/{props} is missing from the suite list")
            return 1
        n += 1
        bad += scan(root, props, top, duts, extra)
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
    # By NAME, not by position. Wave 640: this used rindex("endmodule"), the
    # exact shape that broke the liveness probes in Prop. 95a once a wave
    # appended modules to a property file. dma_controller_props.sv has one
    # module today, so it worked -- which is how the same defect stayed live in
    # a sibling file twice already. A self-test that silently starts injecting
    # into the wrong module stops testing without saying so.
    _m = re.search(r"^module\s+dma_props\b.*?^endmodule", src, re.M | re.S)
    if _m is None:
        print("::error::phantom_scan self-test: module dma_props not found in "
              f"{victim} -- the injection would land in the wrong module")
        return 1
    i = _m.start() + _m.group(0).rindex("endmodule")

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
        # Wave 637c. Every case above injects a ONE-BIT phantom: yosys declares
        # an unknown identifier as a single bit, so all four exercised the only
        # warning form the pattern matched. Multi-bit undriven wires are worded
        # differently -- `Wire m.\x [3] is used but has no driver` -- and were
        # missed entirely, at every width above 1, by the gate whose sole
        # purpose is catching an undriven wire a property proves against. The
        # self-test could not see the hole because it never opened one.
        ("an undriven MULTI-BIT wire",
         src[:i] + "    wire [11:0] fv_ghost_mb;\n"
                   "    always @(posedge clk) if (rst_n) a_x: assert (fv_ghost_mb == 12'd0);\n"
         + src[i:], 1),
        ("an undriven one-bit wire, the width the pattern already handled",
         src[:i] + "    wire fv_ghost_1b;\n"
                   "    always @(posedge clk) if (rst_n) a_x: assert (fv_ghost_1b == 1'b0);\n"
         + src[i:], 1),
    ]
    bad = []
    try:
        for name, text, want in cases:
            open(victim, "w").write(text)
            got = 1 if scan(root, "dma_controller_props.sv", "dma_props",
                            ["dma_controller"],
                            ["formal/axi4_read_slave_model.sv"]) else 0
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
