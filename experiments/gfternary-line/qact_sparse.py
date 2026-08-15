"""W780/T428: quantised activations with learned scales -- the last item on the list.

T420b transcribed what the field's baselines train with, verbatim from PolyLUT-Add:
"Each layer's inputs and outputs are batch normalized and quantized using Brevitas
quantized activation functions, which utilize learned scaling factors."

W779 implemented the first half (per-layer BatchNorm, T422) and closed the gap from
11.95 to 9.39 pp against the field's 4.79. THE SECOND HALF WAS NEVER TRIED: this
stand quantises WEIGHTS only and leaves activations in float, while every baseline
quantises activations too -- which is not an optimisation but a faithfulness
requirement, because the deployed LUT emits a TRIT, not a float.

Implemented here: after BatchNorm, the activation is quantised to a small alphabet
through a LEARNED scale, with a straight-through estimator and a clipped gradient
(the standard LSQ/PACT arrangement). The scale is a trained parameter, one per
layer, exactly as "learned scaling factors" describes.

REGISTERED FORECAST (T44): quantised activations close AT LEAST 2 pp of the
remaining 9.39, landing at or below 7.4 and closer to the field's 4.79. If they
close less than 0.5 pp the transcription is complete and the residual is neither
normalisation nor activation quantisation. If accuracy FALLS, note that this is
still the honest configuration -- the deployed network quantises activations
whether or not training does, so a fall is a cost that was previously hidden.
"""
import json
import sys
import time

import numpy as np

sys.path.insert(0, "/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
from bn_sparse import run_bn                      # the BN stand, reused unchanged
from fanin_accuracy import fwd, levels, load, masks, q


def qact(z, scale, n_lv):
    """Quantise to n_lv symmetric levels through a learned scale, STE on the way back.

    Returns (quantised, mask_in_range). The mask is the clipped-gradient support:
    outside it the straight-through estimator passes nothing, which is what stops
    a learned scale from running away.
    """
    half = (n_lv - 1) // 2
    u = z / (scale + 1e-6)
    c = np.clip(u, -half, half)
    return np.round(c) * (scale + 1e-6), (np.abs(u) <= half)


def run_qact(Xtr, ytr, Xva, yva, Xte, yte, lv, seed, F=3, L=3, hidden=256,
             out_fanin=64, epochs=30, lr=0.05, bs=256, patience=6,
             act_levels=3, lr_scale=0.01):
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
    # one learned activation scale per hidden layer, as "learned scaling factors"
    ascale = [np.float32(1.0) for _ in range(L - 1)]

    def bn(a, li, train, mom=0.1):
        if train:
            m, v = a.mean(0), a.var(0) + 1e-5
            rm[li][:] = (1 - mom) * rm[li] + mom * m
            rv[li][:] = (1 - mom) * rv[li] + mom * v
        else:
            m, v = rm[li], rv[li] + 1e-5
        xh = (a - m) / np.sqrt(v)
        return gam[li] * xh + bet[li], (xh, np.sqrt(v))

    def evaluate(X, y):
        Qs = [q(W, lv) for W in Ws]
        h = X
        for li in range(L):
            z, _ = bn(fwd(h, idxs[li], Qs[li]), li, False)
            h = qact(z, ascale[li], act_levels)[0] if li < L - 1 else z
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
            hs, pre, cache, keep = [x], [], [], []
            for li in range(L):
                a = fwd(hs[-1], idxs[li], Qs[li])
                z, c = bn(a, li, True)
                pre.append(z)
                cache.append(c)
                if li < L - 1:
                    hq, m = qact(z, ascale[li], act_levels)
                    hs.append(hq)
                    keep.append(m)
                else:
                    hs.append(z)
            p = 1 / (1 + np.exp(-np.clip(hs[-1], -30, 30)))
            g = np.where(y > 0.5, wp, wn) * (p - y) / len(b)
            for li in range(L - 1, -1, -1):
                xh, sd = cache[li]
                gam[li] -= lr * (g * xh).sum(0)
                bet[li] -= lr * g.sum(0)
                gx = g * gam[li] / sd
                g = gx - gx.mean(0) - xh * (gx * xh).mean(0)
                gW = np.einsum('no,nof->of', g, hs[li][:, idxs[li]])
                if li > 0:
                    gp = np.zeros_like(hs[li])
                    for o in range(idxs[li].shape[0]):
                        np.add.at(gp, (slice(None), idxs[li][o]),
                                  g[:, o:o + 1] * Qs[li][o][None, :])
                    # STE through the quantiser, gradient clipped to the in-range mask
                    ascale[li - 1] -= lr_scale * float((gp * keep[li - 1]).mean())
                    g = gp * keep[li - 1]
                Ws[li] -= lr * gW
        v = evaluate(Xva, yva)
        if v > best_va:
            best_va, bad = v, 0
            best = ([W.copy() for W in Ws], [x.copy() for x in gam],
                    [x.copy() for x in bet], [x.copy() for x in rm],
                    [x.copy() for x in rv], list(ascale))
        else:
            bad += 1
            if bad >= patience:
                break
    Ws[:], gam[:], bet[:], rm[:], rv[:], ascale[:] = best
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
    print(f"  dense 89.62%.  BN-only stand 80.23% (T422).  Field random-mask gap 4.79 pp")
    res = {}
    r = [run_bn(Xtr, ytr, Xva, yva, Xte, yte, lv, 1000 + s, hidden=256, use_bn=True)
         for s in range(seeds)]
    res["BN only (T422)"] = [x[0] for x in r]
    m0 = np.mean(res["BN only (T422)"]) * 100
    print(f"  {'BN only (T422)':34s} {m0:6.2f}  penalty {89.62-m0:5.2f}", flush=True)
    for nlv in (3, 5, 9):
        t0 = time.time()
        r = [run_qact(Xtr, ytr, Xva, yva, Xte, yte, lv, 1000 + s, act_levels=nlv)
             for s in range(seeds)]
        a = [x[0] for x in r]
        tag = f"BN + quantised act, {nlv} levels"
        res[tag] = a
        m = np.mean(a) * 100
        print(f"  {tag:34s} {m:6.2f}  penalty {89.62-m:5.2f}  d={m-m0:+.2f}"
              f"  ({time.time()-t0:.0f}s)", flush=True)
        json.dump(res, open(out, "w"), indent=1)
    best = max(np.mean(v) * 100 for k, v in res.items() if k != "BN only (T422)")
    closed = best - m0
    v = "CONFIRMED" if closed >= 2 else ("REFUTED" if closed < 0.5 else "PARTIAL")
    print(f"\n  quantised activations closed {closed:+.2f} pp; "
          f"residual {89.62-best:+.2f} pp vs field 4.79 -> {v}")
