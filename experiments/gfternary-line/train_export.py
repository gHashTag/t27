"""W752: train a sparse ternary net and EXPORT IT TO SILICON.

THE ADMISSION THIS CLOSES. Every silicon result this programme has produced --
T219, T302, T323, T329, all of them -- ran a network whose weights came from
`random.Random(seed)`. They proved transport, placement, readback and the
acceptance criterion. **They never proved that a TRAINED network computes on the
die.** No export path from the trainer to the Verilog existed until this file.

THE WIRE CONSTRAINT AND HOW IT IS MET. The BSCANE2 DR is 32 bits and the usable
payload is 31 (T324). Layer 1 reads 593 binary features, so the input cannot
arrive in one pass. Die A therefore carries a 593-bit shift register fed 31 bits
per UPDATE: twenty passes load a complete UNSW row, and the twenty-first CAPTURE
returns the layer's 16 ternary symbols. No arithmetic on the host, as before.

FORECAST, REGISTERED BEFORE THE RUN (T44).
  (1) Silicon will match the software model on 100% of rows, because both are the
      same truth tables -- any mismatch is a transport defect, not a numeric one.
  (2) The exported net's silicon accuracy over 200 real UNSW test rows will land
      within 3 pp of its software test accuracy, the difference being sampling
      noise at n=200 (a 84% accuracy has a +-2.5 pp standard error there).
  (3) Layer 1 restricted to 16 output symbols (one JTAG word) will cost 1-3 pp
      against the 128-wide layer it is cut down from.
If (1) fails on even one row, something in the shift path is wrong and every
earlier cross-die result -- which used 8 hand-picked words, not 200 real rows --
was under-tested rather than correct.
"""
import numpy as np, json, sys, time
sys.path.insert(0,"/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import one_system as OS

LV = OS.LV
THR = 2.0

def train(Xtr,ytr,Xva,yva,mi,seed,H1=16,H2=16,F1=6,F2=3,out_fanin=16,
          epochs=60,lr0=0.1,bs=512,patience=8):
    """593 -> H1 -> H2 -> 1, all sparse. H1=H2=16 so each layer's symbol vector
    is exactly 32 bits and crosses the wire in one pass."""
    rng=np.random.default_rng(seed)
    idx1=OS.masks(Xtr.shape[1],H1,F1,rng,"prop",mi)
    idx2=np.stack([rng.choice(H1,size=min(F2,H1),replace=False) for _ in range(H2)])
    idx3=np.stack([rng.choice(H2,size=min(out_fanin,H2),replace=False)])
    idxs=[idx1,idx2,idx3]
    Ws=[rng.normal(0,1/np.sqrt(ix.shape[1]),ix.shape).astype(np.float32) for ix in idxs]
    p1=ytr.mean(); wp,wn=0.5/p1,0.5/(1-p1)
    act=lambda a: np.tanh(a-THR)*0.5+np.tanh(a+THR)*0.5
    dact=lambda a: 0.5*(1-np.tanh(a-THR)**2)+0.5*(1-np.tanh(a+THR)**2)
    best_va,best,bad=-1.0,None,0
    for ep in range(epochs):
        lr=lr0*(0.5**(ep//12)); pm=rng.permutation(len(Xtr))
        for i in range(0,max(len(Xtr)-bs,1),bs):
            b=pm[i:i+bs]; x,y=Xtr[b],ytr[b][:,None]
            Qs=[OS.q(W) for W in Ws]; hs=[x]; pre=[]; sds=[]
            for li in range(3):
                a=OS.fwd(hs[-1],idxs[li],Qs[li])
                sd=(a.std()+1e-6) if li<2 else 1.0
                if li<2: a=a/sd*THR
                sds.append(sd); pre.append(a); hs.append(act(a) if li<2 else a)
            p=1/(1+np.exp(-np.clip(hs[-1],-30,30)))
            g=np.where(y>0.5,wp,wn)*(p-y)/len(b)
            for li in (2,1,0):
                gW=np.einsum('no,nof->of',g,hs[li][:,idxs[li]])
                if li>0:
                    gp=np.zeros_like(hs[li])
                    for o in range(idxs[li].shape[0]):
                        np.add.at(gp,(slice(None),idxs[li][o]),g[:,o:o+1]*Qs[li][o][None,:])
                    g=gp*dact(pre[li-1])/sds[li-1]*THR
                Ws[li]-=lr*gW
        Qs=[OS.q(W) for W in Ws]; h=Xva
        for li in range(3):
            a=OS.fwd(h,idxs[li],Qs[li])
            if li<2: a=a/(a.std()+1e-6)*THR; h=act(a)
            else: h=a
        sc=h.ravel(); v=0.5*(np.mean(sc[yva>0.5]>0)+np.mean(sc[yva<=0.5]<=0))
        if v>best_va: best_va,best,bad=v,[W.copy() for W in Ws],0
        else:
            bad+=1
            if bad>=patience: break
    return best, idxs, best_va

def quantise_to_int(W):
    """Map trained floats onto the INTEGER levels the truth tables use.
    The hardware knows nothing about the absmean scale -- it sees integers -- so
    the export must be exact, not approximate."""
    s=np.mean(np.abs(W))/(np.mean(np.abs(LV[LV!=0]))+1e-9)+1e-9
    idx=np.argmin(np.abs(W[...,None]/s - LV[None,None,:]),axis=-1)
    return LV[idx].astype(int)     # the integer level, NOT level*scale

if __name__=="__main__":
    out=sys.argv[1]; seed=int(sys.argv[2]) if len(sys.argv)>2 else 1000
    G="/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/gat"
    ds=sys.argv[3] if len(sys.argv)>3 else "unsw"   # W760: any dataset
    d=np.load(f"{G}/{ds}.npz"); tr,te=d["train"],d["test"]
    rng=np.random.default_rng(0); perm=rng.permutation(len(tr)); nva=len(tr)//10
    va,trn=perm[:nva],perm[nva:]
    Xtr=tr[trn,:-1].astype(np.float32)*2-1; ytr=tr[trn,-1].astype(np.float32)
    Xva=tr[va,:-1].astype(np.float32)*2-1;  yva=tr[va,-1].astype(np.float32)
    Xte=te[:,:-1].astype(np.float32)*2-1;   yte=te[:,-1].astype(np.float32)
    mi=OS.mutual_info(Xtr,ytr)
    t0=time.time(); Ws,idxs,bva=train(Xtr,ytr,Xva,yva,mi,seed)
    # software test accuracy of the EXPORTED (integer) network
    Qi=[quantise_to_int(W) for W in Ws]
    def run_int(X):
        h=(X>0).astype(np.int32)*2-1                # +-1
        for li in range(3):
            a=np.einsum('nof,of->no',h[:,idxs[li]],Qi[li].astype(np.int32))
            if li<2: h=np.where(a>2,1,np.where(a<-2,-1,0)).astype(np.int32)
            else:    h=a
        return h.ravel()
    acc=float(np.mean((run_int(Xte)>0)==(yte>0.5)))
    print(f"  trained in {time.time()-t0:.0f}s  val(bal) {bva*100:.2f}%  "
          f"INTEGER-EXPORT test accuracy {acc*100:.2f}%",flush=True)
    json.dump({"idx":[i.tolist() for i in idxs],
               "w":[q.tolist() for q in Qi],
               "test_acc":acc, "seed":seed,
               "n_in":int(Xtr.shape[1]), "dataset":ds}, open(out,"w"))
    print("  written:",out,flush=True)
