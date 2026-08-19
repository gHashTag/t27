"""W749: re-run the alphabet conclusions on the FIXED rig.

WHY THIS MUST BE RUN. Every alphabet result this project holds -- T286's size
effect (+0.844 pp, z=33.9), T286's shape effect (+0.085 pp), T288's Nine-Rung
saturation -- was measured WITHOUT inter-layer normalisation. W748 then showed
that omission is worth +5.49 pp at depth two and +29.15 pp at depth five: the
bench was not merely suboptimal, it was the largest effect in the programme,
uncontrolled. A conclusion measured on an uncontrolled bench is not wrong by
default, but it is unverified, and this project has already retracted 23 claims
for less.

WHAT CHANGES. One line: the pre-activation is normalised to unit scale before
the fixed threshold, so the threshold means the same thing regardless of how the
alphabet scales the sums. NOTE THAT THIS IS EXACTLY THE MECHANISM BY WHICH AN
ALPHABET COULD HAVE BEEN WINNING FOR THE WRONG REASON: a larger alphabet
produces larger sums, and against a FIXED threshold larger sums look better.
Normalisation removes that channel. If the size effect was real it survives; if
it was a scale artefact it disappears.

FORECAST, REGISTERED BEFORE THE RUN (T44). The size effect will SHRINK but
SURVIVE in direction. Predicted +0.3 to +0.8 pp for 3->9 levels, still
significant at 30 seeds, sign unchanged. Reasoning: the absmean quantiser already
renormalises WEIGHTS per layer, so the uncontrolled channel was on the activation
side only, and at depth two W748 measured that as worth 5.49 pp overall -- large,
but not obviously alphabet-dependent. I expect resolution to remain worth
something once scale is pinned.

IF THE EFFECT INVERTS OR VANISHES, T286, T288, the Nine-Rung Law and the
TNF-9 abstract all require retraction, and this file is the reason.
"""
import numpy as np, json, sys, time

PHI = (1 + 5 ** 0.5) / 2
ARMS = [
    ("GA-T0", [1.0]), ("GA-T1", [1.0, PHI]), ("GA-T2", [1.0, PHI, PHI**2]),
    ("GA-T3", [1.0, PHI, PHI**2, PHI**3]),
    ("pot7", [1.0, 2.0, 4.0]), ("pot9", [1.0, 2.0, 4.0, 8.0]),
    ("lin9", [1.0, 2.0, 3.0, 4.0]),
]

def levels(pos):
    return np.array(sorted([-p for p in pos] + [0.0] + list(pos)), dtype=np.float32)

def quantise(W, lv):
    s = np.mean(np.abs(W)) / (np.mean(np.abs(lv[lv != 0])) + 1e-9) + 1e-9
    idx = np.argmin(np.abs(W[..., None] / s - lv[None, None, :]), axis=-1)
    return lv[idx] * s

def run(lv, Xtr, ytr, Xte, yte, seed, hidden=64, epochs=8, lr=0.05, thr=2.0, bs=256, norm=True):
    rng = np.random.default_rng(seed); n_in = Xtr.shape[1]
    W1 = rng.normal(0, 1/np.sqrt(n_in), (n_in, hidden)).astype(np.float32)
    W2 = rng.normal(0, 1/np.sqrt(hidden), (hidden, 1)).astype(np.float32)
    for _ in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, max(len(Xtr)-bs, 1), bs):
            b = perm[i:i+bs]; x, y = Xtr[b], ytr[b][:, None]
            Q1, Q2 = quantise(W1, lv), quantise(W2, lv)
            a1 = x @ Q1
            # THE FIX UNDER TEST: pin the pre-activation scale, so a fixed
            # threshold cannot reward an alphabet merely for producing bigger sums.
            sd = (a1.std() + 1e-6) if norm else 1.0
            if norm: a1 = a1 / sd * thr
            h = np.tanh(a1-thr)*0.5 + np.tanh(a1+thr)*0.5
            o = h @ Q2
            p = 1/(1+np.exp(-np.clip(o, -30, 30))); g = (p-y)/len(b)
            gW2 = h.T @ g
            gh = g @ Q2.T
            ga = gh * (0.5*(1-np.tanh(a1-thr)**2) + 0.5*(1-np.tanh(a1+thr)**2))
            if norm: ga = ga / sd * thr
            W1 -= lr * (x.T @ ga); W2 -= lr * gW2
    Q1, Q2 = quantise(W1, lv), quantise(W2, lv)
    a1 = Xte @ Q1
    if norm: a1 = a1/(a1.std()+1e-6)*thr
    h = np.tanh(a1-thr)*0.5 + np.tanh(a1+thr)*0.5
    return float(np.mean(((h @ Q2).ravel() > 0) == (yte > 0.5)))

if __name__ == "__main__":
    path, out_path = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 30
    d = np.load(path); tr, te = d["train"], d["test"]
    rng = np.random.default_rng(0); sel = rng.permutation(len(tr))[:40000]
    Xtr = tr[sel, :-1].astype(np.float32)*2-1; ytr = tr[sel, -1].astype(np.float32)
    Xte = te[:, :-1].astype(np.float32)*2-1;   yte = te[:, -1].astype(np.float32)
    res = {}
    for name, pos in ARMS:
        lv = levels(pos)
        for tag, nrm in (("norm", True), ("raw", False)):
            k = f"{name}_{tag}"; t0 = time.time()
            res[k] = [run(lv, Xtr, ytr, Xte, yte, 1000+s, norm=nrm) for s in range(seeds)]
        print(f"  {name:<7} norm={np.mean(res[name+'_norm'])*100:6.2f}%  "
              f"raw={np.mean(res[name+'_raw'])*100:6.2f}%  ({time.time()-t0:.0f}s)", flush=True)
    json.dump(res, open(out_path, "w"), indent=1)
    print("  written:", out_path, flush=True)
