"""W780/T431: balanced feature coverage -- the clean form of what ANTI-MI did.

T430 found the control beating the treatment: sampling connectivity by INVERSE
mutual information scored 82.64% against MI-weighted 80.75 and random 80.23.
T430a proposed the mechanism -- MI on UNSW is skewed, so weighting by it makes many
neurons redundant while inverse weighting SPREADS THE DRAW.

If that is the mechanism, inverse MI is a clumsy proxy for it and the clean form is
BALANCED COVERAGE: deal the F*H input slots so every feature is used floor or ceil
of F*H/n times, with no feature used twice in one neuron. No labels are consulted,
which also makes it free of any leakage objection.

REGISTERED FORECAST (T44): balanced coverage beats random by >= 1 pp and lands at
or above ANTI-MI's 82.64, because it is the mechanism without the noise. If it
lands BELOW random, T430a's mechanism is wrong and ANTI-MI's win was seed noise
after all -- in which case say so, since T430 already reported nothing significant.

CONTROL: the same balanced deal, but with the per-neuron triples SHUFFLED so
coverage is identical and the grouping is not. If that scores the same, coverage is
the whole story; if it differs, WHICH features share a neuron matters too.
"""
import json
import sys
import time

import numpy as np

sys.path.insert(0, "/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
from bn_sparse import run_bn
from fanin_accuracy import levels, load


def balanced_masks(n_in, n_out, F, rng, regroup=False):
    """Deal F*H slots so every feature is used floor/ceil(F*H/n) times.

    A neuron never reads the same feature twice: slots are drawn from a shuffled
    deck and any collision within a neuron is swapped out against the remaining
    deck, so the coverage guarantee survives the constraint.
    """
    total = F * n_out
    reps = int(np.ceil(total / n_in))
    deck = np.tile(np.arange(n_in), reps)[:total]
    rng.shuffle(deck)
    out = deck.reshape(n_out, F).copy()
    for i in range(n_out):
        seen = set()
        for j in range(F):
            tries = 0
            while out[i, j] in seen and tries < 64:
                k = rng.integers(0, total)
                a, b = divmod(k, F)
                out[i, j], out[a, b] = out[a, b], out[i, j]
                tries += 1
            seen.add(out[i, j])
    if regroup:
        flat = out.reshape(-1)
        rng.shuffle(flat)
        out = flat.reshape(n_out, F)
    return out


if __name__ == "__main__":
    G, out = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 5
    lv = levels([1., 2., 4., 8.])
    Xtr, ytr, Xte, yte = load(f"{G}/unsw.npz")
    cut = int(len(Xtr) * 0.85)
    Xva, yva = Xtr[cut:], ytr[cut:]
    Xtr, ytr = Xtr[:cut], ytr[:cut]
    n = Xtr.shape[1]
    print(f"  UNSW n={n}, H=256, fan-in 3, L=3, {seeds} seeds")
    print(f"  dense 89.62%.  random 80.23 (T422), ANTI-MI 82.64 (T430). Field gap 4.79")
    print(f"  balanced deal: {3*256} slots over {n} features = "
          f"{3*256/n:.2f} uses each\n")

    import fanin_accuracy as FA
    import bn_sparse
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
        print(f"  {tag:28s} {m:6.2f} +-{np.std(acc, ddof=1)*100:4.2f}"
              f"  penalty {89.62-m:5.2f}  ({time.time()-t0:.0f}s)", flush=True)
        json.dump(res, open(out, "w"), indent=1)
        return m

    m_rand = arm("random (baseline)", lambda a, b, c, r: orig(a, b, c, r))
    m_bal = arm("BALANCED coverage", lambda a, b, c, r: balanced_masks(a, b, c, r))
    m_reg = arm("balanced, regrouped", lambda a, b, c, r: balanced_masks(a, b, c, r, True))

    d = np.array(res["BALANCED coverage"]) - np.array(res["random (baseline)"])
    t = d.mean() / (d.std(ddof=1) / np.sqrt(len(d)) + 1e-12)
    d2 = np.array(res["BALANCED coverage"]) - np.array(res["balanced, regrouped"])
    t2 = d2.mean() / (d2.std(ddof=1) / np.sqrt(len(d2)) + 1e-12)
    closed = m_bal - m_rand
    v = ("CONFIRMED" if closed >= 1.0 and m_bal >= 82.64
         else ("REFUTED" if closed < 0 else "PARTIAL"))
    print(f"\n  balanced vs random     {closed:+.2f} pp  t={t:+.2f}"
          f"  {'ЗНАЧИМО' if abs(t) > 2.78 else 'ns'}")
    print(f"  grouping effect        {(m_bal-m_reg):+.2f} pp  t={t2:+.2f}"
          f"  -> {'WHICH features share a neuron matters' if abs(t2) > 2.78 else 'coverage is the whole story'}")
    print(f"  residual {89.62-max(m_bal, m_reg):+.2f} pp vs field 4.79 -> {v}")
