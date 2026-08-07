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
import os, sys, shutil, tempfile, random, importlib.util

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
vt_spec = importlib.util.spec_from_file_location(
    "vt", os.path.join(ROOT, "tools/verify_trainer_c.py"))
vt = importlib.util.module_from_spec(vt_spec); vt_spec.loader.exec_module(vt)

ROUNDS = int(sys.argv[1]) if len(sys.argv) > 1 else 40
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


def main():
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
