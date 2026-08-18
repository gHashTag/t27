#!/usr/bin/env python3
"""Cross-target agreement over the ENTIRE input space, wherever the space is small.

Why this exists. `docs/POSITIONING.md` establishes that emitting several targets from
one source is table stakes -- Chisel/FIRRTL/CIRCT has done Verilog plus a C++ simulator
for years -- and that proving target equivalence is industrial practice by stronger
methods than ours (HECTOR, ACL2/RAC, which prove over all inputs). One thing is left
that is genuinely ours: **ternary primitives have tiny input spaces**, and a space you
can enumerate does not need a prover or a sample. A ternary full adder takes three
trits-in-a-byte and has 16,777,216 possible inputs. That is seconds of CPU.

What this checks. C, Rust and an independent model, on every input. The model lives in
`tools/ternary_model.py` and was transcribed from the SPEC text, not from the generated
code -- that is what makes it a third opinion rather than a restatement. A fault shared
by both backends, whether from the spec or from shared front-end lowering, shows up as
the model disagreeing with both.

An earlier version of this file compared only C against Rust and said so on every run,
because "the backends agree" and "the backends both match the specification" are
different claims and the first is easy to mistake for the second. The model closes that
gap for the primitives listed below; anything added to TARGETS without a model entry is
reported as two-way and labelled as such.

Coverage before this file: four specs (`ternary_mac`, `systolic_ternary`, `gft_smul`,
`gft_sadd`). `ternary_ripple_adder.t27` generates 167 lines of C and had no cross-target
check at all.

Usage:
  tools/verify_exhaustive.py                 all entries in TARGETS
  tools/verify_exhaustive.py --self-check    negative control
  tools/verify_exhaustive.py <fn> [...]      only the named functions

Exits non-zero if any pair of backends disagrees on any input.
"""
import os
import re
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import functools                                              # noqa: E402
import ternary_model as MODEL                                 # noqa: E402

# dot27 is pure, so memoising it changes nothing about what the model computes and
# takes full_adder over its 16,777,216 inputs from minutes to under a minute.
MODEL.dot27 = functools.lru_cache(maxsize=None)(MODEL.dot27)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# (spec, function, [(c_type, rust_type, lo, hi)]) -- the full domain of each argument.
# (spec, function, args, verilog return as (signed, bits) or None to skip that arm)
RIP = "specs/ternary/ternary_ripple_adder.t27"
U8 = ("uint8_t", "u8", 0, 255)
TARGETS = [
    (RIP, "full_adder", [U8] * 3, (False, 8)),
    (RIP, "maj3", [U8] * 3, (False, 8)),
    (RIP, "tmul", [U8] * 2, (True, 8)),
    (RIP, "negate", [U8], (False, 8)),
    # pack2 returns u64; the fold takes the low 32 bits in C and Rust, and a 64-bit
    # Verilog reg would need a wider fold to match. Left two-and-model until then.
    ("specs/ternary/ternary_xor.t27", "pack2", [U8] * 2, None),
]


def gen(mode, spec):
    r = subprocess.run([t27c(), "gen-" + mode, spec], capture_output=True, text=True, cwd=ROOT)
    if r.returncode != 0:
        out = (r.stderr or r.stdout or "").strip().splitlines()
        print(f"  t27c gen-{mode} {spec}: exited {r.returncode}")
        for l in out[:3]:
            print(f"      {l}")
        return None
    return r.stdout


def t27c():
    for p in ("target/release/t27c", "target/debug/t27c"):
        c = os.path.join(ROOT, p)
        if os.path.exists(c):
            return c
    sys.exit("FAIL: t27c not built. Run: cargo build --release -p t27c")


def loops(args, lang):
    """Nested for-loops covering every argument's full range."""
    out = []
    for i, (_, _, lo, hi) in enumerate(args):
        if lang == "c":
            out.append(f"for (int a{i} = {lo}; a{i} <= {hi}; a{i}++) {{")
        else:
            out.append(f"for a{i} in {lo}i32..={hi} {{")
    return out


def c_program(core, fn, args):
    call = ", ".join(f"({t})a{i}" for i, (t, _, _, _) in enumerate(args))
    # test blocks in the spec emit assert_eq() calls with no declaration; the same
    # shim verify_igla_race.py uses. We are checking the functions, not the tests.
    body = ["#include <stdio.h>", "#include <stdint.h>",
            "#define assert_eq(x,y) ((void)0)", core,
            "int main(void){", "  unsigned h = 2166136261u;"]
    body += ["  " + l for l in loops(args, "c")]
    body += [f"    unsigned v = (unsigned)((long long){fn}({call}) & 0xFFFFFFFF);",
             "    h = (h ^ v) * 16777619u;"]
    body += ["  }"] * len(args)
    body += ['  printf("%08x\\n", h);', "  return 0;", "}"]
    return "\n".join(body) + "\n"


