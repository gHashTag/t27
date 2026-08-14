"""W710: the first trained model in the {-phi, 0, +phi} alphabet.

WHY THIS EXISTS. T158 proved by exact simulation that a network whose nonzero
weights are all in {-phi,+phi} is EXACTLY phi^k times the corresponding
{-1,0,+1} network -- so phi adds nothing THERE. T158a named the escape: a FIVE
level alphabet {0,+/-1,+/-phi} does not factor. T183a added that phi is
fungible: any real c gives the same circuit unless Z[phi] pairs propagate.

None of that is worth anything without a trained model, and there is none --
not in the literature (verified null across arXiv/DBLP/OpenAlex/Semantic
Scholar) and not in this repository. This is that model.

THE DESIGN POINT THAT MAKES IT A REAL TEST. With sign(Wx) and no threshold,
ANY positive scalar factors out and every arm is identical by construction --
a tautology, not an experiment. The project's own hardware uses a FIXED integer
threshold (`quantize(v, threshold)` in specs/ternary/activation_quantizer.t27),
and against a fixed threshold phi does NOT factor out. So the net is built that
way: fixed threshold, straight-through estimator, everything else identical
across arms.

Dataset: UNSW-NB15 binarised (Zenodo 4519767) -- the artefact every paper in the
LUT-network line uses, 593 binary inputs, binary label. Already on disk.
"""
import numpy as np, json, sys, time

PHI = (1 + 5 ** 0.5) / 2
RNG_SEED = 27

def load(path):
    d = np.load(path)
    keys = list(d.keys())
    tr = d[[k for k in keys if 'train' in k.lower()][0]]
    te = d[[k for k in keys if 'test' in k.lower()][0]]
    Xtr, ytr = tr[:, :-1].astype(np.float32), tr[:, -1].astype(np.float32)
    Xte, yte = te[:, :-1].astype(np.float32), te[:, -1].astype(np.float32)
    # centre the binary inputs to {-1,+1}: a 0/1 input through a signed weight
    # wastes half the alphabet, and every ternary paper does this.
    return Xtr * 2 - 1, ytr, Xte * 2 - 1, yte

def quantise(W, levels):
    """Nearest-level projection. `levels` is the sorted alphabet."""
    lv = np.asarray(levels, dtype=np.float32)
    # scale by the mean magnitude so the alphabet spans the weights (BitNet's
    # absmean trick; without it a fixed alphabet either saturates or vanishes)
    s = np.mean(np.abs(W)) / (np.mean(np.abs(lv[lv != 0])) + 1e-9) + 1e-9
    idx = np.argmin(np.abs(W[..., None] / s - lv[None, None, :]), axis=-1)
    return lv[idx] * s

def run_arm(name, levels, Xtr, ytr, Xte, yte, hidden=64, epochs=12, lr=0.05, thr=2.0):
    rng = np.random.default_rng(RNG_SEED)          # SAME seed for every arm
    n_in = Xtr.shape[1]
    W1 = rng.normal(0, 1 / np.sqrt(n_in), (n_in, hidden)).astype(np.float32)
    W2 = rng.normal(0, 1 / np.sqrt(hidden), (hidden, 1)).astype(np.float32)
    bs = 256
    for ep in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, len(Xtr) - bs, bs):
            b = perm[i:i + bs]
            x, y = Xtr[b], ytr[b][:, None]
            Q1, Q2 = quantise(W1, levels), quantise(W2, levels)
            a1 = x @ Q1
            # FIXED THRESHOLD -- this is what stops phi factoring out.
            h = np.tanh(a1 - thr) * 0.5 + np.tanh(a1 + thr) * 0.5
            o = h @ Q2
            p = 1 / (1 + np.exp(-np.clip(o, -30, 30)))
            g = (p - y) / len(b)
            # straight-through: gradient flows to the LATENT weights
            gW2 = h.T @ g
            gh = g @ Q2.T
            ga = gh * (0.5 * (1 - np.tanh(a1 - thr) ** 2) + 0.5 * (1 - np.tanh(a1 + thr) ** 2))
            gW1 = x.T @ ga
            W1 -= lr * gW1
            W2 -= lr * gW2
    Q1, Q2 = quantise(W1, levels), quantise(W2, levels)
    a1 = Xte @ Q1
    h = np.tanh(a1 - thr) * 0.5 + np.tanh(a1 + thr) * 0.5
    o = (h @ Q2).ravel()
    acc = float(np.mean((o > 0) == (yte > 0.5)))
    return {"arm": name, "levels": [float(x) for x in levels], "test_acc": acc}

if __name__ == "__main__":
    Xtr, ytr, Xte, yte = load(sys.argv[1])
    print(f"  train {Xtr.shape}  test {Xte.shape}", flush=True)
    arms = [
        ("ternary  {-1,0,+1}",   [-1.0, 0.0, 1.0]),
        ("phi      {-phi,0,+phi}", [-PHI, 0.0, PHI]),
        ("five     {0,+-1,+-phi}", [-PHI, -1.0, 0.0, 1.0, PHI]),
        ("two-bit  {-2,-1,0,+1}",  [-2.0, -1.0, 0.0, 1.0]),
    ]
    out = []
    for name, lv in arms:
        t0 = time.time()
        r = run_arm(name, lv, Xtr, ytr, Xte, yte)
        r["seconds"] = round(time.time() - t0, 1)
        out.append(r)
        print(f"  {name:<26} acc={r['test_acc']:.4f}  ({r['seconds']}s)", flush=True)
    json.dump(out, open(sys.argv[2], "w"), indent=1)
    print("  DONE", flush=True)
