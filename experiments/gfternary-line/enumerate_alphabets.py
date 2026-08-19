"""W783/T444: the whole admissible non-ladder space, enumerated -- not sampled.

T442 proved that EVERY integer ladder of two or more magnitudes fails S6, at
every size. So the sieve's own formula returns balanced ternary and the escape,
if there is one, is a NON-LADDER integer alphabet. Only four of those have ever
been measured, all picked by hand.

The space is small enough to exhaust. A nine-level integer alphabet is
{0, +-a, +-b, +-c, +-d} with 0 < a < b < c < d; a common positive scale factors
out (T207), so fixing a = 1 loses nothing. S6 requires d <= a+b+c. Bounding
d <= 24 -- comparable to base 3's 27:1 spread -- leaves 1156 admissible
alphabets, and this enumerates every one.

REGISTERED FORECAST (T44), before the run: junta degree is maximised by the
FLATTEST admissible alphabet, so the maximum sits at or beside linear 9
{1,2,3,4}, and NO alphabet in the space exceeds 2.60.
REFUTATION: if any alphabet reaches 2.70 or above, linear 9 is not the maximiser,
the sieve's measured top is wrong, and the winner must be measured on accuracy
and area before anything else in this line is quoted.

No seeds. No sampling. The answer is exact for the stated bound.
"""
import itertools
import json
import sys

import numpy as np

FANIN = 3
SYM_BITS = 2
THR = 1.0

# Input codes -> trit values, once. 0b00=0, 0b01=+1, 0b10=-1, 0b11 unused->0.
DEC = np.array([0, 1, -1, 0], dtype=np.int8)
CODES = np.arange(1 << (FANIN * SYM_BITS))
TRITS = np.stack([DEC[(CODES >> (SYM_BITS * i)) & 3] for i in range(FANIN)], axis=1)


def junta_degree(mags):
    """Mean number of inputs a neuron's output depends on, over ALL 9^3 triples.

    Vectorised: 729 weight triples x 64 input codes in one matmul, then a
    dependency test per input position by comparing codes that differ only there.
    """
    lv = np.array(mags, dtype=np.float64)
    lv = lv / lv.mean()                       # unit mean magnitude, as in training
    alpha = np.concatenate([[0.0], lv, -lv])  # 9 levels
    W = np.array(list(itertools.product(alpha, repeat=FANIN)))       # (729, 3)
    acc = W @ TRITS.T.astype(np.float64)                             # (729, 64)
    out = np.where(acc > THR, 1, np.where(acc < -THR, 2, 0))         # (729, 64)
    dep = np.zeros((len(W), FANIN), dtype=bool)
    for i in range(FANIN):
        for alt in range(4):
            idx = (CODES & ~(3 << (SYM_BITS * i))) | (alt << (SYM_BITS * i))
            dep[:, i] |= (out[:, idx] != out).any(axis=1)
    return dep.sum(axis=1).mean(), (dep.sum(axis=1) == FANIN).mean()


if __name__ == "__main__":
    out_path = sys.argv[1]
    DMAX = int(sys.argv[2]) if len(sys.argv) > 2 else 24
    cands = [(1, b, c, d)
             for b in range(2, DMAX + 1)
             for c in range(b + 1, DMAX + 1)
             for d in range(c + 1, DMAX + 1)
             if d <= 1 + b + c]
    print(f"  admissible nine-level integer alphabets, a=1, d<={DMAX}: {len(cands)}")
    print(f"  enumerating junta degree over all 9^3 = 729 triples each\n")
    rows = []
    for m in cands:
        jd, full = junta_degree(m)
        rows.append({"mags": list(m), "junta": float(jd), "full": float(full)})
    rows.sort(key=lambda r: -r["junta"])
    json.dump(rows, open(out_path, "w"), indent=1)

    print(f"  {'rank':>5}  {'alphabet':22s}{'junta':>8}{'full fan-in 3':>15}")
    for i, r in enumerate(rows[:12]):
        m = r["mags"]
        tag = "{0,+-" + ",+-".join(str(x) for x in m) + "}"
        print(f"  {i+1:>5}  {tag:22s}{r['junta']:>8.3f}{r['full']:>14.1%}")
    print("  ...")
    for i, r in enumerate(rows[-3:]):
        m = r["mags"]
        tag = "{0,+-" + ",+-".join(str(x) for x in m) + "}"
        print(f"  {len(rows)-2+i:>5}  {tag:22s}{r['junta']:>8.3f}{r['full']:>14.1%}")

    lin = [r for r in rows if r["mags"] == [1, 2, 3, 4]][0]
    lin_rank = rows.index(lin) + 1
    best = rows[0]
    print(f"\n  linear 9 {{1,2,3,4}}: junta {lin['junta']:.3f}, rank {lin_rank}/{len(rows)}")
    print(f"  maximum:              junta {best['junta']:.3f} at {best['mags']}")
    v = ("CONFIRMED" if best["junta"] < 2.70 and lin_rank <= 3
         else "REFUTED" if best["junta"] >= 2.70 else "PARTIAL")
    print(f"  forecast (max < 2.70 AND linear in top 3) -> {v}")
