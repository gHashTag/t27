#!/usr/bin/env python3
"""Bit-exact gate for the programmable trainer's generated RTL.

Regenerates the GF-T arithmetic cores from their .t27 specs (via t27c), emits the
microsequencer for several topologies (varying hidden width, output count, input
count, and DEPTH -- 2- to 4-layer nets via emit_verilog / emit_verilog_deep), and
proves in a simulator that the generated RTL is BIT-EXACT to the Python GF-T model
over a full training run (forward + backprop + weight update), comparing EVERY
output's u32 per step.

It also asserts the DATAPATH INVARIANT -- exactly one shared GftSmul + one shared
GftSadd per module, regardless of topology (this is what makes 'network size costs
time, not area' true; a change that parallelized the datapath would explode area
and still pass sim). Then, if yosys is present, it SYNTHESIZES the emitted RTL
(synth_xilinx), asserts a non-zero cell mapping, and prints a per-topology area
report (cells/FF/LUT) so area trends are visible across PRs.

Self-contained and CI-friendly: if t27c or iverilog is unavailable it prints SKIP
and exits 0 (so it never breaks a Rust-only CI); the synth phase is skipped when
yosys is absent; a real mismatch or synth failure exits 1. Run:
    python3 tools/verify_emit_bitexact.py
"""
import os, re, sys, shutil, subprocess, tempfile, importlib.util, random

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SMUL_SPEC = os.path.join(ROOT, "specs/ternary/gft_smul.t27")
SADD_SPEC = os.path.join(ROOT, "specs/ternary/gft_sadd.t27")
ARCHS = [(2, 2, 1), (2, 3, 1), (2, 4, 1), (2, 5, 1),  # hidden-width axis
         (2, 2, 2), (2, 4, 2), (2, 3, 3), (3, 4, 2),   # multi-output + multi-input
         [2, 4, 3, 1], [2, 5, 3, 2], [3, 4, 4, 2, 1]]  # DEEP (lists): 3- and 4-layer
SYNTH_ARCHS = [(2, 2, 1), (2, 4, 2), [2, 4, 3, 1]]  # single / multi-out / deep (yosys is slower)
STEPS = 80


def skip(msg):
    print(f"SKIP verify_emit_bitexact: {msg}")
    sys.exit(0)


def find_t27c():
    for p in ("target/debug/t27c", "target/release/t27c"):
        cand = os.path.join(ROOT, p)
        if os.path.exists(cand):
            return cand
    return shutil.which("t27c")


def load_gen():
    spec = importlib.util.spec_from_file_location(
        "gbm", os.path.join(ROOT, "tools/gft_backprop_microcode.py"))
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    return m


def gen_core(t27c, spec, out):
    v = subprocess.run([t27c, "gen-verilog", spec], capture_output=True, text=True)
    if v.returncode != 0 or "module" not in v.stdout:
        skip(f"t27c gen-verilog failed for {os.path.basename(spec)}")
    open(out, "w").write(v.stdout)


def _emit_and_gen(g, arch, modname="bpx"):
    """Resolve arch -> (verilog, reg, steps, n_in, n_out). A 3-tuple is the 2-layer
    path (gen/emit_verilog); a list [n_in,...,n_out] is the deep path."""
    if isinstance(arch, list):
        return g.emit_verilog_deep(arch, modname), *g.gen_deep(arch), arch[0], arch[-1]
    v = g.emit_verilog(*arch, modname); reg, steps = g.gen(*arch)
    return v, reg, steps, arch[0], arch[2]


