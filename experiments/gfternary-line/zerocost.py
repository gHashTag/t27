"""W747: does ternary's ZERO pay for the code space it occupies?

THE QUESTION (T300). Nine levels {0,+-1,+-2,+-4,+-8} occupy 9 of the 16 codes a
4-bit field offers and waste seven. DenseShift (ICCV 2023) reports that zero
earns nothing at low bit-width and is deliberately zero-free. Our own T291b
points the same way from the accuracy side.

THE DESIGN THIS TESTS. The fair comparison is at equal CODE WIDTH, not at equal
level count -- an alphabet that wastes codes is paying for them in memory and
routing whether it uses them or not.

    3 bits, 8 codes:   {0,+-1,+-2,+-4}         7 levels, 1 code wasted
                  vs   {+-1,+-2,+-4,+-8}       8 levels, none wasted
    4 bits, 16 codes:  {0,+-1,+-2,+-4,+-8}     9 levels, 7 codes wasted
                  vs   {+-1,...,+-128}        16 levels, none wasted

FORECAST, REGISTERED BEFORE THE RUN (T44). Zero WILL pay, and it will pay most
on UNSW-NB15. Zero is how a quantised net prunes: with 593 binary features, most
of them irrelevant, a weight that cannot be zero must contribute noise to every
sum. MNIST-binarised has far less irrelevant input, so the zero-free penalty
should be smaller there. Predicted ordering of the zero-free penalty:
UNSW > Fashion > MNIST, and NEGATIVE (zero-free wins) nowhere.

If this is wrong -- if zero-free wins anywhere -- the nine-level alphabet in
every recommendation this project has made is spending a code for nothing.
"""
import numpy as np, json, sys, time

ARMS = [
    ("z7_3bit",  [1.0, 2.0, 4.0], True),                                   # 7 lv, has zero
    ("nz8_3bit", [1.0, 2.0, 4.0, 8.0], False),                             # 8 lv, no zero
    ("z9_4bit",  [1.0, 2.0, 4.0, 8.0], True),                              # 9 lv, has zero
    ("nz16_4bit",[1.0,2.0,4.0,8.0,16.0,32.0,64.0,128.0], False),           # 16 lv, no zero
]


def levels(pos, with_zero):
    v = [-p for p in pos] + ([0.0] if with_zero else []) + list(pos)
    return np.array(sorted(v), dtype=np.float32)


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
    path, out_path = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 30
    n_train = int(sys.argv[4]) if len(sys.argv) > 4 else 40000
    d = np.load(path); tr, te = d["train"], d["test"]
    rng = np.random.default_rng(0)
    idx = rng.permutation(len(tr))[:n_train]
    Xtr = tr[idx, :-1].astype(np.float32) * 2 - 1; ytr = tr[idx, -1].astype(np.float32)
    Xte = te[:, :-1].astype(np.float32) * 2 - 1;   yte = te[:, -1].astype(np.float32)
    acc = {n: [] for n, _, _ in ARMS}
    for s in range(seeds):
        t0 = time.time()
        for name, pos, wz in ARMS:
            acc[name].append(run(levels(pos, wz), Xtr, ytr, Xte, yte, 1000 + s))
        print(f"  seed {s+1}/{seeds} ({time.time()-t0:.0f}s)  " +
              "  ".join(f"{n}={acc[n][-1]*100:.2f}" for n, _, _ in ARMS), flush=True)
    json.dump(acc, open(out_path, "w"), indent=1)
    print("  written:", out_path, flush=True)
