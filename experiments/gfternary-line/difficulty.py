"""W745: what sets the saturation rung?

T288 measured that accuracy saturates in the alphabet's cardinality on every
task, but at a DIFFERENT rung: UNSW 9, MNIST 5, Fashion 9. A threshold that
varies and is not explained is not a result -- it is a loose end.

THE HYPOTHESIS, registered before running: the rung is set by TASK DIFFICULTY.
An easy task is decided by a few coarse weights; a hard one needs finer ones.

THE DESIGN. Three datasets cannot establish a relationship, and adding a fourth
would still confound "difficulty" with "different data". So the difficulty is
varied WITHIN one dataset: five binary digit-pair tasks from MNIST, from the
famously separable 0-vs-1 to the famously confusable 3-vs-5. Input dimension,
trainer, seeds, subsample size and alphabets are identical across all five --
the ONLY thing that changes is which two digits are being told apart.

Difficulty is then not an opinion: it is the ceiling accuracy the task admits.
"""
import numpy as np, json, sys, math, time

PAIRS = [
    ("0v1", 0, 1),   # famously separable
    ("0v8", 0, 8),
    ("7v9", 7, 9),
    ("4v9", 4, 9),
    ("3v5", 3, 5),   # famously confusable
]
RUNGS = [
    ("3",  [1.0]),
    ("5",  [1.0, 2.0]),
    ("7",  [1.0, 2.0, 4.0]),
    ("9",  [1.0, 2.0, 4.0, 8.0]),
    ("11", [1.0, 2.0, 4.0, 8.0, 16.0]),
]


def levels(pos):
    return np.array(sorted([-p for p in pos] + [0.0] + list(pos)), dtype=np.float32)


def quantise(W, lv):
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
        for i in range(0, max(len(Xtr) - bs, 1), bs):
            b = perm[i:i + bs]
            x, y = Xtr[b], ytr[b][:, None]
            Q1, Q2 = quantise(W1, lv), quantise(W2, lv)
            a1 = x @ Q1
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
    return float(np.mean(((h @ Q2).ravel() > 0) == (yte > 0.5)))


if __name__ == "__main__":
    src, out_path = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 20
    d = np.load(src)
    tr, te = d["train"], d["test"]
    dig_tr, dig_te = d["dig_train"], d["dig_test"]
    res = {}
    for tag, a, b in PAIRS:
        mtr = (dig_tr == a) | (dig_tr == b)
        mte = (dig_te == a) | (dig_te == b)
        Xtr = tr[mtr, :-1].astype(np.float32) * 2 - 1
        ytr = (dig_tr[mtr] == b).astype(np.float32)
        Xte = te[mte, :-1].astype(np.float32) * 2 - 1
        yte = (dig_te[mte] == b).astype(np.float32)
        res[tag] = {}
        t0 = time.time()
        for rung, pos in RUNGS:
            res[tag][rung] = [run(levels(pos), Xtr, ytr, Xte, yte, 1000 + s)
                              for s in range(seeds)]
        best = max(np.mean(res[tag][r]) for r, _ in RUNGS) * 100
        print(f"  {tag}  n_train={mtr.sum():6d}  ceiling={best:6.2f}%  ({time.time()-t0:.0f}s)",
              flush=True)
    json.dump(res, open(out_path, "w"), indent=1)
    print("  written:", out_path, flush=True)
