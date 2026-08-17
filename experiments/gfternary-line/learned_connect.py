"""W782/T441: LEARNED connectivity -- the last named item the field has and we do not.

T439c left the stand at 7.05 pp against the field's 4.79, with nothing from
PolyLUT-Add's stated method untried. What remains is not in that method at all:
SparseLUT (arXiv:2503.12829) LEARNS the sparsity mask and closes 2.13 pp over the
random mask everyone else uses -- and PolyLUT itself calls its variant
"hardware-aware structured pruning".

The faithful cheap version, in three phases:
  1. train with a WIDE first-layer fan-in, enough to see which inputs matter;
  2. prune each neuron to its top-F inputs by |w|;
  3. retrain from scratch under that fixed fan-in-F mask.

REGISTERED FORECAST (T44): learned connectivity closes AT LEAST 1.5 pp of the
7.05, landing at or below 5.5 and within 1 pp of the field's 4.79. Below 0.5 pp
the hypothesis is dead and the residual is something nobody in the field has
named either.

THE CONTROL DECIDES, and T430 is why: an ANTI-LEARNED arm keeps the SMALLEST
weights instead of the largest. It pays the identical wide-training cost, so
learned-vs-anti isolates the SELECTION RULE from the extra training the selection
phase buys. Comparing learned against a plain random mask cannot do that, and any
gain read from that comparison alone would be confounded with training budget.
"""
import json
import sys
import time

import numpy as np

sys.path.insert(0, "/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
from balanced_connect import balanced_masks
from bn_sparse import run_bn
from fanin_accuracy import fwd, levels, load, masks, q


def learn_mask(Xtr, ytr, lv, rng, n_out, F, F_wide=32, epochs=3, lr=0.05, bs=256,
               keep="largest"):
    """Phase 1+2: train wide, then keep F inputs per neuron by |w|.

    One hidden layer straight to a scalar is enough to rank inputs; the point is
    the RANKING, not the accuracy of the probe.
    """
    n_in = Xtr.shape[1]
    wide = masks(n_in, n_out, min(F_wide, n_in), rng)
    W1 = rng.normal(0, 1 / np.sqrt(wide.shape[1]), wide.shape).astype(np.float32)
    W2 = rng.normal(0, 1 / np.sqrt(n_out), (n_out, 1)).astype(np.float32)
    p1 = ytr.mean()
    wp, wn = 0.5 / p1, 0.5 / (1 - p1)
    for _ in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, max(len(Xtr) - bs, 1), bs):
            b = perm[i:i + bs]
            x, y = Xtr[b], ytr[b][:, None]
            Q1 = q(W1, lv)
            a = fwd(x, wide, Q1)
            a = a / (a.std() + 1e-6) * 2.0
            h = np.tanh(a - 2.0) * 0.5 + np.tanh(a + 2.0) * 0.5
            o = h @ W2
            p = 1 / (1 + np.exp(-np.clip(o, -30, 30)))
            g = np.where(y > 0.5, wp, wn) * (p - y) / len(b)
            W2 -= lr * (h.T @ g)
            ga = (g @ W2.T) * (0.5 * (1 - np.tanh(a - 2.0) ** 2)
                               + 0.5 * (1 - np.tanh(a + 2.0) ** 2))
            W1 -= lr * np.einsum('no,nof->of', ga, x[:, wide])
    mag = np.abs(W1)
    order = np.argsort(mag, axis=1)
    pick = order[:, :F] if keep == "smallest" else order[:, -F:]
    return np.take_along_axis(wide, pick, axis=1)


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
    print(f"  dense 89.62.  balanced 82.57 (T439).  FIELD random-mask gap 4.79\n")

    import bn_sparse
    import fanin_accuracy as FA
    orig = FA.masks
    res = {}

    def arm(tag, maker):
        state = {"first": True}

        def patched(n_in, n_out, F, rng):
            if state["first"] and n_in == n:
                state["first"] = False
                return maker(n_in, n_out, F, rng)
            return orig(n_in, n_out, F, rng)
        FA.masks = patched
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

    m_bal = arm("balanced (T439 best)",
                lambda a, b, c, r: balanced_masks(a, b, c, r, True))
    m_lrn = arm("LEARNED (top |w|)",
                lambda a, b, c, r: learn_mask(Xtr, ytr, lv, r, b, c, keep="largest"))
    m_ant = arm("anti-learned (control)",
                lambda a, b, c, r: learn_mask(Xtr, ytr, lv, r, b, c, keep="smallest"))

    def tt(x, y):
        d = (np.array(res[x]) - np.array(res[y])) * 100
        return d.mean(), d.mean() / (d.std(ddof=1) / np.sqrt(len(d)) + 1e-12)
    crit = 2.20 if seeds == 12 else 2.05
    d1, t1 = tt("LEARNED (top |w|)", "balanced (T439 best)")
    d2, t2 = tt("LEARNED (top |w|)", "anti-learned (control)")
    print(f"\n  learned vs balanced  {d1:+.2f} pp  t={t1:+.2f}"
          f"  {'ЗНАЧИМО' if abs(t1) > crit else 'ns'}   (confounded with the probe phase)")
    print(f"  learned vs ANTI      {d2:+.2f} pp  t={t2:+.2f}"
          f"  {'ЗНАЧИМО' if abs(t2) > crit else 'ns'}   (isolates the SELECTION RULE)")
    closed = m_lrn - m_bal
    v = "CONFIRMED" if closed >= 1.5 else ("REFUTED" if closed < 0.5 else "PARTIAL")
    print(f"  residual {89.62-m_lrn:+.2f} pp vs field 4.79 -> {v}")
