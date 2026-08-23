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
import os
import subprocess, re, sys, shutil, subprocess, tempfile, random

def _run_bin(cmd, what, cwd=None):
    """Run a built binary and return its stdout, or None with the reason printed.

    Taking `.stdout` without checking the exit code means a crash arrives as a
    short or empty result list, which then surfaces as a NUMERIC MISMATCH between
    targets -- the most alarming reading available, and the wrong one. A program
    that died on signal 11 did not disagree about arithmetic.
    """
    r = subprocess.run(cmd if isinstance(cmd, list) else [cmd],
                       capture_output=True, text=True, cwd=cwd)
    if r.returncode == 0:
        return r.stdout
    sig = f" (signal {-r.returncode})" if r.returncode < 0 else ""
    print(f"  {what}: exited {r.returncode}{sig}")
    for line in (r.stderr or "").strip().splitlines()[:4]:
        print(f"      {line}")
    return None


def _gen(t27c, mode, spec, root):
    """Run `t27c gen-<mode>` and return its output, or None with the reason printed.

    Every caller below used to take `.stdout` directly, checking neither the exit
    code nor stderr. When the spec failed to PARSE, stdout was empty, the empty
    string flowed downstream, and the failure surfaced as "the C backend failed to
    build/run" -- pointing at a subsystem that had never been reached. The
    compiler's message named the file, the function, the line and the token; it
    was collected by capture_output and discarded. Four days were read in the
    wrong place. Ask the exit code, and print what the tool said.
    """
    r = subprocess.run([t27c, "gen-" + mode, spec], capture_output=True, text=True, cwd=root)
    if r.returncode == 0:
        return r.stdout
    out = (r.stderr or r.stdout or "").strip().splitlines()
    print(f"  t27c gen-{mode} {spec}: exited {r.returncode}"
          + ("" if out else " with no message"))
    for line in out[:4]:
        print(f"      {line}")
    return None


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
SPEC = "specs/igla/race/ternary_mac.t27"
SYS_SPEC = "specs/igla/race/systolic_ternary.t27"
N = 800


def skip(msg):
    """A missing prerequisite: a SKIP locally, a FAILURE under --require.

    T120. The THIRD copy of this helper in the trainer/verifier family, and the
    second found without a --require. Its sibling verify_multitarget.py has had
    one from the start, with a comment saying why: in CI a skip means the
    environment broke, and exit 0 makes "proved" indistinguishable from "never
    ran". Three copies, one rule, applied in one of them.

    The name is read from argv rather than hard-coded, for the reason the copy
    in verify_trainer_c.py needed it: that one announced another tool's name
    when a second script imported and called it.
    """
    who = os.path.basename(sys.argv[0]) or "verify_igla_race"
    if "--require" in sys.argv:
        print(f"FAIL {who}: {msg}")
        print("  --require was given, so a missing prerequisite is a failure, not a skip.")
        print("  The CI job builds t27c and the runner ships cc and rustc; if one is")
        print("  absent the environment is broken and this check did not run.")
        sys.exit(1)
    print(f"SKIP {who}: {msg}"); sys.exit(0)


def self_check():
    """Run the WHOLE program once per verdict reachable without a divergence.

    T120. This gate ran in CI with no negative control in any form, invisible to
    `tri gates sweep` until the selector learned that `sys.exit(0 if ok else 1)`
    is a failure path -- its only non-zero exit is that ternary.

    The skip PAIR is asserted in both directions, because those two exits are
    three lines apart and only the code differs: SKIP reaching 1 and FAIL
    reaching 0 are both silent successes. The clean direction is asserted too --
    without it a gate rewritten to fail unconditionally satisfies every case.

    NOT COVERED, said rather than inferred: the CROSS-TARGET MISMATCH verdict.
    Reaching it needs a backend that genuinely disagrees with the reference,
    which means planting an emitter -- worth building and not built here.
    """
    ok = True

    def spawned(label, args, want, expect, absent, env=None):
        nonlocal ok
        r = subprocess.run([sys.executable, os.path.abspath(__file__), *args],
                           capture_output=True, text=True,
                           env=env or os.environ.copy())
        out = r.stdout + r.stderr
        missing = [s for s in expect if s not in out]
        leaked = [s for s in absent if s in out]
        good = r.returncode == want and not missing and not leaked
        print(f"  {label:<44} " + (f"exit {want}, right branch" if good
                                   else "CONTROL FAILED"))
        if not good:
            ok = False
            print(f"       exit {r.returncode!r} (want {want!r})")
            if missing:
                print(f"       the branch never said: {missing!r}")
            if leaked:
                print(f"       neighbouring marker leaked: {leaked!r}")
            print(f"       said {out[:300]!r}")

    stripped = os.environ.copy()
    stripped["PATH"] = os.pathsep.join(
        d for d in stripped.get("PATH", "").split(os.pathsep)
        if d and not os.path.exists(os.path.join(d, "cc")))

    spawned("a missing compiler skips locally", [], 0,
            ["SKIP verify_igla_race.py: no C compiler"],
            ["FAIL", "CROSS-TARGET MISMATCH", "IGLA RACE: ternary_mul"], env=stripped)

    spawned("--require turns the same state into a failure", ["--require"], 1,
            ["FAIL verify_igla_race.py: no C compiler",
             "--require was given, so a missing prerequisite is a failure"],
            ["SKIP", "CROSS-TARGET MISMATCH", "IGLA RACE: ternary_mul"], env=stripped)

    spawned("a healthy tree exits 0 and says what it proved", ["--require"], 0,
            ["IGLA RACE: ternary_mul EXHAUSTIVE over all 65,536 inputs"],
            ["FAIL", "SKIP", "CROSS-TARGET MISMATCH"])

    print(f"  self-check: the skip pair in both directions plus the clean run; the "
          f"cross-target mismatch verdict is NOT covered here = {ok}")
    return 0 if ok else 1


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


