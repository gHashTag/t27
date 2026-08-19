"""W749 track A: train ONCE properly, because this project never has.

T316's admission: every accuracy number this repository has published -- all the
alphabet results included -- comes from an 8-epoch probe on a 40,000-row
subsample with no early stopping and no learning-rate schedule. The paired
comparisons survive that; the absolute numbers were never entitled to exist.

W749 has now closed the other residue items: connectivity CHOICE is worth at most
+1.6 pp (mi_connect), and -- the surprise -- `top`, where every neuron sees the
SAME features, is the best arm at fan-in 12. Hidden-layer diversity contributes
nothing. So training budget is what is left.

FORECAST, REGISTERED BEFORE THE RUN (T44). Full training will close most but not
all of the 13-point gap. Predicted: 86-90% on UNSW-NB15, against the probe's
83.4% dense / 78.7% sparse and the field's 92%. Reasoning: 8 epochs on 6% of the
data is a severe handicap, and removing it should be worth several points; but
our activation is a hand-rolled tanh-threshold pair with a straight-through
estimator and no batch statistics beyond the W748 scale fix, which is not what
the field runs. If full training reaches 92%, the architecture was never the
problem and every structural conclusion of W748-W749 is about a phantom.
"""
import numpy as np, json, sys, time

LV = np.array([-4,-2,-1,0,1,2,4], dtype=np.float32)

def q(W):
    s = np.mean(np.abs(W))/(np.mean(np.abs(LV[LV!=0]))+1e-9)+1e-9
    return LV[np.argmin(np.abs(W[...,None]/s - LV[None,None,:]), axis=-1)]*s

def train(Xtr, ytr, Xva, yva, Xte, yte, seed, hidden=256, epochs=60, lr0=0.1,
          thr=2.0, bs=512, patience=8):
    rng = np.random.default_rng(seed); n_in = Xtr.shape[1]
    W1 = rng.normal(0, 1/np.sqrt(n_in), (n_in, hidden)).astype(np.float32)
    W2 = rng.normal(0, 1/np.sqrt(hidden), (hidden, 1)).astype(np.float32)
    best_va, best, bad = -1.0, None, 0
    for ep in range(epochs):
        lr = lr0 * (0.5 ** (ep // 12))          # step schedule
        perm = rng.permutation(len(Xtr))
        for i in range(0, max(len(Xtr)-bs, 1), bs):
            b = perm[i:i+bs]; x, y = Xtr[b], ytr[b][:, None]
            Q1, Q2 = q(W1), q(W2)
            a1 = x @ Q1; sd = a1.std()+1e-6; a1 = a1/sd*thr
            h = np.tanh(a1-thr)*0.5 + np.tanh(a1+thr)*0.5
            o = h @ Q2
            p = 1/(1+np.exp(-np.clip(o,-30,30))); g = (p-y)/len(b)
            gW2 = h.T @ g
            ga = (g @ Q2.T)*(0.5*(1-np.tanh(a1-thr)**2)+0.5*(1-np.tanh(a1+thr)**2))/sd*thr
            W1 -= lr*(x.T @ ga); W2 -= lr*gW2
        Q1, Q2 = q(W1), q(W2)
        a1 = Xva @ Q1; a1 = a1/(a1.std()+1e-6)*thr
        h = np.tanh(a1-thr)*0.5 + np.tanh(a1+thr)*0.5
        va = float(np.mean(((h @ Q2).ravel() > 0) == (yva > 0.5)))
        if va > best_va: best_va, best, bad = va, (W1.copy(), W2.copy()), 0
        else:
            bad += 1
            if bad >= patience: break
    W1, W2 = best; Q1, Q2 = q(W1), q(W2)
    a1 = Xte @ Q1; a1 = a1/(a1.std()+1e-6)*thr
    h = np.tanh(a1-thr)*0.5 + np.tanh(a1+thr)*0.5
    return float(np.mean(((h @ Q2).ravel() > 0) == (yte > 0.5))), best_va, ep+1

if __name__ == "__main__":
    path, out_path = sys.argv[1], sys.argv[2]
    seeds = int(sys.argv[3]) if len(sys.argv) > 3 else 5
    d = np.load(path); tr, te = d["train"], d["test"]
    rng = np.random.default_rng(0); perm = rng.permutation(len(tr))
    nva = len(tr)//10
    va, trn = perm[:nva], perm[nva:]                 # FULL training set, not 40k
    Xtr = tr[trn,:-1].astype(np.float32)*2-1; ytr = tr[trn,-1].astype(np.float32)
    Xva = tr[va,:-1].astype(np.float32)*2-1;  yva = tr[va,-1].astype(np.float32)
    Xte = te[:,:-1].astype(np.float32)*2-1;   yte = te[:,-1].astype(np.float32)
    print(f"  train {Xtr.shape}  val {Xva.shape}  test {Xte.shape}", flush=True)
    res = []
    for s in range(seeds):
        t0 = time.time(); acc, bva, eps = train(Xtr, ytr, Xva, yva, Xte, yte, 1000+s)
        res.append(acc)
        print(f"  seed {s+1}: test {acc*100:6.2f}%  (val {bva*100:.2f}%, {eps} epochs, {time.time()-t0:.0f}s)", flush=True)
    print(f"  MEAN {np.mean(res)*100:.2f}% +- {np.std(res,ddof=1)*100:.2f}", flush=True)
    json.dump(res, open(out_path,"w"), indent=1)
