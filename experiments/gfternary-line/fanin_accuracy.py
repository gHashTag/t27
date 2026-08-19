"""W778: does effective fan-in govern FUNCTION, or only area?

T408-T411 established, by exhaustive enumeration over all 729 weight triples,
that a nine-level alphabet's skew decides how many of a fan-in-3 neuron's inputs
its output depends on -- 2.55 for linear {0,+-1,+-2,+-3,+-4}, 2.19 for dyadic,
1.49 for base 3, 1.03 for base 4 -- and that this predicts LUT area at r=+0.991.

T411 scoped that to AREA, because the accuracy benches show nothing: T403 found
base 3 at +0.14 pp (UNSW) and -0.25 pp (Fashion), neither significant. But those
benches are DENSE. A dominant weight among 593 dense inputs is harmless. The
claim "base 3 computes less" is a claim about capability, and it has only been
tested where the mechanism cannot act.

This tests it where the mechanism DOES act: a sparse network at fan-in 3, which
is the architecture S4 mandates and the whole reason the table datapath is cheap.

REGISTERED FORECAST, before the run (T44):

  Accuracy tracks effective fan-in monotonically. Ordering linear >= dyadic >
  base 3 > base 4, and BASE 4 LOSES AT LEAST 3 pp AGAINST LINEAR 9. A neuron
  that reads 1.03 of its 3 inputs cannot be worth as much as one reading 2.55,
  and at fan-in 3 there is nowhere for the network to hide it.

  REFUTATION CONSEQUENCE, stated now: if base 4 lands WITHIN 1 pp of linear 9,
  effective fan-in does not govern function -- training routes around it -- and
  T408-T411 are re-scoped to AREA ONLY, with "base 3 computes less" withdrawn as
  a claim about capability and kept only as a claim about LUT.

  A middle outcome (loses, but by less than 3 pp) is a partial confirmation and
  will be reported as one, not rounded up.

The one control that matters: every arm gets the SAME wiring masks per seed, so
the arms differ only in the alphabet's values.
"""
import json
import sys
import time

import numpy as np

# Magnitudes only; 0 and the negatives are added. Nine levels everywhere, so
# CARDINALITY is held fixed and only SHAPE varies (T286: confounding them is how
# a base wins for the wrong reason).
# W779 CORRECTION. The first version listed `fib [1,1,2,3]` as a nine-level arm.
# A DUPLICATE MAGNITUDE IS ONE LEVEL, NOT TWO: as a set that alphabet has SEVEN
# levels, fails S1 (7 is not a power of three), and had no business in a
# comparison whose whole control is "nine levels everywhere, so cardinality is
# held fixed". The script asserted its control in a docstring and violated it in
# an arm. `assert_same_cardinality` below is that control made executable.
#
# The four S6 survivors at nine levels, and the three ladders that fail S6, so
# the comparison spans the filter rather than sitting on one side of it.
ARMS = [
    ("linear 9",  [1., 2., 3., 4.],   2.55),   # S6 ok, 4 < 6
    ("1,2,3,5",   [1., 2., 3., 5.],   2.42),   # S6 ok, 5 < 6
    ("1,2,3,6",   [1., 2., 3., 6.],   2.27),   # S6 ok, 6 = 6, the boundary
    ("1,2,4,7",   [1., 2., 4., 7.],   2.19),   # S6 ok, 7 = 7, the boundary
    ("dyadic",    [1., 2., 4., 8.],   2.19),   # S6 FAILS, 8 > 7
    ("base 3",    [1., 3., 9., 27.],  1.49),   # S6 FAILS, 27 > 13
    ("base 4",    [1., 4., 16., 64.], 1.03),   # S6 FAILS, 64 > 21
]


def assert_same_cardinality(arms):
    """The control the docstring claimed and the code did not check.

    T286 measured alphabet SIZE at +0.844 pp and SHAPE at +0.085 -- an order of
    magnitude apart -- so an arm with the wrong cardinality does not add noise,
    it adds the larger effect under the smaller effect's name.
    """
    sizes = {}
    for name, pos, _ in arms:
        sizes[name] = 2 * len(set(pos)) + 1
    uniq = set(sizes.values())
    if len(uniq) != 1:
        raise SystemExit(f"GUARD: arms differ in level count -- {sizes}. "
                         f"This comparison measures SIZE, not SHAPE.")
    return uniq.pop()


def levels(pos):
    return np.array(sorted([-p for p in pos] + [0.0] + list(pos)), dtype=np.float32)


def masks(n_in, n_out, F, rng):
    return np.stack([rng.choice(n_in, size=min(F, n_in), replace=False)
                     for _ in range(n_out)])


def q(W, lv):
    """BitNet absmean scaling, so the alphabet's SCALE is normalised away."""
    s = np.mean(np.abs(W)) / (np.mean(np.abs(lv[lv != 0])) + 1e-9) + 1e-9
    return lv[np.argmin(np.abs(W[..., None] / s - lv[None, None, :]), axis=-1)] * s


def fwd(X, idx, Q):
    return np.einsum('nof,of->no', X[:, idx], Q)


