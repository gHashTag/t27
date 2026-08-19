"""W783/T445: learn the mask JOINTLY, the way SparseLUT actually does it.

T443 refuted a magnitude PROBE and said so plainly: "a negative result about a
cheap proxy is not a negative result about the published method". SparseLUT
(arXiv:2503.12829) learns the sparsity pattern jointly with the weights through a
differentiable relaxation, and reports 2.13 pp over a random mask. This is that
method, not a probe.

  - a score s[i,o] per (input, neuron) pair, trained by gradient;
  - each neuron's forward pass uses a STRAIGHT-THROUGH TOP-K mask over its scores,
    so the hard fan-in-F constraint holds in the forward direction while the
    gradient reaches every score;
  - after training the mask is hardened to the top-F and the network is retrained
    under it, which is the configuration that would actually be built.

REGISTERED FORECAST (T44): joint mask learning closes AT LEAST 1.5 pp over
balanced coverage (82.52), i.e. reaches 84.0 or better and brings the residual
under 5.6 against the field's 4.79. Under 0.5 pp and the whole connectivity axis
is declared closed: three distinct methods will have failed to beat a label-free
rule that costs nothing.

THE CONTROL, and by now it is not optional: an arm with the identical machinery
whose scores are FROZEN at random initialisation. It pays every parameter and
every epoch the learned arm pays and learns nothing about connectivity, so
learned-minus-frozen isolates MASK LEARNING from the extra capacity and training
the mechanism drags in. T430 and T443 are why this arm exists.
"""
import json
import sys
import time

import numpy as np

sys.path.insert(0, "/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
from balanced_connect import balanced_masks
from bn_sparse import run_bn
from fanin_accuracy import levels, load, masks, q


def learn_soft_mask(Xtr, ytr, lv, rng, n_out, F, epochs=4, lr=0.05,
                    lr_score=0.5, bs=256, freeze=False):
    """Joint score+weight training with a straight-through top-F mask."""
    n_in = Xtr.shape[1]
    S = rng.normal(0, 0.1, (n_out, n_in)).astype(np.float32)
    W = rng.normal(0, 1 / np.sqrt(F), (n_out, n_in)).astype(np.float32)
    V = rng.normal(0, 1 / np.sqrt(n_out), (n_out, 1)).astype(np.float32)
    p1 = ytr.mean()
    wp, wn = 0.5 / p1, 0.5 / (1 - p1)
    for _ in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, max(len(Xtr) - bs, 1), bs):
            b = perm[i:i + bs]
            x, y = Xtr[b], ytr[b][:, None]
            # straight-through top-F: hard mask forward, soft scores backward
            keep = np.argsort(-S, axis=1)[:, :F]
            M = np.zeros_like(S)
            np.put_along_axis(M, keep, 1.0, axis=1)
            soft = 1.0 / (1.0 + np.exp(-S))
            Meff = M + (soft - soft)                      # value = M, grad via soft
            Wq = q(W[None, :, :], lv)[0]
            a = x @ (Wq * M).T
            a = a / (a.std() + 1e-6) * 2.0
            h = np.tanh(a - 2.0) * 0.5 + np.tanh(a + 2.0) * 0.5
            p = 1 / (1 + np.exp(-np.clip(h @ V, -30, 30)))
            g = np.where(y > 0.5, wp, wn) * (p - y) / len(b)
            V -= lr * (h.T @ g)
            ga = (g @ V.T) * (0.5 * (1 - np.tanh(a - 2.0) ** 2)
                              + 0.5 * (1 - np.tanh(a + 2.0) ** 2))
            gWM = ga.T @ x                                # (n_out, n_in)
            W -= lr * (gWM * M)
            if not freeze:
                # the score gradient is the weight gradient seen through the STE:
                # a score rises when using that input would have reduced the loss
                S -= lr_score * (gWM * Wq) * soft * (1 - soft)
    return np.argsort(-S, axis=1)[:, :F]


if __name__ == "__main__":
    G, out = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 12
    lv = levels([1., 2., 4., 8.])
    Xtr, ytr, Xte, yte = load(f"{G}/unsw.npz")
    cut = int(len(Xtr) * 0.85)
    Xva, yva = Xtr[cut:], ytr[cut:]
    Xtr, ytr = Xtr[:cut], ytr[:cut]
    n = Xtr.shape[1]
    print(f"  UNSW n={n}, H=256, fan-in 3, L=3, {seeds} seeds")
    print(f"  balanced 82.52 (T439).  magnitude probe 82.21 (T443).  FIELD gap 4.79\n")

    import bn_sparse
    import fanin_accuracy as FA
    orig = FA.masks
    res = {}

    def arm(tag, maker):
        st = {"f": True}

        def patched(a, b, c, r):
            if st["f"] and a == n:
                st["f"] = False
                return maker(a, b, c, r)
            return orig(a, b, c, r)
        FA.masks = patched
        bn_sparse.masks = patched
        t0 = time.time()
        acc = []
        for s in range(seeds):
            st["f"] = True
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

    m_bal = arm("balanced (T439)",
                lambda a, b, c, r: balanced_masks(a, b, c, r, True))
    m_soft = arm("JOINT soft mask",
                 lambda a, b, c, r: learn_soft_mask(Xtr, ytr, lv, r, b, c))
    m_frz = arm("frozen scores (control)",
                lambda a, b, c, r: learn_soft_mask(Xtr, ytr, lv, r, b, c, freeze=True))

    def tt(x, y):
        d = (np.array(res[x]) - np.array(res[y])) * 100
        return d.mean(), d.mean() / (d.std(ddof=1) / np.sqrt(len(d)) + 1e-12)
    crit = 2.20 if seeds == 12 else 2.05
    d1, t1 = tt("JOINT soft mask", "balanced (T439)")
    d2, t2 = tt("JOINT soft mask", "frozen scores (control)")
    print(f"\n  joint vs balanced  {d1:+.2f} pp  t={t1:+.2f}"
          f"  {'ЗНАЧИМО' if abs(t1) > crit else 'ns'}")
    print(f"  joint vs FROZEN    {d2:+.2f} pp  t={t2:+.2f}"
          f"  {'ЗНАЧИМО' if abs(t2) > crit else 'ns'}   (isolates MASK LEARNING)")
    v = "CONFIRMED" if d1 >= 1.5 else ("REFUTED" if d1 < 0.5 else "PARTIAL")
    print(f"  residual {89.62-m_soft:+.2f} pp vs field 4.79 -> {v}")
