#!/usr/bin/env python3
# GF-T GENERALIZATION demo: does GF-T learning GENERALIZE to unseen data, not just
# fit the training points? A linear 4-class classifier (2D points labeled by
# quadrant) is trained by SGD on a random TRAIN split using ONLY the GF-T integer
# models from gft_train_demo.py (bit-exact to the synthesized hardware), then
# evaluated on a held-out TEST split. Rising test accuracy => GF-T learning
# generalizes.  Run:  python3 tools/gft_generalize_demo.py
import sys, os, random
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gft_train_demo import f2gft, gft_value, sadd, smul, neg, softmax

def quadrant(x0, x1):
    if x0 >= 0 and x1 >= 0: return 0
    if x0 < 0 and x1 >= 0:  return 1
    if x0 < 0 and x1 < 0:   return 2
    return 3

def gen(n, rng):
    pts = []
    while len(pts) < n:
        x0 = rng.uniform(-1, 1); x1 = rng.uniform(-1, 1)
        if abs(x0) < 0.15 or abs(x1) < 0.15:   # avoid axis-boundary ambiguity
            continue
        pts.append(((x0, x1), quadrant(x0, x1)))
    return pts

def forward(W, x):
    out = []
    for c in range(4):
        acc = 0
        for j in range(2): acc = sadd(acc, smul(W[c][j], x[j]))
        out.append(acc)
    return out

def accuracy(W, data):
    ok = 0
    for (x0, x1), t in data:
        lg = forward(W, [f2gft(x0), f2gft(x1)])
        pred = max(range(4), key=lambda c: gft_value(lg[c]))
        ok += (pred == t)
    return ok / len(data)

def main():
    rng = random.Random(3201)
    train = gen(120, rng); test = gen(60, rng)
    W = [[f2gft(rng.uniform(-0.2, 0.2)) for _ in range(2)] for _ in range(4)]
    eta_g = f2gft(0.2)
    print(f"train={len(train)} test={len(test)} points (2D, 4 quadrant classes)")
    print(f"{'epoch':>5} {'train_acc':>10} {'test_acc':>9}")
    print(f"{0:5d} {accuracy(W,train):10.3f} {accuracy(W,test):9.3f}")
    for ep in range(1, 26):
        random.Random(ep).shuffle(train)
        for (x0, x1), t in train:
            xg = [f2gft(x0), f2gft(x1)]; lg = forward(W, xg)
            for c in range(4):
                pc = softmax(lg, c)
                gc = sadd(pc, neg(f2gft(1.0))) if c == t else pc
                for j in range(2):
                    W[c][j] = sadd(W[c][j], neg(smul(eta_g, smul(gc, xg[j]))))
        if ep % 5 == 0 or ep <= 2:
            print(f"{ep:5d} {accuracy(W,train):10.3f} {accuracy(W,test):9.3f}")
    print(f"\nFINAL held-out test accuracy: {accuracy(W,test):.1%}")

if __name__ == "__main__":
    main()
