#!/usr/bin/env python3
"""Gate 15: a proof under assumptions is worthless if no trace satisfies them.

Every other gate in this directory reads text. This one runs the solver, because
the failure it catches is invisible in source: yosys reports "proof succeeded"
for an UNSATISFIABLE assumption set, with no diagnostic. Every property in the
run passes. So does `assert (1'b0)`.

Wave 666 hit this twice in one session. Both experiments -- `weight_words != 0`
and `num_layers == 1` -- were written as `always @(posedge clk) if (rst_n)
assume (R == k)` with k nonzero. Under `-set-init-zero` every register is zero
at t=0, so the assumption is contradicted at the first cycle rst_n holds, no
trace satisfies it, and the solver vacuously proves whatever you asked. The
first experiment was read as "root cause confirmed" for a full minute.

The check: inject `assert (1'b0)` and require it to REFUTE. If a literally false
assertion proves, the trace set is empty and every result from that
configuration is meaningless.

This gate is the reason the campaign can now state that its 30 integration
properties are non-vacuous. Before Wave 666 that was an assumption about
assumptions.
"""
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
RTL = ROOT / "build" / "rtl"

MODULES = [
    "bitnet_engine_top", "multilayer_sequencer", "layer_sequencer",
    "double_buffer_ctrl", "weight_bram", "pipeline_stage2_compute",
    "activation_requant", "weight_prefetch_ctrl", "interrupt_controller",
    "axi_lite_slave", "dma_controller", "trit_stdlib",
]

# The define sets the suite is actually run under. A configuration that is
# never exercised does not need to be live; one that gates a published claim
# does.
CONFIGS = [
    ("baseline", ["-DT27_FORMAL"]),
    ("deep", ["-DT27_FORMAL", "-DT27_FORMAL_DEEP"]),
    ("open", ["-DT27_FORMAL", "-DT27_FORMAL_OPEN"]),
]

# Anchor for the probe: a declaration that exists in the top module and is
# followed by module body, not ports. Matched exactly once or the gate fails
# rather than silently probing nothing -- the mistake this campaign has made
# more than any other.
ANCHOR = "    wire [15:0] neurons_per_layer = reg_neurons[15:0];"

PROBE = """
`ifdef T27_VACUITY_PROBE
    // If this PROVES, no trace satisfies the assumptions and every other
    // property in the run proved for the same empty reason.
    always @(posedge clk) if (rst_n) a_vacuity_probe: assert (1'b0);
`endif"""


def run(cfg_defines, work, probe):
    srcs = " ".join(str(work / f"{m}.sv") for m in MODULES)
    defines = " ".join(cfg_defines + (["-DT27_VACUITY_PROBE"] if probe else []))
    script = (
        f"read_verilog -sv -formal {defines} {srcs}; "
        "chparam -set DEPTH 4 weight_bram; "
        "prep -top bitnet_engine_top -flatten; memory_map; async2sync; "
        "chformal -lower; "
        "sat -verify -prove-asserts -seq 40 -set-init-zero -set-assumes"
    )
    r = subprocess.run(["yosys", "-q", "-p", script],
                       capture_output=True, text=True)
    return r.returncode


def main():
    missing = [m for m in MODULES if not (RTL / f"{m}.sv").exists()]
    # Prop. 103 shape two -- a decline not counted -- is why these are
    # failures rather than skips. A gate that exits 0 when the design is
    # absent reports success for having checked nothing, and this gate was
    # written to catch precisely that class of lie.
    if missing:
        print(f"vacuity gate: FAIL -- RTL not generated ({missing[0]}.sv absent); "
              "nothing was checked")
        return 1
    if not shutil.which("yosys"):
        print("vacuity gate: FAIL -- yosys not on PATH; nothing was checked")
        return 1

    problems = []
    with tempfile.TemporaryDirectory() as td:
        work = pathlib.Path(td)
        for m in MODULES:
            shutil.copy(RTL / f"{m}.sv", work / f"{m}.sv")

        top = work / "bitnet_engine_top.sv"
        text = top.read_text()
        n = text.count(ANCHOR)
        if n != 1:
            print(f"vacuity gate: FAIL -- probe anchor matched {n} times, not 1. "
                  "A probe that does not land tests nothing.")
            return 1
        top.write_text(text.replace(ANCHOR, ANCHOR + PROBE, 1))

        # The probe must be present in the text the solver reads.
        if "a_vacuity_probe" not in top.read_text():
            print("vacuity gate: FAIL -- probe not written")
            return 1

        for name, defines in CONFIGS:
            code = run(defines, work, probe=True)
            if code == 0:
                problems.append(
                    f"{name}: assert(1'b0) PROVED -- the assumption set is "
                    f"unsatisfiable and every property in this configuration "
                    f"passed vacuously")
            elif code != 1:
                problems.append(f"{name}: solver returned {code}, "
                                f"neither proof nor refutation")
            else:
                print(f"  live    {name}: assert(1'b0) refutes, traces exist")

    # Any assumption in the RTL is what makes this gate necessary; report the
    # count so a future wave adding one cannot claim it was always covered.
    def strip_comments(t):
        t = re.sub(r"/\*.*?\*/", "", t, flags=re.S)
        return re.sub(r"//[^\n]*", "", t)

    n_assume = sum(
        len(re.findall(r"\bassume\s*\(", strip_comments((RTL / f"{m}.sv").read_text())))
        for m in MODULES)

    if problems:
        print(f"vacuity gate: FAIL -- {len(problems)} vacuous configuration(s)")
        for p in problems:
            print(f"  {p}")
        return 1

    print(f"vacuity gate: {len(CONFIGS)} configurations live, "
          f"{n_assume} assumption(s) in RTL, 0 problems")
    return 0


if __name__ == "__main__":
    # Prop. 116: a step that dies with a bare traceback proves nothing about
    # whether it reads its subject -- it fails when starved and would fail
    # just as readily if it were simply broken. Diagnose the absence instead.
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"vacuity gate: FAIL -- could not run the emptiness probe "
              f"({type(exc).__name__}: {exc}); nothing was checked")
        sys.exit(1)
