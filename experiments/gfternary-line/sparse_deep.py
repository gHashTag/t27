"""W748: does DEPTH buy back what sparsity costs? (T311a)

T311 left the field 14 points ahead at area parity, with depth named as the
untested difference: our network is two layers, LogicNets/PolyLUT/NeuraLUT are
four to five. Narrow fan-in limits what ONE neuron can see; stacking layers lets
a path through the network reach F^L inputs. At F=6 and L=4 that is 1296 -- more
than UNSW's 593 -- so depth should in principle restore full receptive field.

FORECAST, REGISTERED BEFORE THE RUN (T44). Depth WILL buy accuracy back, and the
gain will be largest going 2->3 layers and then flatten, mirroring the alphabet
saturation of T288. Predicted: L=4 at fan-in 6 reaches 78-83%, i.e. matching or
beating our DENSE two-layer 83.4% at ~1/400th the area. If depth does NOT help,
the residual is training method alone and no structural change we can make will
close it.

AREA NOTE: each extra layer costs 2 LUT/neuron, so L=4 with 64-wide hidden is
about 8*64 = 512 LUT -- still under 1% of the dense design.
"""
import numpy as np, json, sys, time

LV = np.array([-4,-2,-1,0,1,2,4], dtype=np.float32)

def masks(n_in, n_out, F, rng):
    return np.stack([rng.choice(n_in, size=min(F, n_in), replace=False) for _ in range(n_out)])

def q(W):
    s = np.mean(np.abs(W)) / (np.mean(np.abs(LV[LV != 0])) + 1e-9) + 1e-9
    return LV[np.argmin(np.abs(W[..., None]/s - LV[None, None, :]), axis=-1)] * s

def fwd(X, idx, Q):
    return np.einsum('nof,of->no', X[:, idx], Q)

def run(Xtr, ytr, Xte, yte, seed, F=6, L=3, hidden=64, out_fanin=64,
        epochs=8, lr=0.05, thr=2.0, bs=256):
    rng = np.random.default_rng(seed)
    sizes = [Xtr.shape[1]] + [hidden]*(L-1) + [1]
    idxs, Ws = [], []
    for li in range(L):
        f = out_fanin if li == L-1 else F
        idxs.append(masks(sizes[li], sizes[li+1], f, rng))
        Ws.append(rng.normal(0, 1/np.sqrt(idxs[-1].shape[1]), idxs[-1].shape).astype(np.float32))
    act = lambda a: np.tanh(a-thr)*0.5 + np.tanh(a+thr)*0.5
    dact = lambda a: 0.5*(1-np.tanh(a-thr)**2) + 0.5*(1-np.tanh(a+thr)**2)
    for _ in range(epochs):
        perm = rng.permutation(len(Xtr))
        for i in range(0, max(len(Xtr)-bs, 1), bs):
            b = perm[i:i+bs]; x, y = Xtr[b], ytr[b][:, None]
            Qs = [q(W) for W in Ws]
            hs, pre = [x], []
            for li in range(L):
                a = fwd(hs[-1], idxs[li], Qs[li]); pre.append(a)
                hs.append(act(a) if li < L-1 else a)
            p = 1/(1+np.exp(-np.clip(hs[-1], -30, 30)))
            g = (p - y)/len(b)
            for li in range(L-1, -1, -1):
                gW = np.einsum('no,nof->of', g, hs[li][:, idxs[li]])
                if li > 0:
                    gprev = np.zeros_like(hs[li])
                    for o in range(idxs[li].shape[0]):
                        np.add.at(gprev, (slice(None), idxs[li][o]), g[:, o:o+1] * Qs[li][o][None, :])
                    g = gprev * dact(pre[li-1])
                Ws[li] -= lr * gW
    Qs = [q(W) for W in Ws]
    h = Xte
    for li in range(L):
        a = fwd(h, idxs[li], Qs[li]); h = act(a) if li < L-1 else a
    return float(np.mean((h.ravel() > 0) == (yte > 0.5)))

if __name__ == "__main__":
    path, out_path = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 10
    d = np.load(path); tr, te = d["train"], d["test"]
    rng = np.random.default_rng(0); sel = rng.permutation(len(tr))[:40000]
    Xtr = tr[sel, :-1].astype(np.float32)*2-1; ytr = tr[sel, -1].astype(np.float32)
    Xte = te[:, :-1].astype(np.float32)*2-1;   yte = te[:, -1].astype(np.float32)
    res = {}
    for F in [6, 12]:
        for L in [2, 3, 4, 5]:
            k = f"F{F}_L{L}"; t0 = time.time()
            res[k] = [run(Xtr, ytr, Xte, yte, 1000+s, F=F, L=L) for s in range(seeds)]
            print(f"  {k:>8}: {np.mean(res[k])*100:6.2f}%  ({time.time()-t0:.0f}s)", flush=True)
    json.dump(res, open(out_path, "w"), indent=1)
    print("  written:", out_path, flush=True)
