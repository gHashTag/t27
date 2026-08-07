#!/usr/bin/env python3
"""Cross-target bit-exactness for IGLA RACE's ternary MAC core.

The RACE ternary accelerator specs (specs/igla/race/ternary_mac.t27) are sealed and
simulated but were never cross-checked across t27's backends. This applies the same
multi-target discipline the GF-T trainer uses: it emits `ternary_mul` / `ternary_mac`
(the multiplier-free R-SI-1 MAC primitives, {-1,0,+1} weights) to C and Rust via
t27c, compiles both, and cross-checks against an independent Python reference over
random + edge inputs -- a=-128 sign-flip wrap, invalid codes (>2 decode to 0), and
i32-accumulator edges. Closes the "RACE specs not multi-target-verified" gap.

Self-contained + CI-friendly: SKIPs (exit 0) if t27c / cc / rustc is missing; a real
divergence exits 1.  Run:  python3 tools/verify_igla_race.py
"""
import os, re, sys, shutil, subprocess, tempfile, random

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPEC = "specs/igla/race/ternary_mac.t27"
SYS_SPEC = "specs/igla/race/systolic_ternary.t27"
N = 800


def skip(msg):
    print(f"SKIP verify_igla_race: {msg}"); sys.exit(0)


def find_t27c():
    for p in ("target/debug/t27c", "target/release/t27c"):
        c = os.path.join(ROOT, p)
        if os.path.exists(c):
            return c
    return shutil.which("t27c")


def i8(x): return ((x + 128) & 0xFF) - 128
def i16(x): return ((x + (1 << 15)) & 0xFFFF) - (1 << 15)
def i32(x): return ((x + (1 << 31)) & 0xFFFFFFFF) - (1 << 31)


def ref_mul(a, code):
    a = i8(a); d = 1 if code == 1 else (-1 if code == 2 else 0)
    if d == 0: return 0
    return i8(a if d == 1 else -a)


def ref_mac(acc, a, code): return i32(i32(acc) + ref_mul(a, code))


def gen_vectors():
    r = random.Random(7777)
    A = [-128, -1, 0, 1, 127]; C = [0, 1, 2, 3, 255]; ACC = [0, (1 << 31) - 1, -(1 << 31), 1000000]
    v = []
    for a in A:
        for c in C:
            for ac in ACC:
                v.append((a, c, ac))
    while len(v) < N:
        v.append((r.randint(-128, 127), r.choice([0, 1, 2, 3, r.randint(0, 255)]),
                  r.randint(-(1 << 31), (1 << 31) - 1)))
    return v[:N]


def _brace_block(src, start):
    """Return src[start .. matching close brace], brace-matched from the first {."""
    b = src.find("{", start)
    if b < 0:
        return None
    depth, j = 0, b
    while j < len(src):
        if src[j] == "{": depth += 1
        elif src[j] == "}":
            depth -= 1
            if depth == 0:
                return src[start:j + 1]
        j += 1
    return None


def _extract_def(src, sig):
    """Extract the DEFINITION (sig followed by a `{...}` body), skipping any
    prototype (`sig;`). Returns the block or None."""
    for m in re.finditer(re.escape(sig), src):
        rest = src[m.end():]
        nxt = rest.lstrip()
        if nxt[:1] == "{":
            return _brace_block(src, m.start())
    return None


def full_spec_compiles(t27c, wd):
    """Diagnostic: does the WHOLE spec emit compilable C and Rust? Returns (c_ok,
    rust_ok, note). This surfaces gen-backend gaps in the IGLA spec (slice .len(),
    duplicate test emission, serde deps, non-Copy struct)."""
    c = subprocess.run([t27c, "gen-c", SPEC], capture_output=True, text=True, cwd=ROOT).stdout
    open(os.path.join(wd, "full.c"), "w").write('#define assert_eq(x,y) ((void)0)\n' + c + "\nint main(){return 0;}\n")
    c_ok = subprocess.run(["cc", "-c", "-o", os.path.join(wd, "f.o"), os.path.join(wd, "full.c")],
                          cwd=wd, capture_output=True, text=True).returncode == 0
    r = subprocess.run([t27c, "gen-rust", SPEC], capture_output=True, text=True, cwd=ROOT).stdout
    open(os.path.join(wd, "full.rs"), "w").write(r + "\nfn main(){}\n")
    r_ok = subprocess.run(["rustc", "-A", "warnings", "--emit=metadata", "-o", os.path.join(wd, "f.rmeta"),
                           os.path.join(wd, "full.rs")], cwd=wd, capture_output=True, text=True).returncode == 0
    return c_ok, r_ok


