#!/usr/bin/env python3
"""Cross-target bit-exactness for the GF-T primitives the trainer is built on.

`smul` and `sadd` (the exact functions the microsequencer's shared datapath uses)
must compute IDENTICALLY across t27's backends. verify_emit_bitexact already proves
Verilog == the independent Python GF-T model over a full training run; this proves
C == model and Rust == model on the same operands -- closing the "one spec -> any
target, bit-exact" claim across {Verilog, C, Rust, model}. Operands span BOTH the
moderate range and the EXTREME range (raw GF-T u32 over the full offset span 0..127,
both signs, saturation-adjacent) -- overflow/underflow/carry edges a [-4,4] sweep
never reaches (weights land here during training).

Self-contained + CI-friendly: SKIPs (exit 0) if t27c / a C compiler / rustc is
missing; a real cross-target divergence exits 1. Run:
    python3 tools/verify_multitarget.py
"""
import os, sys, shutil, subprocess, tempfile, importlib.util, random

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPECS = {"gft_smul": "smul", "gft_sadd": "sadd"}   # spec file -> top function
N = 600


def skip(msg):
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
    # moderate range (typical activations/weights)
    vals = [g.enc(round(random.uniform(-4, 4), 3)) for _ in range(40)]
    vals += [g.enc(0.0), g.enc(1.0), g.enc(-1.0), g.enc(2.0), g.enc(0.5), g.enc(-2.0), g.enc(0.25)]
    # EXTREME range: raw GF-T u32 across the full offset span (0..127) and both signs,
    # incl. saturation-adjacent offsets -- exercises overflow/underflow/carry edges that
    # a [-4,4]-only sweep never reaches (large weights during training land here).
    for _ in range(40):
        off = random.choice([0, 1, 2, 38, 39, 40, 41, 79, 80, 120, 126, 127])
        vals.append((random.randint(0, 1) << 16) | (off << 9) | random.randint(0, 511))
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
    if subprocess.run(["cc", "-O2", "-o", os.path.join(wd, "cbin"), os.path.join(wd, "main.c")],
                      cwd=wd, capture_output=True, text=True).returncode != 0:
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
    if subprocess.run(["rustc", "-A", "warnings", "-O", "-o", os.path.join(wd, "rbin"), rs],
                      cwd=wd, capture_output=True, text=True).returncode != 0:
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
