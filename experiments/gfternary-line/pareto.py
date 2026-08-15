"""W757: the middle ground. We have two points and no curve between them.

  16 wide, depth 3  ->  126 LUT, 78.45%   (T350, whole artefact, output included)
  64 wide, depth 4  ->  ~857 LUT, 84.64%  (T331 + T340's output correction)

Six-fold area for six points of accuracy, and NOTHING measured in between. A
Pareto front needs the middle, and the middle is where a deployable design lives.

THE SIX-BIT RULE (T331) fixes the fan-ins: layer 1 reads BINARY inputs so its
fan-in is 6; every deeper layer reads TERNARY symbols at two bits each, so its
fan-in is 3. Both cost 2.00 LUT/neuron. Violating it costs 20x, measured.

FORECAST, REGISTERED BEFORE THE RUN (T44).
 (1) Accuracy rises with width and depth and SATURATES: 48-wide will be within
     0.5 pp of 64-wide, so the knee sits at 32-48 neurons.
 (2) The best accuracy-at-area point will be **32 wide, depth 3**, landing at
     82-84% for 250-400 LUT.
 (3) No configuration reaches 88%: the T347a ceiling is a property of six-input
     truth tables and width does not lift it (W751 measured +0.86 pp for a 4x
     width increase).
If (3) breaks, the ceiling claim closed in W755 was premature and T347a must be
reopened.
"""
import numpy as np, json, sys, time
sys.path.insert(0,"/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import one_system as OS

LV = OS.LV; THR = 2.0

def train(Xtr,ytr,Xva,yva,Xte,yte,mi,seed,H=32,L=3,F1=6,Fd=3,out_fanin=None,
          epochs=60,lr0=0.1,bs=512,patience=8):
    rng=np.random.default_rng(seed)
    sizes=[Xtr.shape[1]]+[H]*(L-1)+[1]
    of = out_fanin or H
    idxs,Ws=[],[]
    for li in range(L):
        if li==L-1: ix=np.stack([rng.choice(sizes[li],size=min(of,sizes[li]),replace=False)])
        elif li==0: ix=OS.masks(sizes[li],sizes[li+1],F1,rng,"prop",mi)
        else:       ix=np.stack([rng.choice(sizes[li],size=min(Fd,sizes[li]),replace=False) for _ in range(sizes[li+1])])
        idxs.append(ix); Ws.append(rng.normal(0,1/np.sqrt(ix.shape[1]),ix.shape).astype(np.float32))
    p1=ytr.mean(); wp,wn=0.5/p1,0.5/(1-p1)
    act=lambda a: np.tanh(a-THR)*0.5+np.tanh(a+THR)*0.5
    dact=lambda a: 0.5*(1-np.tanh(a-THR)**2)+0.5*(1-np.tanh(a+THR)**2)
    best_va,best,bad=-1.0,None,0
    for ep in range(epochs):
        lr=lr0*(0.5**(ep//12)); pm=rng.permutation(len(Xtr))
        for i in range(0,max(len(Xtr)-bs,1),bs):
            b=pm[i:i+bs]; x,y=Xtr[b],ytr[b][:,None]
            Qs=[OS.q(W) for W in Ws]; hs=[x]; pre=[]; sds=[]
            for li in range(L):
                a=OS.fwd(hs[-1],idxs[li],Qs[li])
                sd=(a.std()+1e-6) if li<L-1 else 1.0
                if li<L-1: a=a/sd*THR
                sds.append(sd); pre.append(a); hs.append(act(a) if li<L-1 else a)
            p=1/(1+np.exp(-np.clip(hs[-1],-30,30)))
            g=np.where(y>0.5,wp,wn)*(p-y)/len(b)
            for li in range(L-1,-1,-1):
                gW=np.einsum('no,nof->of',g,hs[li][:,idxs[li]])
                if li>0:
                    gp=np.zeros_like(hs[li])
                    for o in range(idxs[li].shape[0]):
                        np.add.at(gp,(slice(None),idxs[li][o]),g[:,o:o+1]*Qs[li][o][None,:])
                    g=gp*dact(pre[li-1])/sds[li-1]*THR
                Ws[li]-=lr*gW
        Qs=[OS.q(W) for W in Ws]; h=Xva
        for li in range(L):
            a=OS.fwd(h,idxs[li],Qs[li])
            if li<L-1: a=a/(a.std()+1e-6)*THR; h=act(a)
            else: h=a
        sc=h.ravel(); v=0.5*(np.mean(sc[yva>0.5]>0)+np.mean(sc[yva<=0.5]<=0))
        if v>best_va: best_va,best,bad=v,([W.copy() for W in Ws],idxs),0
        else:
            bad+=1
            if bad>=patience: break
    Ws2,ix2=best; Qs=[OS.q(W) for W in Ws2]; h=Xte
    for li in range(L):
        a=OS.fwd(h,ix2[li],Qs[li])
        if li<L-1: a=a/(a.std()+1e-6)*THR; h=act(a)
        else: h=a
    sc=h.ravel()
    return float(np.mean((sc>0)==(yte>0.5))), ([q.astype(int).tolist() for q in
            [LV[np.argmin(np.abs(W[...,None]/(np.mean(np.abs(W))/(np.mean(np.abs(LV[LV!=0]))+1e-9)+1e-9)
              - LV[None,None,:]),axis=-1)] for W in Ws2]], [i.tolist() for i in ix2])

if __name__=="__main__":
    out=sys.argv[1]; seeds=int(sys.argv[2]) if len(sys.argv)>2 else 3
    G="/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/gat"
    d=np.load(f"{G}/unsw.npz"); tr,te=d["train"],d["test"]
    rng=np.random.default_rng(0); perm=rng.permutation(len(tr)); nva=len(tr)//10
    va,trn=perm[:nva],perm[nva:]
    Xtr=tr[trn,:-1].astype(np.float32)*2-1; ytr=tr[trn,-1].astype(np.float32)
    Xva=tr[va,:-1].astype(np.float32)*2-1;  yva=tr[va,-1].astype(np.float32)
    Xte=te[:,:-1].astype(np.float32)*2-1;   yte=te[:,-1].astype(np.float32)
    mi=OS.mutual_info(Xtr,ytr); res={}; best_net={}
    for H in (16,32,48,64):
        for L in (2,3,4):
            k=f"H{H}_L{L}"; t0=time.time(); accs=[]; keep=None
            for s in range(seeds):
                a,w,ix=train(Xtr,ytr,Xva,yva,Xte,yte,mi,1000+s,H=H,L=L)
                accs.append(a)
                if keep is None or a==max(accs): keep=(w,ix,a)
            res[k]=accs; best_net[k]={"w":keep[0],"idx":keep[1],"acc":keep[2]}
            print(f"  {k:<8}: {np.mean(accs)*100:6.2f}% +- {np.std(accs,ddof=1)*100:4.2f}   "
                  f"best {max(accs)*100:6.2f}%   ({time.time()-t0:.0f}s)",flush=True)
    json.dump({"acc":res,"nets":best_net},open(out,"w"))
    print("  written:",out,flush=True)
