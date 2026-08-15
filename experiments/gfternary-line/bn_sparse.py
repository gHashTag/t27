"""W779/T422: give the sparse stand what the field's baselines actually train with.

T420 calibrated this project's 12-13 pp sparse penalty against published work:
SparseLUT Table IV shows 4.79 pp for the COMPARABLE configuration (fan-in 6,
random fixed mask, one LUT per neuron) and NeuraLUT-Assemble reaches <=1 pp. Ours
is 2.5x the random-mask figure, so the excess is the instrument.

T420b named the difference, verbatim from PolyLUT-Add: "Each layer's inputs and
outputs are batch normalized and quantized using Brevitas quantized activation
functions, which utilize learned scaling factors."

This stand had NONE of that. W778 added only a per-batch variance rescale of the
pre-activation -- no mean centring, no learned parameters, and only on one side.
That partial measure was already worth 17-21 pp (T413), which is the reason to
expect the full version to matter more.

REGISTERED FORECAST (T44), before the run: full per-layer batch normalisation
with a learned scale and shift closes AT LEAST 5 pp of the ~13, landing at or
below 8 pp and within reach of the field's 4.79. If it closes less than 2 pp the
instrument gap is something else again and this hypothesis is dead. Between 2 and
5, report the split and do not round.
"""
import json
import sys
import time

import numpy as np

sys.path.insert(0, "/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
from fanin_accuracy import fwd, levels, load, masks, q


def run_bn(Xtr, ytr, Xva, yva, Xte, yte, lv, seed, F=3, L=3, hidden=256,
           out_fanin=64, epochs=30, lr=0.05, thr=2.0, bs=256, patience=6,
           use_bn=True, momentum=0.1):
    """Sparse LUT-style net with real per-layer BatchNorm and learned scale.

    gamma/beta are trained by gradient like any other parameter; running mean and
    variance are kept for inference, as in every framework the baselines use.
    """
    rng = np.random.default_rng(seed)
    sizes = [Xtr.shape[1]] + [hidden] * (L - 1) + [1]
    idxs, Ws, gam, bet, rm, rv = [], [], [], [], [], []
    for li in range(L):
        f = out_fanin if li == L - 1 else F
        idxs.append(masks(sizes[li], sizes[li + 1], f, rng))
        Ws.append(rng.normal(0, 1 / np.sqrt(idxs[-1].shape[1]),
                             idxs[-1].shape).astype(np.float32))
        gam.append(np.ones(sizes[li + 1], dtype=np.float32))
        bet.append(np.zeros(sizes[li + 1], dtype=np.float32))
        rm.append(np.zeros(sizes[li + 1], dtype=np.float32))
        rv.append(np.ones(sizes[li + 1], dtype=np.float32))

    def bn_fwd(a, li, train):
        if not use_bn:
            return a, None
        if train:
            m, v = a.mean(0), a.var(0) + 1e-5
            rm[li][:] = (1 - momentum) * rm[li] + momentum * m
            rv[li][:] = (1 - momentum) * rv[li] + momentum * v
        else:
            m, v = rm[li], rv[li] + 1e-5
        xh = (a - m) / np.sqrt(v)
        return gam[li] * xh + bet[li], (xh, np.sqrt(v))

    act = lambda z: np.tanh(z - thr) * 0.5 + np.tanh(z + thr) * 0.5
    dact = lambda z: 0.5 * (1 - np.tanh(z - thr) ** 2) + 0.5 * (1 - np.tanh(z + thr) ** 2)

    def evaluate(X, y):
        Qs = [q(W, lv) for W in Ws]
        h = X
        for li in range(L):
            z, _ = bn_fwd(fwd(h, idxs[li], Qs[li]), li, False)
            h = act(z) if li < L - 1 else z
        return float(np.mean((h.ravel() > 0) == (y > 0.5)))

    p1 = ytr.mean()
    wp, wn = 0.5 / p1, 0.5 / (1 - p1)
    best_va, best, bad = -1.0, None, 0
    for ep in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, max(len(Xtr) - bs, 1), bs):
            b = perm[i:i + bs]
            x, y = Xtr[b], ytr[b][:, None]
            Qs = [q(W, lv) for W in Ws]
            hs, pre, cache = [x], [], []
            for li in range(L):
                a = fwd(hs[-1], idxs[li], Qs[li])
                z, c = bn_fwd(a, li, True)
                pre.append(z)
                cache.append(c)
                hs.append(act(z) if li < L - 1 else z)
            p = 1 / (1 + np.exp(-np.clip(hs[-1], -30, 30)))
            g = np.where(y > 0.5, wp, wn) * (p - y) / len(b)
            for li in range(L - 1, -1, -1):
                if use_bn:
                    xh, sd = cache[li]
                    gam[li] -= lr * (g * xh).sum(0)
                    bet[li] -= lr * g.sum(0)
                    n = len(b)
                    gx = g * gam[li] / sd
                    g = gx - gx.mean(0) - xh * (gx * xh).mean(0)
                gW = np.einsum('no,nof->of', g, hs[li][:, idxs[li]])
                if li > 0:
                    gp = np.zeros_like(hs[li])
                    for o in range(idxs[li].shape[0]):
                        np.add.at(gp, (slice(None), idxs[li][o]),
                                  g[:, o:o + 1] * Qs[li][o][None, :])
                    g = gp * dact(pre[li - 1])
                Ws[li] -= lr * gW
        v = evaluate(Xva, yva)
        if v > best_va:
            best_va, bad = v, 0
            best = ([W.copy() for W in Ws], [x.copy() for x in gam],
                    [x.copy() for x in bet], [x.copy() for x in rm], [x.copy() for x in rv])
        else:
            bad += 1
            if bad >= patience:
                break
    Ws2, g2, b2, m2, v2 = best
    Ws[:], gam[:], bet[:], rm[:], rv[:] = Ws2, g2, b2, m2, v2
    return evaluate(Xte, yte), ep + 1


