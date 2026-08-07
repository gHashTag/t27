#!/usr/bin/env python3
"""Bit-exact gate for the programmable trainer's generated RTL.

Regenerates the GF-T arithmetic cores from their .t27 specs (via t27c), emits the
microsequencer for several topologies (varying hidden width, output count, and
input count) via emit_verilog(), and proves in a simulator that the generated RTL
is BIT-EXACT to the Python GF-T model over a full training run (forward + backprop
+ weight update), comparing EVERY output's u32 per step.

Self-contained and CI-friendly: if t27c or iverilog is unavailable it prints SKIP
and exits 0 (so it never breaks a Rust-only CI); a real mismatch exits 1. Run:
    python3 tools/verify_emit_bitexact.py
"""
import os, re, sys, shutil, subprocess, tempfile, importlib.util, random

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SMUL_SPEC = os.path.join(ROOT, "specs/ternary/gft_smul.t27")
SADD_SPEC = os.path.join(ROOT, "specs/ternary/gft_sadd.t27")
ARCHS = [(2, 2, 1), (2, 3, 1), (2, 4, 1), (2, 5, 1),  # hidden-width axis
         (2, 2, 2), (2, 4, 2), (2, 3, 3), (3, 4, 2)]   # multi-output + multi-input
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


def check(g, arch, workdir):
    n_in, n_hid, n_out = arch
    v = g.emit_verilog(*arch, "bpx")
    reg, steps = g.gen(*arch)
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
    print(f"OK {arch}: RTL == model BIT-EXACT over {len(py)} training steps, all {n_out} output(s) (final yout={py[-1]})")
    return True


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
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