def run(Xtr, ytr, Xte, yte, lv, seed, F=3, L=3, hidden=64, out_fanin=64,
        epochs=8, lr=0.05, thr=2.0, bs=256, scale_thr=False):
    """scale_thr: T412c. The confound this run was built to separate.

    With a FIXED threshold the alphabet's dynamic range (8:1 dyadic to 64:1 base
    4) decides how many neurons ever cross it, and T207 established that a fixed
    threshold is exactly what makes an alphabet's scale matter. Normalising each
    pre-activation by its own std before the threshold removes the scale and
    leaves only the SHAPE -- which is what bases.py already does on the dense
    stand. If the UNSW inversion survives this, it is the alphabet; if it
    vanishes, it was the trainer.
    """
    rng = np.random.default_rng(seed)
    sizes = [Xtr.shape[1]] + [hidden] * (L - 1) + [1]
    idxs, Ws = [], []
    for li in range(L):
        f = out_fanin if li == L - 1 else F
        idxs.append(masks(sizes[li], sizes[li + 1], f, rng))
        Ws.append(rng.normal(0, 1 / np.sqrt(idxs[-1].shape[1]),
                             idxs[-1].shape).astype(np.float32))
    def nrm(a):
        return a / (a.std() + 1e-6) * thr if scale_thr else a
    act = lambda a: np.tanh(nrm(a) - thr) * 0.5 + np.tanh(nrm(a) + thr) * 0.5
    dact = lambda a: 0.5 * (1 - np.tanh(nrm(a) - thr) ** 2) \
                   + 0.5 * (1 - np.tanh(nrm(a) + thr) ** 2)
    for _ in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, max(len(Xtr) - bs, 1), bs):
            b = perm[i:i + bs]
            x, y = Xtr[b], ytr[b][:, None]
            Qs = [q(W, lv) for W in Ws]
            hs, pre = [x], []
            for li in range(L):
                a = fwd(hs[-1], idxs[li], Qs[li])
                pre.append(a)
                hs.append(act(a) if li < L - 1 else a)
            p = 1 / (1 + np.exp(-np.clip(hs[-1], -30, 30)))
            g = (p - y) / len(b)
            for li in range(L - 1, -1, -1):
                gW = np.einsum('no,nof->of', g, hs[li][:, idxs[li]])
                if li > 0:
                    gprev = np.zeros_like(hs[li])
                    for o in range(idxs[li].shape[0]):
                        np.add.at(gprev, (slice(None), idxs[li][o]),
                                  g[:, o:o + 1] * Qs[li][o][None, :])
                    g = gprev * dact(pre[li - 1])
                Ws[li] -= lr * gW
    Qs = [q(W, lv) for W in Ws]
    h = Xte
    for li in range(L):
        a = fwd(h, idxs[li], Qs[li])
        h = act(a) if li < L - 1 else a
    return float(np.mean((h.ravel() > 0) == (yte > 0.5)))


def load(path):
    d = np.load(path)
    tr, te = d["train"], d["test"]
    return (tr[:, :-1].astype(np.float32) * 2 - 1, tr[:, -1].astype(np.float32),
            te[:, :-1].astype(np.float32) * 2 - 1, te[:, -1].astype(np.float32))


if __name__ == "__main__":
    G, out_path = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 8
    SCALE = len(sys.argv) > 4 and sys.argv[4] == "scale"
    print(f"  threshold: {'SCALED per arm (T412c control)' if SCALE else 'FIXED at 2.0'}")
    K = assert_same_cardinality(ARMS)
    print(f"  cardinality control: every arm has {K} levels")
    res = {}
    for tname, fn in (("UNSW", "unsw.npz"), ("Fashion", "fashion_bin.npz")):
        Xtr, ytr, Xte, yte = load(f"{G}/{fn}")
        print(f"\n  === {tname} ===  sparse, fan-in 3, L=3, {seeds} seeds", flush=True)
        res[tname] = {}
        for name, pos, ef in ARMS:
            lv = levels(pos)
            # W776: PRINT THE ALPHABET ACTUALLY USED. A parameterisation that
            # silently did nothing once reported a run identical to baseline.
            t0 = time.time()
            a = [run(Xtr, ytr, Xte, yte, lv, 1000 + s, scale_thr=SCALE)
                 for s in range(seeds)]
            res[tname][name] = a
            m, sd = float(np.mean(a)) * 100, float(np.std(a, ddof=1)) * 100
            print(f"  {name:10s} eff.fan-in {ef:.2f}  lv={list(lv)}  "
                  f"{m:6.2f}% +-{sd:4.2f}  ({time.time()-t0:.0f}s)", flush=True)
            json.dump(res, open(out_path, "w"), indent=1)   # per-item persistence

    print("\n  === verdict against the registered forecast ===")
    for tname in res:
        lin = np.mean(res[tname]["linear 9"]) * 100
        b4 = np.mean(res[tname]["base 4"]) * 100
        gap = lin - b4
        v = ("CONFIRMED" if gap >= 3 else
             "REFUTED" if gap <= 1 else "PARTIAL")
        print(f"  {tname}: linear 9 {lin:.2f}% - base 4 {b4:.2f}% = {gap:+.2f} pp  -> {v}")