if __name__ == "__main__":
    G, out = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 5
    lv = levels([1., 2., 4., 8.])
    Xtr, ytr, Xte, yte = load(f"{G}/unsw.npz")
    cut = int(len(Xtr) * 0.85)
    Xva, yva = Xtr[cut:], ytr[cut:]
    Xtr, ytr = Xtr[:cut], ytr[:cut]
    print(f"  UNSW sparse fan-in 3, L=3, H=256, {seeds} seeds")
    print(f"  dense reference 89.62%.  Field: SparseLUT random-mask gap 4.79 pp (MNIST)")
    res = {}
    for tag, kw in (("no BN  (as measured)", dict(use_bn=False)),
                    ("PER-LAYER BATCH NORM", dict(use_bn=True))):
        t0 = time.time()
        r = [run_bn(Xtr, ytr, Xva, yva, Xte, yte, lv, 1000 + s, **kw) for s in range(seeds)]
        a = [x[0] for x in r]
        res[tag] = a
        m = np.mean(a) * 100
        print(f"  {tag:24s} {m:6.2f}+-{np.std(a, ddof=1)*100:4.2f}  penalty {89.62-m:5.2f}"
              f"  stopped ~ep{int(np.mean([x[1] for x in r]))}  ({time.time()-t0:.0f}s)", flush=True)
        json.dump(res, open(out, "w"), indent=1)
    a0 = np.mean(res["no BN  (as measured)"]) * 100
    a1 = np.mean(res["PER-LAYER BATCH NORM"]) * 100
    closed = a1 - a0
    v = "CONFIRMED" if closed >= 5 else ("REFUTED" if closed < 2 else "PARTIAL")
    print(f"\n  batch norm closed {closed:+.2f} pp; residual {89.62-a1:+.2f} pp -> {v}")
