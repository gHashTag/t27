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
SPECS = {"gft_smul": "smul", "gft_sadd": "sadd"}   # spec file -> top function
N = 600


REQUIRE = "--require" in sys.argv


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
    hdr = _gen(t27c, "c", f"specs/ternary/{spec}.t27", ROOT)
    if hdr is None:
        return None
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
    out = _run_bin(os.path.join(wd, "cbin"), "C target run")
    if out is None:
        return None
    return [int(x) for x in out.split()]


def run_rust(t27c, spec, fn, pairs, wd):
    src = _gen(t27c, "rust", f"specs/ternary/{spec}.t27", ROOT)
    if src is None:
        return None
    if "fn " not in src:
        return None
    a = ",".join(str(x) for x, _ in pairs); b = ",".join(str(y) for _, y in pairs)
    src += (f'\nfn main(){{let a:[u32;{len(pairs)}]=[{a}];let b:[u32;{len(pairs)}]=[{b}];'
            f'for i in 0..a.len(){{println!("{{}}",{fn}(a[i],b[i]) as u32);}}}}\n')
    rs = os.path.join(wd, "m.rs"); open(rs, "w").write(src)
    if not _build(["rustc", "-A", "warnings", "-O", "-o", os.path.join(wd, "rbin"), rs], wd, "Rust target"):
        return None
    out = _run_bin(os.path.join(wd, "rbin"), "Rust target run")
    if out is None:
        return None
    return [int(x) for x in out.split()]


def self_check():
    """Prove the skip pair can reach BOTH of its exits, for the right reason.

    T114. This gate ran in CI for the whole campaign with no negative control in
    any form -- invisible to `tri gates sweep` because it is named `verify_*`
    rather than `check_*`, which is how a name came to stand in for a property.

    The branch covered here is the one whose entire purpose is to stop a silent
    non-run: without --require a missing prerequisite is a SKIP and exit 0, with
    it the same state is a FAILURE. Those two exits are three lines apart and
    the exit code is the only thing that differs, so each case names the other's
    marker as forbidden -- "SKIP" reaching exit 1 and "FAIL" reaching exit 0 are
    both silent successes of the kind this campaign exists to find.

    The world is planted by COPYING THIS SCRIPT INTO AN EMPTY TREE. find_t27c()
    resolves target/*/t27c against ROOT, and ROOT is __file__'s parent's parent,
    so the copy makes the absence real rather than simulated. A temp cwd was
    tried first and does nothing -- the control said so on its first run, which
    is what a control is for. `t27c` is also removed from PATH, because
    find_t27c() falls back to shutil.which and a runner with it installed would
    otherwise find one.

    No flag aims this gate anywhere: nothing here adds a way to make the live
    check pass.

    NOT COVERED, said out loud rather than left to be inferred: the cross-target
    MISMATCH verdict at the end of main(). Reaching it needs t27c to emit a
    backend that disagrees with the Python model, which means planting a fake
    compiler that generates deliberately wrong C and Rust -- a control worth
    building and not built here. Until it exists, this gate is proven able to
    fail on a broken ENVIRONMENT and unproven on a broken ARITHMETIC.
    """
    ok = True

    def case(label, args, want_rc, want, forbid):
        nonlocal ok
        with tempfile.TemporaryDirectory() as td:
            import shutil as _sh
            tools = os.path.join(td, "tools")
            os.makedirs(tools)
            me = os.path.join(tools, os.path.basename(__file__))
            _sh.copy(os.path.abspath(__file__), me)
            # T122: the shared prerequisite module travels with the gate. The
            # extraction broke this control on its first run -- the planted tree
            # had the script and not the module it imports, so the child died at
            # import with empty stdout, which the case correctly refused to read
            # as a skip.
            _sh.copy(os.path.join(os.path.dirname(os.path.abspath(__file__)), "_prereq.py"),
                     os.path.join(tools, "_prereq.py"))
            env = dict(os.environ)
            env["PATH"] = os.pathsep.join(
                d for d in env.get("PATH", "").split(os.pathsep)
                if d and not os.path.exists(os.path.join(d, "t27c")))
            r = subprocess.run([sys.executable, me, *args],
                               capture_output=True, text=True, cwd=td, env=env)
        missing = [s for s in want if s not in r.stdout]
        leaked = [s for s in forbid if s in r.stdout]
        good = r.returncode == want_rc and not missing and not leaked
        print(f"  {label:<34} " + (f"exit {want_rc}, right branch" if good
                                   else "CONTROL FAILED"))
        if not good:
            ok = False
            print(f"       exit {r.returncode!r} (want {want_rc!r})")
            if missing:
                print(f"       the branch never said: {missing!r}")
            if leaked:
                print(f"       neighbouring marker leaked: {leaked!r}")
            print(f"       stdout {r.stdout[:300]!r}")

    case("missing prerequisite skips", [], 0,
         ["SKIP verify_multitarget.py: t27c binary not found"],
         ["FAIL", "--require was given", "ALL TARGETS BIT-EXACT", "CROSS-TARGET MISMATCH"])

    # The same world, the opposite verdict. The explanation is asserted with the
    # exit code because main() has other paths to 1, and "it went red" does not
    # say it went red for this reason.
    case("--require turns it into a failure", ["--require"], 1,
         ["FAIL verify_multitarget.py: t27c binary not found",
          "--require was given, so a missing prerequisite is a failure"],
         ["SKIP", "ALL TARGETS BIT-EXACT", "CROSS-TARGET MISMATCH"])

    print("  self-check: both exits of the skip pair reached, each by its own message; "
          f"the cross-target mismatch verdict is NOT covered here = {ok}")
    return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        sys.exit(self_check())
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
