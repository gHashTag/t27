"""W750 track B: the UNSW val/test gap is PRIOR SHIFT, and prior shift is fixable.

MEASURED FIRST, so the fix is not a guess. We are on the official Moustafa & Slay
partition (175,341 / 82,332 -- verified by shape). Per-feature marginal drift
between train and test is tiny: mean 0.019, max 0.085, and NOT ONE of the 593
features drifts past 0.10. What moves is the LABEL PRIOR: 68.06% positive in
train against 55.06% in test, a thirteen-point shift.

Our decision rule thresholds the output at zero, which is calibrated for whatever
prior the training set carried. Against a test set with a different prior it
over-predicts the training majority. That is textbook prior probability shift.

TWO FIXES, ONLY ONE OF WHICH IS HONEST:
  balanced  -- reweight the training loss so each class contributes equally.
               Uses NO test information. This is the fix we may keep.
  oracle    -- move the threshold to the value that maximises TEST accuracy.
               This LEAKS the test set and is reported ONLY as an upper bound on
               what any calibration could buy. It is not a result.

FORECAST, REGISTERED BEFORE THE RUN (T44). `balanced` recovers 2-5 pp on test
over the 86.66% baseline; `oracle` recovers 4-8 pp and bounds the rest. If
`balanced` recovers nothing, the gap is not prior shift and the drift measurement
above is misleading me.
"""
import numpy as np, json, sys, time

LV = np.array([-4,-2,-1,0,1,2,4], dtype=np.float32)

def q(W):
    s = np.mean(np.abs(W))/(np.mean(np.abs(LV[LV!=0]))+1e-9)+1e-9
    return LV[np.argmin(np.abs(W[...,None]/s - LV[None,None,:]), axis=-1)]*s

def train(Xtr,ytr,Xva,yva,seed,hidden=256,epochs=60,lr0=0.1,thr=2.0,bs=512,
          patience=8,balanced=False):
    rng=np.random.default_rng(seed); n_in=Xtr.shape[1]
    W1=rng.normal(0,1/np.sqrt(n_in),(n_in,hidden)).astype(np.float32)
    W2=rng.normal(0,1/np.sqrt(hidden),(hidden,1)).astype(np.float32)
    # class weights that make the two classes contribute equally
    if balanced:
        p1=ytr.mean(); w_pos, w_neg = 0.5/max(p1,1e-6), 0.5/max(1-p1,1e-6)
    else:
        w_pos = w_neg = 1.0
    best_va,best,bad=-1.0,None,0
    for ep in range(epochs):
        lr=lr0*(0.5**(ep//12)); perm=rng.permutation(len(Xtr))
        for i in range(0,max(len(Xtr)-bs,1),bs):
            b=perm[i:i+bs]; x,y=Xtr[b],ytr[b][:,None]
            Q1,Q2=q(W1),q(W2)
            a1=x@Q1; sd=a1.std()+1e-6; a1=a1/sd*thr
            h=np.tanh(a1-thr)*0.5+np.tanh(a1+thr)*0.5
            o=h@Q2; p=1/(1+np.exp(-np.clip(o,-30,30)))
            wt=np.where(y>0.5,w_pos,w_neg)
            g=wt*(p-y)/len(b)
            gW2=h.T@g
            ga=(g@Q2.T)*(0.5*(1-np.tanh(a1-thr)**2)+0.5*(1-np.tanh(a1+thr)**2))/sd*thr
            W1-=lr*(x.T@ga); W2-=lr*gW2
        Q1,Q2=q(W1),q(W2)
        a1=Xva@Q1; a1=a1/(a1.std()+1e-6)*thr
        h=np.tanh(a1-thr)*0.5+np.tanh(a1+thr)*0.5
        sc=(h@Q2).ravel()
        # balanced accuracy on validation, so early stopping is not itself biased
        va = 0.5*(np.mean(sc[yva>0.5]>0) + np.mean(sc[yva<=0.5]<=0)) if balanced \
             else float(np.mean((sc>0)==(yva>0.5)))
        if va>best_va: best_va,best,bad=va,(W1.copy(),W2.copy()),0
        else:
            bad+=1
            if bad>=patience: break
    return best, best_va

def scores(W, Xte, thr=2.0):
    W1,W2=W; Q1,Q2=q(W1),q(W2)
    a1=Xte@Q1; a1=a1/(a1.std()+1e-6)*thr
    h=np.tanh(a1-thr)*0.5+np.tanh(a1+thr)*0.5
    return (h@Q2).ravel()

if __name__=="__main__":
    path,out_path=sys.argv[1],sys.argv[2]
    seeds=int(sys.argv[3]) if len(sys.argv)>3 else 5
    d=np.load(path); tr,te=d["train"],d["test"]
    rng=np.random.default_rng(0); perm=rng.permutation(len(tr)); nva=len(tr)//10
    va,trn=perm[:nva],perm[nva:]
    Xtr=tr[trn,:-1].astype(np.float32)*2-1; ytr=tr[trn,-1].astype(np.float32)
    Xva=tr[va,:-1].astype(np.float32)*2-1;  yva=tr[va,-1].astype(np.float32)
    Xte=te[:,:-1].astype(np.float32)*2-1;   yte=te[:,-1].astype(np.float32)
    print(f"  train prior {ytr.mean()*100:.2f}% -> test prior {yte.mean()*100:.2f}%",flush=True)
    res={}
    for tag,bal in (("plain",False),("balanced",True)):
        accs=[];orc=[]
        for s in range(seeds):
            W,_=train(Xtr,ytr,Xva,yva,1000+s,balanced=bal)
            sc=scores(W,Xte)
            accs.append(float(np.mean((sc>0)==(yte>0.5))))
            # ORACLE: best possible threshold on the test set. An upper bound, not a result.
            cand=np.quantile(sc,np.linspace(0.01,0.99,199))
            orc.append(max(float(np.mean((sc>c)==(yte>0.5))) for c in cand))
        res[tag]=accs; res[tag+"_oracle"]=orc
        print(f"  {tag:<9}: test {np.mean(accs)*100:6.2f}% +- {np.std(accs,ddof=1)*100:4.2f}   "
              f"[oracle threshold {np.mean(orc)*100:6.2f}%]",flush=True)
    json.dump(res,open(out_path,"w"),indent=1)
    print("  written:",out_path,flush=True)
