"""W715: what each GA-T rung can REPRESENT, measured exactly.

WHY THIS AND NOT TRAINING. The question "does the seventh level earn its cost"
was going to be answered by thirty training seeds. Training answers it through
one dataset, one architecture and one optimiser, and T207 already had to state
in its own conclusions that it established only relative order under identical
conditions. The representation question is prior to all of that and has an
EXACT answer: given weights drawn from the distribution every quantisation
paper assumes, how much of the achievable accuracy does each alphabet capture?

This is the standard object in the quantisation literature -- APoT (Li et al.,
ICLR 2020) and LQ-Nets (Zhang et al., ECCV 2018) are both, at bottom, arguments
about which fixed level set best fits a bell-shaped weight distribution. It is
measured here for the GA-T line against dyadic sets of EQUAL CARDINALITY
and against the Lloyd-Max optimum, which no fixed alphabet can beat.

Every alphabet gets its own optimal scale, chosen by minimising the same
objective -- otherwise the comparison measures scaling, not shape.
"""
import numpy as np, json, sys

PHI = (1 + 5 ** 0.5) / 2

# Symmetric alphabets, given by their POSITIVE levels; 0 and the negatives are
# added automatically, so cardinality is 2*len(pos)+1.
ALPHABETS = {
    3: [("GA-T0  {0,+-1}",          [1.0])],
    5: [("GA-T1  {0,+-1,+-phi}",    [1.0, PHI]),
        ("lin5  {0,+-1,+-2}",      [1.0, 2.0]),
        ("pot5  {0,+-1,+-2}",      [1.0, 2.0])],
    7: [("GA-T2  {0,+-1,+-phi,+-phi^2}", [1.0, PHI, PHI ** 2]),
        ("lin7  {0,+-1,+-2,+-3}",       [1.0, 2.0, 3.0]),
        ("pot7  {0,+-1,+-2,+-4}",       [1.0, 2.0, 4.0])],
    9: [("GA-T3  {0,..,+-phi^3}",    [1.0, PHI, PHI ** 2, PHI ** 3]),
        ("lin9  {0,+-1,..,+-4}",    [1.0, 2.0, 3.0, 4.0]),
        ("pot9  {0,+-1,+-2,+-4,+-8}", [1.0, 2.0, 4.0, 8.0])],
    11: [("GA-T4  {0,..,+-phi^4}",   [1.0, PHI, PHI ** 2, PHI ** 3, PHI ** 4]),
         ("lin11 {0,+-1,..,+-5}",   [1.0, 2.0, 3.0, 4.0, 5.0]),
         ("pot11 {0,+-1,+-2,..,+-16}", [1.0, 2.0, 4.0, 8.0, 16.0])],
    13: [("GA-T5  {0,..,+-phi^5}",   [1.0, PHI, PHI**2, PHI**3, PHI**4, PHI**5]),
         ("lin13 {0,+-1,..,+-6}",   [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
         ("pot13 {0,+-1,..,+-32}",  [1.0, 2.0, 4.0, 8.0, 16.0, 32.0])],
}

# Gauss-Legendre on [-8,8] is exact to machine precision for a Gaussian and
# needs no sampling noise argument. 20001 nodes is far past convergence.
GRID = np.linspace(-8.0, 8.0, 20001)
PDF = np.exp(-GRID ** 2 / 2) / np.sqrt(2 * np.pi)
DX = GRID[1] - GRID[0]


def full(pos):
    return np.array(sorted([-p for p in pos] + [0.0] + list(pos)), dtype=np.float64)


def mse_at_scale(levels, s):
    """E[(w - s*Q(w/s))^2] for w ~ N(0,1), Q = nearest level."""
    lv = levels * s
    d = np.abs(GRID[:, None] - lv[None, :]).min(axis=1)
    return float((d ** 2 * PDF).sum() * DX)


def best_mse(levels):
    """Minimise over the scale. Coarse sweep then golden-section refine, so the
    result does not depend on where the grid happened to land."""
    ss = np.geomspace(0.02, 5.0, 400)
    vals = [mse_at_scale(levels, s) for s in ss]
    i = int(np.argmin(vals))
    lo, hi = ss[max(i - 1, 0)], ss[min(i + 1, len(ss) - 1)]
    gr = (5 ** 0.5 - 1) / 2
    a, b = lo, hi
    for _ in range(80):
        c, d = b - gr * (b - a), a + gr * (b - a)
        if mse_at_scale(levels, c) < mse_at_scale(levels, d):
            b = d
        else:
            a = c
    s = (a + b) / 2
    return mse_at_scale(levels, s), s


def lloyd_max(k, iters=500):
    """Optimal k-level symmetric quantiser for N(0,1): the bound no fixed
    alphabet can pass. Lloyd's algorithm on the same grid."""
    lv = np.linspace(-2.5, 2.5, k)
    for _ in range(iters):
        idx = np.abs(GRID[:, None] - lv[None, :]).argmin(axis=1)
        new = lv.copy()
        for j in range(k):
            m = idx == j
            wsum = (PDF[m]).sum()
            if wsum > 0:
                new[j] = (GRID[m] * PDF[m]).sum() / wsum
        if np.allclose(new, lv, atol=1e-12):
            lv = new
            break
        lv = new
    d = np.abs(GRID[:, None] - lv[None, :]).min(axis=1)
    return float((d ** 2 * PDF).sum() * DX), lv


if __name__ == "__main__":
    out = []
    print(f"  {'K':>3} {'alphabet':<28} {'MSE':>10} {'SQNR dB':>9} {'scale':>7} {'% of Lloyd-Max':>15}")
    for k in sorted(ALPHABETS):
        lm, lmlv = lloyd_max(k)
        for name, pos in ALPHABETS[k]:
            lv = full(pos)
            assert len(lv) == k, (name, len(lv), k)
            m, s = best_mse(lv)
            # variance of N(0,1) is 1, so SQNR = -10 log10(MSE)
            sqnr = -10 * np.log10(m)
            eff = lm / m * 100.0
            out.append({"K": k, "arm": name.split()[0], "alphabet": name,
                        "mse": m, "sqnr_db": sqnr, "scale": s,
                        "lloyd_max_mse": lm, "pct_of_optimal": eff})
            print(f"  {k:>3} {name:<28} {m:>10.6f} {sqnr:>9.3f} {s:>7.4f} {eff:>14.2f}%")
        print(f"  {k:>3} {'Lloyd-Max optimum':<28} {lm:>10.6f} {-10*np.log10(lm):>9.3f} {'--':>7} {100.0:>14.2f}%")
    if len(sys.argv) > 1:
        json.dump(out, open(sys.argv[1], "w"), indent=1)
        print("  written:", sys.argv[1])