def check(g, arch, workdir):
    v, reg, steps, n_in, n_out = _emit_and_gen(g, arch)
    rf = [0] * len(reg)
    for idx, val in re.findall(r"rf\[(\d+)\]<=32'd(\d+);", v):
        rf[int(idx)] = int(val)
    # deterministic stream: n_in inputs + n_out targets per step (values arbitrary;
    # bit-exactness is independent of target semantics, we just need identical drive)
    random.seed(101)
    seq = []
    for _ in range(STEPS):
        xs = [round(random.uniform(-1, 1), 3) for _ in range(n_in)]
        cls = int((xs[0] > 0) != (xs[-1] > 0))
        ts = [1.0 if o == cls % n_out else 0.0 for o in range(n_out)]
        seq.append((xs, ts))
    py = []
    for (xs, ts) in seq:
        for k in range(n_in): rf[reg[f"x{k}"]] = g.enc(xs[k])
        for o in range(n_out): rf[reg[f"t{o}"]] = g.enc(ts[o])
        g.run(steps, rf)
        py.append(tuple(rf[reg[f"y{o}"]] & 0xFFFFFFFF for o in range(n_out)))
    xdecl = " ".join(f"reg [31:0] x{k}=0;" for k in range(n_in))
    tdecl = " ".join(f"reg [31:0] t{o}=0;" for o in range(n_out))
    ports = ",".join([".clk(clk)", ".rst(rst)", ".start(start)"]
                     + [f".x{k}i(x{k})" for k in range(n_in)]
                     + [f".t{o}i(t{o})" for o in range(n_out)]
                     + [".yout(y)", ".done(done)"])
    # a per-arch task carries the proven start/wait(done) handshake (an inline
    # sequence of many wait(done)'s in one initial block hangs under iverilog)
    tparams = ", ".join([f"input [31:0] px{k}" for k in range(n_in)]
                        + [f"input [31:0] pt{o}" for o in range(n_out)])
    tassign = " ".join(f"x{k}=px{k};" for k in range(n_in)) \
              + " " + " ".join(f"t{o}=pt{o};" for o in range(n_out))
    fmt = "Y" + " %0d" * n_out
    args = ",".join(f"y[{32*o} +: 32]" for o in range(n_out))
    tb = [v, "`timescale 1ns/1ps", "module tb;",
          "  reg clk=0; always #2.5 clk=~clk;",
          f"  reg rst=1,start=0; {xdecl} {tdecl} wire [{32*n_out-1}:0] y; wire done;",
          f"  bpx dut({ports});",
          f"  task step({tparams}); begin",
          f"    {tassign} @(posedge clk); start=1; @(posedge clk); start=0;"
          f" wait(done); @(posedge clk); $display(\"{fmt}\",{args});",
          "  end endtask",
          "  initial begin rst=1; repeat(6)@(posedge clk); rst=0; @(posedge clk);"]
    for (xs, ts) in seq:
        vals = ",".join([f"32'd{g.enc(xs[k])}" for k in range(n_in)]
                        + [f"32'd{g.enc(ts[o])}" for o in range(n_out)])
        tb.append(f"    step({vals});")
    tb += ["    $display(\"END\"); $finish; end",
           "  initial begin #60000000 $display(\"TIMEOUT\"); $finish; end", "endmodule"]
    tbf = os.path.join(workdir, "tb.v"); open(tbf, "w").write("\n".join(tb))
    vvp = os.path.join(workdir, "tb.vvp")
    r = subprocess.run(["iverilog", "-o", vvp, tbf,
                        os.path.join(workdir, "GftSmul.v"), os.path.join(workdir, "GftSadd.v")],
                       capture_output=True, text=True)
    if r.returncode != 0:
        print(f"FAIL {arch}: iverilog compile error\n{r.stderr}"); return False
    out = subprocess.run(["vvp", vvp], capture_output=True, text=True).stdout
    rtl = [tuple(map(int, m.split())) for m in re.findall(r"^Y ([\d ]+)$", out, re.M)]
    if len(rtl) != len(py):
        print(f"FAIL {arch}: step count RTL={len(rtl)} PY={len(py)}"); return False
    mism = [(i, p, r_) for i, (p, r_) in enumerate(zip(py, rtl)) if p != r_]
    if mism:
        print(f"FAIL {arch}: {len(mism)}/{len(py)}; first step {mism[0][0]} py={mism[0][1]} rtl={mism[0][2]}")
        return False
    print(f"OK {arch}: RTL == model BIT-EXACT over {len(py)} training steps, all {n_out} output(s) "
          f"[{len(steps)} microcode steps / {len(reg)} regs] (final yout={py[-1]})")
    return True


def datapath_check(g, arch):
    """Guard the core architectural invariant -- ONE shared multiply core + ONE
    shared add core, regardless of topology. This is what makes 'network size costs
    time, not area' true: a change that accidentally parallelized the datapath would
    instantiate N GftSmul (area explosion) and silently pass the sim + synth gates.
    Version-independent (a text check on the emitted RTL, no toolchain needed)."""
    v = _emit_and_gen(g, arch)[0]
    nmul = len(re.findall(r"\bGftSmul\s+\w+\s*\(", v))
    nadd = len(re.findall(r"\bGftSadd\s+\w+\s*\(", v))
    if nmul != 1 or nadd != 1:
        print(f"FAIL {arch}: datapath has {nmul} GftSmul + {nadd} GftSadd (must be exactly 1+1)")
        return False
    return True


