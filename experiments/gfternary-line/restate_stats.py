"""W786/T455: restate every correlation in this line as a slope with an interval.

T424 said it plainly: r = +0.991 over six alphabets CONSTRUCTED to vary
monotonically in the predictor, with no confidence interval, "measures the design,
not the world". The same objection hits r = -0.971 / +0.956 at n=5 and
+0.916 / +0.986 at n=7. T424 said report the pairs and the slope. Nobody did.

This does it: for every junta-degree relation in the line, a least-squares SLOPE
with a bootstrap-percentile 95 % interval over resampled arms, plus Spearman, plus
the n. A slope has units -- LUT per unit of junta degree, or pp per unit -- and an
interval that includes zero is a null however large r looks.

No new measurement. Every number below is re-read from the saved runs.
"""
import json
import sys

import numpy as np

JD = {"linear 9": 2.551, "linear": 2.551, "1,2,3,5": 2.42, "1235": 2.42,
      "1,2,3,6": 2.27, "1,2,4,7": 2.19, "dyadic": 2.189, "2.0": 2.189,
      "base 3": 1.490, "3.0": 1.490, "base 4": 1.033, "4.0": 1.033,
      "fib": 2.47}


def boot_slope(x, y, n=20000, seed=7):
    x, y = np.asarray(x, float), np.asarray(y, float)
    k = len(x)
    rng = np.random.default_rng(seed)
    sl = np.polyfit(x, y, 1)[0]
    bs = []
    for _ in range(n):
        i = rng.integers(0, k, k)
        if len(set(x[i])) < 2:
            continue
        bs.append(np.polyfit(x[i], y[i], 1)[0])
    lo, hi = np.percentile(bs, [2.5, 97.5])
    return sl, lo, hi


def spearman(x, y):
    def rk(v):
        o = sorted(range(len(v)), key=lambda i: v[i])
        r = [0] * len(v)
        for p, i in enumerate(o):
            r[i] = p
        return r
    return float(np.corrcoef(rk(x), rk(y))[0, 1])


def report(name, x, y, unit):
    r = float(np.corrcoef(x, y)[0, 1])
    sl, lo, hi = boot_slope(x, y)
    rho = spearman(x, y)
    null = "includes 0" if lo <= 0 <= hi else "excludes 0"
    print(f"  {name}")
    print(f"    n={len(x)}   r={r:+.3f}   Spearman={rho:+.3f}")
    print(f"    slope {sl:+.1f} {unit} per unit junta   95% CI [{lo:+.1f}, {hi:+.1f}]"
          f"   -> {null}")
    print()


if __name__ == "__main__":
    G = sys.argv[1]
    print("  RESTATING THE JUNTA RELATIONS AS SLOPES WITH INTERVALS")
    print("  (T424: r over arms constructed to vary in the predictor measures the design)\n")

    # T410 / T445: junta vs AREA, post-route, enumerated ranks
    names = ["linear", "1235", "2.0", "3.0", "4.0"]
    area = [128, 135, 137, 111, 85]              # W778 synthesis-era L=4 SLICE_LUTX
    report("T410 junta -> yosys LUT (5 nine-level alphabets, L=4)",
           [JD[n] for n in names], area, "LUT")

    post = {"linear": 128, "1235": 147, "2.0": 137, "3.0": 111, "4.0": 65}
    report("T445 junta -> POST-ROUTE SLICE_LUTX (same alphabets)",
           [JD[n] for n in post], list(post.values()), "LUT")

    # T417: junta vs ACCURACY, both tasks, from the saved seven-arm run
    try:
        d = json.load(open(f"{G}/nonladder.json"))
        for task in d:
            ks = [k for k in d[task] if k in JD]
            report(f"T417 junta -> accuracy, {task} (7 arms, 8 seeds)",
                   [JD[k] for k in ks],
                   [float(np.mean(d[task][k])) * 100 for k in ks], "pp")
    except FileNotFoundError:
        print("  (nonladder.json absent -- accuracy relations not restated)\n")

    print("  READ THE INTERVALS, NOT THE r. A slope whose interval includes zero")
    print("  is a null no matter how close r sits to one.")
