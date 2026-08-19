"""W759: what PREDICTS the sparse penalty? Turning a word into a measure.

W758 measured the dense-minus-sparse penalty at +3.48 (Fashion), +6.70 (UNSW)
and +14.85 (MNIST) and explained the spread with "how concentrated a task's
evidence is". That is a WORD. Three points would fit any word one cares to
propose, which is why this file does not stop at three.

THE TASKS. Eleven, not three: the three originals plus eight MNIST digit-pairs
of graded difficulty. The pairs share input dimension, trainer, seeds and
subsample -- only the decision changes -- so nothing about the setup is confounded
with the quantity under test.

THE MEASURE, defined BEFORE any correlation is computed. For each task,
    C6 = (sum of the 6 largest per-feature mutual informations) / (sum of all)
i.e. what fraction of the total single-feature evidence a SIX-input neuron could
see if it chose perfectly. Six because that is the fan-in the six-bit rule fixes.
No free parameters, no threshold, nothing to tune after the fact.

FORECAST, REGISTERED BEFORE THE RUN (T44).
 (1) C6 correlates NEGATIVELY with the penalty: more concentrated evidence means
     a six-feature neuron loses less. Predicted Pearson r <= -0.7 over 11 tasks.
 (2) Ordering of the three originals by C6: Fashion > UNSW > MNIST, mirroring
     their penalties 3.48 < 6.70 < 14.85.
 (3) The relation will be MONOTONE but not linear -- penalty should flatten once
     C6 is small, because past some point a neuron sees nothing useful either way.
If r is weaker than -0.5, "evidence concentration" is not the mechanism and W758's
explanation must be withdrawn to a bare observation that the penalty varies.
"""
import numpy as np, json, sys, time
sys.path.insert(0,"/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import one_system as OS, pareto as P, task_or_arch as TA

def concentration(mi, k=6):
    tot = mi.sum()
    if tot <= 0: return 0.0
    return float(np.sort(mi)[-k:].sum() / tot)

def make_pair(dig_tr, dig_te, tr, te, a, b):
    mtr=(dig_tr==a)|(dig_tr==b); mte=(dig_te==a)|(dig_te==b)
    Xtr=tr[mtr,:-1].astype(np.float32)*2-1; ytr=(dig_tr[mtr]==b).astype(np.float32)
    Xte=te[mte,:-1].astype(np.float32)*2-1; yte=(dig_te[mte]==b).astype(np.float32)
    n=len(Xtr); nva=max(n//10,1); idx=np.random.default_rng(0).permutation(n)
    return (Xtr[idx[nva:]],ytr[idx[nva:]],Xtr[idx[:nva]],ytr[idx[:nva]],Xte,yte)

if __name__=="__main__":
    out=sys.argv[1]; seeds=int(sys.argv[2]) if len(sys.argv)>2 else 3
    G="/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/gat"
    tasks=[]
    for nm,pth in (("UNSW",f"{G}/unsw.npz"),("MNIST",f"{G}/mnist_bin.npz"),("Fashion",f"{G}/fashion_bin.npz")):
        tasks.append((nm,)+TA.load(pth))
    d=np.load(f"{G}/mnist_dig.npz")
    tr,te,dtr,dte=d["train"],d["test"],d["dig_train"],d["dig_test"]
    for a,b in [(0,1),(0,8),(3,5),(4,9),(7,9),(2,7),(1,7),(5,6)]:
        tasks.append((f"{a}v{b}",)+make_pair(dtr,dte,tr,te,a,b))
    print(f"  {'task':<9}{'C6':>8}{'dense':>9}{'sparse':>9}{'penalty':>9}",flush=True)
    rows=[]
    for nm,Xtr,ytr,Xva,yva,Xte,yte in tasks:
        t0=time.time(); mi=OS.mutual_info(Xtr,ytr); c6=concentration(mi,6)
        dn=np.mean([TA.dense(Xtr,ytr,Xva,yva,Xte,yte,1000+s) for s in range(seeds)])*100
        sp=np.mean([P.train(Xtr,ytr,Xva,yva,Xte,yte,mi,1000+s,H=48,L=3)[0] for s in range(seeds)])*100
        rows.append({"task":nm,"c6":c6,"dense":float(dn),"sparse":float(sp),"penalty":float(dn-sp)})
        print(f"  {nm:<9}{c6:>8.4f}{dn:>8.2f}%{sp:>8.2f}%{dn-sp:>+9.2f}   ({time.time()-t0:.0f}s)",flush=True)
    json.dump(rows,open(out,"w"),indent=1)
    c=np.array([r["c6"] for r in rows]); p=np.array([r["penalty"] for r in rows])
    r=float(np.corrcoef(c,p)[0,1])
    from math import sqrt
    n=len(rows); t=r*sqrt((n-2)/max(1-r*r,1e-12))
    print(f"\n  n={n}   Pearson r(C6, penalty) = {r:+.3f}   t={t:+.2f}   "
          f"(|t|>2.26 significant at n=11)")
    print("  ->", "CONCENTRATION PREDICTS THE PENALTY" if r<=-0.5 else
          "NOT THE MECHANISM -- W758's explanation must be withdrawn")
