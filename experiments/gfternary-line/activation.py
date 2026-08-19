"""W755: the hand-rolled activation, the last untested ceiling candidate (T330a).

WHAT IS UNDER TEST. Every network in this programme uses
    h = 0.5*tanh(a - thr) + 0.5*tanh(a + thr)
with thr FIXED at 2.0 and its own derivative as the backward pass. That is a
smooth surrogate someone (me) invented; the field uses a hard ternary threshold
with a STRAIGHT-THROUGH estimator, and often a LEARNABLE threshold. T330a left it
as the only structural knob never turned, after alphabet, depth, width,
connectivity and normalisation were all controlled and none explained the
84-87% sparse ceiling.

ARMS
  tanh      the incumbent, thr = 2.0 fixed          (control)
  ste       hard threshold forward, identity-in-window backward (the field's)
  ste_lrn   as `ste`, with thr learned per layer
  ste_wide  as `ste`, backward window widened 2x -- tests whether the estimator's
            window, not its shape, is what matters

FORECAST, REGISTERED BEFORE THE RUN (T44). `ste` beats `tanh` by 0.5-2.0 pp and
`ste_lrn` adds another 0-1 pp; none of them lifts the ORACLE ceiling by more than
2 pp, because a six-input truth table's capacity does not depend on how it was
trained. Predicted: the ceiling is architecture, and the activation is worth less
than normalisation (+29 pp) and more than the alphabet (+0.7 pp).
If `ste` lifts the oracle past 90%, the ceiling was never architectural and
T330a's conclusion is wrong.
"""
import numpy as np, json, sys, time
sys.path.insert(0,"/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import one_system as OS

LV = OS.LV

def act_fwd(a, thr, kind):
    if kind == "tanh":
        return np.tanh(a-thr)*0.5 + np.tanh(a+thr)*0.5
    return np.where(a > thr, 1.0, np.where(a < -thr, -1.0, 0.0)).astype(np.float32)

def act_bwd(a, thr, kind):
    if kind == "tanh":
        return 0.5*(1-np.tanh(a-thr)**2) + 0.5*(1-np.tanh(a+thr)**2)
    w = thr*(2.0 if kind == "ste_wide" else 1.0)
    # straight-through: pass the gradient where the pre-activation is in range
    return (np.abs(a) < 2*w).astype(np.float32)

def train(Xtr,ytr,Xva,yva,Xte,yte,mi,seed,kind="tanh",F=6,hidden=64,L=2,
          out_fanin=64,epochs=60,lr0=0.1,thr0=2.0,bs=512,patience=8):
    rng=np.random.default_rng(seed)
    sizes=[Xtr.shape[1]]+[hidden]*(L-1)+[1]
    idxs,Ws=[],[]
    for li in range(L):
        if li==L-1: ix=np.stack([rng.choice(sizes[li],size=min(out_fanin,sizes[li]),replace=False)])
        elif li==0: ix=OS.masks(sizes[li],sizes[li+1],F,rng,"prop",mi)
        else:       ix=np.stack([rng.choice(sizes[li],size=min(3,sizes[li]),replace=False) for _ in range(sizes[li+1])])
        idxs.append(ix); Ws.append(rng.normal(0,1/np.sqrt(ix.shape[1]),ix.shape).astype(np.float32))
    thrs=[float(thr0)]*max(L-1,1)
    learn = (kind=="ste_lrn")
    p1=ytr.mean(); wp,wn=0.5/p1,0.5/(1-p1)
    best_va,best,bad=-1.0,None,0
    for ep in range(epochs):
        lr=lr0*(0.5**(ep//12)); pm=rng.permutation(len(Xtr))
        for i in range(0,max(len(Xtr)-bs,1),bs):
            b=pm[i:i+bs]; x,y=Xtr[b],ytr[b][:,None]
            Qs=[OS.q(W) for W in Ws]; hs=[x]; pre=[]; sds=[]
            for li in range(L):
                a=OS.fwd(hs[-1],idxs[li],Qs[li])
                sd=(a.std()+1e-6) if li<L-1 else 1.0
                if li<L-1: a=a/sd*thr0
                sds.append(sd); pre.append(a)
                hs.append(act_fwd(a,thrs[min(li,len(thrs)-1)],kind) if li<L-1 else a)
            p=1/(1+np.exp(-np.clip(hs[-1],-30,30)))
            g=np.where(y>0.5,wp,wn)*(p-y)/len(b)
            for li in range(L-1,-1,-1):
                gW=np.einsum('no,nof->of',g,hs[li][:,idxs[li]])
                if li>0:
                    gp=np.zeros_like(hs[li])
                    for o in range(idxs[li].shape[0]):
                        np.add.at(gp,(slice(None),idxs[li][o]),g[:,o:o+1]*Qs[li][o][None,:])
                    t=thrs[min(li-1,len(thrs)-1)]
                    if learn:
                        # d h / d thr for the STE window: pushing thr shrinks the live set
                        thrs[min(li-1,len(thrs)-1)] = float(np.clip(t - lr*0.01*np.sign(np.mean(gp*np.sign(pre[li-1]))), 0.2, 8.0))
                    g=gp*act_bwd(pre[li-1],t,kind)/sds[li-1]*thr0
                Ws[li]-=lr*gW
        Qs=[OS.q(W) for W in Ws]; h=Xva
        for li in range(L):
            a=OS.fwd(h,idxs[li],Qs[li])
            if li<L-1: a=a/(a.std()+1e-6)*thr0; h=act_fwd(a,thrs[min(li,len(thrs)-1)],kind)
            else: h=a
        sc=h.ravel(); v=0.5*(np.mean(sc[yva>0.5]>0)+np.mean(sc[yva<=0.5]<=0))
        if v>best_va: best_va,best,bad=v,([W.copy() for W in Ws],list(thrs)),0
        else:
            bad+=1
            if bad>=patience: break
    Ws2,th=best; Qs=[OS.q(W) for W in Ws2]; h=Xte
    for li in range(L):
        a=OS.fwd(h,idxs[li],Qs[li])
        if li<L-1: a=a/(a.std()+1e-6)*thr0; h=act_fwd(a,th[min(li,len(th)-1)],kind)
        else: h=a
    sc=h.ravel(); acc=float(np.mean((sc>0)==(yte>0.5)))
    cand=np.quantile(sc,np.linspace(0.01,0.99,199))
    return acc, max(float(np.mean((sc>c)==(yte>0.5))) for c in cand), th[0]

if __name__=="__main__":
    out=sys.argv[1]; seeds=int(sys.argv[2]) if len(sys.argv)>2 else 5
    G="/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/gat"
    d=np.load(f"{G}/unsw.npz"); tr,te=d["train"],d["test"]
    rng=np.random.default_rng(0); perm=rng.permutation(len(tr)); nva=len(tr)//10
    va,trn=perm[:nva],perm[nva:]
    Xtr=tr[trn,:-1].astype(np.float32)*2-1; ytr=tr[trn,-1].astype(np.float32)
    Xva=tr[va,:-1].astype(np.float32)*2-1;  yva=tr[va,-1].astype(np.float32)
    Xte=te[:,:-1].astype(np.float32)*2-1;   yte=te[:,-1].astype(np.float32)
    mi=OS.mutual_info(Xtr,ytr); res={}
    for kind in ("tanh","ste","ste_lrn","ste_wide"):
        t0=time.time(); r=[train(Xtr,ytr,Xva,yva,Xte,yte,mi,1000+s,kind=kind) for s in range(seeds)]
        a=np.array([x[0] for x in r])*100; o=np.array([x[1] for x in r])*100
        res[kind]=[a.tolist(),o.tolist(),[x[2] for x in r]]
        print(f"  {kind:<9}: test {a.mean():6.2f}% +- {a.std(ddof=1):4.2f}   oracle {o.mean():6.2f}%"
              f"   thr {np.mean([x[2] for x in r]):.2f}   ({time.time()-t0:.0f}s)",flush=True)
    json.dump(res,open(out,"w"),indent=1); print("  written:",out,flush=True)
