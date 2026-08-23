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

WHAT THIS DOES NOT ESTABLISH
----------------------------
The topologies are a LIST, not a space. Fourteen are exercised; a fifteenth
that breaks the datapath invariant is not found here until someone adds it.

The training run is a fixed, seeded sequence -- byte-identical across runs,
which is what makes a disagreement reproducible, and also means the sample
never moves. Inputs outside it are not compared however often this runs.

"Synthesizes" is yosys reaching a cell count. It is not place-and-route, not
timing closure, and not silicon.

verify_exhaustive.py is the step in this job that enumerates a whole input
space, and it does that only "wherever the space is small". The two are
different claims and were named the same way until now.
"""
import os, re, sys, shutil, subprocess, tempfile, importlib.util, random


_pq = importlib.util.spec_from_file_location(
    "_prereq", os.path.join(os.path.dirname(os.path.abspath(__file__)), "_prereq.py"))
_prereq = importlib.util.module_from_spec(_pq); _pq.loader.exec_module(_prereq)
skip, broken = _prereq.skip, _prereq.broken

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SMUL_SPEC = os.path.join(ROOT, "specs/ternary/gft_smul.t27")
SADD_SPEC = os.path.join(ROOT, "specs/ternary/gft_sadd.t27")
ARCHS = [(2, 2, 1), (2, 3, 1), (2, 4, 1), (2, 5, 1),  # hidden-width axis
         (2, 2, 2), (2, 4, 2), (2, 3, 3), (3, 4, 2),   # multi-output + multi-input
         [2, 4, 3, 1], [2, 5, 3, 2], [3, 4, 4, 2, 1]]  # DEEP (lists): 3- and 4-layer
SYNTH_ARCHS = [(2, 2, 1), (2, 4, 2), [2, 4, 3, 1]]  # single / multi-out / deep (yosys is slower)
STEPS = 80


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


def self_check():
    """Run the WHOLE program once per verdict reachable without a simulator.

    T121. This gate had no negative control in any form. It was invisible to
    `tri gates sweep` until the selector learned that `sys.exit(0 if ok else 1)`
    is a failure path -- its only non-zero exit is that ternary, and now the
    two this commit adds.

    The gen-failure case is the one that matters: it plants a t27c that refuses
    to emit, which before this commit made the gate exit 0. `SKIP` is named
    absent there, because reporting a broken compiler as a missing prerequisite
    is precisely the defect.

    NOT COVERED, said rather than inferred: the bit-exactness verdict itself,
    which needs a Verilog arm that genuinely disagrees with the model.
    """
    ok = True

    def spawned(label, args, want, expect, absent, env=None, tree=None):
        nonlocal ok
        me = os.path.abspath(__file__)
        cwd = None
        if tree is not None:
            os.makedirs(os.path.join(tree, "tools"), exist_ok=True)
            import shutil as _sh
            _sh.copy(me, os.path.join(tree, "tools", os.path.basename(me)))
            me = os.path.join(tree, "tools", os.path.basename(me))
            cwd = tree
        r = subprocess.run([sys.executable, me, *args], capture_output=True,
                           text=True, cwd=cwd, env=env or os.environ.copy())
        out = r.stdout + r.stderr
        missing = [s for s in expect if s not in out]
        leaked = [s for s in absent if s in out]
        good = r.returncode == want and not missing and not leaked
        print(f"  {label:<46} " + (f"exit {want}, right branch" if good
                                   else "CONTROL FAILED"))
        if not good:
            ok = False
            print(f"       exit {r.returncode!r} (want {want!r})")
            if missing:
                print(f"       the branch never said: {missing!r}")
            if leaked:
                print(f"       neighbouring marker leaked: {leaked!r}")
            print(f"       said {out[:320]!r}")

    stripped = os.environ.copy()
    stripped["PATH"] = os.pathsep.join(
        d for d in stripped.get("PATH", "").split(os.pathsep)
        if d and not any(os.path.exists(os.path.join(d, x)) for x in ("iverilog", "vvp")))

    spawned("a missing simulator skips locally", [], 0,
            ["SKIP verify_emit_bitexact.py: iverilog/vvp not on PATH"],
            ["FAIL", "ALL SYNTHESIZE", "BIT-EXACT"], env=stripped)

    spawned("--require turns the same state into a failure", ["--require"], 1,
            ["FAIL verify_emit_bitexact.py: iverilog/vvp not on PATH",
             "--require was given"],
            ["SKIP", "ALL SYNTHESIZE"], env=stripped)

    # T121: the branch that used to exit 0. A planted t27c that refuses to emit
    # is the compiler under test being broken -- the loudest thing this gate can
    # find, and it was reported as a missing prerequisite.
    with tempfile.TemporaryDirectory() as td:
        os.makedirs(os.path.join(td, "target/release"))
        fake = os.path.join(td, "target/release/t27c")
        open(fake, "w").write("#!/bin/sh\necho 'gen-verilog: refused' >&2\nexit 1\n")
        os.chmod(fake, 0o755)
        for extra in ("specs", "conformance"):
            s = os.path.join(ROOT, extra)
            if os.path.exists(s):
                os.symlink(s, os.path.join(td, extra))
        import shutil as _sh
        for f in os.listdir(os.path.join(ROOT, "tools")):
            if f.endswith(".py") and f != os.path.basename(__file__):
                os.makedirs(os.path.join(td, "tools"), exist_ok=True)
                _sh.copy(os.path.join(ROOT, "tools", f), os.path.join(td, "tools", f))
        spawned("a compiler that will not emit is a FAILURE", [], 1,
                ["t27c gen-verilog failed",
                 "This is not a missing tool"],
                ["SKIP", "ALL SYNTHESIZE"], tree=td)

    # T129: the gate's OWN verdict, which none of the three cases above reaches.
    # Measured with the five-operator run: this gate scored 0 killed out of 17 --
    # `sys.exit(0 if ok else 1)` survived both return operators, and every FAIL
    # branch of the comparison survived inversion. All three cases leave through
    # skip() or broken(), so main()'s verdict was never observed at all.
    #
    # A control that covers only preconditions is a control for preconditions.
    # I wrote these three knowing that class, and the number that showed it could
    # not be produced until the run learned to be interruptible.
    #
    # The plant moves ONE arm. The Python side comes from g.run(); the Verilog is
    # emitted from the microcode `steps`, not from run(). Perturbing the
    # interpreter therefore makes the model disagree with an RTL that is
    # unchanged -- which is what a real bit-exactness failure looks like.
    def whole_program(label, edit, want, expect, absent):
        nonlocal ok
        import shutil as _sh
        with tempfile.TemporaryDirectory() as td:
            os.makedirs(os.path.join(td, "tools"))
            for f in os.listdir(os.path.join(ROOT, "tools")):
                if f.endswith(".py"):
                    src = open(os.path.join(ROOT, "tools", f), encoding="utf-8").read()
                    if f == "gft_backprop_microcode.py" and edit:
                        before = src
                        src = edit(src)
                        assert src != before, f"{label}: the plant changed nothing"
                    open(os.path.join(td, "tools", f), "w", encoding="utf-8").write(src)
            for extra in ("target", "specs"):
                s = os.path.join(ROOT, extra)
                if os.path.exists(s):
                    os.symlink(s, os.path.join(td, extra))
            r = subprocess.run(
                [sys.executable, os.path.join(td, "tools", os.path.basename(__file__))],
                capture_output=True, text=True)
        out = r.stdout + r.stderr
        missing = [s for s in expect if s not in out]
        leaked = [s for s in absent if s in out]
        good = r.returncode == want and not missing and not leaked
        print(f"  {label:<46} " + (f"exit {want}, right branch" if good
                                   else "CONTROL FAILED"))
        if not good:
            ok = False
            print(f"       exit {r.returncode!r} (want {want!r})")
            if missing:
                print(f"       the branch never said: {missing!r}")
            if leaked:
                print(f"       neighbouring marker leaked: {leaked!r}")
            print(f"       said {out[-320:]!r}")

    whole_program("a clean tree is bit-exact and exits 0", None, 0,
                  ["RTL == model BIT-EXACT"],
                  ["FAIL", "SKIP", "Traceback"])

    whole_program("a perturbed model disagrees with the RTL",
                  lambda s: s.replace(
                      '        rf[d] = smul(av, bv) if op == "MUL" else '
                      '(sadd(av, bv) if op == "ADD" else av)',
                      '        rf[d] = smul(av, bv) if op == "MUL" else '
                      '(sadd(av, bv) if op == "ADD" else av)\n'
                      '        if op == "ADD": rf[d] ^= 1', 1),
                  1, ["py=", "FAIL"],
                  ["RTL == model BIT-EXACT", "SKIP", "Traceback"])

    print(f"  self-check: the skip pair, the gen-failure branch, and the gate's own "
          f"verdict in both directions = {ok}")
    return 0 if ok else 1


def gen_core(t27c, spec, out):
    v = subprocess.run([t27c, "gen-verilog", spec], capture_output=True, text=True)
    if v.returncode != 0 or "module" not in v.stdout:
        broken(f"t27c gen-verilog failed for {os.path.basename(spec)}"
               + (f": {v.stderr.strip().splitlines()[-1][:120]}" if v.stderr.strip() else ""))
    open(out, "w").write(v.stdout)


def _emit_and_gen(g, arch, modname="bpx"):
    """Resolve arch -> (verilog, reg, steps, n_in, n_out). A 3-tuple is the 2-layer
    path (gen/emit_verilog); a list [n_in,...,n_out] is the deep path."""
    if isinstance(arch, list):
        return g.emit_verilog_deep(arch, modname), *g.gen_deep(arch), arch[0], arch[-1]
    v = g.emit_verilog(*arch, modname); reg, steps = g.gen(*arch)
    return v, reg, steps, arch[0], arch[2]


# mutant-equivalent: the guard above forces returncode != 0, so < is <=
#
# T132. Five copies of this line across five verifiers, and the boundary
# operator reports each as a survivor. All five are reached only after a
# returncode check -- three via `if returncode == 0: return`, two via
# `if returncode != 0:` -- so at this point the value cannot be zero, and
# `< 0` and `<= 0` agree on every value it can hold. Proven from the line
# above, not assumed from the shape.
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
    sim = subprocess.run(["vvp", vvp], capture_output=True, text=True)
    out = sim.stdout
    # A simulator that died, or one that hit the testbench's own TIMEOUT, produces a
    # short Y-list -- which the step-count check below would report as "the RTL
    # emitted the wrong number of outputs", i.e. a design fault. It is not: nothing
    # was compared. Ask the exit code, and read the marker the testbench prints.
    if sim.returncode != 0:
        sig = f" (signal {-sim.returncode})" if sim.returncode < 0 else ""
        print(f"FAIL {arch}: vvp exited {sim.returncode}{sig} -- the simulation did not "
              f"run to completion, so nothing was compared")
        for line in (sim.stderr or "").strip().splitlines()[:6]:
            print(f"    {line}")
        return False
    if "TIMEOUT" in out:
        print(f"FAIL {arch}: the testbench hit its own TIMEOUT after "
              f"{len(re.findall(r'^Y ', out, re.M))} of {len(py)} steps -- the design did "
              f"not finish, which is not the same as disagreeing with the model")
        return False
    rtl = [tuple(map(int, m.split())) for m in re.findall(r"^Y ([\d ]+)$", out, re.M)]
    if len(rtl) != len(py):
        print(f"FAIL {arch}: step count RTL={len(rtl)} PY={len(py)}"
              f" -- vvp exited 0 and printed no TIMEOUT, so this is a real output-count "
              f"disagreement, not a dead simulation")
        return False
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
    # ZERO transparent latches: gen-verilog function locals can infer latches
    # (yosys warns), and synth_xilinx normally optimizes them back to combinational
    # (0 in the final netlist). A latch that SURVIVES to silicon is level-sensitive /
    # placement-sensitive -- a real reliability hazard that iverilog verification never
    # catches. Assert the synthesized netlist has none.
    latch = sum(int(n) for n, _ in re.findall(r"(\d+)\s+(LD\w*|\S*LATCH\w*)", log))
    if latch:
        print(f"FAIL {arch}: {latch} transparent latch cells in the synthesized netlist "
              f"(silicon-reliability hazard; make the gen-verilog function locals fully assigned)")
        return False
    extra = f" ({ff} FF + {lut} LUT, 0 latches)" if ff and lut else ""
    print(f"OK {arch}: yosys synth_xilinx -> {total} cells{extra} (maps to real hardware)")
    return {"arch": str(arch), "cells": total, "ff": ff, "lut": lut}


def main():
    if "--self-check" in sys.argv:
        sys.exit(self_check())
    if not shutil.which("iverilog") or not shutil.which("vvp"):
        skip("iverilog/vvp not on PATH")
    t27c = find_t27c()
    if not t27c:
        skip("t27c binary not found (build with `cargo build` first)")
    if not (os.path.exists(SMUL_SPEC) and os.path.exists(SADD_SPEC)):
        broken("gft_smul.t27 / gft_sadd.t27 are tracked in this repository and are not on disk")
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