def exhaustive_mul_digest():
    """FNV-1a over ternary_mul(a, code) for EVERY (a, code) pair.

    The input space of ternary_mul is 256 x 256 = 65,536 -- small enough to
    enumerate. gen_vectors() samples 800 triples from it, which is a subsample of
    a space that can be covered completely, and "agree on 800 random operands" is
    a strictly weaker statement than "agree on every possible input" when the
    latter costs milliseconds.

    ternary_mac is NOT exhaustible: its accumulator is i32, so the space is
    256 x 256 x 2^32 ~ 2.8e14. It stays sampled, with its edge cases, and the
    verdict line says which of the two it is.
    """
    h = 2166136261
    for a in range(-128, 128):
        for c in range(256):
            v = ref_mul(a, c) & 0xFF
            h = ((h ^ v) * 16777619) & 0xFFFFFFFF
    return h


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
    c = _gen(t27c, "c", SPEC, ROOT)
    if c is None:
        return False, False
    open(os.path.join(wd, "full.c"), "w").write('#define assert_eq(x,y) ((void)0)\n' + c + "\nint main(){return 0;}\n")
    c_ok = _build(["cc", "-c", "-o", os.path.join(wd, "f.o"), os.path.join(wd, "full.c")], wd, "full-spec gen-c")
    r = _gen(t27c, "rust", SPEC, ROOT)
    if r is None:
        return False, False
    open(os.path.join(wd, "full.rs"), "w").write(r + "\nfn main(){}\n")
    r_ok = _build(["rustc", "-A", "warnings", "--emit=metadata", "-o", os.path.join(wd, "f.rmeta"),
                   os.path.join(wd, "full.rs")], wd, "full-spec gen-rust")
    return c_ok, r_ok


def _core_c(t27c):
    src = _gen(t27c, "c", SPEC, ROOT)
    if src is None:
        return None
    st = re.search(r"typedef struct\s*\{[^}]*\}\s*TernaryWeight\s*;", src)
    defs = [_extract_def(src, s) for s in (
        "int8_t ternary_decode(TernaryWeight w)",
        "int8_t ternary_mul(int8_t a, TernaryWeight w)",
        "int32_t ternary_mac(int32_t acc, int8_t a, TernaryWeight w)")]
    if not st or any(d is None for d in defs):
        return None
    return "#include <stdint.h>\n#include <stdio.h>\n" + st.group(0) + "\n" + "\n".join(defs)


def _core_rust(t27c):
    src = _gen(t27c, "rust", SPEC, ROOT)
    if src is None:
        return None
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


def exhaustive_c(t27c, wd):
    """ternary_mul over EVERY (a, code) pair, in C. 256 x 256 = 65,536 inputs.

    gen_vectors() samples 800 triples out of that space. Sampling a space you can
    enumerate is a weaker statement than enumerating it, and here enumeration costs
    milliseconds -- so this is not a sample, it is the whole domain.

    ternary_mac is deliberately NOT covered this way: its accumulator is i32, so its
    space is 256 x 256 x 2^32 ~ 2.8e14. It stays sampled, and the verdict says so.
    """
    core = _core_c(t27c)
    if core is None:
        return None
    main = [
        "int main(void){",
        "  unsigned h = 2166136261u;",
        "  for (int a = -128; a < 128; a++) {",
        "    for (int c = 0; c < 256; c++) {",
        "      TernaryWeight w; w.code = (uint8_t)c;",
        "      unsigned v = (unsigned)((int)ternary_mul((int8_t)a, w) & 0xFF);",
        "      h = (h ^ v) * 16777619u;",
        "    }",
        "  }",
        '  printf("%08x\\n", h);',
        "  return 0;",
        "}",
    ]
    src = core + "\n" + "\n".join(main) + "\n"
    f = os.path.join(wd, "exh.c")
    open(f, "w").write(src)
    if not _build(["cc", "-O2", "-o", os.path.join(wd, "exhc"), f], wd, "exhaustive C"):
        return None
    out = _run_bin(os.path.join(wd, "exhc"), "exhaustive C run")
    return None if out is None else out.strip()


