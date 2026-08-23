#!/usr/bin/env python3
"""Whole-trainer cross-target bit-exactness (C == model).

verify_multitarget proves the GF-T PRIMITIVES (smul/sadd) are bit-exact across
Verilog/C/Rust. This proves the WHOLE trainer is: it emits the microsequencer as a
C program -- the C GF-T primitives (t27c gen-c) + a microcode interpreter + the
operand-modifier `modf` -- runs a full 80-step training run (forward+backprop+
update), and checks every output per step against the independent Python GF-T
model. Verilog == model is already proven by verify_emit_bitexact, so C == model
here closes "one spec -> any target, bit-exact" for the entire training loop, not
just the arithmetic.

Self-contained + CI-friendly: SKIPs (exit 0) if t27c / a C compiler is missing; a
real divergence exits 1. Run:  python3 tools/verify_trainer_c.py
"""
import os, re, sys, shutil, subprocess, tempfile, importlib.util, random


_pq = importlib.util.spec_from_file_location(
    "_prereq", os.path.join(os.path.dirname(os.path.abspath(__file__)), "_prereq.py"))
_prereq = importlib.util.module_from_spec(_pq); _pq.loader.exec_module(_prereq)
skip, broken = _prereq.skip, _prereq.broken

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
    # mutant-equivalent: the guard above forces returncode != 0, so < is <=
    #
    # T132. Five copies of this line across five verifiers, and the boundary
    # operator reports each as a survivor. All five are reached only after a
    # returncode check -- three via `if returncode == 0: return`, two via
    # `if returncode != 0:` -- so at this point the value cannot be zero, and
    # `< 0` and `<= 0` agree on every value it can hold. Proven from the line
    # above, not assumed from the shape.
    sig = f" (signal {-r.returncode})" if r.returncode < 0 else ""
    print(f"  {what}: exited {r.returncode}{sig}")
    for line in (r.stderr or "").strip().splitlines()[:4]:
        print(f"      {line}")
    return None


