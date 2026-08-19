"""W766: the file contains two numbers for one quantity. Resolve it.

T317 records the alphabet-SIZE effect (3 levels -> 9) at **+0.735 pp**, pooled
over three tasks. W765 measured the same step on the same architecture at
**+0.10 pp** (UNSW, not significant) and **+0.25** (Fashion). Two numbers, one
quantity, both in the theorem file. That is a defect regardless of which is right.

THE ONLY DIFFERENCE BETWEEN THE BENCHES is the training budget:
`alphabet_fixed.py` runs **8 epochs**; `bases.py` runs **30**. Hidden width,
quantiser, normalisation, class balancing and task set are otherwise the same.

THE HYPOTHESIS. A larger alphabet is a CRUTCH FOR AN UNDER-TRAINED NETWORK.
Given enough gradient steps the network compensates for a coarse alphabet by
moving other weights, so the benefit of extra levels decays with budget. If true,
then the programme's own ranking is not a list of independent effects: the
alphabet's +0.735 pp was measured in a regime the training-budget fix (+3.30 pp,
W749) has since removed.

FORECAST, REGISTERED BEFORE THE RUN (T44).
 (1) The size effect DECAYS MONOTONICALLY with epochs: about +0.7 pp at 8 epochs,
     +0.15 to +0.3 at 30, and under +0.1 and not significant at 60.
 (2) Therefore BOTH recorded numbers are correct measurements of DIFFERENT
     regimes, and T317 needs its budget stated rather than its value corrected.
 (3) The same decay will appear on both tasks, since the mechanism is about
     optimisation and not about the data.
If the effect does NOT decay -- if it is flat in epochs -- then the two benches
differ in something I have not identified and the contradiction is unexplained,
which is worse than either number being wrong.
"""
import numpy as np, json, sys, time
sys.path.insert(0,"/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import task_or_arch as TA, bases as B
from math import sqrt

def lv(n):
    k=(n-1)//2; pos=[2.0**i for i in range(k)]
    return np.array(sorted([-p for p in pos]+[0.0]+pos),dtype=np.float32)

if __name__=="__main__":
    out=sys.argv[1]; seeds=int(sys.argv[2]) if len(sys.argv)>2 else 8
    G="/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/gat"
    res={}
    print(f"  {'task':<9}{'epochs':>8}{'3 lvl':>9}{'9 lvl':>9}{'SIZE effect':>13}{'t':>8}",flush=True)
    for tname,path in (("UNSW","unsw.npz"),("Fashion","fashion_bin.npz")):
        Xtr,ytr,Xva,yva,Xte,yte=TA.load(f"{G}/{path}")
        for ep in (8,15,30,60):
            t0=time.time()
            a3=np.array([B.train(Xtr,ytr,Xva,yva,Xte,yte,lv(3),1000+s,epochs=ep,patience=99) for s in range(seeds)])*100
            a9=np.array([B.train(Xtr,ytr,Xva,yva,Xte,yte,lv(9),1000+s,epochs=ep,patience=99) for s in range(seeds)])*100
            d=a9-a3; t=d.mean()/(d.std(ddof=1)/sqrt(len(d))+1e-12)
            res[f"{tname}_{ep}"]={"a3":a3.tolist(),"a9":a9.tolist(),"delta":float(d.mean()),"t":float(t)}
            print(f"  {tname:<9}{ep:>8}{a3.mean():>8.2f}%{a9.mean():>8.2f}%{d.mean():>+12.3f}{t:>8.2f}"
                  f"   {'*' if abs(t)>2.36 else ''}  ({time.time()-t0:.0f}s)",flush=True)
            json.dump(res,open(out,"w"),indent=1)
    print("  written:",out,flush=True)
