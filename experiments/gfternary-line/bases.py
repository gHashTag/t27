"""W763: WHICH NUMBERS suit ternary weights? A measured top list.

Dmitrii asked for a top of numbers for ternary weights, drawn from our
catalogues. NOTE THE DISTINCTION FIRST: the 83-format catalogue
(`docs/metrics/NUMERIC_FORMATS_83_METRICS.md`) enumerates FLOAT ENCODINGS --
bit widths, exponent/mantissa splits. This asks a different question: what
VALUES should the weight levels take. Those catalogues do not answer it, so
this file builds the candidate list from the project's own theorems and from
the quantisation literature.

THE ALPHABET SHAPE, held fixed so only the base varies:
    A(b) = {0} u {+-b^k : 0 <= k <= 3}      -> NINE levels for every base
Nine because T288's Nine-Rung Law puts the ceiling there, and fixing cardinality
is what makes this a test of the NUMBER rather than of the size (T286: size is
worth +0.844 pp, shape +0.085 -- confounding them is how a base wins for the
wrong reason).

THE CANDIDATES, with why each is in the list:
  1.0      degenerate: all levels equal, {0,+-1}. The control that must lose.
  2.0      DYADIC / power-of-two (PoT). One shift, one lane, zero DSP. The
           incumbent, and the literature's (Li et al. ICLR 2020).
  phi      1.6180  GOLDEN. Our line is named for it. Z[phi] closed; degree 2
           admits phi ALONE as a multiplier-free scale (proved).
  rho      1.3247  PLASTIC NUMBER, root of r^3 = r+1. The d=3 member of the
           same family as phi (r^2=r+1); T-theorem says fineness costs
           registers, not adders.
  psi4     1.2207  root of r^4 = r+1, the d=4 member.
  supergold 1.4656 root of r^3 = r^2+1. Adjacent family, different recurrence.
  trib     1.8393  TRIBONACCI constant, root of r^3=r^2+r+1.
  sqrt2    1.4142  the obvious algebraic irrational that is NOT in the r^d=r+1
           family -- included to separate "algebraic" from "our family".
  silver   2.4142  1+sqrt2, root of r^2=2r+1. The other metallic mean.
  3.0      TERNARY BASE. The project is ternary; nobody has measured base 3 as
           a weight scale, which is an omission worth naming.
  e        2.7183  transcendental control.
  lin      linear spacing {0,+-1,+-2,+-3,+-4}, not a geometric base at all.

FORECAST, REGISTERED BEFORE THE RUN (T44).
 (1) ACCURACY: all bases within 1.0 pp of each other except b=1.0, which loses
     by 2-6 pp because it is really a 3-level alphabet. Reasoning: T286 measured
     alphabet SHAPE at +0.085 pp pooled and W749 cut that to +0.149 with the
     bench fixed -- the shape effect is tiny and a base is a shape.
 (2) AREA: bases 2.0 and 3.0 and lin are cheapest; every irrational base pays
     for a multiply or a two-lane representation. Predicted ordering by LUT:
     2.0 < 3.0 ~ lin << phi ~ sqrt2 ~ rho ~ others.
 (3) THE TOP will therefore be decided by AREA, not accuracy, and 2.0 wins it.
If any base beats 2.0 by more than 1.0 pp in accuracy, the shape effect is
larger than three waves of measurement say and T286/T317 need revisiting.
"""
import numpy as np, json, sys, time
sys.path.insert(0,"/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import one_system as OS, task_or_arch as TA

PHI=(1+5**0.5)/2
def root(poly, lo, hi):
    for _ in range(200):
        m=(lo+hi)/2
        if poly(lo)*poly(m)<=0: hi=m
        else: lo=m
    return (lo+hi)/2
RHO   = root(lambda r: r**3-r-1, 1.0, 2.0)
PSI4  = root(lambda r: r**4-r-1, 1.0, 2.0)
SUPER = root(lambda r: r**3-r*r-1, 1.0, 2.0)
TRIB  = root(lambda r: r**3-r*r-r-1, 1.0, 2.0)

BASES=[("b1.0",1.0),("dyadic 2.0",2.0),("phi",PHI),("plastic",RHO),("psi4",PSI4),
       ("supergold",SUPER),("tribonacci",TRIB),("sqrt2",2**0.5),("silver",1+2**0.5),
       ("ternary 3.0",3.0),("e",np.e)]

def levels_for(b):
    pos=[b**k for k in range(4)]                 # 4 magnitudes -> 9 levels
    return np.array(sorted([-p for p in pos]+[0.0]+pos),dtype=np.float32)
LIN=np.array([-4,-3,-2,-1,0,1,2,3,4],dtype=np.float32)

def quant(W,lv):
    s=np.mean(np.abs(W))/(np.mean(np.abs(lv[lv!=0]))+1e-9)+1e-9
    return lv[np.argmin(np.abs(W[...,None]/s-lv[None,None,:]),axis=-1)]*s

def train(Xtr,ytr,Xva,yva,Xte,yte,lv,seed,hidden=64,epochs=30,lr0=0.1,thr=2.0,bs=512,patience=6):
    rng=np.random.default_rng(seed); n=Xtr.shape[1]
    W1=rng.normal(0,1/np.sqrt(n),(n,hidden)).astype(np.float32)
    W2=rng.normal(0,1/np.sqrt(hidden),(hidden,1)).astype(np.float32)
    p1=ytr.mean(); wp,wn=0.5/p1,0.5/(1-p1); best_va,best,bad=-1.0,None,0
    for ep in range(epochs):
        lr=lr0*(0.5**(ep//10)); pm=rng.permutation(len(Xtr))
        for i in range(0,max(len(Xtr)-bs,1),bs):
            b=pm[i:i+bs]; x,y=Xtr[b],ytr[b][:,None]
            Q1,Q2=quant(W1,lv),quant(W2,lv)
            a1=x@Q1; sd=a1.std()+1e-6; a1=a1/sd*thr      # W748 fix, always on
            h=np.tanh(a1-thr)*0.5+np.tanh(a1+thr)*0.5
            o=h@Q2; p=1/(1+np.exp(-np.clip(o,-30,30)))
            g=np.where(y>0.5,wp,wn)*(p-y)/len(b)
            gW2=h.T@g
            ga=(g@Q2.T)*(0.5*(1-np.tanh(a1-thr)**2)+0.5*(1-np.tanh(a1+thr)**2))/sd*thr
            W1-=lr*(x.T@ga); W2-=lr*gW2
        Q1,Q2=quant(W1,lv),quant(W2,lv)
        a1=Xva@Q1; a1=a1/(a1.std()+1e-6)*thr
        h=np.tanh(a1-thr)*0.5+np.tanh(a1+thr)*0.5; sc=(h@Q2).ravel()
        v=0.5*(np.mean(sc[yva>0.5]>0)+np.mean(sc[yva<=0.5]<=0))
        if v>best_va: best_va,best,bad=v,(W1.copy(),W2.copy()),0
        else:
            bad+=1
            if bad>=patience: break
    W1,W2=best; Q1,Q2=quant(W1,lv),quant(W2,lv)
    a1=Xte@Q1; a1=a1/(a1.std()+1e-6)*thr
    h=np.tanh(a1-thr)*0.5+np.tanh(a1+thr)*0.5
    return float(np.mean(((h@Q2).ravel()>0)==(yte>0.5)))

if __name__=="__main__":
    out=sys.argv[1]; seeds=int(sys.argv[2]) if len(sys.argv)>2 else 5
    G="/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/gat"
    res={}
    for tname,path in (("UNSW","unsw.npz"),("Fashion","fashion_bin.npz")):
        Xtr,ytr,Xva,yva,Xte,yte=TA.load(f"{G}/{path}")
        print(f"\n  === {tname} ===   (9 уровней у КАЖДОГО основания)",flush=True)
        print(f"  {'основание':<13}{'значение':>10}{'точность':>11}",flush=True)
        rows=[]
        for nm,b in BASES:
            lv=levels_for(b); t0=time.time()
            a=[train(Xtr,ytr,Xva,yva,Xte,yte,lv,1000+s) for s in range(seeds)]
            rows.append({"base":nm,"value":float(b),"acc":[float(x) for x in a]})
            print(f"  {nm:<13}{b:>10.4f}{np.mean(a)*100:>10.2f}%   ({time.time()-t0:.0f}s)",flush=True)
        a=[train(Xtr,ytr,Xva,yva,Xte,yte,LIN,1000+s) for s in range(seeds)]
        rows.append({"base":"linear","value":0.0,"acc":[float(x) for x in a]})
        print(f"  {'linear':<13}{'--':>10}{np.mean(a)*100:>10.2f}%",flush=True)
        res[tname]=rows
        json.dump(res,open(out,"w"),indent=1)
    print("\n  written:",out,flush=True)
