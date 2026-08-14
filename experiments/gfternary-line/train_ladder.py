"""W740: train the GA-T ladder, at the rung where representation says phi pays.

WHY THIS EXPERIMENT AND NOT THE EARLIER ONE. T205/T207 trained {0,+-1,+-phi}
against {0,+-1} and a 2-bit dyadic set and found phi adds nothing: cardinality
did all the work. T215-T217 then measured what each alphabet can REPRESENT,
exactly, against the Lloyd-Max optimum, and found something the training had not
looked for:

    K=7   pot7 {0,+-1,+-2,+-4}  92.52%  BEATS  GA-T2  86.67%
    K=9   GA-T3                 91.82%  beats  pot9   69.85%   <-- 22 points
                                        beats  lin9   90.74%   <-- 1.1 points

So the golden ladder's advantage, if it has one, lives at NINE levels, and no
rung above GA-T1 has ever been trained. This is that test.

THE DESIGN POINT, unchanged from T207 because it is what makes the test real:
with sign(Wx) and no threshold ANY positive scalar factors out and every arm is
identical by construction. The project's own hardware uses a FIXED integer
threshold, and against that phi does not factor out.

Dataset: UNSW-NB15 binarised (Zenodo 4519767), 593 binary inputs, binary label
-- the artefact the LUT-network line uses.
"""
# NAMING (W746). `pot<N>` is THIS REPOSITORY'S TAG, not a format name: "powers of
# two, N levels", e.g. pot9 = {0, +-1, +-2, +-4, +-8}. In the literature this is
# POWER-OF-TWO (PoT) QUANTISATION -- Li, Dong & Wang, ICLR 2020, arXiv:1909.13144
# Eq. 3 coins "PoT"; Zhou et al. ICLR 2017 (arXiv:1702.03044) Eq. 1 defines the
# identical set as P_l. The literature name carries NO level count, so any prose
# must write the set out. Do NOT call it APoT (that is sums of PoT terms), INQ
# (a training procedure), or logarithmic quantisation (unsigned, free base).
# The tag stays in code -- stable and greppable -- and stays OUT of prose.
import numpy as np, json, sys, time

PHI = (1 + 5 ** 0.5) / 2

# Positive levels; 0 and the negatives are added, so |alphabet| = 2n+1.
ARMS = [
    ("GA-T0", [1.0]),                                   # 3   balanced ternary
    ("GA-T1", [1.0, PHI]),                              # 5
    ("GA-T2", [1.0, PHI, PHI ** 2]),                    # 7
    ("GA-T3", [1.0, PHI, PHI ** 2, PHI ** 3]),          # 9   <- the rung in question
    ("pot7",  [1.0, 2.0, 4.0]),                         # 7   beats GA-T2 on representation
    ("pot9",  [1.0, 2.0, 4.0, 8.0]),                    # 9   loses badly to GA-T3
    ("lin9",  [1.0, 2.0, 3.0, 4.0]),                    # 9   ties GA-T3
]

# W744: the ONLY effect that replicated across three tasks is CARDINALITY
# (+0.844 pp pooled, z=33.9, direction never varies). Its limit is unmeasured --
# the ladder stopped at nine. This is the saturation sweep, on the cheapest
# family, since T286 showed the shape effect is an order of magnitude smaller.
SAT_ARMS = [
    ("pot3",  [1.0]),                                        #  3
    ("pot5",  [1.0, 2.0]),                                   #  5
    ("pot7",  [1.0, 2.0, 4.0]),                              #  7
    ("pot9",  [1.0, 2.0, 4.0, 8.0]),                         #  9
    ("pot11", [1.0, 2.0, 4.0, 8.0, 16.0]),                   # 11
    ("pot13", [1.0, 2.0, 4.0, 8.0, 16.0, 32.0]),             # 13
    ("pot15", [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0]),       # 15
    ("pot17", [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0]),# 17
]
import os
if os.environ.get("T27_SAT"):
    ARMS = SAT_ARMS


def levels(pos):
    return np.array(sorted([-p for p in pos] + [0.0] + list(pos)), dtype=np.float32)


def load(path, n_train):
    d = np.load(path)
    tr, te = d["train"], d["test"]
    rng = np.random.default_rng(0)          # ONE fixed subsample for every arm
    idx = rng.permutation(len(tr))[:n_train]
    Xtr = tr[idx, :-1].astype(np.float32) * 2 - 1
    ytr = tr[idx, -1].astype(np.float32)
    Xte = te[:, :-1].astype(np.float32) * 2 - 1
    yte = te[:, -1].astype(np.float32)
    return Xtr, ytr, Xte, yte


def quantise(W, lv):
    # BitNet's absmean scale, so a fixed alphabet neither saturates nor vanishes.
    s = np.mean(np.abs(W)) / (np.mean(np.abs(lv[lv != 0])) + 1e-9) + 1e-9
    idx = np.argmin(np.abs(W[..., None] / s - lv[None, None, :]), axis=-1)
    return lv[idx] * s


def run(lv, Xtr, ytr, Xte, yte, seed, hidden=64, epochs=8, lr=0.05, thr=2.0, bs=256):
    rng = np.random.default_rng(seed)
    n_in = Xtr.shape[1]
    W1 = rng.normal(0, 1 / np.sqrt(n_in), (n_in, hidden)).astype(np.float32)
    W2 = rng.normal(0, 1 / np.sqrt(hidden), (hidden, 1)).astype(np.float32)
    for _ in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, len(Xtr) - bs, bs):
            b = perm[i:i + bs]
            x, y = Xtr[b], ytr[b][:, None]
            Q1, Q2 = quantise(W1, lv), quantise(W2, lv)
            a1 = x @ Q1
            # FIXED THRESHOLD -- this is what stops phi factoring out.
            h = np.tanh(a1 - thr) * 0.5 + np.tanh(a1 + thr) * 0.5
            o = h @ Q2
            p = 1 / (1 + np.exp(-np.clip(o, -30, 30)))
            g = (p - y) / len(b)
            gW2 = h.T @ g
            gh = g @ Q2.T
            ga = gh * (0.5 * (1 - np.tanh(a1 - thr) ** 2) + 0.5 * (1 - np.tanh(a1 + thr) ** 2))
            W1 -= lr * (x.T @ ga)
            W2 -= lr * gW2
    Q1, Q2 = quantise(W1, lv), quantise(W2, lv)
    a1 = Xte @ Q1
    h = np.tanh(a1 - thr) * 0.5 + np.tanh(a1 + thr) * 0.5
    o = (h @ Q2).ravel()
    return float(np.mean((o > 0) == (yte > 0.5)))


if __name__ == "__main__":
    path, out_path = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 30
    n_train = int(sys.argv[4]) if len(sys.argv) > 4 else 40000
    Xtr, ytr, Xte, yte = load(path, n_train)
    print(f"  train {Xtr.shape}  test {Xte.shape}  seeds {seeds}", flush=True)
    acc = {name: [] for name, _ in ARMS}
    for s in range(seeds):
        t0 = time.time()
        for name, pos in ARMS:
            acc[name].append(run(levels(pos), Xtr, ytr, Xte, yte, seed=1000 + s))
        print(f"  seed {s + 1}/{seeds}  ({time.time() - t0:.0f}s)  "
              + "  ".join(f"{n}={acc[n][-1] * 100:.2f}" for n, _ in ARMS), flush=True)
    json.dump({k: v for k, v in acc.items()}, open(out_path, "w"), indent=1)
    print("  written:", out_path, flush=True)
