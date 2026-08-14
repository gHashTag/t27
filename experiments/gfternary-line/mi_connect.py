"""W749: choose the sparse connectivity instead of throwing a die (T316).

Our sparse layers pick `F` inputs uniformly at random. LogicNets and PolyLUT do
not. With 593 binary features of which most are irrelevant -- the property that
made T306's forecast so wrong -- a uniform draw spends most of a neuron's tiny
fan-in on noise. This is the cheapest untested item on T316's residue list: one
pass over the data buys the mutual information of every feature with the label.

THE DESIGN POINT. Deterministic top-K is a trap: every neuron would pick the SAME
features and the layer would collapse to one repeated neuron. So the arms are:
    uniform   -- the current scheme, the control
    top       -- deterministic top-F by MI, the collapse case, included to MEASURE
                 the collapse rather than assume it
    prop      -- sample WITHOUT replacement with probability proportional to MI,
                 which keeps diversity while biasing toward signal
    top2f     -- uniform draw from the top 2F features: a middle point

FORECAST, REGISTERED BEFORE THE RUN (T44). `prop` beats `uniform` by 3-8 pp at
fan-in 6, `top` collapses to near the majority baseline through lost diversity,
and `top2f` lands between `prop` and `uniform`. If `top` does NOT collapse, then
diversity is worth less than signal and the whole random-connectivity premise of
this architecture family deserves a second look.
"""
import numpy as np, json, sys, time

LV = np.array([-4,-2,-1,0,1,2,4], dtype=np.float32)

def mutual_info(X, y):
    """X is +-1 binarised, y is {0,1}. Exact MI for two binary variables, per feature."""
    n = len(y); out = np.zeros(X.shape[1], dtype=np.float64)
    yb = y > 0.5
    for c in range(X.shape[1]):
        xb = X[:, c] > 0
        mi = 0.0
        for xv in (False, True):
            for yv in (False, True):
                p = np.mean((xb == xv) & (yb == yv))
                if p <= 0: continue
                px = np.mean(xb == xv); py = np.mean(yb == yv)
                mi += p * np.log(p / (px * py + 1e-12) + 1e-12)
        out[c] = max(mi, 0.0)
    return out

def masks(n_in, n_out, F, rng, mode, mi):
    F = min(F, n_in); order = np.argsort(-mi)
    rows = []
    for _ in range(n_out):
        if mode == "uniform":  rows.append(rng.choice(n_in, size=F, replace=False))
        elif mode == "top":    rows.append(order[:F].copy())
        elif mode == "top2f":  rows.append(rng.choice(order[:min(2*F, n_in)], size=F, replace=False))
        elif mode == "prop":
            p = mi + 1e-9; p = p / p.sum()
            rows.append(rng.choice(n_in, size=F, replace=False, p=p))
    return np.stack(rows)

def q(W):
    s = np.mean(np.abs(W))/(np.mean(np.abs(LV[LV!=0]))+1e-9)+1e-9
    return LV[np.argmin(np.abs(W[...,None]/s - LV[None,None,:]), axis=-1)]*s

def fwd(X, idx, Q): return np.einsum('nof,of->no', X[:, idx], Q)

def run(Xtr, ytr, Xte, yte, mi, seed, F=6, mode="uniform", hidden=64, out_fanin=64,
        epochs=8, lr=0.05, thr=2.0, bs=256):
    rng = np.random.default_rng(seed)
    idx1 = masks(Xtr.shape[1], hidden, F, rng, mode, mi)
    f2 = min(out_fanin, hidden)
    idx2 = np.stack([rng.choice(hidden, size=f2, replace=False)])
    W1 = rng.normal(0, 1/np.sqrt(idx1.shape[1]), idx1.shape).astype(np.float32)
    W2 = rng.normal(0, 1/np.sqrt(f2), idx2.shape).astype(np.float32)
    for _ in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, max(len(Xtr)-bs,1), bs):
            b = perm[i:i+bs]; x, y = Xtr[b], ytr[b][:,None]
            Q1, Q2 = q(W1), q(W2)
            a1 = fwd(x, idx1, Q1); sd = a1.std()+1e-6; a1 = a1/sd*thr   # W748 fix, always on
            h = np.tanh(a1-thr)*0.5 + np.tanh(a1+thr)*0.5
            o = fwd(h, idx2, Q2)
            p = 1/(1+np.exp(-np.clip(o,-30,30))); g = (p-y)/len(b)
            gW2 = np.einsum('no,nof->of', g, h[:, idx2])
            gh = np.zeros_like(h); np.add.at(gh,(slice(None),idx2[0]), g*Q2[0][None,:])
            ga = gh*(0.5*(1-np.tanh(a1-thr)**2)+0.5*(1-np.tanh(a1+thr)**2))/sd*thr
            W1 -= lr*np.einsum('no,nof->of', ga, x[:, idx1]); W2 -= lr*gW2
    Q1, Q2 = q(W1), q(W2)
    a1 = fwd(Xte, idx1, Q1); a1 = a1/(a1.std()+1e-6)*thr
    h = np.tanh(a1-thr)*0.5 + np.tanh(a1+thr)*0.5
    return float(np.mean((fwd(h, idx2, Q2).ravel() > 0) == (yte > 0.5)))

if __name__ == "__main__":
    path, out_path = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 15
    d = np.load(path); tr, te = d["train"], d["test"]
    rng = np.random.default_rng(0); sel = rng.permutation(len(tr))[:40000]
    Xtr = tr[sel,:-1].astype(np.float32)*2-1; ytr = tr[sel,-1].astype(np.float32)
    Xte = te[:,:-1].astype(np.float32)*2-1;   yte = te[:,-1].astype(np.float32)
    t0 = time.time(); mi = mutual_info(Xtr, ytr)
    print(f"  MI computed in {time.time()-t0:.0f}s; top feature MI={mi.max():.4f}, "
          f"median={np.median(mi):.5f}, zero-MI features={int((mi<1e-9).sum())}/{len(mi)}", flush=True)
    res = {}
    for F in [4, 6, 12]:
        for mode in ["uniform", "top", "top2f", "prop"]:
            k = f"F{F}_{mode}"; t0 = time.time()
            res[k] = [run(Xtr, ytr, Xte, yte, mi, 1000+s, F=F, mode=mode) for s in range(seeds)]
            print(f"  {k:<14}: {np.mean(res[k])*100:6.2f}%  ({time.time()-t0:.0f}s)", flush=True)
    json.dump(res, open(out_path, "w"), indent=1)
    print("  written:", out_path, flush=True)
