#!/usr/bin/env python3
# GF-T on-device training DEMO -- SELF-CONTAINED (no external deps).
#
# Proves the spec-first GF-T primitive stack (specs/ternary/gft_softmax4.t27,
# gft_softmax_grad4.t27, gft_sgd_step.t27, gft_exp2.t27, ...) does not merely
# compute correct arithmetic -- it composes into real gradient-descent LEARNING.
# The GF-T ops below are the EXACT integer models that every spec's iverilog
# conformance test is bit-exact to (500/1600/2000/... vectors), i.e. they compute
# what the synthesized hardware computes, bit for bit. A linear 4-class classifier
# logits = W @ x is trained by SGD on a 4-point toy set; we run the same loop in
# float64 as a reference and confirm both losses fall and track each other.
#
# Run:  python3 tools/gft_train_demo.py
import math, random
BIAS = 40

# ---- GF-T16 signed arithmetic (bit-exact to the .t27 specs) ----
def gft_value(u):
    if u == 0: return 0.0
    s = -1.0 if (u >> 16) == 1 else 1.0
    mag = u & 65535
    return s * (1 + (mag & 511)/512) * 2 ** ((mag >> 9) - BIAS)

def f2gft(x):
    if x == 0.0: return 0
    s = 1 if x < 0 else 0
    ax = abs(x); exp = math.floor(math.log2(ax)); frac = ax/(2**exp)
    mant = round((frac-1.0)*512); off = exp + BIAS
    if mant == 512: mant = 0; off += 1
    if off < 1: off = 1; mant = 0
    if off > 80: off = 80; mant = 511
    return (s << 16) | (off << 9) | mant

def magadd(a, b):
    ao,am=a>>9,a&511; bo,bm=b>>9,b&511
    ho,hm,lo,lm=(ao,am,bo,bm) if ao>=bo else (bo,bm,ao,am)
    hs,ls=512+hm,512+lm; d=min(ho-lo,11)
    losh=ls>>d; rem=ls-(losh<<d); s=hs+losh; off=ho; mant=s-512
    if s>=1024:
        gg=s&1; pre=s>>1; mant=pre-512
        if gg==1: mant += 1 if rem>0 else (1 if (pre&1)==1 else 0)
        off=ho+1; off=min(off,80)
    else:
        t=rem<<1; hf=1<<d
        if t>hf: mant+=1
        elif t==hf and (s&1)==1: mant+=1
    if mant>=512: mant=0; off=min(off+1,80)
    return (off<<9)|mant

def magsub(hi, lo):
    if hi==lo: return 0
    ho,hm=hi>>9,hi&511; lo_o,lm=lo>>9,lo&511
    d=ho-lo_o; hs=(512+hm)<<14; la=0; sticky=0
    if d<0: return 0   # hi<lo: spec discards this call (recomputes magsub(mb,ma))
    if d>=26: la=0; sticky=1
    else:
        ls=(512+lm)<<14; la=ls>>d
        if (ls-(la<<d))>0: sticky=1
    diff=hs-la; off=ho
    for _ in range(12):
        if diff<8388608 and off>1: diff<<=1; off-=1
    q=diff>>14; rem=diff-(q<<14); half=8192; mant=q-512
    if rem>half: mant+=1
    elif rem==half: mant += 1 if sticky==1 else (1 if (q&1)==1 else 0)
    if mant>=512: mant=0; off=min(off+1,80)
    return (off<<9)|mant

def sadd(a, b):
    if a==0: return b
    if b==0: return a
    sa,ma=a>>16,a&65535; sb,mb=b>>16,b&65535
    if sa==sb: return ((sa<<16)|magadd(ma,mb))&0xffffffff
    bsign=sa; r=magsub(ma,mb)
    if ma<mb: r=magsub(mb,ma); bsign=sb
    if r==0: return 0
    return ((bsign<<16)|r)&0xffffffff

def neg(v): return 0 if v==0 else v^65536

def magmul(a16, b16):
    ao,am=a16>>9,a16&511; bo,bm=b16>>9,b16&511
    prod=(512+am)*(512+bm); carry=1 if prod>=524288 else 0
    if carry: q=prod>>10; r=prod&1023; half=512
    else: q=prod>>9; r=prod&511; half=256
    mant=q-512
    if r>half: mant+=1
    elif r==half and (q&1)==1: mant+=1
    sm_=ao+bo+carry; out_off=0
    if sm_>=40: out_off=min(sm_-40,80)
    if mant>=512: mant=0; out_off=min(out_off+1,80)
    return (out_off<<9)|mant

def smul(a, b):
    if a==0 or b==0: return 0
    sign=((a>>16)&1)^((b>>16)&1); mag=magmul(a&65535,b&65535)
    return 0 if mag==0 else (sign<<16)|mag

# exp2 (Q16 quartic, <=1 ULP vs true 2^x)
K=[6,29,123,354]; H=1<<15
def pow2_frac(f):
    p=K[0]
    for c in K[1:]: p=((p*f+H)>>16)+c
    p=(p*f+H)>>16
    return min(p,511)
