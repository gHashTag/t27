#!/usr/bin/env python3
"""Differential fuzzer: C trainer vs the Python GF-T model over RANDOM topologies
and RANDOM training inputs, with edge values injected (offset-saturation-large,
tiny, exact 0/+-1/+-2/+-0.5). Where the fixed-sequence gate (verify_trainer_c)
proves bit-exactness on a few chosen nets, this widens it to a randomized space --
a single divergence is a reproducible counterexample of a spec-vs-model edge bug.

Reuses run_model / run_c from verify_trainer_c (one shared C emission, no drift).
CI-friendly: SKIPs (exit 0) if t27c / cc is missing; any mismatch exits 1.
    python3 tools/fuzz_trainer.py [rounds]   # default 40; try 500 locally
"""
import os, sys, shutil, tempfile, random, importlib.util, subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
vt_spec = importlib.util.spec_from_file_location(
    "vt", os.path.join(ROOT, "tools/verify_trainer_c.py"))
vt = importlib.util.module_from_spec(vt_spec); vt_spec.loader.exec_module(vt)

# T118: flags are not round counts. This read sys.argv[1] directly, so
# `fuzz_trainer.py --require` -- the spelling its sibling uses, and the one this
# commit adds to CI -- died with a ValueError at import time, before main() and
# before any verdict. A crash at import is the least informative red there is.
_ROUND_ARGS = [a for a in sys.argv[1:] if not a.startswith("-")]
ROUNDS = int(_ROUND_ARGS[0]) if _ROUND_ARGS else 40
STEPS = 16
EDGES = [0.0, 1.0, -1.0, 2.0, -2.0, 0.5, -0.5, 0.25, -0.25]


def rnd_val(r):
    p = r.random()
    if p < 0.15: return r.choice(EDGES)               # exact GF-T-representable
    if p < 0.25: return r.uniform(-1e5, 1e5)          # large -> offset saturation
    if p < 0.30: return r.choice([1e-6, -1e-6, 1e-9]) # tiny -> underflow edge
    return round(r.uniform(-4, 4), 3)


def rnd_sizes(r):
    n_in = r.randint(1, 3)
    depth = r.randint(1, 3)                            # hidden layers
    sizes = [n_in] + [r.randint(1, 5) for _ in range(depth)] + [r.randint(1, 3)]
    return sizes


