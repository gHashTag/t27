"""W750: ONE system that is both small and accurate. T319b said we had none.

THE ADMISSION THIS ANSWERS. Our 86.66% is dense at ~200k LUT; our 128 LUT is
sparse at 78.7%. Two configurations quoted as if they were one system. The field
has ONE: 89 LUT at 92%. Every ingredient has now been measured separately --
sparse truth tables (429x area, 3.6x Fmax, -4.7 pp), inter-layer normalisation
(+29 pp at depth), full training budget (+3.3 pp) -- and never combined.

THE HARDWARE CONSTRAINT THAT PICKS THE FAN-IN. A neuron of F one-bit inputs is a
function of F bits. At F <= 6 that is exactly one LUT6 per output bit: 2.00
LUT/neuron, measured. At F = 12 the function needs 2^12 entries and yosys must
cascade -- roughly 12-16 LUT/neuron, six to eight times the cost, for the +2 pp
that W749 measured. So the fan-in choice is not free and both must be built.

FORECAST, REGISTERED BEFORE THE RUN (T44).
  (1) Sparse + full training + normalisation reaches 81-84% at fan-in 6, i.e.
      +3 pp over the probe's 78.7%, mirroring the dense +3.3 pp.
  (2) Fan-in 12 buys ~+2 pp of accuracy for ~6x the LUT, so LUT-per-point of
      accuracy will FAVOUR fan-in 6 and the small system will be the right one.
  (3) The combined system will NOT reach 92%: predicted final gap 8-11 points at
      an area within 5x of the field's 89 LUT.
If (3) is wrong and the combination reaches the field, then the ingredients were
super-additive and every "residue" estimate in T316/T319 was too pessimistic.
"""
import numpy as np, json, sys, time

LV = np.array([-4,-2,-1,0,1,2,4], dtype=np.float32)

def masks(n_in, n_out, F, rng, mode, mi):
    F = min(F, n_in); order = np.argsort(-mi)
    if mode == "top":     return np.stack([order[:F].copy() for _ in range(n_out)])
    if mode == "uniform": return np.stack([rng.choice(n_in, size=F, replace=False) for _ in range(n_out)])
    p = mi + 1e-9; p = p/p.sum()
    return np.stack([rng.choice(n_in, size=F, replace=False, p=p) for _ in range(n_out)])

def mutual_info(X, y):
    n = X.shape[1]; out = np.zeros(n); yb = y > 0.5
    py1 = yb.mean(); 
    for c in range(n):
        xb = X[:, c] > 0; mi = 0.0
        for xv in (False, True):
            px = np.mean(xb == xv)
            if px <= 0: continue
            for yv in (False, True):
                p = np.mean((xb == xv) & (yb == yv))
                if p <= 0: continue
                py = py1 if yv else 1-py1
                mi += p*np.log(p/(px*py + 1e-12) + 1e-12)
        out[c] = max(mi, 0.0)
    return out

def q(W):
    s = np.mean(np.abs(W))/(np.mean(np.abs(LV[LV!=0]))+1e-9)+1e-9
    return LV[np.argmin(np.abs(W[...,None]/s - LV[None,None,:]), axis=-1)]*s

def fwd(X, idx, Q): return np.einsum('nof,of->no', X[:, idx], Q)

def train(Xtr,ytr,Xva,yva,Xte,yte,mi,seed,F=6,mode="prop",hidden=64,out_fanin=64,
          epochs=60,lr0=0.1,thr=2.0,bs=512,patience=8):
    rng=np.random.default_rng(seed)
    idx1=masks(Xtr.shape[1],hidden,F,rng,mode,mi)
    f2=min(out_fanin,hidden); idx2=np.stack([rng.choice(hidden,size=f2,replace=False)])
    W1=rng.normal(0,1/np.sqrt(idx1.shape[1]),idx1.shape).astype(np.float32)
    W2=rng.normal(0,1/np.sqrt(f2),idx2.shape).astype(np.float32)
    best_va,best,bad=-1.0,None,0
    for ep in range(epochs):
        lr=lr0*(0.5**(ep//12)); perm=rng.permutation(len(Xtr))
        for i in range(0,max(len(Xtr)-bs,1),bs):
            b=perm[i:i+bs]; x,y=Xtr[b],ytr[b][:,None]
            Q1,Q2=q(W1),q(W2)
            a1=fwd(x,idx1,Q1); sd=a1.std()+1e-6; a1=a1/sd*thr
            h=np.tanh(a1-thr)*0.5+np.tanh(a1+thr)*0.5
            o=fwd(h,idx2,Q2); p=1/(1+np.exp(-np.clip(o,-30,30))); g=(p-y)/len(b)
            gW2=np.einsum('no,nof->of',g,h[:,idx2])
            gh=np.zeros_like(h); np.add.at(gh,(slice(None),idx2[0]),g*Q2[0][None,:])
            ga=gh*(0.5*(1-np.tanh(a1-thr)**2)+0.5*(1-np.tanh(a1+thr)**2))/sd*thr
            W1-=lr*np.einsum('no,nof->of',ga,x[:,idx1]); W2-=lr*gW2
        Q1,Q2=q(W1),q(W2)
        a1=fwd(Xva,idx1,Q1); a1=a1/(a1.std()+1e-6)*thr
        h=np.tanh(a1-thr)*0.5+np.tanh(a1+thr)*0.5
        va=float(np.mean((fwd(h,idx2,Q2).ravel()>0)==(yva>0.5)))
        if va>best_va: best_va,best,bad=va,(W1.copy(),W2.copy()),0
        else:
            bad+=1
            if bad>=patience: break
    W1,W2=best; Q1,Q2=q(W1),q(W2)
    a1=fwd(Xte,idx1,Q1); a1=a1/(a1.std()+1e-6)*thr
    h=np.tanh(a1-thr)*0.5+np.tanh(a1+thr)*0.5
    return float(np.mean((fwd(h,idx2,Q2).ravel()>0)==(yte>0.5))), best_va, ep+1

if __name__=="__main__":
    path,out_path=sys.argv[1],sys.argv[2]
    seeds=int(sys.argv[3]) if len(sys.argv)>3 else 5
    d=np.load(path); tr,te=d["train"],d["test"]
    rng=np.random.default_rng(0); perm=rng.permutation(len(tr)); nva=len(tr)//10
    va,trn=perm[:nva],perm[nva:]
    Xtr=tr[trn,:-1].astype(np.float32)*2-1; ytr=tr[trn,-1].astype(np.float32)
    Xva=tr[va,:-1].astype(np.float32)*2-1;  yva=tr[va,-1].astype(np.float32)
    Xte=te[:,:-1].astype(np.float32)*2-1;   yte=te[:,-1].astype(np.float32)
    print(f"  train {Xtr.shape} val {Xva.shape} test {Xte.shape}",flush=True)
    mi=mutual_info(Xtr,ytr)
    res={}
    for F,mode in [(6,"prop"),(6,"top"),(12,"prop"),(12,"top")]:
        k=f"F{F}_{mode}"; t0=time.time(); accs=[]
        for s in range(seeds):
            a,bv,ep=train(Xtr,ytr,Xva,yva,Xte,yte,mi,1000+s,F=F,mode=mode)
            accs.append(a)
        res[k]=accs
        print(f"  {k:<10}: test {np.mean(accs)*100:6.2f}% +- {np.std(accs,ddof=1)*100:4.2f}  ({time.time()-t0:.0f}s)",flush=True)
    json.dump(res,open(out_path,"w"),indent=1)
    print("  written:",out_path,flush=True)
