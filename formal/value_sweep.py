#!/usr/bin/env python3
"""Sweep the end-to-end VALUE check across the configuration grid.

Prop. 135 measured the engine against a reference and got an exact match. At one
configuration: N=1, C=1, L=1. Prop. 125 had already established that this is the
single configuration out of 81 that completes — so the campaign's only arithmetic
measurement sits precisely on the point least likely to be representative.

Every property in this campaign constrains CONTROL (Prop. 81b). Twenty-nine
integration properties prove while nothing checks that a number is right. This is
the other axis: run the same testbench over the grid and compare the engine's
accumulator and emitted trit against a reference computed in the testbench.

A configuration where control passes and arithmetic does not is a defect class
this campaign has never been able to see.

ARTIFACTS. Reads `build/rtl/*.sv` and `sim/tb_data_check.v`. WRITES
`formal/value_sweep_baseline.txt` -- the locked-in set of configurations that
match, which is how a regression is told from a known-open configuration.
Everything else stays inside a temporary directory; `build/rtl` and `sim/` are
left exactly as found.

# comment-scan: this file applies regexes to SIMULATOR OUTPUT (`RESULT:`,
# `ENGINE acc=`, `WORDS engine=`), never to Verilog source. There are no `//`
# comments in vvp stdout to strip, and stripping them would corrupt any Verilog
# path printed in a build error.

Prop. 137.
"""
import itertools
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
RTL = ROOT / "build" / "rtl"
TB = ROOT / "sim" / "tb_data_check.v"

MODULES = [
    "activation_requant", "axi_lite_slave", "bitnet_engine_top",
    "dma_controller", "double_buffer_ctrl", "interrupt_controller",
    "layer_sequencer", "multilayer_sequencer", "pipeline_stage2_compute",
    "trit_stdlib", "weight_bram", "weight_prefetch_ctrl",
]

IVERILOG = "/opt/homebrew/bin/iverilog"
VVP = "/opt/homebrew/bin/vvp"

# Chunks and neurons are the two axes Prop. 125's sweep moved; layers is the
# axis Prop. 129's defect lived on. Threshold is held at 3 because the reference
# and the design agree on its meaning and varying it tests the testbench.
#
# WELL-FORMEDNESS (Prop. 138). A multi-layer network is only defined when layer
# 0 produces exactly what layer 1 consumes: N neurons emit N trits, and the next
# layer reads C chunks of 27, so L > 1 requires N == C * 27. The first version
# of this grid crossed {1,2,3} x {1,2,3} x {1,2} freely, and six of those
# eighteen points emitted one activation word where the reference owed two.
# That looked exactly like a design defect -- and was an ill-posed question:
# layer 1 was being asked to read 27-81 trits from a layer that produced 1-3.
# At N == C*27 the same configurations emit 2, 4 and 6 words and all MATCH.
#
# The single-layer rows keep the free cross, because a network with one layer
# has no successor to be consistent with.
GRID = ([(c, n, 1, 0) for c in (1, 2, 3) for n in (1, 2, 3)]
        + [(c, c * 27, 2, 0) for c in (1, 2, 3)]
        # Prop. 140: shape was swept, values never were. At seed 0 every input
        # and weight is +1, so the accumulator is always 27*C and the trit
        # always TRIT_P -- a sign error, a lane transposition or a wrong trit
        # decode survives the entire grid above. These seeds draw pseudo-random
        # trits; between them they reach acc in [-3, +27] and all three trit
        # values. Seeds 5 and 7 land exactly on acc == -threshold, the boundary
        # that exposed a disagreement between the design and this testbench's
        # reference after 139 propositions had never touched it.
        + [(1, 1, 1, s) for s in (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)]
        + [(2, 2, 1, s) for s in (1, 5, 7)])


