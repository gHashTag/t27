"""W760: sixty labellings of ONE dataset, and an honest search for a predictor.

W759 tested one pre-registered predictor on eleven tasks and it failed at
r = +0.128. The obvious next move -- try more predictors -- is exactly how a
programme fools itself: ten candidates on eleven points will find one by chance.

THE DESIGN THAT PREVENTS THAT.
  * SIXTY tasks, all labellings of MNIST: 45 digit pairs, 10 one-vs-rest, and 5
    groupings (even/odd, low/high, and three arbitrary splits). The INPUT
    DISTRIBUTION IS IDENTICAL across all sixty -- only the decision changes. This
    is T359's observation turned into the experiment's design.
  * The sixty are split into a DISCOVERY half (30) and a CONFIRMATION half (30)
    by a fixed seed, before anything is measured.
  * Candidate predictors are proposed and ranked on DISCOVERY only. The single
    best is then tested ONCE on CONFIRMATION. That number is the result; the
    discovery correlations are not.

CANDIDATES, all defined before any measurement:
  c6      fraction of total per-feature MI in the top 6 features (W759's, refuted)
  c48     the same at 48 features -- the layer's whole first-layer view
  mi_tot  total per-feature MI summed over features
  mi_max  the single strongest feature's MI
  dense   the dense reference's accuracy
  head    100 - dense
  ntrain  training rows for that labelling
  bal     class balance, |p - 0.5|

FORECAST, REGISTERED BEFORE THE RUN (T44). At least one candidate reaches
|r| >= 0.5 on DISCOVERY -- with eight candidates on thirty points that is nearly
certain by chance alone. Predicted: the winner does NOT replicate, landing at
|r| < 0.3 on CONFIRMATION, and the honest conclusion stays "no predictor known".
I expect `head` to be the discovery winner, because a task with more headroom has
more room to lose, and I expect it to shrink on confirmation.
If a candidate holds |r| >= 0.5 on BOTH halves, the project has its first real
rule for choosing tasks and T359a can be strengthened.
"""
import numpy as np, json, sys, time
sys.path.insert(0,"/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import one_system as OS, pareto as P, task_or_arch as TA

def build(dtr, dte, tr, te, sel_tr, sel_te, lab_tr, lab_te):
    Xtr=tr[sel_tr,:-1].astype(np.float32)*2-1; ytr=lab_tr.astype(np.float32)
    Xte=te[sel_te,:-1].astype(np.float32)*2-1; yte=lab_te.astype(np.float32)
    n=len(Xtr); nva=max(n//10,1); idx=np.random.default_rng(0).permutation(n)
    return Xtr[idx[nva:]],ytr[idx[nva:]],Xtr[idx[:nva]],ytr[idx[:nva]],Xte,yte

def tasks(tr,te,dtr,dte):
    out=[]
    for a in range(10):
        for b in range(a+1,10):
            mtr=(dtr==a)|(dtr==b); mte=(dte==a)|(dte==b)
            out.append((f"p{a}{b}", build(dtr,dte,tr,te,mtr,mte,(dtr[mtr]==b),(dte[mte]==b))))
    for a in range(10):
        allt=np.ones(len(dtr),bool); alle=np.ones(len(dte),bool)
        out.append((f"r{a}", build(dtr,dte,tr,te,allt,alle,(dtr==a),(dte==a))))
    rng=np.random.default_rng(11)
    grouping=[("even",set(range(0,10,2))),("low",set(range(5)))]
    for i in range(3):
        g=set(rng.choice(10,size=5,replace=False).tolist()); grouping.append((f"g{i}",g))
    for nm,g in grouping:
        allt=np.ones(len(dtr),bool); alle=np.ones(len(dte),bool)
        out.append((nm, build(dtr,dte,tr,te,allt,alle,
                              np.isin(dtr,list(g)),np.isin(dte,list(g)))))
    return out

def feats(mi, Xtr, ytr, dense_acc):
    s=mi.sum()+1e-12
    return {"c6":float(np.sort(mi)[-6:].sum()/s), "c48":float(np.sort(mi)[-48:].sum()/s),
            "mi_tot":float(s), "mi_max":float(mi.max()),
            "dense":float(dense_acc), "head":float(100-dense_acc),
            "ntrain":float(len(Xtr)), "bal":float(abs(ytr.mean()-0.5))}

if __name__=="__main__":
    out=sys.argv[1]; seeds=int(sys.argv[2]) if len(sys.argv)>2 else 2
    G="/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/gat"
    d=np.load(f"{G}/mnist_dig.npz")
    T=tasks(d["train"],d["test"],d["dig_train"],d["dig_test"])
    print(f"  {len(T)} labellings of ONE dataset",flush=True)
    rows=[]
    for i,(nm,(Xtr,ytr,Xva,yva,Xte,yte)) in enumerate(T):
        t0=time.time(); mi=OS.mutual_info(Xtr,ytr)
        dn=np.mean([TA.dense(Xtr,ytr,Xva,yva,Xte,yte,1000+s) for s in range(seeds)])*100
        sp=np.mean([P.train(Xtr,ytr,Xva,yva,Xte,yte,mi,1000+s,H=48,L=3)[0] for s in range(seeds)])*100
        r=feats(mi,Xtr,ytr,dn); r.update({"task":nm,"sparse":float(sp),"penalty":float(dn-sp)})
        rows.append(r)
        if i%10==0 or i==len(T)-1:
            print(f"    [{i+1}/{len(T)}] {nm:<5} dense {dn:6.2f} sparse {sp:6.2f} pen {dn-sp:+6.2f} ({time.time()-t0:.0f}s)",flush=True)
    json.dump(rows,open(out,"w"),indent=1); print("  written:",out,flush=True)