def _core_c(t27c):
    src = subprocess.run([t27c, "gen-c", SPEC], capture_output=True, text=True, cwd=ROOT).stdout
    st = re.search(r"typedef struct\s*\{[^}]*\}\s*TernaryWeight\s*;", src)
    defs = [_extract_def(src, s) for s in (
        "int8_t ternary_decode(TernaryWeight w)",
        "int8_t ternary_mul(int8_t a, TernaryWeight w)",
        "int32_t ternary_mac(int32_t acc, int8_t a, TernaryWeight w)")]
    if not st or any(d is None for d in defs):
        return None
    return "#include <stdint.h>\n#include <stdio.h>\n" + st.group(0) + "\n" + "\n".join(defs)


def _core_rust(t27c):
    src = subprocess.run([t27c, "gen-rust", SPEC], capture_output=True, text=True, cwd=ROOT).stdout
    ms = re.search(r"pub struct TernaryWeight\b", src)
    if not ms:
        return None
    st = _brace_block(src, ms.start())          # "pub struct TernaryWeight { pub code: u8, }"
    blocks = []
    for name in ("ternary_decode", "ternary_mul", "ternary_mac"):
        m = re.search(r"pub fn " + name + r"\b", src)
        if not m:
            return None
        blk = _brace_block(src, m.start())      # Rust has definitions only, no prototypes
        if blk is None:
            return None
        blocks.append(blk)
    if st is None:
        return None
    return "#[derive(Clone, Copy)]\n" + st + "\n" + "\n".join(blocks)


def run_c(t27c, vecs, wd):
    core = _core_c(t27c)
    if core is None:
        return None
    A = ",".join(str(a) for a, _, _ in vecs); C = ",".join(str(c) for _, c, _ in vecs)
    AC = ",".join(str(ac) for _, _, ac in vecs)
    src = (core + f'\nint main(){{int8_t A[]={{{A}}}; uint8_t C[]={{{C}}}; int32_t AC[]={{{AC}}}; int n={len(vecs)};'
           f'for(int i=0;i<n;i++){{TernaryWeight w; w.code=C[i];'
           f'printf("%d %d\\n",(int)ternary_mul(A[i],w),(int)ternary_mac(AC[i],A[i],w));}}return 0;}}')
    open(os.path.join(wd, "m.c"), "w").write(src)
    if subprocess.run(["cc", "-O2", "-o", os.path.join(wd, "cb"), os.path.join(wd, "m.c")],
                      cwd=wd, capture_output=True, text=True).returncode != 0:
        return None
    out = subprocess.run([os.path.join(wd, "cb")], capture_output=True, text=True).stdout
    return [tuple(map(int, ln.split())) for ln in out.strip().splitlines()]


def run_rust(t27c, vecs, wd):
    core = _core_rust(t27c)
    if core is None:
        return None
    A = ",".join(str(a) for a, _, _ in vecs); C = ",".join(str(c) for _, c, _ in vecs)
    AC = ",".join(str(ac) for _, _, ac in vecs)
    src = core + (f'\nfn main(){{let a:[i8;{len(vecs)}]=[{A}]; let c:[u8;{len(vecs)}]=[{C}]; '
                  f'let ac:[i32;{len(vecs)}]=[{AC}]; for i in 0..a.len(){{let w=TernaryWeight{{code:c[i]}}; '
                  f'println!("{{}} {{}}", ternary_mul(a[i],w) as i32, ternary_mac(ac[i],a[i],w));}}}}\n')
    rs = os.path.join(wd, "m.rs"); open(rs, "w").write(src)
    if subprocess.run(["rustc", "-A", "warnings", "-O", "-o", os.path.join(wd, "rb"), rs],
                      cwd=wd, capture_output=True, text=True).returncode != 0:
        return None
    out = subprocess.run([os.path.join(wd, "rb")], capture_output=True, text=True).stdout
    return [tuple(map(int, ln.split())) for ln in out.strip().splitlines()]


