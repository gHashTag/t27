#!/usr/bin/env python3
"""Cross-target bit-exactness for the GF-T primitives the trainer is built on.

`smul` and `sadd` (the exact functions the microsequencer's shared datapath uses)
must compute IDENTICALLY across t27's backends. verify_emit_bitexact already proves
Verilog == the independent Python GF-T model over a full training run; this proves
C == model and Rust == model on the same random operands -- closing the
"one spec -> any target, bit-exact" claim across {Verilog, C, Rust, model}.

Self-contained. Locally it SKIPs (exit 0) when t27c, a C compiler or rustc is
absent, because a contributor without rustc should not be blocked. In CI that
tolerance is wrong: the workflow builds t27c itself and the runner ships cc and
rustc, so a skip there means the environment broke, and exit 0 makes "proved"
indistinguishable from "never ran". Pass --require to turn every skip into a
failure. A real cross-target divergence exits 1 in both modes.

    python3 tools/verify_multitarget.py             # local, tolerant
    python3 tools/verify_multitarget.py --require   # CI, asserts it actually ran
"""
import os, sys, shutil, subprocess, tempfile, importlib.util, random

def _build(cmd, cwd, what):
    """Run a compiler and, if it fails, print what IT said before giving up.

    Every caller below used to test `.returncode` on a capture_output=True run and
    discard the message. That is how a missing brace in a spec came to be reported
    as "the C backend failed to build" for four days: the compiler named the file,
    the function, the line and the token, and the wrapper threw it away. A
    diagnostic that names the wrong subsystem costs more than none.
    """
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if r.returncode == 0:
        return True
    out = (r.stderr or r.stdout or "").strip().splitlines()
    print(f"  {what}: {os.path.basename(cmd[0])} exited {r.returncode}"
          + ("" if out else " with no message"))
    for line in out[:6]:
        print(f"      {line}")
    if len(out) > 6:
        print(f"      ... {len(out) - 6} more line(s)")
    return False


ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPECS = {"gft_smul": "smul", "gft_sadd": "sadd"}   # spec file -> top function
N = 600


REQUIRE = "--require" in sys.argv


def skip(msg):
    if REQUIRE:
        print(f"FAIL verify_multitarget: {msg}")
        print("  --require was given, so a missing prerequisite is a failure, not a skip.")
        print("  The CI job builds t27c and the runner ships cc and rustc; if one is")
        print("  absent the environment is broken and this check did not run.")
        sys.exit(1)
    print(f"SKIP verify_multitarget: {msg}")
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


def gen_pairs(g):
    random.seed(202)
    vals = [g.enc(round(random.uniform(-4, 4), 3)) for _ in range(48)]
    vals += [g.enc(0.0), g.enc(1.0), g.enc(-1.0), g.enc(2.0), g.enc(0.5), g.enc(-2.0), g.enc(0.25)]
    return [(random.choice(vals), random.choice(vals)) for _ in range(N)]


def py_ref(g, fn, pairs):
    f = getattr(g, fn)
    return [f(a, b) & 0xFFFFFFFF for a, b in pairs]


def run_c(t27c, spec, fn, pairs, wd):
    hdr = subprocess.run([t27c, "gen-c", f"specs/ternary/{spec}.t27"],
                         capture_output=True, text=True, cwd=ROOT).stdout
    if "GFTSMUL_H" not in hdr and "GFTSADD_H" not in hdr:
        return None
    open(os.path.join(wd, "mod.h"), "w").write(hdr)
    a = ",".join(str(x) for x, _ in pairs); b = ",".join(str(y) for _, y in pairs)
    # the spec's `test` blocks emit `assert_eq(...)` calls (undeclared in C); we
    # call the functions directly, so stub it out before including the module
    main = (f'#define assert_eq(x,y) ((void)0)\n#include "mod.h"\n#include <stdio.h>\nint main(){{'
            f'uint32_t A[]={{{a}}},B[]={{{b}}};int n={len(pairs)};'
            f'for(int i=0;i<n;i++)printf("%u\\n",(unsigned){fn}(A[i],B[i]));return 0;}}')
    open(os.path.join(wd, "main.c"), "w").write(main)
    if not _build(["cc", "-O2", "-o", os.path.join(wd, "cbin"), os.path.join(wd, "main.c")], wd, "C target"):
        return None
    out = subprocess.run([os.path.join(wd, "cbin")], capture_output=True, text=True).stdout
    return [int(x) for x in out.split()]


def run_rust(t27c, spec, fn, pairs, wd):
    src = subprocess.run([t27c, "gen-rust", f"specs/ternary/{spec}.t27"],
                         capture_output=True, text=True, cwd=ROOT).stdout
    if "fn " not in src:
        return None
    a = ",".join(str(x) for x, _ in pairs); b = ",".join(str(y) for _, y in pairs)
    src += (f'\nfn main(){{let a:[u32;{len(pairs)}]=[{a}];let b:[u32;{len(pairs)}]=[{b}];'
            f'for i in 0..a.len(){{println!("{{}}",{fn}(a[i],b[i]) as u32);}}}}\n')
    rs = os.path.join(wd, "m.rs"); open(rs, "w").write(src)
    if not _build(["rustc", "-A", "warnings", "-O", "-o", os.path.join(wd, "rbin"), rs], wd, "Rust target"):
        return None
    out = subprocess.run([os.path.join(wd, "rbin")], capture_output=True, text=True).stdout
    return [int(x) for x in out.split()]


def main():
    t27c = find_t27c()
    if not t27c:
        skip("t27c binary not found")
    if not shutil.which("cc"):
        skip("no C compiler (cc) on PATH")
    if not shutil.which("rustc"):
        skip("rustc not on PATH")
    g = load_gen()
    ok = True
    with tempfile.TemporaryDirectory() as wd:
        for spec, fn in SPECS.items():
            pairs = gen_pairs(g)
            ref = py_ref(g, fn, pairs)
            for tgt, runner in (("C", run_c), ("Rust", run_rust)):
                got = runner(t27c, spec, fn, pairs, wd)
                if got is None:
                    print(f"FAIL {spec}.{fn}: {tgt} backend failed to build/run"); ok = False; continue
                if len(got) != len(ref):
                    print(f"FAIL {spec}.{fn}: {tgt} produced {len(got)} of {len(ref)} outputs"); ok = False; continue
                mism = [(i, r, x) for i, (r, x) in enumerate(zip(ref, got)) if r != x]
                if mism:
                    i, r, x = mism[0]
                    print(f"FAIL {spec}.{fn}: {tgt} != model in {len(mism)}/{len(ref)}; "
                          f"first pair {pairs[i]} model={r} {tgt}={x}"); ok = False
                else:
                    print(f"OK {spec}.{fn}: {tgt} == Python model BIT-EXACT over {len(ref)} operand pairs")
    print("ALL TARGETS BIT-EXACT (Verilog[via emit gate] + C + Rust + model agree)" if ok else "CROSS-TARGET MISMATCH")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