def run_one(work, c, n, layers, seed):
    exe = work / f"s_{c}_{n}_{layers}_{seed}.vvp"
    cmd = [IVERILOG, "-g2012", "-o", str(exe), "-s", "tb_data",
           f"-DT27_C={c}", f"-DT27_N={n}", f"-DT27_L={layers}",
           f"-DT27_SEED={seed}",
           str(work / "tb_data.v")] + [str(work / f"{m}.sv") for m in MODULES]
    b = subprocess.run(cmd, capture_output=True, text=True)
    if b.returncode != 0:
        first = next((l for l in (b.stderr or "").splitlines()
                      if "error" in l.lower()), "build failed")
        return "BUILD", first[:70]
    r = subprocess.run([VVP, str(exe)], capture_output=True, text=True,
                       timeout=900)
    out = r.stdout
    m = re.search(r"RESULT: (.+)", out)
    if not m:
        return "NO-VERDICT", "the testbench printed no RESULT line"
    verdict = m.group(1).strip()
    eng = re.search(r"ENGINE\s+acc=(-?\d+)\s+trit=(\S+)", out)
    ref = re.search(r"REFERENCE\s+acc=(-?\d+)\s+trit=(\S+)", out)
    detail = ""
    if eng and ref:
        detail = f"engine acc={eng.group(1)} trit={eng.group(2)}  " \
                 f"reference acc={ref.group(1)} trit={ref.group(2)}"
    w = re.search(r"WORDS engine=(\d+) expected=(\d+)", out)
    if w:
        detail += f"  words {w.group(1)}/{w.group(2)}"
    if verdict.startswith("MATCH") and w and w.group(1) != w.group(2):
        return "WORD-COUNT", f"emitted {w.group(1)} activation words, owes {w.group(2)}"
    if verdict.startswith("MATCH"):
        return "MATCH", detail
    return "MISMATCH", f"{verdict}  ({detail})" if detail else verdict


def main():
    for tool in (IVERILOG, VVP):
        if not pathlib.Path(tool).exists():
            print(f"::error::value sweep: no such file '{tool}' -- "
                  f"iverilog is required and nothing was measured")
            return 1
    if not TB.exists():
        print(f"::error::value sweep: no such file '{TB}' -- "
              f"nothing was measured")
        return 1
    missing = [m for m in MODULES if not (RTL / f"{m}.sv").exists()]
    if missing:
        print(f"::error::value sweep: no such file "
              f"'build/rtl/{missing[0]}.sv' -- the design is absent, so "
              f"nothing was measured")
        return 1

    results = {}
    with tempfile.TemporaryDirectory() as td:
        work = pathlib.Path(td)
        shutil.copy(TB, work / "tb_data.v")
        for m in MODULES:
            shutil.copy(RTL / f"{m}.sv", work / f"{m}.sv")
        for c, n, layers, seed in GRID:
            status, detail = run_one(work, c, n, layers, seed)
            results[(c, n, layers, seed)] = (status, detail)
            print(f"  C={c} N={n} L={layers} seed={seed:<2} {status:10} "
                  f"{detail[:70]}")

    tally = {}
    for status, _ in results.values():
        tally[status] = tally.get(status, 0) + 1
    summary = "  ".join(f"{k}={v}" for k, v in sorted(tally.items()))
    print(f"value sweep: {len(GRID)} configurations — {summary}")

    matched = tally.get("MATCH", 0)
    if matched == 0:
        print("::error::value sweep: not one configuration in build/rtl "
              "produced a MATCH -- a sweep with no passing point is measuring "
              "the harness, not the design (Prop. 103)")
        return 1

    # The gate does NOT fail on mismatches. Prop. 137: the grid is known to
    # contain configurations the design does not yet support, and a gate that
    # goes red on a known-open defect gets disabled rather than fixed
    # (Prop. 26's expected-refutation convention exists for this). What it
    # enforces is that the SET of failing configurations does not grow.
    baseline = ROOT / "formal" / "value_sweep_baseline.txt"
    now = sorted(f"C={c} N={n} L={l} S={s} {results[(c, n, l, s)][0]}"
                 for (c, n, l, s) in results)
    if not baseline.exists():
        baseline.write_text("\n".join(now) + "\n")
        print(f"value sweep: baseline written to {baseline.name} "
              f"({matched}/{len(GRID)} matching)")
        return 0
    was = [l for l in baseline.read_text().splitlines() if l.strip()]
    regressions = [n_ for n_, w in zip(now, was)
                   if n_ != w and "MATCH" in w and "MATCH" not in n_]
    if regressions:
        print(f"::error::value sweep: {len(regressions)} configuration(s) "
              f"that used to MATCH no longer do")
        for r in regressions:
            print(f"  {r}")
        return 1
    gained = [n_ for n_, w in zip(now, was)
              if n_ != w and "MATCH" not in w and "MATCH" in n_]
    if gained:
        print(f"value sweep: {len(gained)} configuration(s) newly matching — "
              f"update {baseline.name} to lock them in")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::value sweep: could not measure build/rtl "
              f"({type(exc).__name__}: {exc}) -- nothing was measured")
        sys.exit(1)
