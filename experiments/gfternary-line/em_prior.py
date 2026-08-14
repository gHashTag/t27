"""W751 track A: take the calibration points LEGALLY.

T326a showed a test-set-tuned threshold reaches 92.05% where honest training
reaches 89.94%. That 2.1 pp is calibration, and the oracle that revealed it
leaks the test labels, so it is a bound and not a method.

THE LEGITIMATE METHOD. Saerens, Latinne & Decaestecker (2002) estimate the
target-domain class prior by EM on the UNLABELLED target features: iterate
    p(y|x) reweighted by (pi_new / pi_train)   ->   pi_new = mean of reweighted
until the prior converges. Test FEATURES are available at deployment; test LABELS
are not. Using the former is transduction, not leakage -- and this is the standard
correction for prior probability shift, which is exactly what T326 measured
(68.06% -> 55.06%, with per-feature drift under 0.10 everywhere).

FORECAST, REGISTERED BEFORE THE RUN (T44). EM recovers 1.0-2.0 of the 2.1 pp the
oracle bounds, landing at 91.0-92.0% dense; and the estimated prior lands within
3 points of the true 55.06%. If EM recovers nothing, the oracle's 2.1 pp is not
prior shift after all and T326's diagnosis needs revisiting.
"""
import numpy as np, json, sys, time
sys.path.insert(0,"/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a/experiments/gfternary-line")
import prior_shift as PS

def em_prior(p_src, pi_src, iters=200, tol=1e-7):
    """p_src: P(y=1|x) under the SOURCE prior. Returns the EM estimate of the
    target prior. Saerens et al. 2002, eq. 6-7."""
    pi = pi_src
    for _ in range(iters):
        r = (pi/pi_src)*p_src
        r = r / (r + ((1-pi)/(1-pi_src))*(1-p_src) + 1e-12)
        pi_new = float(r.mean())
        if abs(pi_new - pi) < tol: break
        pi = pi_new
    return pi

if __name__=="__main__":
    path,out_path=sys.argv[1],sys.argv[2]
    seeds=int(sys.argv[3]) if len(sys.argv)>3 else 5
    d=np.load(path); tr,te=d["train"],d["test"]
    rng=np.random.default_rng(0); perm=rng.permutation(len(tr)); nva=len(tr)//10
    va,trn=perm[:nva],perm[nva:]
    Xtr=tr[trn,:-1].astype(np.float32)*2-1; ytr=tr[trn,-1].astype(np.float32)
    Xva=tr[va,:-1].astype(np.float32)*2-1;  yva=tr[va,-1].astype(np.float32)
    Xte=te[:,:-1].astype(np.float32)*2-1;   yte=te[:,-1].astype(np.float32)
    pi_true=float(yte.mean())
    print(f"  train prior {ytr.mean()*100:.2f}%   TRUE test prior {pi_true*100:.2f}%",flush=True)
    res={}
    for tag,bal in (("plain",False),("balanced",True)):
        rows=[]
        for s in range(seeds):
            W,_=PS.train(Xtr,ytr,Xva,yva,1000+s,balanced=bal)
            sc_va=PS.scores(W,Xva); sc_te=PS.scores(W,Xte)
            # calibrate a logistic on VALIDATION only -- source-domain probabilities
            from math import exp
            lo,hi=-10.0,10.0
            for _ in range(60):                       # 1-D fit of a temperature
                mid=(lo+hi)/2
                p=1/(1+np.exp(-sc_va*np.exp(mid)))
                if p.mean() > yva.mean(): hi=mid
                else: lo=mid
            T=np.exp((lo+hi)/2)
            pi_src=float(yva.mean())
            p_te=1/(1+np.exp(-sc_te*T))
            pi_hat=em_prior(p_te,pi_src)
            # shifted decision rule: P_target(y=1|x) > 0.5
            r=(pi_hat/pi_src)*p_te
            r=r/(r+((1-pi_hat)/(1-pi_src))*(1-p_te)+1e-12)
            acc_plain=float(np.mean((sc_te>0)==(yte>0.5)))
            acc_em=float(np.mean((r>0.5)==(yte>0.5)))
            cand=np.quantile(sc_te,np.linspace(0.01,0.99,199))
            orc=max(float(np.mean((sc_te>c)==(yte>0.5))) for c in cand)
            rows.append((acc_plain,acc_em,orc,pi_hat))
        a=np.array(rows)
        res[tag]=a.tolist()
        print(f"  {tag:<9}: plain {a[:,0].mean()*100:6.2f}%  EM {a[:,1].mean()*100:6.2f}%  "
              f"[oracle {a[:,2].mean()*100:6.2f}%]   pi_hat {a[:,3].mean()*100:5.2f}% (true {pi_true*100:.2f}%)",flush=True)
    json.dump(res,open(out_path,"w"),indent=1); print("  written:",out_path,flush=True)