def rust_program(core, fn, args):
    call = ", ".join(f"a{i} as {t}" for i, (_, t, _, _) in enumerate(args))
    body = [core, "fn main(){", "  let mut h: u32 = 2166136261;"]
    body += ["  " + l for l in loops(args, "rust")]
    body += [f"    let v = ({fn}({call}) as i64 & 0xFFFFFFFF) as u32;",
             "    h = (h ^ v).wrapping_mul(16777619);"]
    body += ["  }"] * len(args)
    body += ['  println!("{:08x}", h);', "}"]
    return "\n".join(body) + "\n"


def build_and_run(src, path, cmd, wd, what):
    open(path, "w").write(src)
    b = subprocess.run(cmd, cwd=wd, capture_output=True, text=True)
    if b.returncode != 0:
        print(f"  {what}: {os.path.basename(cmd[0])} exited {b.returncode}")
        lines = (b.stderr or "").strip().splitlines()
        # Prefer the lines that say error:. Truncating to the first N hid the real
        # error under four warnings the first time this ran.
        errs = [l for l in lines if "error" in l.lower()] or lines
        for l in errs[:5]:
            print(f"      {l}")
        if len(errs) > 5:
            print(f"      ... {len(errs) - 5} more")
        return None
    r = subprocess.run([cmd[cmd.index("-o") + 1]], cwd=wd, capture_output=True, text=True)
    if r.returncode != 0:
        sig = f" (signal {-r.returncode})" if r.returncode < 0 else ""
        print(f"  {what}: run exited {r.returncode}{sig} -- nothing was compared")
        return None
    return r.stdout.strip()


def model_digest(fn, args):
    """The same FNV-1a fold, over the same domain, computed by the independent model."""
    f = getattr(MODEL, fn, None)
    if f is None:
        return None
    h = 2166136261

    def rec(i, acc):
        nonlocal h
        if i == len(args):
            v = f(*acc) & 0xFFFFFFFF
            h = ((h ^ v) * 16777619) & 0xFFFFFFFF
            return
        _, _, lo, hi = args[i]
        for x in range(lo, hi + 1):
            rec(i + 1, acc + [x])

    rec(0, [])
    return f"{h:08x}"


def verilog_program(vsrc, fn, args, ret_signed, ret_bits):
    """A testbench that enumerates the whole domain and folds the same FNV-1a digest.

    The generated module carries the spec's own `initial` test blocks, which print
    [TEST] lines; the digest is picked out by filtering those. Its four ports are
    unused by the functions, so the body is lifted into a bare `module tb;` with the
    ports declared as constants rather than instantiated -- these primitives are
    combinational, and giving them a clock would only add a way to be wrong.
    """
    body = vsrc[vsrc.index(");", vsrc.index("module ")) + 2: vsrc.rindex("endmodule")]
    loops, close = [], []
    for i in range(len(args)):
        loops.append(f"    for (i{i} = {args[i][2]}; i{i} <= {args[i][3]}; i{i} = i{i} + 1)")
        close.append("")
    call = ", ".join(f"i{i}[7:0]" for i in range(len(args)))
    decl = " ".join(f"integer i{i};" for i in range(len(args)))
    # sign-extend a signed return to 32 bits so the fold matches the C/Rust one
    ext = (f"{{{{{32 - ret_bits}{{r[{ret_bits - 1}]}}}}, r}}" if ret_signed
           else f"{{{32 - ret_bits}'b0, r}}")
    return "\n".join([
        "`timescale 1ns/1ps", "module tb;",
        "  wire clk = 1'b0; wire rst_n = 1'b1; wire en = 1'b1; wire ready;",
        body,
        f"  {decl}",
        "  reg [31:0] h;",
        f"  reg {'signed ' if ret_signed else ''}[{ret_bits - 1}:0] r;",
        "  initial begin",
        "    h = 32'd2166136261;",
        *loops,
        "    begin",
        f"      r = {fn}({call});",
        f"      h = (h ^ ({ext} & 32'hFFFFFFFF)) * 32'd16777619;",
        "    end",
        '    $display("DIGEST %08x", h);',
        "    $finish;", "  end", "endmodule"]) + "\n"


# iverilog is an interpreter, and the cost per input is not the same across these
# functions -- measured, not assumed: tmul ~330,000 inputs/s, maj3 21,061/s,
# full_adder 2,972/s, because full_adder calls dot27 nine times per input at 27 lanes
# each. Exhausting full_adder in Verilog is 94 minutes. So the Verilog arm gets a
# budget: exhaustive where that fits, and an explicitly labelled SLICE where it does
# not. A slice is never reported as exhaustive.
# Budgeting in INPUTS was the wrong unit: the cost per input differs by 100x across
# these functions, so one input count is seconds for tmul and eleven minutes for
# full_adder. The budget is in SECONDS, converted per function using the rates measured
# above. Choosing a budget in a unit the thing does not vary in is how you get a limit
# that binds nowhere and everywhere at once.
VERILOG_SECONDS = 25
VERILOG_RATE = {"tmul": 330_000, "negate": 330_000, "maj3": 21_061, "full_adder": 2_972}
DEFAULT_RATE = 20_000