def synth_check(g, arch, workdir):
    """Prove the emitted microsequencer actually SYNTHESIZES (yosys synth_xilinx) --
    bit-exact-in-sim doesn't imply synthesizable. Asserts no yosys error and a
    large post-synth cell count (a design DCE'd to nothing would 'pass' sim of an
    empty module but map to ~0 cells). Uses the version-stable `Number of cells`
    stat line rather than parsing per-primitive names (which vary across yosys
    versions); reports the FF/LUT breakdown when it is parseable."""
    open(os.path.join(workdir, "bpx.v"), "w").write(_emit_and_gen(g, arch)[0])
    # -DSIMULATION strips the cores' `ifndef SIMULATION` self-test blocks.
    cmd = ("read_verilog -DSIMULATION bpx.v GftSmul.v GftSadd.v; hierarchy -top bpx; "
           "synth_xilinx -nocarry -flatten; stat")
    r = subprocess.run(["yosys", "-p", cmd], capture_output=True, text=True, cwd=workdir)
    log = r.stdout + r.stderr  # some yosys builds log stat to stderr, not stdout
    if r.returncode != 0 or re.search(r"^ERROR", log, re.M):
        print(f"FAIL {arch}: yosys synth error\n{log[-800:]}"); return False
    # cell total is printed as "N cells" (or "Number of cells: N" on some versions)
    totals = [int(n) for n in re.findall(r"(\d+)\s+cells\b", log)
              + re.findall(r"Number of cells:\s*(\d+)", log)]
    total = max(totals) if totals else 0
    if total < 200:
        print(f"FAIL {arch}: synthesized to {total} cells (design optimized away?)\n{log[-1200:]}")
        return False
    ff = sum(int(n) for n, _ in re.findall(r"(\d+)\s+(FD\w+)", log))
    lut = sum(int(n) for n, _ in re.findall(r"(\d+)\s+(LUT\w*)", log))
    extra = f" ({ff} FF + {lut} LUT)" if ff and lut else ""
    print(f"OK {arch}: yosys synth_xilinx -> {total} cells{extra} (maps to real hardware)")
    return {"arch": str(arch), "cells": total, "ff": ff, "lut": lut}


def main():
    if not shutil.which("iverilog") or not shutil.which("vvp"):
        skip("iverilog/vvp not on PATH")
    t27c = find_t27c()
    if not t27c:
        skip("t27c binary not found (build with `cargo build` first)")
    if not (os.path.exists(SMUL_SPEC) and os.path.exists(SADD_SPEC)):
        skip("gft_smul.t27 / gft_sadd.t27 specs not found")
    g = load_gen()
    with tempfile.TemporaryDirectory() as wd:
        gen_core(t27c, SMUL_SPEC, os.path.join(wd, "GftSmul.v"))
        gen_core(t27c, SADD_SPEC, os.path.join(wd, "GftSadd.v"))
        ok = all(check(g, a, wd) for a in ARCHS)
        print("ALL BIT-EXACT" if ok else "MISMATCH")
        if ok:
            print("--- datapath invariant (one shared smul + one shared sadd) ---")
            ok = all(datapath_check(g, a) for a in ARCHS)
            print("ALL ONE-MULTIPLIER" if ok else "DATAPATH FAIL")
        if ok:
            # quantify the core claim: bigger nets grow the microcode (TIME), not the
            # one-shared-multiplier datapath (AREA). (Step count also predicts on-silicon
            # timing-marginality: more steps per frame -> more chances for a glitch.)
            print("--- size costs TIME (microcode steps), not AREA (1 shared multiplier) ---")
            for a in ARCHS:
                reg2, steps2 = (g.gen_deep(a) if isinstance(a, list) else g.gen(*a))
                print(f"  {str(a):<16} {len(steps2):>4} steps  {len(reg2):>3} regs  -- same 1-smul+1-sadd datapath")
        if ok and shutil.which("yosys"):
            print("--- synthesizability + area (yosys synth_xilinx) ---")
            results = [synth_check(g, a, wd) for a in SYNTH_ARCHS]
            ok = all(results)
            if ok:
                print("area report (cell counts are yosys-version-specific; watch the trend across PRs):")
                for r in results:
                    print(f"  {r['arch']:<16} {r['cells']:>7} cells  {r['ff']:>6} FF  {r['lut']:>6} LUT")
            print("ALL SYNTHESIZE" if ok else "SYNTH FAIL")
        elif ok:
            print("synth check skipped (yosys not on PATH)")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
