"""W748: attack the 617x architecture gap, not the alphabet.

T301 measured our dense design at 54,914 LUT / 83.4% against the field's 89 LUT
/ 92%. Seven waves went into the weight alphabet of a network three orders of
magnitude off the pace. This changes the ONE thing never questioned: dense
connectivity.

THE FIELD'S TRICK (LogicNets, FPL 2020; PolyLUT; NeuraLUT; TreeLUT). Give each
neuron a small fixed fan-in F over RANDOMLY CHOSEN inputs, then absorb the whole
neuron -- weights, sum, threshold -- into a TRUTH TABLE. A neuron with F inputs
of W bits each is a function of F*W bits; at F*W <= 6 that is exactly one Xilinx
LUT6 per output bit. No adders. No weights in memory. The arithmetic disappears
into the routing that was going to exist anyway.

FOR A TERNARY NODE the accounting is: layer 1 sees 1-bit binarised inputs, so
F <= 6 gives 2 LUT per neuron (two output bits). Deeper layers see 2-bit ternary
inputs, so F <= 3 gives 2 LUT per neuron. That is the whole reason 89 LUT is
reachable and 54,914 is not.

FORECAST, REGISTERED BEFORE THE RUN (T44). Sparse fan-in will COST accuracy and
the area will collapse. Specifically:
  (1) F=6 on layer 1 lands at 78-81% -- below our dense 83.4%, above chance.
  (2) Area falls by at least 100x, to the low hundreds of LUT.
  (3) THE DECOMPOSITION THIS BUYS: if area reaches ~100 LUT while accuracy stays
      near 80%, then the residual gap to the field's 92% is TRAINING, not
      architecture -- our trainer is an 8-epoch probe and theirs are not. If
      accuracy instead RISES toward 92%, the gap was architectural all along and
      the alphabet work was even further off-target than T301 says.
Predicted: (3) resolves as TRAINING. I expect area parity and accuracy failure.
"""
import numpy as np, json, sys, time

def build_masks(n_in, n_out, fanin, rng):
    """Each neuron sees `fanin` inputs drawn without replacement."""
    return np.stack([rng.choice(n_in, size=fanin, replace=False) for _ in range(n_out)])

def quantise(W, lv):
    s = np.mean(np.abs(W)) / (np.mean(np.abs(lv[lv != 0])) + 1e-9) + 1e-9
    idx = np.argmin(np.abs(W[..., None] / s - lv[None, None, :]), axis=-1)
    return lv[idx] * s

def sparse_forward(X, idx, Q, thr):
    """X: (N, n_in). idx: (n_out, F). Q: (n_out, F). -> (N, n_out) pre-activation."""
    return np.einsum('nof,of->no', X[:, idx], Q)

def run(Xtr, ytr, Xte, yte, seed, fanin=6, hidden=64, epochs=8, lr=0.05, thr=2.0, bs=256,
        levels=np.array([-4,-2,-1,0,1,2,4], dtype=np.float32)):
    rng = np.random.default_rng(seed)
    n_in = Xtr.shape[1]
    idx1 = build_masks(n_in, hidden, min(fanin, n_in), rng)
    f2 = min(fanin, hidden)
    idx2 = build_masks(hidden, 1, f2, rng)
    W1 = rng.normal(0, 1/np.sqrt(idx1.shape[1]), idx1.shape).astype(np.float32)
    W2 = rng.normal(0, 1/np.sqrt(f2), idx2.shape).astype(np.float32)
    for _ in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, max(len(Xtr) - bs, 1), bs):
            b = perm[i:i+bs]; x, y = Xtr[b], ytr[b][:, None]
            Q1, Q2 = quantise(W1, levels), quantise(W2, levels)
            a1 = sparse_forward(x, idx1, Q1, thr)
            h = np.tanh(a1 - thr)*0.5 + np.tanh(a1 + thr)*0.5
            o = sparse_forward(h, idx2, Q2, thr)
            p = 1/(1 + np.exp(-np.clip(o, -30, 30)))
            g = (p - y)/len(b)
            # grad wrt W2 (sparse): accumulate over the selected indices
            gW2 = np.einsum('no,nof->of', g, h[:, idx2])
            gh = np.zeros_like(h)
            np.add.at(gh, (slice(None), idx2[0]), g * Q2[0][None, :])
            ga = gh * (0.5*(1 - np.tanh(a1-thr)**2) + 0.5*(1 - np.tanh(a1+thr)**2))
            gW1 = np.einsum('no,nof->of', ga, x[:, idx1])
            W1 -= lr*gW1; W2 -= lr*gW2
    Q1, Q2 = quantise(W1, levels), quantise(W2, levels)
    a1 = sparse_forward(Xte, idx1, Q1, thr)
    h = np.tanh(a1-thr)*0.5 + np.tanh(a1+thr)*0.5
    o = sparse_forward(h, idx2, Q2, thr).ravel()
    return float(np.mean((o > 0) == (yte > 0.5)))

if __name__ == "__main__":
    path, out_path = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 20
    d = np.load(path); tr, te = d["train"], d["test"]
    rng = np.random.default_rng(0); sel = rng.permutation(len(tr))[:40000]
    Xtr = tr[sel, :-1].astype(np.float32)*2-1; ytr = tr[sel, -1].astype(np.float32)
    Xte = te[:, :-1].astype(np.float32)*2-1;   yte = te[:, -1].astype(np.float32)
    res = {}
    for F in [2, 3, 4, 6, 8, 12, 24]:
        t0 = time.time()
        res[str(F)] = [run(Xtr, ytr, Xte, yte, 1000+s, fanin=F) for s in range(seeds)]
        m = np.mean(res[str(F)])*100
        print(f"  fan-in {F:>3}: {m:6.2f}%  ({time.time()-t0:.0f}s)", flush=True)
    json.dump(res, open(out_path, "w"), indent=1)
    print("  written:", out_path, flush=True)
