"""W784/T447: re-measure the Nine-Rung Law on a stand that normalises.

T286 (alphabet SIZE worth +0.844 pp, z=33.9) and T288 (the Nine-Rung Law: no step
above nine levels is significant on any task) are two of this programme's
foundations. Both were produced by experiments/gfternary-line/train_ladder.py.

That file has NO normalisation. `run()` takes thr=2.0 and applies it to a raw
pre-activation; `grep std()` over the file returns nothing.

T413c then proved that a fixed threshold does not merely lower every arm -- it
REORDERS them, flipping a measured correlation from -0.971 to +0.956. So the two
foundations sit on the stand later shown to manufacture orderings.

THE MECHANISM PREDICTS A SPECIFIC DIRECTION, and it is why this is worth a wave.
A larger alphabet has a wider spread of weight magnitudes; against a FIXED
threshold, wider spread means more neurons cross it at all. So part of the
measured "cardinality effect" may be the threshold, not the cardinality.

REGISTERED FORECAST (T44):
  (a) The cardinality effect SHRINKS under normalisation. T286 measured 3->9 at
      +0.844 pp pooled; predicted under 0.5 pp on the normalised stand.
  (b) The Nine-Rung CEILING survives: no step above 9 levels is significant.
  REFUTATION: if the 3->9 gain holds at or above 0.844 the mechanism is wrong and
  T286 stands unmodified. If the ceiling MOVES -- saturation at 5, or 3 tying 9 --
  then T288 is not a law but a property of an unnormalised trainer, and every
  document calling it the Nine-Rung Law must be corrected.

Both stands are run here, same seeds, same data, one flag apart, so the
difference IS the threshold and nothing else.
"""
import json
import sys
import time

import numpy as np

RUNGS = [("3", [1.]), ("5", [1., 2.]), ("7", [1., 2., 4.]), ("9", [1., 2., 4., 8.]),
         ("11", [1., 2., 4., 8., 16.]), ("13", [1., 2., 4., 8., 16., 32.])]


def levels(pos):
    return np.array(sorted([-p for p in pos] + [0.0] + list(pos)), dtype=np.float32)


def quantise(W, lv):
    s = np.mean(np.abs(W)) / (np.mean(np.abs(lv[lv != 0])) + 1e-9) + 1e-9
    idx = np.argmin(np.abs(W[..., None] / s - lv[None, None, :]), axis=-1)
    return lv[idx] * s


def run(lv, Xtr, ytr, Xte, yte, seed, normalise, hidden=64, epochs=8, lr=0.05,
        thr=2.0, bs=256):
    """train_ladder.run(), with the ONE line that separates the two stands."""
    rng = np.random.default_rng(seed)
    n_in = Xtr.shape[1]
    W1 = rng.normal(0, 1 / np.sqrt(n_in), (n_in, hidden)).astype(np.float32)
    W2 = rng.normal(0, 1 / np.sqrt(hidden), (hidden, 1)).astype(np.float32)
    for _ in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, max(len(Xtr) - bs, 1), bs):
            b = perm[i:i + bs]
            x, y = Xtr[b], ytr[b][:, None]
            Q1, Q2 = quantise(W1, lv), quantise(W2, lv)
            a1 = x @ Q1
            sd = a1.std() + 1e-6
            if normalise:
                a1 = a1 / sd * thr
            h = np.tanh(a1 - thr) * 0.5 + np.tanh(a1 + thr) * 0.5
            p = 1 / (1 + np.exp(-np.clip(h @ Q2, -30, 30)))
            g = (p - y) / len(b)
            gW2 = h.T @ g
            ga = (g @ Q2.T) * (0.5 * (1 - np.tanh(a1 - thr) ** 2)
                               + 0.5 * (1 - np.tanh(a1 + thr) ** 2))
            if normalise:
                ga = ga / sd * thr
            W1 -= lr * (x.T @ ga)
            W2 -= lr * gW2
    Q1, Q2 = quantise(W1, lv), quantise(W2, lv)
    a1 = Xte @ Q1
    if normalise:
        a1 = a1 / (a1.std() + 1e-6) * thr
    h = np.tanh(a1 - thr) * 0.5 + np.tanh(a1 + thr) * 0.5
    return float(np.mean(((h @ Q2).ravel() > 0) == (yte > 0.5)))


def load(path, n=40000):
    d = np.load(path)
    tr, te = d["train"], d["test"]
    rng = np.random.default_rng(0)
    idx = rng.permutation(len(tr))[:n]
    return (tr[idx, :-1].astype(np.float32) * 2 - 1, tr[idx, -1].astype(np.float32),
            te[:, :-1].astype(np.float32) * 2 - 1, te[:, -1].astype(np.float32))


if __name__ == "__main__":
    G, out = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 10
    crit = 2.26 if seeds == 10 else 2.05
    res = {}
    for tname, fn in (("UNSW", "unsw.npz"), ("Fashion", "fashion_bin.npz")):
        Xtr, ytr, Xte, yte = load(f"{G}/{fn}")
        res[tname] = {}
        for tag, norm in (("FIXED thr (T286/T288 stand)", False),
                          ("NORMALISED", True)):
            print(f"\n  === {tname} / {tag} ===  {seeds} seeds, dense, hidden=64",
                  flush=True)
            res[tname][tag] = {}
            for rung, pos in RUNGS:
                t0 = time.time()
                a = [run(levels(pos), Xtr, ytr, Xte, yte, 1000 + s, norm)
                     for s in range(seeds)]
                res[tname][tag][rung] = a
                print(f"    {rung:>3} levels  {np.mean(a)*100:6.2f} "
                      f"+-{np.std(a, ddof=1)*100:4.2f}  ({time.time()-t0:.0f}s)",
                      flush=True)
                json.dump(res, open(out, "w"), indent=1)

    print("\n  === THE TWO FORECASTS ===")
    for tname in res:
        for tag in res[tname]:
            r = res[tname][tag]
            base = np.array(r["3"])
            d9 = (np.array(r["9"]) - base) * 100
            t9 = d9.mean() / (d9.std(ddof=1) / np.sqrt(len(d9)) + 1e-12)
            last = None
            for a_, b_ in (("9", "11"), ("11", "13")):
                dd = (np.array(r[b_]) - np.array(r[a_])) * 100
                tt_ = dd.mean() / (dd.std(ddof=1) / np.sqrt(len(dd)) + 1e-12)
                if abs(tt_) > crit:
                    last = b_
            print(f"    {tname:8s} {tag:28s} 3->9 {d9.mean():+6.2f} pp t={t9:+6.2f}"
                  f" {'ЗНАЧИМО' if abs(t9) > crit else 'ns':8s}"
                  f"  step above 9 significant: {last or 'none'}")
