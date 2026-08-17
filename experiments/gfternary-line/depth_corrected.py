"""W785/T450: re-measure DEPTH and FAN-IN on the corrected stand.

T414b declared the residual "not architectural" on the strength of two nulls:
fan-in 3->6 buys 0.68 pp, depth 3->4 costs 0.99, both inside one sd. Both were
measured on a stand that had only a variance rescale -- BEFORE per-layer
BatchNorm (T422), before ternary activations (T428), before balanced coverage
(T439). Between them those three moved the stand 11.95 -> 6.90 pp.

A structural null measured on a trainer that could not train is not a structural
null. And the field runs FOUR TO SIX layers where this stand runs three.

REGISTERED FORECAST (T44): on the corrected stand depth now HELPS -- L=3 to L=5
buys AT LEAST 1.0 pp, because the earlier null was a trainer that could not carry
gradient through depth. Under 0.3 pp and depth is genuinely exhausted, T414b
stands as re-measured, and the residual is neither connectivity (T446) nor depth.

Everything corrected is on: per-layer BatchNorm with learned scale, ternary
quantised activations through a learned scale with STE, balanced-coverage
connectivity, class weighting, validation early stopping.
"""
import json
import sys
import time

import numpy as np

sys.path.insert(0, "/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
from balanced_connect import balanced_masks
from fanin_accuracy import levels, load
from qact_sparse import run_qact

if __name__ == "__main__":
    G, out = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 8
    lv = levels([1., 2., 4., 8.])
    Xtr, ytr, Xte, yte = load(f"{G}/unsw.npz")
    cut = int(len(Xtr) * 0.85)
    Xva, yva = Xtr[cut:], ytr[cut:]
    Xtr, ytr = Xtr[:cut], ytr[:cut]
    n = Xtr.shape[1]
    print(f"  UNSW n={n}, H=256, {seeds} seeds, CORRECTED stand")
    print(f"  (per-layer BN + ternary quantised activations + balanced coverage)")
    print(f"  dense 89.62.  best so far 82.72 at L=3, F=3 (T446)\n")
    print(f"  {'F':>3}{'L':>3}{'accuracy':>12}{'penalty':>10}")

    import bn_sparse
    import fanin_accuracy as FA
    orig = FA.masks
    res = {}
    for F in (3, 6):
        for L in (2, 3, 4, 5, 6):
            st = {"f": True}

            def patched(a, b, c, r):
                if st["f"] and a == n:
                    st["f"] = False
                    return balanced_masks(a, b, c, r, True)
                return orig(a, b, c, r)
            FA.masks = patched
            bn_sparse.masks = patched
            t0 = time.time()
            acc = []
            for s in range(seeds):
                st["f"] = True
                acc.append(run_qact(Xtr, ytr, Xva, yva, Xte, yte, lv, 1000 + s,
                                    F=F, L=L, hidden=256, act_levels=3)[0])
            FA.masks = orig
            bn_sparse.masks = orig
            m = np.mean(acc) * 100
            res[f"F{F}L{L}"] = acc
            print(f"  {F:>3}{L:>3}{m:>10.2f} +-{np.std(acc, ddof=1)*100:4.2f}"
                  f"{89.62-m:>10.2f}   ({time.time()-t0:.0f}s)", flush=True)
            json.dump(res, open(out, "w"), indent=1)

    crit = 2.36 if seeds == 8 else 2.05
    print("\n  === THE FORECAST ===")
    for F in (3, 6):
        a3 = np.array(res[f"F{F}L3"])
        a5 = np.array(res[f"F{F}L5"])
        d = (a5 - a3) * 100
        t = d.mean() / (d.std(ddof=1) / np.sqrt(len(d)) + 1e-12)
        v = "CONFIRMED" if d.mean() >= 1.0 else ("REFUTED" if d.mean() < 0.3 else "PARTIAL")
        print(f"    F={F}  L3->L5 {d.mean():+6.2f} pp  t={t:+6.2f}"
              f"  {'ЗНАЧИМО' if abs(t) > crit else 'ns':8s} -> {v}")
    best = max(np.mean(v) * 100 for v in res.values())
    bk = [k for k in res if np.mean(res[k]) * 100 == best][0]
    print(f"    best {best:.2f} at {bk}, residual {89.62-best:+.2f} pp vs field 4.79")