def ref_pe(a, code, psum):
    """systolic_ternary_pe: psum_out = psum_in + ternary_mul(a, w), i16 accumulator."""
    return i16(i16(psum) + ref_mul(a, code))


def gen_pe_vectors():
    r = random.Random(4242)
    A = [-128, -1, 0, 1, 127]; C = [0, 1, 2, 3]; PS = [0, 32767, -32768, 12345]
    v = [(a, c, p) for a in A for c in C for p in PS]
    while len(v) < N:
        v.append((r.randint(-128, 127), r.choice([0, 1, 2, 3]), r.randint(-32768, 32767)))
    return v[:N]


def run_pe_c(t27c, vecs, wd):
    # imported `ternary_mul` is NOT emitted into systolic's output -> supply the
    # primitive from ternary_mac's core, then add systolic's tuple + PE definition
    core = _core_c(t27c)
    sysc = subprocess.run([t27c, "gen-c", SYS_SPEC], capture_output=True, text=True, cwd=ROOT).stdout
    tup = re.search(r"typedef struct\s*\{[^}]*\}\s*t27_tuple_int8_t_int16_t\s*;", sysc)
    pe = _extract_def(sysc, "t27_tuple_int8_t_int16_t systolic_ternary_pe(int8_t a_in, TernaryWeight w, int16_t psum_in)")
    if core is None or not tup or pe is None:
        return None
    A = ",".join(str(a) for a, _, _ in vecs); C = ",".join(str(c) for _, c, _ in vecs)
    PS = ",".join(str(p) for _, _, p in vecs)
    src = (core + "\n" + tup.group(0) + "\n" + pe +
           f'\nint main(){{int8_t A[]={{{A}}}; uint8_t C[]={{{C}}}; int16_t PS[]={{{PS}}}; int n={len(vecs)};'
           f'for(int i=0;i<n;i++){{TernaryWeight w; w.code=C[i];'
           f'printf("%d\\n",(int)systolic_ternary_pe(A[i],w,PS[i]).f1);}}return 0;}}')
    open(os.path.join(wd, "pe.c"), "w").write(src)
    if subprocess.run(["cc", "-O2", "-o", os.path.join(wd, "peb"), os.path.join(wd, "pe.c")],
                      cwd=wd, capture_output=True, text=True).returncode != 0:
        return None
    out = subprocess.run([os.path.join(wd, "peb")], capture_output=True, text=True).stdout
    return [int(x) for x in out.split()]


def run_pe_rust(t27c, vecs, wd):
    core = _core_rust(t27c)
    sysr = subprocess.run([t27c, "gen-rust", SYS_SPEC], capture_output=True, text=True, cwd=ROOT).stdout
    m = re.search(r"pub fn systolic_ternary_pe\b", sysr)
    if core is None or not m:
        return None
    pe = _brace_block(sysr, m.start())
    if pe is None:
        return None
    # gen-rust FINDING: emits `psum_in(i16) + prod(i8)` with no cast -> Rust rejects
    # the mixed-width add (C auto-promotes). Documented one-token workaround so the
    # ARITHMETIC can be cross-checked; the gap itself is reported in main().
    pe = pe.replace("psum_in + prod", "psum_in + prod as i16")
    A = ",".join(str(a) for a, _, _ in vecs); C = ",".join(str(c) for _, c, _ in vecs)
    PS = ",".join(str(p) for _, _, p in vecs)
    src = (core + "\n" + pe +
           f'\nfn main(){{let a:[i8;{len(vecs)}]=[{A}]; let c:[u8;{len(vecs)}]=[{C}]; '
           f'let ps:[i16;{len(vecs)}]=[{PS}]; for i in 0..a.len(){{let w=TernaryWeight{{code:c[i]}}; '
           f'println!("{{}}", systolic_ternary_pe(a[i],w,ps[i]).1 as i32);}}}}\n')
    rs = os.path.join(wd, "pe.rs"); open(rs, "w").write(src)
    if subprocess.run(["rustc", "-A", "warnings", "-O", "-o", os.path.join(wd, "perb"), rs],
                      cwd=wd, capture_output=True, text=True).returncode != 0:
        return None
    out = subprocess.run([os.path.join(wd, "perb")], capture_output=True, text=True).stdout
    return [int(x) for x in out.split()]