def self_check():
    """Plant a divergence in the C arm and prove this whole program reports it.

    T118. This was the last gate in the tree with no negative control in any
    form. It was invisible to `tri gates sweep` until selection moved from names
    to properties -- `fuzz_*` matches neither `check_*` nor "gate".

    The plant is a REAL tree: tools/ copied, target/ symlinked, and the copy of
    verify_trainer_c.py edited so its run_c returns a value one step off. The
    gate under test loads that module by path from its own ROOT, so the
    divergence arrives exactly where a real one would -- through the arm being
    compared, not through a stub of it.

    Three verdicts, each asserting its own message, because all three exit 1 and
    the code cannot tell them apart. The clean direction is asserted too: every
    other case demands RED, so a gate rewritten to fail unconditionally would
    satisfy all of them.

    NOT COVERED: the "C trainer failed to build/run" branch, which needs a
    compiler that is present and then fails -- a plant worth building and not
    built here.
    """
    ok = True

    def spawned(label, edit, want, expect, absent, rounds="2"):
        nonlocal ok
        with tempfile.TemporaryDirectory() as td:
            os.makedirs(os.path.join(td, "tools"))
            for f in os.listdir(os.path.join(ROOT, "tools")):
                if f.endswith(".py"):
                    src = open(os.path.join(ROOT, "tools", f), encoding="utf-8").read()
                    if f == "verify_trainer_c.py" and edit:
                        src = edit(src)
                    open(os.path.join(td, "tools", f), "w", encoding="utf-8").write(src)
            os.symlink(os.path.join(ROOT, "target"), os.path.join(td, "target"))
            for extra in ("specs", "conformance"):
                s = os.path.join(ROOT, extra)
                if os.path.exists(s):
                    os.symlink(s, os.path.join(td, extra))
            r = subprocess.run(
                [sys.executable, os.path.join(td, "tools", "fuzz_trainer.py"), rounds],
                capture_output=True, text=True)
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
            print(f"       said {out[:400]!r}")

    # The clean direction first, or the two below pass for free on a gate that
    # reds unconditionally.
    spawned("an unperturbed tree finds no divergence", None, 0,
            ["FUZZ OK: C trainer == model"],
            ["COUNTEREXAMPLE", "FAIL round", "Traceback"])

    def perturb_run_c(replacement):
        """Edit the C arm ONLY.

        The first version of this used a bare str.replace on the whole file and
        hit `run_model`'s return instead -- the MODEL arm, three functions
        earlier. The counterexample case then failed with a traceback, and the
        length case PASSED for the wrong reason: a shortened model is also a
        length mismatch, so the assertion was satisfied by a divergence planted
        somewhere it was never meant to be. Splitting at `def run_c(` and
        editing only the tail is what makes the case measure the arm it names.
        """
        def edit(src):
            head, sep, tail = src.partition("def run_c(")
            assert sep, "run_c disappeared from verify_trainer_c.py"
            anchor = "    return [tuple(map(int, ln.split())) for ln in out.strip().splitlines()]"
            assert anchor in tail, "run_c's result line moved; the plant must move with it"
            return head + sep + tail.replace(anchor, replacement, 1)
        return edit

    # One step of the C arm moved: the counterexample verdict, and the only one
    # that says a spec-vs-model bug exists.
    spawned("a perturbed C arm is a counterexample",
            perturb_run_c("    _o = [tuple(map(int, ln.split())) for ln in out.strip().splitlines()]\n"
                          "    return ([(_o[0][0] + 1,) + _o[0][1:]] + _o[1:]) if _o else _o"),
            1, ["COUNTEREXAMPLE round"],
            ["FUZZ OK", "Traceback"])

    # A C arm one step SHORT: the length verdict. Distinct from the one above --
    # both exit 1, and only the message separates "they disagree" from "they are
    # not even the same shape".
    spawned("a short C arm is a length mismatch",
            perturb_run_c("    _o = [tuple(map(int, ln.split())) for ln in out.strip().splitlines()]\n"
                          "    return _o[:-1] if _o else _o"),
            1, ["vs model", "steps"],
            ["COUNTEREXAMPLE", "FUZZ OK", "Traceback"])

    print(f"  self-check: three verdicts reached, each by its own message; the "
          f"build-failure branch is NOT covered here = {ok}")
    return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        sys.exit(self_check())
    t27c = vt.find_t27c()
    if not t27c:
        vt.skip("t27c binary not found")
    if not shutil.which("cc"):
        vt.skip("no C compiler (cc) on PATH")
    g = vt.load_gen()
    r = random.Random(20260807)
    total_steps = 0
    with tempfile.TemporaryDirectory() as wd:
        for rd in range(ROUNDS):
            sizes = rnd_sizes(r)
            reg, steps = g.gen_deep(sizes)
            n_in, n_out = sizes[0], sizes[-1]
            # random init over every weight (W..) / bias (b..) register; scratch and
            # activation regs stay 0 (as the RTL zero-inits them)
            init = [(idx, g.enc(rnd_val(r))) for name, idx in reg.items()
                    if name.startswith("W") or name.startswith("b")]
            # random training sequence with edge-injected inputs/targets
            seq = [([rnd_val(r) for _ in range(n_in)], [rnd_val(r) for _ in range(n_out)])
                   for _ in range(STEPS)]
            py = vt.run_model(g, reg, steps, init, n_in, n_out, seq)
            cy = vt.run_c(g, reg, steps, init, n_in, n_out, seq, t27c, wd)
            if cy is None:
                print(f"FAIL round {rd} sizes={sizes}: C trainer failed to build/run"); sys.exit(1)
            if len(cy) != len(py):
                print(f"FAIL round {rd} sizes={sizes}: C {len(cy)} vs model {len(py)} steps"); sys.exit(1)
            for i, (p, c) in enumerate(zip(py, cy)):
                if p != c:
                    print(f"COUNTEREXAMPLE round {rd} sizes={sizes} step {i}: model={p} C={c}")
                    print(f"  init={init}")
                    print(f"  seq={seq}")
                    sys.exit(1)
            total_steps += len(py)
    print(f"FUZZ OK: C trainer == model over {ROUNDS} random topologies x {STEPS} steps "
          f"({total_steps} step-comparisons), edge values injected -- no divergence")
    sys.exit(0)


if __name__ == "__main__":
    main()
