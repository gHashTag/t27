"""W767: rank candidate tasks by what a sparse ternary datapath will cost.

WHAT THIS IS NOT. It is not a predictor of the penalty in percentage points.
T371 fitted exactly that -- `penalty = -2.28*ln(mi_tot) + 8.90`, in-sample RMSE
0.64 pp -- and it under-predicted three held-out datasets by 2.4, 3.3 and 8.6 pp,
every error in the same direction. A tool printing those numbers would have been
confidently, optimistically wrong on every dataset we care about.

WHAT IT IS. A RANKER. `mi_tot` -- the sum of per-feature mutual information with
the label, one pass over the data -- correlates with the penalty at r = -0.81
within a dataset (45 labellings, confound-controlled) and r = -0.68 across three
datasets. That licenses an ORDERING and a BRACKET taken from the observed spread.
It does not license a point estimate, and this file refuses to print one.

USAGE
    python3 taskfit.py data1.npz data2.npz ...
Prints the tasks ordered by expected cost, cheapest first, with the observed
penalty range of comparable tasks as a bracket.
"""
import numpy as np, sys, json

# Observed penalties, from the measurements this bracket is drawn from.
# 60 labellings of MNIST (W760/W761) plus three whole datasets (W758/W759).
OBSERVED = {"min": 0.24, "max": 14.85, "median": 3.00,
            "n_tasks": 63, "source": "W758-W762"}
# Anchors: (mi_tot, measured penalty). Used ONLY to place a new task between
# known ones -- never to interpolate a value.
ANCHORS = [(3.14, 14.85, "MNIST-bin"), (5.21, 5.66, "4v9"), (11.04, 6.70, "UNSW"),
           (21.89, 0.90, "0v8"), (30.40, 3.48, "Fashion"), (44.75, 0.28, "0v1")]

def mutual_info(X, y):
    n = X.shape[1]; out = np.zeros(n); yb = y > 0.5; py1 = yb.mean()
    for c in range(n):
        xb = X[:, c] > 0; mi = 0.0
        for xv in (False, True):
            px = np.mean(xb == xv)
            if px <= 0: continue
            for yv in (False, True):
                p = np.mean((xb == xv) & (yb == yv))
                if p <= 0: continue
                py = py1 if yv else 1 - py1
                mi += p * np.log(p / (px * py + 1e-12) + 1e-12)
        out[c] = max(mi, 0.0)
    return out

def load(path):
    d = np.load(path); tr = d["train"]
    return tr[:, :-1].astype(np.float32) * 2 - 1, tr[:, -1].astype(np.float32)

def bracket(mi):
    """Place mi between the two nearest anchors and report THEIR penalties."""
    lo = [a for a in ANCHORS if a[0] <= mi]
    hi = [a for a in ANCHORS if a[0] >= mi]
    l = max(lo, key=lambda a: a[0]) if lo else None
    h = min(hi, key=lambda a: a[0]) if hi else None
    if l and h and l is not h:
        p = sorted([l[1], h[1]])
        return f"{p[0]:.1f}-{p[1]:.1f} pp", f"between {l[2]} and {h[2]}"
    a = l or h
    return f"~{a[1]:.1f} pp", f"nearest anchor {a[2]}"

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__); sys.exit(1)
    rows = []
    for path in sys.argv[1:]:
        X, y = load(path)
        mi = mutual_info(X, y)
        rows.append((path.split("/")[-1], float(mi.sum()), int(X.shape[1])))
    rows.sort(key=lambda r: -r[1])      # high mi_tot = low penalty = cheapest first
    print(f"\n  RANKED BY EXPECTED COST TO A SPARSE TERNARY DATAPATH -- cheapest first")
    print(f"  {'task':<22}{'features':>10}{'mi_tot':>10}   {'bracket':<14} basis")
    for nm, m, nf in rows:
        b, why = bracket(m)
        print(f"  {nm:<22}{nf:>10}{m:>10.2f}   {b:<14} {why}")
    print(f"\n  THIS IS A RANKING, NOT A PREDICTION.")
    print(f"  mi_tot correlates with the penalty at r = -0.81 within a dataset and")
    print(f"  r = -0.68 across datasets, over {OBSERVED['n_tasks']} measured tasks spanning")
    print(f"  {OBSERVED['min']}-{OBSERVED['max']} pp (median {OBSERVED['median']}).")
    print(f"  A curve fitted to this relation under-predicted three held-out")
    print(f"  datasets by 2.4, 3.3 and 8.6 pp (T371), so no point estimate is given.")