def _gen(t27c, mode, spec, root):
    """Run `t27c gen-<mode>` and return its output, or None with the reason printed.

    Taking `.stdout` while checking neither the exit code nor stderr is how a spec
    that failed to PARSE surfaced as "the C backend failed to build" for four days.
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
SMUL_SPEC = os.path.join(ROOT, "specs/ternary/gft_smul.t27")
ARCHS = [(2, 2, 1), (2, 4, 2), [2, 4, 3, 1]]   # 2-layer, multi-output, deep
STEPS = 80

MODF_C = r"""
static uint32_t modf_(uint32_t v, int m){
  unsigned off, mant; int sgn = (v>>16)&1;
  if(m==0) return v;
  if(m==1) return (v==0||sgn)?0u:v;
  if(m==2) return (v==0||sgn)?0u:20480u;
  if(m==3) return (v==0)?0u:(v^65536u);
  if(m==4){ if(v==0) return 0u; off=(v>>9)&0x7fu; mant=v&0x1ffu;
            if(off<4u) return 0u;
            return (((v&0x10000u)^0x10000u) | (((off-3u)<<9)|mant)); }
  return v;
}
"""


def find_t27c():
    for p in ("target/debug/t27c", "target/release/t27c"):
        c = os.path.join(ROOT, p)
        if os.path.exists(c):
            return c
    return shutil.which("t27c")


def load_gen():
    spec = importlib.util.spec_from_file_location(
        "gbm", os.path.join(ROOT, "tools/gft_backprop_microcode.py"))
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    return m


def emit_and_gen(g, arch):
    if isinstance(arch, list):
        return g.emit_verilog_deep(arch, "bpx"), *g.gen_deep(arch), arch[0], arch[-1]
    return g.emit_verilog(*arch, "bpx"), *g.gen(*arch), arch[0], arch[2]


def build_seq(n_in, n_out):
    # identical stream to verify_emit_bitexact.check()
    random.seed(101); seq = []
    for _ in range(STEPS):
        xs = [round(random.uniform(-1, 1), 3) for _ in range(n_in)]
        cls = int((xs[0] > 0) != (xs[-1] > 0))
        ts = [1.0 if o == cls % n_out else 0.0 for o in range(n_out)]
        seq.append((xs, ts))
    return seq


def run_model(g, reg, steps, init_pairs, n_in, n_out, seq):
    """Run the training sequence through the Python GF-T model. init_pairs = list of
    (reg_index, u32); seq = list of (xs_floats, ts_floats). Returns per-step output
    tuples (yout u32 for each of n_out)."""
    rf = [0] * len(reg)
    for i, val in init_pairs:
        rf[i] = val
    out = []
    for xs, ts in seq:
        for k in range(n_in): rf[reg[f"x{k}"]] = g.enc(xs[k])
        for o in range(n_out): rf[reg[f"t{o}"]] = g.enc(ts[o])
        g.run(steps, rf)
        out.append(tuple(rf[reg[f"y{o}"]] & 0xFFFFFFFF for o in range(n_out)))
    return out


def run_c(g, reg, steps, init_pairs, n_in, n_out, seq, t27c, wd):
    """Emit the trainer as a C program (t27c gen-c primitives + microcode interpreter
    + modf), compile, run the same sequence. Returns per-step output tuples, or None
    on a build failure. gftmod.h is (re)written into wd."""
    hdr = _gen(t27c, "c", "specs/ternary/gft_smul.t27", ROOT)
    if hdr is None:
        return None
    if "GFTSMUL_H" not in hdr:
        return None
    open(os.path.join(wd, "gftmod.h"), "w").write(hdr)
    op = ",".join("2" if o == "MOV" else ("1" if o == "ADD" else "0") for o, *_ in steps)
    ai = ",".join(str(s[1]) for s in steps); am = ",".join(str(s[2]) for s in steps)
    bi = ",".join(str(s[3]) for s in steps); bm = ",".join(str(s[4]) for s in steps)
    di = ",".join(str(s[5]) for s in steps)
    initc = "".join(f"rf[{i}]={val}u;" for i, val in init_pairs)
    xidx = [reg[f"x{k}"] for k in range(n_in)]; tidx = [reg[f"t{o}"] for o in range(n_out)]
    yidx = [reg[f"y{o}"] for o in range(n_out)]
    rows = ["{" + ",".join(str(x) for x in ([g.enc(x) for x in xs] + [g.enc(t) for t in ts])) + "}"
            for xs, ts in seq]
    samples = ",".join(rows)
    main = f"""#define assert_eq(a,b) ((void)0)
