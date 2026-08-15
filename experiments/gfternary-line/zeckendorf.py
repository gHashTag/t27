"""W769: phi's last untested form -- as an ADDITIVE basis, not a base of powers.

Every measurement of phi in this programme used it as a BASE: levels {0, +-phi^k}.
That form gave +0.735 pp of size effect, +0.149 of shape, and a pair resolve
costing 8 DSP48E1. Dmitrii asked about a "golden sieve"; the repo has none, but
it does have ZECKENDORF -- every integer uniquely a sum of NON-CONSECUTIVE
Fibonacci numbers, which is exactly base-phi's forbidden `11`.

That suggests the one form never tried: a weight as a SUM of Fibonacci terms.
In the literature the powers-of-two analogue is APoT (Li, Dong & Wang, ICLR 2020,
arXiv:1909.13144), which exists precisely because plain PoT has "rigid
resolution". Nobody here has asked whether the Fibonacci version does better.

THE 2x2, at MATCHED CARDINALITY so shape is tested and not size:
                    single term          additive (2 terms)
  powers of two     pot9  {0,+-2^k}       apot9  {0,+-(2^i+2^j)}
  Fibonacci         fib9  {0,+-F_k}       zeck9  {0,+-(F_i+F_j)}, i,j non-consecutive

Nine levels in every arm. `fib9` is the control that separates "is it the
ADDITIVE structure or the FIBONACCI values?" -- without it a win by zeck9 would
be unattributable.

FORECAST, REGISTERED BEFORE THE RUN (T44).
 (1) All four arms land within 0.5 pp of each other, none significantly best.
     Reasoning: every shape comparison this programme has run came in under
     0.5 pp (T286 +0.085 pooled, T317 +0.149, W763 spread 0.49 across ELEVEN
     bases). A 2x2 of shapes is another shape comparison.
 (2) Additive arms cost about 2x the dense adder tree, because each weight
     contributes two terms instead of one; in the TABLE architecture all four
     cost the same, because the table absorbs the arithmetic (T366).
 (3) Therefore Zeckendorf closes phi's last door rather than opening it.
If zeck9 beats pot9 by more than 0.5 pp, the additive-Fibonacci form is the one
phi variant this programme never found, and T365c's "the number is an area
decision" needs reopening.
"""
import numpy as np, json, sys, time
sys.path.insert(0,"/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import task_or_arch as TA, bases as B

def fib(n):
    a,b=1,2; out=[]
    for _ in range(n): out.append(a); a,b=b,a+b
    return out                                    # 1,2,3,5,8,13,...

def levels_single(vals, k):
    pos=sorted(vals)[:k]
    return np.array(sorted([-p for p in pos]+[0.0]+list(pos)),dtype=np.float32)

def levels_additive(vals, k, nonconsec):
    """Sums of TWO distinct terms; if nonconsec, indices may not be adjacent."""
    pos=set()
    for i in range(len(vals)):
        for j in range(i+1,len(vals)):
            if nonconsec and j==i+1: continue
            pos.add(vals[i]+vals[j])
    pos=sorted(pos)[:k]
    return np.array(sorted([-float(p) for p in pos]+[0.0]+[float(p) for p in pos]),dtype=np.float32)

POW=[2.0**i for i in range(8)]
FIB=[float(x) for x in fib(8)]
ARMS={
 "pot9  (single, 2^k)"     : levels_single(POW,4),
 "fib9  (single, F_k)"     : levels_single(FIB,4),
 "apot9 (additive, 2^i+2^j)": levels_additive(POW,4,False),
 "zeck9 (additive, Zeckendorf)": levels_additive(FIB,4,True),
}

if __name__=="__main__":
    out=sys.argv[1]; seeds=int(sys.argv[2]) if len(sys.argv)>2 else 8
    G="/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/gat"
    for nm,lv in ARMS.items():
        print(f"  {nm:<30} {[round(float(x),1) for x in lv]}")
    print()
    res={}
    from math import sqrt
    for tname,path in (("UNSW","unsw.npz"),("Fashion","fashion_bin.npz")):
        Xtr,ytr,Xva,yva,Xte,yte=TA.load(f"{G}/{path}")
        base=None
        print(f"  === {tname} ===",flush=True)
        for nm,lv in ARMS.items():
            t0=time.time()
            a=np.array([B.train(Xtr,ytr,Xva,yva,Xte,yte,lv,1000+s,epochs=30) for s in range(seeds)])*100
            res[f"{tname}|{nm}"]=a.tolist()
            if base is None: base=a
            d=a-base; t=d.mean()/(d.std(ddof=1)/sqrt(len(d))+1e-12) if nm!=list(ARMS)[0] else 0
            print(f"    {nm:<30}{a.mean():>7.2f}%  {d.mean():+6.3f}  t={t:+5.2f}"
                  f"{'  *' if abs(t)>2.36 else ''}   ({time.time()-t0:.0f}s)",flush=True)
            json.dump(res,open(out,"w"),indent=1)
    print("  written:",out,flush=True)