def exp2(x):
    if x==0: return 20480
    ng=1 if (x>>16)==1 else 0
    off_in=(x>>9)&127; mant_in=x&511
    if off_in>=48: return 512 if ng else ((80<<9)|511)
    num=512+mant_in; sh=off_in-33
    mq=num<<sh if sh>=0 else num>>(-sh)
    ki=mq>>16; ff=mq&65535
    if ng: (k,f)=(-ki,0) if ff==0 else (-(ki+1),65536-ff)
    else: k,f=ki,ff
    mant=pow2_frac(f); off=k+40
    if off<1: return 512
    if off>80: return (80<<9)|511
    return (off<<9)|mant

def recip(x):
    if x==0: return (80<<9)|511
    s=(x>>16)&1; o=(x>>9)&127; m=x&511; den=512+m
    mp=(524288+(den>>1))//den-512; off=79-o
    if mp>=512: mp-=512; off+=1
    if off<1: off=1; mp=0
    if off>80: off=80; mp=511
    return (s<<16)|(off<<9)|mp

def _cat(a):
    if a==0: return 1
    return 2 if (a>>16)==0 else 0
def _gt(a,b):
    ca,cb=_cat(a),_cat(b)
    if ca!=cb: return 1 if ca>cb else 0
    ma,mb=a&65535,b&65535
    if ca==2: return 1 if ma>mb else 0
    if ca==0: return 1 if ma<mb else 0
    return 0

def softmax(ls, sel):
    mx=ls[0]
    for i in range(1,4):
        if _gt(ls[i],mx)==1: mx=ls[i]
    es=[exp2(sadd(ls[i],neg(mx))) for i in range(4)]
    S=sadd(sadd(es[0],es[1]),sadd(es[2],es[3])); r=recip(S)
    return magmul(es[sel]&65535, r&65535) & 0xffff

# ---- toy task + training loop ----
X=[(1.0,0.0),(0.0,1.0),(-1.0,0.0),(0.0,-1.0)]; Y=[0,1,2,3]

def forward(W,x):
    out=[]
    for c in range(4):
        acc=0
        for j in range(2): acc=sadd(acc,smul(W[c][j],x[j]))
        out.append(acc)
    return out

def loss(W):
    tot=0.0
    for (x0,x1),t in zip(X,Y):
        lg=forward(W,[f2gft(x0),f2gft(x1)])
        pt=gft_value(softmax(lg,t))
        tot+=-math.log2(max(pt,1e-9))
    return tot/len(X)

def main():
    rng=random.Random(2901)
    Wf=[[rng.uniform(-0.3,0.3) for _ in range(2)] for _ in range(4)]
    W=[[f2gft(Wf[c][j]) for j in range(2)] for c in range(4)]
    eta=0.5; eta_g=f2gft(eta)
    def floss(Wf):
        tot=0.0
        for (x0,x1),t in zip(X,Y):
            lg=[Wf[c][0]*x0+Wf[c][1]*x1 for c in range(4)]
            m=max(lg); ex=[2**(v-m) for v in lg]; s=sum(ex)
            tot+=-math.log2(max(ex[t]/s,1e-9))
        return tot/len(X)
    print(f"{'epoch':>5} {'gft_loss':>10} {'float_loss':>10}")
    print(f"{0:5d} {loss(W):10.4f} {floss(Wf):10.4f}")
    for ep in range(1,21):
        for (x0,x1),t in zip(X,Y):
            xg=[f2gft(x0),f2gft(x1)]; lg=forward(W,xg)
            for c in range(4):
                pc=softmax(lg,c); gc=sadd(pc,neg(f2gft(1.0))) if c==t else pc
                for j in range(2):
                    W[c][j]=sadd(W[c][j],neg(smul(eta_g,smul(gc,xg[j]))))
            lgf=[Wf[c][0]*x0+Wf[c][1]*x1 for c in range(4)]
            m=max(lgf); ex=[2**(v-m) for v in lgf]; s=sum(ex); p=[v/s for v in ex]
            for c in range(4):
                gc=p[c]-(1.0 if c==t else 0.0)
                for j in range(2): Wf[c][j]-=eta*gc*[x0,x1][j]
        if ep%2==0 or ep<=3: print(f"{ep:5d} {loss(W):10.4f} {floss(Wf):10.4f}")
    print("\nfinal GF-T predictions:")
    ok=0
    for (x0,x1),t in zip(X,Y):
        lg=forward(W,[f2gft(x0),f2gft(x1)]); pred=max(range(4),key=lambda c: gft_value(lg[c]))
        ok+=(pred==t); print(f"  x=({x0:+.0f},{x1:+.0f}) target={t} pred={pred} {'OK' if pred==t else 'XX'}")
    print(f"accuracy: {ok}/{len(X)}")

if __name__=="__main__": main()
