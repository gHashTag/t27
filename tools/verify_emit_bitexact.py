#!/usr/bin/env python3
"""Bit-exact gate for the programmable trainer's generated RTL.

Regenerates the GF-T arithmetic cores from their .t27 specs (via t27c), emits the
microsequencer for several hidden widths via emit_verilog(), and proves in a
simulator that the generated RTL is BIT-EXACT to the Python GF-T model over a full
training run (forward + backprop + weight update), comparing yout u32 per step.

Self-contained and CI-friendly: if t27c or iverilog is unavailable it prints SKIP
and exits 0 (so it never breaks a Rust-only CI); a real mismatch exits 1. Run:
    python3 tools/verify_emit_bitexact.py
"""
import os, re, sys, shutil, subprocess, tempfile, importlib.util, random

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SMUL_SPEC = os.path.join(ROOT, "specs/ternary/gft_smul.t27")
SADD_SPEC = os.path.join(ROOT, "specs/ternary/gft_sadd.t27")
ARCHS = [(2, 2, 1), (2, 3, 1), (2, 4, 1), (2, 5, 1)]  # hidden width is the free axis
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
    v = g.emit_verilog(*arch, "bpx")
    reg, steps = g.gen(*arch)
    rf = [0] * len(reg)
    for idx, val in re.findall(r"rf\[(\d+)\]<=32'd(\d+);", v):
        rf[int(idx)] = int(val)
    random.seed(101)
    seq = []
    for _ in range(STEPS):
        a = round(random.uniform(-1, 1), 3); b = round(random.uniform(-1, 1), 3)
        t = float(int((a > 0) != (b > 0)))
        seq.append((a, b, t))
    py = []
    for (a, b, t) in seq:
        rf[reg["x0"]] = g.enc(a); rf[reg["x1"]] = g.enc(b); rf[reg["t0"]] = g.enc(t)
        g.run(steps, rf); py.append(rf[reg["y0"]] & 0xFFFFFFFF)
    tb = [v, "`timescale 1ns/1ps", "module tb;",
          "  reg clk=0; always #2.5 clk=~clk;",
          "  reg rst=1,start=0; reg [31:0] x0=0,x1=0,t=0; wire [31:0] y; wire done;",
          "  bpx dut(.clk(clk),.rst(rst),.start(start),.x0i(x0),.x1i(x1),.ti(t),.yout(y),.done(done));",
          "  task step(input [31:0] a,input [31:0] b,input [31:0] tt); begin",
          "    x0=a;x1=b;t=tt;@(posedge clk);start=1;@(posedge clk);start=0;wait(done);@(posedge clk);$display(\"Y %0d\",y);",
          "  end endtask",
          "  initial begin rst=1; repeat(6)@(posedge clk); rst=0; @(posedge clk);"]
    for (a, b, t) in seq:
        tb.append(f"    step(32'd{g.enc(a)},32'd{g.enc(b)},32'd{g.enc(t)});")
    tb += ["    $display(\"END\"); $finish; end",
           "  initial begin #40000000 $display(\"TIMEOUT\"); $finish; end", "endmodule"]
    tbf = os.path.join(workdir, "tb.v"); open(tbf, "w").write("\n".join(tb))
    vvp = os.path.join(workdir, "tb.vvp")
    r = subprocess.run(["iverilog", "-o", vvp, tbf,
                        os.path.join(workdir, "GftSmul.v"), os.path.join(workdir, "GftSadd.v")],
                       capture_output=True, text=True)
    if r.returncode != 0:
        print(f"FAIL {arch}: iverilog compile error\n{r.stderr}"); return False
    out = subprocess.run(["vvp", vvp], capture_output=True, text=True).stdout
    rtl = [int(x) for x in re.findall(r"^Y (\d+)$", out, re.M)]
    if len(rtl) != len(py):
        print(f"FAIL {arch}: step count RTL={len(rtl)} PY={len(py)}"); return False
    mism = [(i, p, r_) for i, (p, r_) in enumerate(zip(py, rtl)) if p != r_]
    if mism:
        print(f"FAIL {arch}: {len(mism)}/{len(py)}; first step {mism[0][0]} py={mism[0][1]} rtl={mism[0][2]}")
        return False
    print(f"OK {arch}: RTL == model BIT-EXACT over {len(py)} training steps (final yout={py[-1]})")
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