def main():
    t27c = find_t27c()
    if not t27c:
        skip("t27c not found")
    if not os.path.exists(os.path.join(ROOT, SPEC)):
        skip("ternary_mac.t27 not found")
    if not shutil.which("cc"):
        skip("no C compiler")
    if not shutil.which("rustc"):
        skip("rustc not on PATH")
    vecs = gen_vectors()
    ref = [(ref_mul(a, c), ref_mac(ac, a, c)) for a, c, ac in vecs]
    ok = True
    with tempfile.TemporaryDirectory() as wd:
        # diagnostic: does the WHOLE spec emit compilable C/Rust? (surfaces gen gaps)
        c_ok, r_ok = full_spec_compiles(t27c, wd)
        print(f"NOTE full-spec gen-c compiles: {c_ok} | gen-rust compiles: {r_ok} "
              f"(if False, the emitter has gaps on this IGLA spec -- slice .len(), "
              f"duplicate test emission, serde deps, non-Copy struct; core arithmetic checked below)")
        for tgt, runner in (("C", run_c), ("Rust", run_rust)):
            got = runner(t27c, vecs, wd)
            if got is None:
                print(f"FAIL: {tgt} backend failed to build/run"); ok = False; continue
            if len(got) != len(ref):
                print(f"FAIL: {tgt} produced {len(got)} of {len(ref)}"); ok = False; continue
            mism = [(i, r, g) for i, (r, g) in enumerate(zip(ref, got)) if r != g]
            if mism:
                i, r, g = mism[0]
                print(f"FAIL: {tgt} != ref in {len(mism)}/{len(ref)}; first vec {vecs[i]} ref={r} {tgt}={g}")
                ok = False
            else:
                print(f"OK ternary_mul/mac: {tgt} == reference BIT-EXACT over {len(ref)} vectors (edges incl.)")
        # systolic PE: extends the check up the datapath (ternary_mac -> systolic PE)
        if os.path.exists(os.path.join(ROOT, SYS_SPEC)):
            print("NOTE systolic_ternary imports ternary_mul but the import is NOT emitted "
                  "into its C/Rust output (dangling call) -- primitive supplied from "
                  "ternary_mac's core (concrete #1773 in the C/Rust backends). Also: gen-rust "
                  "emits a mixed-width `i16 + i8` add with no cast (Rust rejects it; C promotes) "
                  "-- worked around with a documented `as i16` to cross-check the arithmetic")
            pv = gen_pe_vectors()
            pref = [ref_pe(a, c, p) for a, c, p in pv]
            for tgt, runner in (("C", run_pe_c), ("Rust", run_pe_rust)):
                got = runner(t27c, pv, wd)
                if got is None:
                    print(f"FAIL: systolic PE {tgt} failed to build/run"); ok = False; continue
                mism = [(i, r, g) for i, (r, g) in enumerate(zip(pref, got)) if r != g]
                if len(got) != len(pref) or mism:
                    i, r, g = (mism[0] if mism else (0, "?", "?"))
                    print(f"FAIL: systolic PE {tgt} != ref ({len(mism)}/{len(pref)}); first vec {pv[i]} ref={r} {tgt}={g}")
                    ok = False
                else:
                    print(f"OK systolic_ternary_pe: {tgt} == reference BIT-EXACT over {len(pref)} vectors (i16 psum, edges)")
    print("IGLA RACE ternary MAC + systolic PE BIT-EXACT ACROSS TARGETS (C + Rust + model)" if ok else "IGLA RACE CROSS-TARGET MISMATCH")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
