#!/usr/bin/env python3
"""Cross-target agreement over the ENTIRE input space, wherever the space is small.

Why this exists. `docs/POSITIONING.md` establishes that emitting several targets from
one source is table stakes -- Chisel/FIRRTL/CIRCT has done Verilog plus a C++ simulator
for years -- and that proving target equivalence is industrial practice by stronger
methods than ours (HECTOR, ACL2/RAC, which prove over all inputs). One thing is left
that is genuinely ours: **ternary primitives have tiny input spaces**, and a space you
can enumerate does not need a prover or a sample. A ternary full adder takes three
trits-in-a-byte and has 16,777,216 possible inputs. That is seconds of CPU.

What this checks and what it does NOT. It compares the C backend against the Rust
backend on every input. If both are wrong in the same way -- a bug in the spec, or in
shared front-end lowering -- this will not see it. That is a weaker statement than
`verify_igla_race.py` makes for `ternary_mul`, where an independent Python model is the
third opinion. The distinction is printed with every result, because "C == Rust on all
inputs" and "C == Rust == independent model on all inputs" are different claims and the
first is easy to mistake for the second.

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

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# (spec, function, [(c_type, rust_type, lo, hi)]) -- the full domain of each argument.
TARGETS = [
    ("specs/ternary/ternary_ripple_adder.t27", "full_adder",
     [("uint8_t", "u8", 0, 255)] * 3),
    ("specs/ternary/ternary_ripple_adder.t27", "maj3",
     [("uint8_t", "u8", 0, 255)] * 3),
    ("specs/ternary/ternary_ripple_adder.t27", "tmul",
     [("uint8_t", "u8", 0, 255)] * 2),
    ("specs/ternary/ternary_ripple_adder.t27", "negate",
     [("uint8_t", "u8", 0, 255)]),
    ("specs/ternary/ternary_xor.t27", "pack2",
     [("uint8_t", "u8", 0, 255)] * 2),
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


def check(spec, fn, args, wd):
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
    print(f"OK   {fn:<12} C == Rust on ALL {space:>12,} inputs   digest {cd}   {dt:.1f}s")
    return True


def self_check(wd):
    """Plant a divergence and prove the comparison sees it."""
    spec, fn, args = TARGETS[2]           # tmul, 65,536 inputs
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
    ok = good_c is not None and bad_c is not None and good_c != bad_c
    print(f"  self-check: one-input perturbation changes the digest = {ok}"
          + (f"  ({good_c} -> {bad_c})" if ok else ""))
    return 0 if ok else 1


def main():
    only = [a for a in sys.argv[1:] if not a.startswith("--")]
    with tempfile.TemporaryDirectory() as wd:
        if "--self-check" in sys.argv:
            return self_check(wd)
        print("Cross-target agreement over the ENTIRE input space.\n")
        print("  This compares C against Rust. It does NOT include an independent model, so a")
        print("  fault shared by both backends -- a spec bug, or shared front-end lowering --")
        print("  is invisible here. verify_igla_race.py carries the stronger form for")
        print("  ternary_mul, where a Python model is the third opinion.\n")
        results = []
        for spec, fn, args in TARGETS:
            if only and fn not in only:
                continue
            results.append(check(spec, fn, args, wd))
        bad = [r for r in results if r is not True]
        print()
        if not results:
            print("FAIL: no targets selected")
            return 1
        if bad:
            print(f"FAIL: {len(bad)} of {len(results)} targets did not agree or did not run")
            return 1
        print(f"ALL {len(results)} PRIMITIVES: C == Rust EXHAUSTIVELY (no sampling, no model)")
        return 0


if __name__ == "__main__":
    sys.exit(main())
