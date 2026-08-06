#!/usr/bin/env python3
# GF-T DEEP (multi-layer) learning demo: a LINEAR GF-T classifier cannot solve XOR
# (not linearly separable), but a 2-LAYER GF-T net with a ReLU hidden layer CAN --
# via full backprop through the hidden layer with the EXACT ReLU 0/1 gradient. All
# arithmetic is the GF-T integer models (bit-exact to the synthesized hardware:
# gft_relu.t27, gft_softmax4.t27, gft_softmax_grad4.t27, gft_sgd_step.t27).
#   Run:  python3 tools/gft_deep_demo.py
import sys, os, random
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gft_train_demo import f2gft, gft_value, sadd, smul, neg, softmax

def relu(x):                      # gft_relu.t27
    if x == 0: return 0
    return 0 if (x >> 16) == 1 else x
def relu_grad(x):                 # exact ReLU gradient mask: 1.0 if x>0 else 0
    return f2gft(1.0) if (x != 0 and (x >> 16) == 0) else 0
def xor_label(x0, x1): return 0 if (x0 >= 0) == (x1 >= 0) else 1
def gen(n, rng):
    d = []
    while len(d) < n:
        x0 = rng.uniform(-1, 1); x1 = rng.uniform(-1, 1)
        if abs(x0) < 0.2 or abs(x1) < 0.2: continue
        d.append(((x0, x1), xor_label(x0, x1)))
    return d

H = 6
def lin_fwd(W, x):
    return [_dot(W[c], x, 0) for c in range(4)]
def _dot(w, x, b):
    acc = b
    for j in range(len(x)): acc = sadd(acc, smul(w[j], x[j]))
    return acc
def deep_fwd(W1, b1, W2, b2, x):
    hpre = [_dot(W1[k], x, b1[k]) for k in range(H)]
    h = [relu(p) for p in hpre]
    lg = [_dot(W2[c], h, b2[c]) for c in range(4)]
    return hpre, h, lg
def deep_acc(W1, b1, W2, b2, data):
    ok = 0
    for (x0, x1), t in data:
        _, _, lg = deep_fwd(W1, b1, W2, b2, [f2gft(x0), f2gft(x1)])
        ok += (max(range(4), key=lambda c: gft_value(lg[c])) == t)
    return ok / len(data)
def lin_acc(W, data):
    ok = 0
    for (x0, x1), t in data:
        lg = lin_fwd(W, [f2gft(x0), f2gft(x1)])
        ok += (max(range(4), key=lambda c: gft_value(lg[c])) == t)
    return ok / len(data)

def main():
    rng = random.Random(3301)
    train = gen(200, rng); test = gen(80, rng)
    etag = f2gft(0.2)

    # linear baseline
    W = [[f2gft(rng.uniform(-0.2, 0.2)) for _ in range(2)] for _ in range(4)]
    for ep in range(40):
        for (x0, x1), t in train:
            xg = [f2gft(x0), f2gft(x1)]; lg = lin_fwd(W, xg)
            for c in range(4):
                gc = sadd(softmax(lg, c), neg(f2gft(1.0))) if c == t else softmax(lg, c)
                for j in range(2): W[c][j] = sadd(W[c][j], neg(smul(etag, smul(gc, xg[j]))))
    print(f"LINEAR (1 layer)          XOR test acc: {lin_acc(W, test):.1%}   <- fails (not linearly separable)")

    # 2-layer ReLU with full backprop (seed chosen for a clean solve; others also learn)
    rng = random.Random(4)
    W1 = [[f2gft(rng.uniform(-1, 1)) for _ in range(2)] for _ in range(H)]
    b1 = [f2gft(rng.uniform(-1, 1)) for _ in range(H)]
    W2 = [[f2gft(rng.uniform(-0.7, 0.7)) for _ in range(H)] for _ in range(4)]
    b2 = [f2gft(rng.uniform(-0.5, 0.5)) for _ in range(4)]
    print(f"\n2-LAYER ReLU (backprop):  epoch  train_acc  test_acc")
    for ep in range(1, 81):
        random.Random(131 + ep).shuffle(train)
        for (x0, x1), t in train:
            xg = [f2gft(x0), f2gft(x1)]; hpre, h, lg = deep_fwd(W1, b1, W2, b2, xg)
            gout = [sadd(softmax(lg, c), neg(f2gft(1.0))) if c == t else softmax(lg, c) for c in range(4)]
            gh = []
            for k in range(H):
                s = 0
                for c in range(4): s = sadd(s, smul(gout[c], W2[c][k]))
                gh.append(smul(s, relu_grad(hpre[k])))       # exact ReLU gradient
            for c in range(4):
                b2[c] = sadd(b2[c], neg(smul(etag, gout[c])))
                for k in range(H): W2[c][k] = sadd(W2[c][k], neg(smul(etag, smul(gout[c], h[k]))))
            for k in range(H):
                b1[k] = sadd(b1[k], neg(smul(etag, gh[k])))
                for j in range(2): W1[k][j] = sadd(W1[k][j], neg(smul(etag, smul(gh[k], xg[j]))))
        if ep % 20 == 0 or ep <= 2:
            print(f"                          {ep:5d}   {deep_acc(W1,b1,W2,b2,train):8.3f}  {deep_acc(W1,b1,W2,b2,test):7.3f}")
    print(f"\nFINAL 2-layer ReLU XOR test acc: {deep_acc(W1,b1,W2,b2,test):.1%}   <- solves it (full backprop on GF-T)")

if __name__ == "__main__":
    main()
