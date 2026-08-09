#!/usr/bin/env python3
"""Report which integration properties survive a larger bound.

Wave 583. Every engine property proves at `-seq 40` with `chparam DEPTH 4`.
Prop. 29a showed that a bounded proof establishes nothing about a property whose
counterexample lies beyond the bound: two modules "proved" an address never
wraps while both contained a wrap needing 4096 writes to reach.

So "proved" is not a property of the design, it is a property of the pair
(design, scale). This measures the curve. Usage:

    python3 formal/scale_probe.py 40 4          # aggregate verdict + timing
    python3 formal/scale_probe.py 40 4 --each   # isolate every property

`--each` neutralises all but one property per run, so a refutation is
attributable rather than merely present -- the lesson of Prop. 30d, where two
properties were reported together and only one discriminated.
"""

import re
import subprocess
import sys
import time

REST = ["multilayer_sequencer", "layer_sequencer", "double_buffer_ctrl", "weight_bram",
        "pipeline_stage2_compute", "activation_requant", "weight_prefetch_ctrl",
        "interrupt_controller", "axi_lite_slave", "dma_controller", "trit_stdlib"]

TOP = "build/rtl/bitnet_engine_top.sv"


def neutralise(src, keep):
    """Replace every assertion body except `keep` with 1'b1, matching parens."""
    out, i = [], 0
    while True:
        m = re.search(r"\b(a_[a-z0-9_]+): assert \(", src[i:])
        if not m:
            out.append(src[i:])
            break
        st = i + m.start()
        j = i + m.end() - 1
        d = 0
        while j < len(src):
            if src[j] == "(":
                d += 1
            elif src[j] == ")":
                d -= 1
                if d == 0:
                    break
            j += 1
        lab = m.group(1)
        out.append(src[i:st])
        out.append(f"{lab}: assert (1'b1)" if lab != keep else src[st:j + 1])
        i = j + 1
    return "".join(out)


def run(path, seq, depth, timeout):
    rest = " ".join(f"build/rtl/{m}.sv" for m in REST)
    cmd = (f"read_verilog -sv -formal -DT27_FORMAL {path} {rest}; "
           f"chparam -set DEPTH {depth} weight_bram; "
           "prep -top bitnet_engine_top -flatten; memory_map; async2sync; chformal -lower; "
           f"sat -verify -prove-asserts -seq {seq} -set-init-zero -set-assumes")
    t0 = time.time()
    try:
        r = subprocess.run(["yosys", "-q", "-p", cmd], capture_output=True,
                           text=True, timeout=timeout)
        return ("PROVED" if r.returncode == 0 else "REFUTED"), time.time() - t0
    except subprocess.TimeoutExpired:
        return f"timeout>{timeout}s", time.time() - t0


def main():
    seq = int(sys.argv[1]) if len(sys.argv) > 1 else 40
    depth = int(sys.argv[2]) if len(sys.argv) > 2 else 4
    each = "--each" in sys.argv
    timeout = 600
    src = open(TOP).read()

    if not each:
        v, dt = run(TOP, seq, depth, timeout)
        print(f"seq={seq} DEPTH={depth}  {v}  ({dt:.1f}s)")
        return 0 if v == "PROVED" else 1

    labels = sorted(set(re.findall(r"\b(a_[a-z0-9_]+): assert \(", src)))
    print(f"seq={seq} DEPTH={depth}, {len(labels)} properties, isolated", flush=True)
    bad = []
    for lab in labels:
        open("build/scale_iso.sv", "w").write(neutralise(src, lab))
        v, dt = run("build/scale_iso.sv", seq, depth, timeout)
        print(f"  {lab:34s} {v:>14s} {dt:7.1f}s", flush=True)
        if v != "PROVED":
            bad.append((lab, v))
    if bad:
        print("\nnot proved at this scale:")
        for lab, v in bad:
            print(f"  {lab}: {v}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