def exhaustive_rust(t27c, wd):
    """The same whole-domain sweep, in Rust."""
    core = _core_rust(t27c)
    if core is None:
        return None
    main = [
        "fn main(){",
        "  let mut h: u32 = 2166136261;",
        "  for a in -128i32..128 {",
        "    for c in 0u32..256 {",
        "      let w = TernaryWeight { code: c as u8 };",
        "      let v = ((ternary_mul(a as i8, w) as i32) & 0xFF) as u32;",
        "      h = (h ^ v).wrapping_mul(16777619);",
        "    }",
        "  }",
        '  println!("{:08x}", h);',
        "}",
    ]
    src = core + "\n" + "\n".join(main) + "\n"
    f = os.path.join(wd, "exh.rs")
    open(f, "w").write(src)
    if not _build(["rustc", "-A", "warnings", "-O", "-o", os.path.join(wd, "exhr"), f], wd, "exhaustive Rust"):
        return None
    out = _run_bin(os.path.join(wd, "exhr"), "exhaustive Rust run")
    return None if out is None else out.strip()


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
    if not _build(["cc", "-O2", "-o", os.path.join(wd, "cb"), os.path.join(wd, "m.c")], wd, "core C"):
        return None
    out = _run_bin(os.path.join(wd, "cb"), "core C run")
    if out is None:
        return None
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
    if not _build(["rustc", "-A", "warnings", "-O", "-o", os.path.join(wd, "rb"), rs], wd, "core Rust"):
        return None
    out = _run_bin(os.path.join(wd, "rb"), "core Rust run")
    if out is None:
        return None
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
    sysc = _gen(t27c, "c", SYS_SPEC, ROOT)
    if sysc is None:
        return None
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
    if not _build(["cc", "-O2", "-o", os.path.join(wd, "peb"), os.path.join(wd, "pe.c")], wd, "systolic PE C"):
        return None
    out = _run_bin(os.path.join(wd, "peb"), "systolic PE C run")
    if out is None:
        return None
    return [int(x) for x in out.split()]


def run_pe_rust(t27c, vecs, wd):
    core = _core_rust(t27c)
    sysr = _gen(t27c, "rust", SYS_SPEC, ROOT)
    if sysr is None:
        return None
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
    if not _build(["rustc", "-A", "warnings", "-O", "-o", os.path.join(wd, "perb"), rs], wd, "systolic PE Rust"):
        return None
    out = _run_bin(os.path.join(wd, "perb"), "systolic PE Rust run")
    if out is None:
        return None
    return [int(x) for x in out.split()]


def main():
    if "--self-check" in sys.argv:
        sys.exit(self_check())
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
                print(f"OK ternary_mul/mac: {tgt} == reference over {len(ref)} SAMPLED vectors "
                      f"(edge cases included; ternary_mac's i32 accumulator makes its space "
                      f"~2.8e14, so this arm is a sample and says so)")

        # ternary_mul alone has a space of 256 x 256 = 65,536 -- small enough to enumerate.
        # Sampling a domain you can exhaust is a weaker claim than exhausting it, and here
        # exhausting it costs milliseconds. This arm is therefore not a sample.
        want = f"{exhaustive_mul_digest():08x}"
        for tgt, runner in (("C", exhaustive_c), ("Rust", exhaustive_rust)):
            got = runner(t27c, wd)
            if got is None:
                print(f"FAIL: exhaustive {tgt} failed to build/run"); ok = False; continue
            if got != want:
                print(f"FAIL: exhaustive {tgt} digest {got} != model {want} -- the two disagree "
                      f"on at least one of the 65,536 possible (a, code) inputs")
                ok = False
            else:
                print(f"OK ternary_mul: {tgt} == reference on ALL 65,536 possible inputs "
                      f"(exhaustive, FNV-1a digest {got})")
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
    print("IGLA RACE: ternary_mul EXHAUSTIVE over all 65,536 inputs; mac + systolic PE agree on "
          "sampled vectors (C + Rust + model)" if ok else "IGLA RACE CROSS-TARGET MISMATCH")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
