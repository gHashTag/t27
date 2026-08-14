"""W751: raise the 128-LUT system's ceiling. T327 says it is 84.43%.

THE CONSTRAINT. The sparse system's ORACLE -- the best any threshold could do on
its scores -- stops at 84.43%, where the dense model's reaches 92.05%. That is a
representational ceiling, not a calibration one, and it is now the binding limit
on everything small this project builds.

WHICH LEVER. T314 already measured depth WITH normalisation and found it flat:
75.98 / 77.28 / 77.23 / 77.66 for two to five layers. So depth is not the lever,
and this project has already paid once for ignoring its own record (T298). The
untested lever is WIDTH, and width is nearly free here: 2.00 LUT per neuron at
fan-in <= 6, measured, flat in fan-in up to six. Going 64 -> 512 neurons costs
128 -> 1024 LUT and stays an order of magnitude under the dense design.

FORECAST, REGISTERED BEFORE THE RUN (T44).
  (1) WIDTH raises the oracle ceiling: 64 -> 512 neurons takes 84.43% to 87-90%.
  (2) DEPTH stays flat under full training too, within 1 pp of the two-layer
      number at every depth, confirming T314 rather than extending it.
  (3) Achieved accuracy will trail the oracle by ~1.5 pp throughout, the same
      calibration gap balancing left behind at 128 LUT.
If (2) is wrong and depth now helps, then T314's flatness was an artefact of the
probe trainer and the depth conclusion needs restating -- which is exactly why
depth is re-run here rather than assumed.
"""
import numpy as np, json, sys, time
sys.path.insert(0, "/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import one_system as OS

LV = OS.LV

def train(Xtr,ytr,Xva,yva,Xte,yte,mi,seed,F=6,mode="prop",hidden=64,L=2,
          out_fanin=64,epochs=60,lr0=0.1,thr=2.0,bs=512,patience=8):
    rng=np.random.default_rng(seed)
    sizes=[Xtr.shape[1]]+[hidden]*(L-1)+[1]
    idxs,Ws=[],[]
    for li in range(L):
        if li==L-1: idx=np.stack([rng.choice(sizes[li],size=min(out_fanin,sizes[li]),replace=False)])
        elif li==0: idx=OS.masks(sizes[li],sizes[li+1],F,rng,mode,mi)
        else:       idx=np.stack([rng.choice(sizes[li],size=min(F,sizes[li]),replace=False) for _ in range(sizes[li+1])])
        idxs.append(idx); Ws.append(rng.normal(0,1/np.sqrt(idx.shape[1]),idx.shape).astype(np.float32))
    p1=ytr.mean(); wp,wn=0.5/p1,0.5/(1-p1)
    act=lambda a: np.tanh(a-thr)*0.5+np.tanh(a+thr)*0.5
    dact=lambda a: 0.5*(1-np.tanh(a-thr)**2)+0.5*(1-np.tanh(a+thr)**2)
    best_va,best,bad=-1.0,None,0
    for ep in range(epochs):
        lr=lr0*(0.5**(ep//12)); pm=rng.permutation(len(Xtr))
        for i in range(0,max(len(Xtr)-bs,1),bs):
            b=pm[i:i+bs]; x,y=Xtr[b],ytr[b][:,None]
            Qs=[OS.q(W) for W in Ws]; hs=[x]; pre=[]; sds=[]
            for li in range(L):
                a=OS.fwd(hs[-1],idxs[li],Qs[li])
                sd=(a.std()+1e-6) if li<L-1 else 1.0
                if li<L-1: a=a/sd*thr
                sds.append(sd); pre.append(a); hs.append(act(a) if li<L-1 else a)
            p=1/(1+np.exp(-np.clip(hs[-1],-30,30)))
            g=np.where(y>0.5,wp,wn)*(p-y)/len(b)
            for li in range(L-1,-1,-1):
                gW=np.einsum('no,nof->of',g,hs[li][:,idxs[li]])
                if li>0:
                    gp=np.zeros_like(hs[li])
                    for o in range(idxs[li].shape[0]):
                        np.add.at(gp,(slice(None),idxs[li][o]),g[:,o:o+1]*Qs[li][o][None,:])
                    g=gp*dact(pre[li-1])/sds[li-1]*thr
                Ws[li]-=lr*gW
        Qs=[OS.q(W) for W in Ws]; h=Xva
        for li in range(L):
            a=OS.fwd(h,idxs[li],Qs[li])
            if li<L-1: a=a/(a.std()+1e-6)*thr; h=act(a)
            else: h=a
        sc=h.ravel(); v=0.5*(np.mean(sc[yva>0.5]>0)+np.mean(sc[yva<=0.5]<=0))
        if v>best_va: best_va,best,bad=v,([W.copy() for W in Ws],idxs),0
        else:
            bad+=1
            if bad>=patience: break
    Ws,idxs=best; Qs=[OS.q(W) for W in Ws]; h=Xte
    for li in range(L):
        a=OS.fwd(h,idxs[li],Qs[li])
        if li<L-1: a=a/(a.std()+1e-6)*thr; h=act(a)
        else: h=a
    sc=h.ravel()
    acc=float(np.mean((sc>0)==(yte>0.5)))
    cand=np.quantile(sc,np.linspace(0.01,0.99,199))
    return acc, max(float(np.mean((sc>c)==(yte>0.5))) for c in cand)

if __name__=="__main__":
    path,out_path=sys.argv[1],sys.argv[2]
    seeds=int(sys.argv[3]) if len(sys.argv)>3 else 3
    d=np.load(path); tr,te=d["train"],d["test"]
    rng=np.random.default_rng(0); perm=rng.permutation(len(tr)); nva=len(tr)//10
    va,trn=perm[:nva],perm[nva:]
    Xtr=tr[trn,:-1].astype(np.float32)*2-1; ytr=tr[trn,-1].astype(np.float32)
    Xva=tr[va,:-1].astype(np.float32)*2-1;  yva=tr[va,-1].astype(np.float32)
    Xte=te[:,:-1].astype(np.float32)*2-1;   yte=te[:,-1].astype(np.float32)
    mi=OS.mutual_info(Xtr,ytr); res={}
    print("  WIDTH sweep (L=2, fan-in 6, 2.00 LUT/neuron measured):",flush=True)
    for H in [64,128,256,512]:
        t0=time.time(); r=[train(Xtr,ytr,Xva,yva,Xte,yte,mi,1000+s,hidden=H,L=2) for s in range(seeds)]
        a=np.array([x[0] for x in r])*100; o=np.array([x[1] for x in r])*100
        res[f"H{H}_L2"]=[list(map(float,a)),list(map(float,o))]
        print(f"    {H:>4} neurons ({2*H:>5} LUT): test {a.mean():6.2f}%  oracle {o.mean():6.2f}%  ({time.time()-t0:.0f}s)",flush=True)
    print("  DEPTH sweep (H=128, re-checking T314 under FULL training):",flush=True)
    for L in [2,3,4]:
        t0=time.time(); r=[train(Xtr,ytr,Xva,yva,Xte,yte,mi,1000+s,hidden=128,L=L) for s in range(seeds)]
        a=np.array([x[0] for x in r])*100; o=np.array([x[1] for x in r])*100
        res[f"H128_L{L}"]=[list(map(float,a)),list(map(float,o))]
        print(f"    L={L}: test {a.mean():6.2f}%  oracle {o.mean():6.2f}%  ({time.time()-t0:.0f}s)",flush=True)
    json.dump(res,open(out_path,"w"),indent=1); print("  written:",out_path,flush=True)
