"""W758: is the 84% ceiling a property of the TASK or of the ARCHITECTURE?

Nine waves improved one answer on one dataset and hit its ceiling. T354 asserted
the limit is "the capacity of six-input truth tables on UNSW-NB15" -- but that
sentence contains a dataset, and nothing has ever varied it.

THE TEST. Run the SAME architecture family on three tasks and measure the GAP
between a dense reference and the sparse truth-table network on each. A gap that
is CONSTANT across tasks is a property of the architecture. A gap that MOVES is a
property of the task, and T354 overreached.

Absolute ceilings differ by task and tell us nothing on their own -- MNIST is
easier than UNSW. The gap is the invariant to look at, and it has never been
computed even once.

FORECAST, REGISTERED BEFORE THE RUN (T44).
 (1) The dense-minus-sparse gap will be LARGER on MNIST and Fashion than on
     UNSW. Reasoning: UNSW's 593 binary features are mostly irrelevant (T306
     showed most carry near-zero mutual information), so a 6-of-593 draw loses
     little; MNIST's 784 pixels are spatially correlated and informative almost
     everywhere, so throwing away 778 of them per neuron should cost more.
     Predicted gaps: UNSW 5-7 pp, MNIST 10-18 pp, Fashion 8-15 pp.
 (2) Therefore T354's phrasing is too strong and will need narrowing to
     "on tasks whose features are individually weak".
If the gap is CONSTANT within 2 pp across all three, T354 stands as written and
the ceiling is architectural in the strong sense.
"""
import numpy as np, json, sys, time
sys.path.insert(0,"/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import one_system as OS
import pareto as P

LV = OS.LV; THR = 2.0

def dense(Xtr,ytr,Xva,yva,Xte,yte,seed,hidden=256,epochs=60,lr0=0.1,bs=512,patience=8):
    """The dense reference: same quantiser, same normalisation, same balancing."""
    rng=np.random.default_rng(seed); n_in=Xtr.shape[1]
    W1=rng.normal(0,1/np.sqrt(n_in),(n_in,hidden)).astype(np.float32)
    W2=rng.normal(0,1/np.sqrt(hidden),(hidden,1)).astype(np.float32)
    p1=ytr.mean(); wp,wn=0.5/p1,0.5/(1-p1)
    best_va,best,bad=-1.0,None,0
    for ep in range(epochs):
        lr=lr0*(0.5**(ep//12)); pm=rng.permutation(len(Xtr))
        for i in range(0,max(len(Xtr)-bs,1),bs):
            b=pm[i:i+bs]; x,y=Xtr[b],ytr[b][:,None]
            Q1,Q2=OS.q(W1),OS.q(W2)
            a1=x@Q1; sd=a1.std()+1e-6; a1=a1/sd*THR
            h=np.tanh(a1-THR)*0.5+np.tanh(a1+THR)*0.5
            o=h@Q2; p=1/(1+np.exp(-np.clip(o,-30,30)))
            g=np.where(y>0.5,wp,wn)*(p-y)/len(b)
            gW2=h.T@g
            ga=(g@Q2.T)*(0.5*(1-np.tanh(a1-THR)**2)+0.5*(1-np.tanh(a1+THR)**2))/sd*THR
            W1-=lr*(x.T@ga); W2-=lr*gW2
        Q1,Q2=OS.q(W1),OS.q(W2)
        a1=Xva@Q1; a1=a1/(a1.std()+1e-6)*THR
        h=np.tanh(a1-THR)*0.5+np.tanh(a1+THR)*0.5
        sc=(h@Q2).ravel()
        v=0.5*(np.mean(sc[yva>0.5]>0)+np.mean(sc[yva<=0.5]<=0))
        if v>best_va: best_va,best,bad=v,(W1.copy(),W2.copy()),0
        else:
            bad+=1
            if bad>=patience: break
    W1,W2=best; Q1,Q2=OS.q(W1),OS.q(W2)
    a1=Xte@Q1; a1=a1/(a1.std()+1e-6)*THR
    h=np.tanh(a1-THR)*0.5+np.tanh(a1+THR)*0.5
    return float(np.mean(((h@Q2).ravel()>0)==(yte>0.5)))

def load(path):
    d=np.load(path); tr,te=d["train"],d["test"]
    rng=np.random.default_rng(0); perm=rng.permutation(len(tr)); nva=len(tr)//10
    va,trn=perm[:nva],perm[nva:]
    return (tr[trn,:-1].astype(np.float32)*2-1, tr[trn,-1].astype(np.float32),
            tr[va,:-1].astype(np.float32)*2-1,  tr[va,-1].astype(np.float32),
            te[:,:-1].astype(np.float32)*2-1,   te[:,-1].astype(np.float32))

if __name__=="__main__":
    out=sys.argv[1]; seeds=int(sys.argv[2]) if len(sys.argv)>2 else 3
    G="/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/gat"
    TASKS=[("UNSW",f"{G}/unsw.npz"),("MNIST",f"{G}/mnist_bin.npz"),("Fashion",f"{G}/fashion_bin.npz")]
    res={}
    print(f"  {'task':<9}{'baseline':>9}{'dense':>9}{'sparse':>9}{'GAP':>8}",flush=True)
    for name,path in TASKS:
        Xtr,ytr,Xva,yva,Xte,yte=load(path)
        base=max(yte.mean(),1-yte.mean())*100
        mi=OS.mutual_info(Xtr,ytr)
        t0=time.time()
        dn=[dense(Xtr,ytr,Xva,yva,Xte,yte,1000+s) for s in range(seeds)]
        sp=[P.train(Xtr,ytr,Xva,yva,Xte,yte,mi,1000+s,H=48,L=3)[0] for s in range(seeds)]
        d_,s_=np.mean(dn)*100, np.mean(sp)*100
        res[name]={"base":base,"dense":dn,"sparse":sp}
        print(f"  {name:<9}{base:>8.2f}%{d_:>8.2f}%{s_:>8.2f}%{d_-s_:>+8.2f}   ({time.time()-t0:.0f}s)",flush=True)
    json.dump(res,open(out,"w"),indent=1)
    gaps=[np.mean(v["dense"])*100-np.mean(v["sparse"])*100 for v in res.values()]
    print(f"\n  gaps: {[f'{g:+.2f}' for g in gaps]}   spread {max(gaps)-min(gaps):.2f} pp")
    print("  ->", "ARCHITECTURAL (constant within 2 pp)" if max(gaps)-min(gaps)<2.0
          else "TASK-DEPENDENT -- T354 overreached")