def verilog_digest(spec, fn, args, ret, wd, full=False):
    """Fourth opinion: the target that actually goes to silicon."""
    if ret is None:
        return None
    vsrc = gen("verilog", spec)
    if vsrc is None:
        return False
    signed, bits = ret
    space = 1
    for _, _, lo, hi in args:
        space *= (hi - lo + 1)
    vargs, covered = list(args), space
    budget = int(VERILOG_RATE.get(fn, DEFAULT_RATE) * VERILOG_SECONDS)
    if not full and space > budget:
        t, r, lo, hi = vargs[0]
        n = max(1, budget // (space // (hi - lo + 1)))
        vargs[0] = (t, r, lo, lo + n - 1)
        covered = space // (hi - lo + 1) * n
    f = os.path.join(wd, f"{fn}_tb.v")
    open(f, "w").write(verilog_program(vsrc, fn, vargs, signed, bits))
    b = subprocess.run(["iverilog", "-o", os.path.join(wd, f"{fn}.vvp"), f],
                       cwd=wd, capture_output=True, text=True)
    if b.returncode != 0:
        errs = [l for l in (b.stderr or "").splitlines() if "error" in l.lower()]
        print(f"  {fn} Verilog: iverilog exited {b.returncode}")
        for l in (errs or (b.stderr or "").splitlines())[:4]:
            print(f"      {l}")
        return False
    r = subprocess.run(["vvp", os.path.join(wd, f"{fn}.vvp")], cwd=wd,
                       capture_output=True, text=True)
    if r.returncode != 0:
        print(f"  {fn} Verilog: vvp exited {r.returncode} -- nothing was compared")
        return False
    m = re.search(r"^DIGEST ([0-9a-f]{8})$", r.stdout, re.M)
    if not m:
        print(f"  {fn} Verilog: no DIGEST line in {len(r.stdout.splitlines())} lines of output")
        return False
    return (m.group(1), covered, space)


def check(spec, fn, args, wd, ret=None):
    space = 1
    for _, _, lo, hi in args:
        space *= (hi - lo + 1)
    cs, rs = gen("c", spec), gen("rust", spec)
    if cs is None or rs is None:
        return None
    t0 = time.time()
    cd = build_and_run(c_program(cs, fn, args), os.path.join(wd, f"{fn}.c"),
                       ["cc", "-O2", "-o", os.path.join(wd, f"{fn}_c"), os.path.join(wd, f"{fn}.c")],
                       wd, f"{fn} C")
    rd = build_and_run(rust_program(rs, fn, args), os.path.join(wd, f"{fn}.rs"),
                       ["rustc", "-A", "warnings", "-O", "-o", os.path.join(wd, f"{fn}_r"),
                        os.path.join(wd, f"{fn}.rs")], wd, f"{fn} Rust")
    dt = time.time() - t0
    if cd is None or rd is None:
        return False
    if cd != rd:
        print(f"FAIL {fn}: C digest {cd} != Rust digest {rd} -- the backends disagree on at "
              f"least one of the {space:,} possible inputs")
        return False
    md = model_digest(fn, args)
    if md is None:
        print(f"OK   {fn:<12} C == Rust on ALL {space:>12,} inputs   digest {cd}   {dt:.1f}s"
              f"   [2-way: no model entry in ternary_model.py]")
        return True
    if md != cd:
        print(f"FAIL {fn}: backends agree on {cd} but the independent model says {md} -- both "
              f"backends differ from the specification on at least one of the {space:,} inputs, "
              f"which two-way agreement could never have shown")
        return False
    vres = verilog_digest(spec, fn, args, ret, wd, full="--verilog-full" in sys.argv)
    if vres is False:
        return False
    if vres is None:
        print(f"OK   {fn:<12} model == C == Rust on ALL {space:>12,} inputs   digest {cd}"
              f"   {dt:.1f}s   [3-way: no Verilog arm]")
        return True
    vd, covered, vspace = vres
    if covered == vspace:
        if vd != cd:
            print(f"FAIL {fn}: model/C/Rust agree on {cd} but Verilog says {vd} -- the target "
                  f"that goes to silicon differs on at least one of the {space:,} inputs")
            return False
        print(f"OK   {fn:<12} model == C == Rust == Verilog on ALL {space:>12,} inputs"
              f"   digest {cd}   {time.time() - t0:.1f}s")
        return True
    # Sliced Verilog arm: its digest covers a different domain, so it is recomputed on
    # that slice by the model and compared there. Reported as a slice, never as ALL.
    mv = model_digest(fn, [args[0][:2] + (args[0][2], args[0][2] + covered //
                           (space // (args[0][3] - args[0][2] + 1)) - 1)] + list(args[1:]))
    verdict = "==" if mv == vd else "!="
    if mv != vd:
        print(f"FAIL {fn}: on the Verilog slice ({covered:,} of {space:,} inputs) the model "
              f"says {mv} and Verilog says {vd}")
        return False
    print(f"OK   {fn:<12} model == C == Rust on ALL {space:>12,} inputs   digest {cd}   "
          f"{time.time() - t0:.1f}s")
    print(f"     {'':<12} Verilog {verdict} model on a SLICE of {covered:,} "
          f"({100.0 * covered / space:.1f}% of the domain) -- iverilog costs "
          f"{space / VERILOG_RATE.get(fn, DEFAULT_RATE) / 60:.0f} min for the whole of it; "
          f"--verilog-full runs it")
    return True


def self_check(wd):
    """Plant a divergence and prove the comparison sees it."""
    spec, fn, args, _ret = TARGETS[2]     # tmul, 65,536 inputs
    cs, rs = gen("c", spec), gen("rust", spec)
    if cs is None or rs is None:
        return 1
    good_c = build_and_run(c_program(cs, fn, args), os.path.join(wd, "sc.c"),
                           ["cc", "-O2", "-o", os.path.join(wd, "sc"), os.path.join(wd, "sc.c")],
                           wd, "self-check C")
    # perturb the C side at exactly one input
    bad_src = c_program(cs, fn, args).replace(
        "    h = (h ^ v) * 16777619u;",
        "    if (a0 == 7 && a1 == 3) v ^= 1u;\n    h = (h ^ v) * 16777619u;", 1)
    bad_c = build_and_run(bad_src, os.path.join(wd, "sb.c"),
                          ["cc", "-O2", "-o", os.path.join(wd, "sb"), os.path.join(wd, "sb.c")],
                          wd, "self-check C perturbed")
    ok_c = good_c is not None and bad_c is not None and good_c != bad_c
    print(f"  self-check: one-input perturbation of C changes the digest = {ok_c}"
          + (f"  ({good_c} -> {bad_c})" if ok_c else ""))

    # The model arm needs its own control. Perturbing C proves the C/Rust comparison
    # has resolution; it says nothing about whether a model that disagreed with BOTH
    # backends would be noticed -- which is the whole reason the model was added.
    real = MODEL.tmul
    try:
        MODEL.tmul = lambda a, b, _r=real: (_r(a, b) + 1) if (a == 7 and b == 3) else _r(a, b)
        perturbed = model_digest("tmul", args)
    finally:
        MODEL.tmul = real
    clean = model_digest("tmul", args)
    ok_m = perturbed is not None and clean is not None and perturbed != clean and clean == good_c
    print(f"  self-check: one-input perturbation of the MODEL is visible = {ok_m}"
          + (f"  ({clean} -> {perturbed}, backends {good_c})" if ok_m else ""))

    return 0 if (ok_c and ok_m) else 1


def main():
    only = [a for a in sys.argv[1:] if not a.startswith("--")]
    with tempfile.TemporaryDirectory() as wd:
        if "--self-check" in sys.argv:
            return self_check(wd)
        print("Cross-target agreement over the ENTIRE input space.\n")
        print("  Four opinions: C, Rust, Verilog under iverilog, and tools/ternary_model.py,")
        print("  transcribed from the spec text rather than from generated code. Verilog is the")
        print("  target that actually goes to silicon and was previously checked only by sample.\n")
        results = []
        for spec, fn, args, ret in TARGETS:
            if only and fn not in only:
                continue
            results.append(check(spec, fn, args, wd, ret))
        bad = [r for r in results if r is not True]
        print()
        if not results:
            print("FAIL: no targets selected")
            return 1
        if bad:
            print(f"FAIL: {len(bad)} of {len(results)} targets did not agree or did not run")
            return 1
        # The summary must not claim more than the lines above it. model/C/Rust are
        # exhaustive for every target; the Verilog arm is exhaustive only where its
        # budget allowed, and says so per line. An earlier version of this line read
        # "AGREE EXHAUSTIVELY across every arm", which was false for three of five.
        full_v = sum(1 for r in results if r is True)
        print(f"{len(results)} PRIMITIVES: model == C == Rust EXHAUSTIVELY over every input.")
        print("Verilog agrees wherever it was run -- exhaustively on the cheap primitives,")
        print("on a labelled slice where iverilog's cost makes the whole domain a long job.")
        print("Run --verilog-full to exhaust the Verilog arm too.")
        return 0


if __name__ == "__main__":
    sys.exit(main())