#include "gftmod.h"
#include <stdio.h>
{MODF_C}
static uint32_t rf[{len(reg)}];
static const int OP[]={{{op}}}, AI[]={{{ai}}}, AM[]={{{am}}}, BI[]={{{bi}}}, BM[]={{{bm}}}, DI[]={{{di}}};
#define NP {len(steps)}
static void run_steps(void){{
  int pc; for(pc=0; pc<NP; pc++){{
    uint32_t a=modf_(rf[AI[pc]],AM[pc]), b=modf_(rf[BI[pc]],BM[pc]);
    rf[DI[pc]] = OP[pc]==2 ? a : (OP[pc] ? sadd(a,b) : smul(a,b));
  }}
}}
static const uint32_t SAMP[{len(seq)}][{n_in + n_out}]={{{samples}}};
static const int XIDX[]={{{",".join(map(str,xidx))}}}, TIDX[]={{{",".join(map(str,tidx))}}}, YIDX[]={{{",".join(map(str,yidx))}}};
int main(void){{
  int s,k;
  {initc}
  for(s=0;s<{len(seq)};s++){{
    for(k=0;k<{n_in};k++) rf[XIDX[k]]=SAMP[s][k];
    for(k=0;k<{n_out};k++) rf[TIDX[k]]=SAMP[s][{n_in}+k];
    run_steps();
    for(k=0;k<{n_out};k++) printf("%u%s", rf[YIDX[k]], k=={n_out}-1?"\\n":" ");
  }}
  return 0;
}}
"""
    cf = os.path.join(wd, "trainer.c"); open(cf, "w").write(main)
    b = os.path.join(wd, "tbin")
    if not _build(["cc", "-O2", "-o", b, cf], wd, "trainer C"):
        return None
    out = _run_bin(b, "trainer C run")
    if out is None:
        return None
    return [tuple(map(int, ln.split())) for ln in out.strip().splitlines()]


def check(g, arch, t27c, wd):
    v, reg, steps, n_in, n_out = emit_and_gen(g, arch)
    init = [(int(i), int(val)) for i, val in re.findall(r"rf\[(\d+)\]<=32'd(\d+);", v)]
    seq = build_seq(n_in, n_out)
    py = run_model(g, reg, steps, init, n_in, n_out, seq)
    cy = run_c(g, reg, steps, init, n_in, n_out, seq, t27c, wd)
    if cy is None:
        print(f"FAIL {arch}: C trainer failed to build/run"); return False
    if len(cy) != len(py):
        print(f"FAIL {arch}: C produced {len(cy)} of {len(py)} steps"); return False
    mism = [(i, p, c) for i, (p, c) in enumerate(zip(py, cy)) if p != c]
    if mism:
        i, p, c = mism[0]
        print(f"FAIL {arch}: C != model in {len(mism)}/{len(py)}; first step {i} model={p} C={c}"); return False
    print(f"OK {arch}: C trainer == Python model BIT-EXACT over {len(py)} training steps, all {n_out} output(s)")
    return True


def self_check():
    """Plant a divergence in the C arm and prove this whole program reports it.

    T123. The last of the four cross-target verifiers to get one, and the one
    whose skip() every other copied.

    The plant is a real tree -- tools/ copied, target/ and specs/ symlinked --
    with THIS FILE's own run_c edited so its result is one step off. The gate
    under test is the copy in that tree, so the divergence arrives through the
    arm being compared rather than through a stub of it.

    The edit is scoped to the text after `def run_c(`, for the reason the same
    plant in fuzz_trainer.py needed it: a bare replace hit run_model's return
    three functions earlier, and one case then PASSED on a divergence planted
    somewhere it was never meant to be.

    NOT COVERED, said rather than inferred: the per-architecture "C produced N
    of M steps" branch, which needs a C arm that truncates rather than one that
    disagrees.
    """
    ok = True

    def spawned(label, edit, want, expect, absent):
        nonlocal ok
        import shutil as _sh
        with tempfile.TemporaryDirectory() as td:
            os.makedirs(os.path.join(td, "tools"))
            for f in os.listdir(os.path.join(ROOT, "tools")):
                if f.endswith(".py"):
                    src = open(os.path.join(ROOT, "tools", f), encoding="utf-8").read()
                    if f == os.path.basename(__file__) and edit:
                        src = edit(src)
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
            print(f"       said {out[:320]!r}")

    def perturb_run_c(replacement):
        anchor = "    return [tuple(map(int, ln.split())) for ln in out.strip().splitlines()]"
        def edit(src):
            head, sep, tail = src.partition("def run_c(")
            assert sep, "run_c disappeared"
            assert anchor in tail, "run_c's result line moved; the plant must move with it"
            return head + sep + tail.replace(anchor, replacement, 1)
        return edit

    # The clean direction first, or the case below passes for free on a gate
    # that reds unconditionally.
    spawned("an unperturbed tree is bit-exact", None, 0,
            ["WHOLE TRAINER BIT-EXACT ACROSS TARGETS"],
            ["TRAINER CROSS-TARGET MISMATCH", "FAIL", "Traceback"])

    spawned("a perturbed C arm is a mismatch",
            perturb_run_c("    _o = [tuple(map(int, ln.split())) for ln in out.strip().splitlines()]\n"
                          "    return ([(_o[0][0] + 1,) + _o[0][1:]] + _o[1:]) if _o else _o"),
            1, ["C != model", "TRAINER CROSS-TARGET MISMATCH"],
            ["WHOLE TRAINER BIT-EXACT", "Traceback"])

    print(f"  self-check: both directions of the cross-target verdict; the step-count "
          f"branch is NOT covered here = {ok}")
    return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        sys.exit(self_check())
    t27c = find_t27c()
    if not t27c:
        skip("t27c binary not found")
    if not shutil.which("cc"):
        skip("no C compiler (cc) on PATH")
    g = load_gen()
    with tempfile.TemporaryDirectory() as wd:
        ok = all(check(g, a, t27c, wd) for a in ARCHS)
    print("WHOLE TRAINER BIT-EXACT ACROSS TARGETS (C == model == Verilog)" if ok else "TRAINER CROSS-TARGET MISMATCH")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
