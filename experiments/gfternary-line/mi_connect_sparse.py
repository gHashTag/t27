"""W780/T430: connectivity chosen by mutual information, not by a coin.

T428b left 7.94 pp against the field's 4.79 with nothing remaining on
PolyLUT-Add's stated list. One named candidate remains: CONNECTIVITY QUALITY.
SparseLUT (arXiv:2503.12829) learns the sparsity mask and closes 2.13 pp on MNIST
HDR(D=1) over PolyLUT's random mask -- and the baseline everyone uses, including
this stand, is `RandomFixedSparsityMask2D`.

This project already measured the ingredient. T376: per-feature mutual information
with the label predicts the sparse penalty at rho = -0.902 out of sample. It has
only ever been used to PREDICT. Here it CHOOSES: the first layer samples its
fan-in weighted by MI instead of uniformly.

REGISTERED FORECAST (T44): MI-weighted connectivity closes AT LEAST 1.5 pp of the
7.94, i.e. lands at or below 6.4, on the grounds that SparseLUT's learned masks
close 2.13 pp of the same kind and MI is a cheaper approximation to the same
quantity. If it closes less than 0.5 pp, MI ranks features but does not select
them, and T376's predictor is confirmed as diagnostic only -- which is exactly
what T376 itself said about calibration, so that outcome is informative too.

CONTROL, and it is the one that matters: an ANTI-MI arm that samples by INVERSE
MI. If MI-weighted and anti-MI perform the same, the sampling weight is doing
nothing and any gain in the MI arm was seed noise.
"""
import json
import sys
import time

import numpy as np

sys.path.insert(0, "/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
from bn_sparse import run_bn
from fanin_accuracy import levels, load


def feature_mi(X, y, eps=1e-12):
    """Per-feature mutual information with a binary label, X in {-1,+1}.

    Closed form for two binary variables -- no estimator, no binning, no
    hyper-parameter to get wrong.
    """
    n = len(y)
    py = np.array([1 - y.mean(), y.mean()])
    xb = (X > 0).astype(np.int8)
    mi = np.zeros(X.shape[1], dtype=np.float64)
    for xv in (0, 1):
        mx = (xb == xv)
        px = mx.mean(0)
        for yv in (0, 1):
            sel = (y > 0.5) if yv else (y <= 0.5)
            pxy = (mx[sel]).sum(0) / n
            d = pxy / (px * py[yv] + eps)
            mi += np.where(pxy > 0, pxy * np.log2(np.maximum(d, eps)), 0.0)
    return mi


def masks_weighted(n_in, n_out, F, rng, w):
    """Sample fan-in WITHOUT replacement, weighted by w. Falls back to uniform
    for any layer whose input is not the feature vector."""
    p = np.asarray(w, dtype=np.float64)
    p = np.maximum(p, 0)
    if p.sum() <= 0:
        p = np.ones(n_in)
    p = p / p.sum()
    return np.stack([rng.choice(n_in, size=min(F, n_in), replace=False, p=p)
                     for _ in range(n_out)])


if __name__ == "__main__":
    G, out = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 5
    lv = levels([1., 2., 4., 8.])
    Xtr, ytr, Xte, yte = load(f"{G}/unsw.npz")
    cut = int(len(Xtr) * 0.85)
    Xva, yva = Xtr[cut:], ytr[cut:]
    Xtr, ytr = Xtr[:cut], ytr[:cut]

    mi = feature_mi(Xtr, ytr)
    print(f"  UNSW n={Xtr.shape[1]} features, {seeds} seeds, H=256, fan-in 3, L=3")
    print(f"  MI: min {mi.min():.4f}  median {np.median(mi):.4f}  max {mi.max():.4f}"
          f"  zero-MI features {int((mi <= 1e-9).sum())}")
    print(f"  dense 89.62%.  BN+qact stand 81.68% (T428).  Field random-mask gap 4.79\n")

    import fanin_accuracy as FA
    orig = FA.masks
    res = {}

    def with_weight(w, tag):
        """Patch the mask sampler for the FIRST layer only; deeper layers keep
        uniform sampling, because MI is a property of the input features."""
        state = {"first": True}

        def patched(n_in, n_out, F, rng):
            if state["first"] and w is not None and n_in == len(w):
                state["first"] = False
                return masks_weighted(n_in, n_out, F, rng, w)
            return orig(n_in, n_out, F, rng)
        FA.masks = patched
        import bn_sparse
        bn_sparse.masks = patched
        t0 = time.time()
        acc = []
        for s in range(seeds):
            state["first"] = True
            acc.append(run_bn(Xtr, ytr, Xva, yva, Xte, yte, lv, 1000 + s,
                              hidden=256, use_bn=True)[0])
        FA.masks = orig
        bn_sparse.masks = orig
        m = np.mean(acc) * 100
        res[tag] = acc
        print(f"  {tag:26s} {m:6.2f} +-{np.std(acc, ddof=1)*100:4.2f}"
              f"  penalty {89.62-m:5.2f}  ({time.time()-t0:.0f}s)", flush=True)
        json.dump(res, open(out, "w"), indent=1)
        return m

    m_rand = with_weight(None, "random (baseline)")
    m_mi = with_weight(mi, "MI-weighted")
    inv = mi.max() - mi + 1e-6
    m_anti = with_weight(inv, "ANTI-MI (control)")

    a = np.array(res["MI-weighted"]) - np.array(res["random (baseline)"])
    t = a.mean() / (a.std(ddof=1) / np.sqrt(len(a)) + 1e-12) * 1
    b = np.array(res["MI-weighted"]) - np.array(res["ANTI-MI (control)"])
    t2 = b.mean() / (b.std(ddof=1) / np.sqrt(len(b)) + 1e-12)
    closed = m_mi - m_rand
    v = "CONFIRMED" if closed >= 1.5 else ("REFUTED" if closed < 0.5 else "PARTIAL")
    print(f"\n  MI vs random  {closed:+.2f} pp  t={t:+.2f}")
    print(f"  MI vs ANTI-MI {(m_mi-m_anti):+.2f} pp  t={t2:+.2f}"
          f"   {'sampling weight DOES something' if abs(t2) > 2.78 else 'weight does nothing -- gain was noise'}")
    print(f"  residual {89.62-m_mi:+.2f} pp vs field 4.79 -> {v}")
