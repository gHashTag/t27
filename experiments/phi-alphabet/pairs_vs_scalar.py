"""W712: does propagating Z[phi] PAIRS compute something a scalar collapse cannot?

T183a named this as phi's last branch: keep un-collapsed (a,b) coordinates
meaning a + b*phi across layers, so weight application is the Fibonacci step
(a,b) -> (b, a+b) -- one integer add, exact, no multiplier. T207 closed the
scalar form: phi adds nothing there. This is the only form left.

THE PREDICTION, REGISTERED BEFORE MEASURING: a pair (a,b) is an EXACT
REPRESENTATION of the real number a + b*phi. The scalar path computes that same
real number in float. So the two must agree to float precision, and the
experiment tests the IMPLEMENTATION, not the hypothesis. If so, T183a's second
branch is about HARDWARE COST -- exactness and multiplier-freedom -- and not
about what the network can express.
"""
import numpy as np

PHI = (1 + 5 ** 0.5) / 2
rng = np.random.default_rng(27)

def layer_scalar(x, W):
    """The collapsed path: weights are the real numbers +-1, +-phi, 0."""
    return x @ W

def layer_pairs(xa, xb, code):
    """Exact Z[phi] path. `code` in {-2,-1,0,1,2} meaning {-phi,-1,0,+1,+phi}.

    A value is (a, b) = a + b*phi. Applying +-1 scales both coordinates.
    Applying +-phi is the Fibonacci step: phi*(a + b*phi) = b + (a+b)*phi,
    i.e. (a,b) -> (b, a+b). Both are pure integer, no multiplication.
    """
    n_out = code.shape[1]
    oa = np.zeros((xa.shape[0], n_out))
    ob = np.zeros((xa.shape[0], n_out))
    for j in range(n_out):
        c = code[:, j]
        # +-1 : (a,b) -> (+-a, +-b)
        s1 = (c == 1).astype(float) - (c == -1).astype(float)
        oa[:, j] += xa @ s1
        ob[:, j] += xb @ s1
        # +-phi : (a,b) -> (+-b, +-(a+b))
        sp = (c == 2).astype(float) - (c == -2).astype(float)
        oa[:, j] += xb @ sp
        ob[:, j] += (xa + xb) @ sp
    return oa, ob

CODE_TO_REAL = {-2: -PHI, -1: -1.0, 0: 0.0, 1: 1.0, 2: PHI}

def main():
    n, d_in, d_h, d_out = 512, 64, 48, 32
    # integer inputs, so the pair path is EXACT integer arithmetic throughout
    xa = rng.integers(-8, 9, (n, d_in)).astype(float)
    xb = np.zeros_like(xa)                      # inputs are plain integers: b = 0
    c1 = rng.integers(-2, 3, (d_in, d_h))
    c2 = rng.integers(-2, 3, (d_h, d_out))

    W1 = np.vectorize(CODE_TO_REAL.get)(c1).astype(float)
    W2 = np.vectorize(CODE_TO_REAL.get)(c2).astype(float)

    # --- scalar path, two layers, no activation (pure linear comparison) ---
    h_s = layer_scalar(xa, W1)
    o_s = layer_scalar(h_s, W2)

    # --- pair path, two layers, exact ---
    ha, hb = layer_pairs(xa, xb, c1)
    oa, ob = layer_pairs(ha, hb, c2)
    o_p = oa + ob * PHI

    err = np.abs(o_s - o_p)
    rel = err / (np.abs(o_s) + 1e-12)
    print(f"  выходов: {o_s.size}")
    print(f"  максимальная абсолютная разность: {err.max():.3e}")
    print(f"  максимальная относительная:       {rel.max():.3e}")
    print(f"  совпадает до точности float64:    {'ДА' if rel.max() < 1e-9 else 'НЕТ'}")

    # and the sign agreement, which is what an activation actually uses
    agree = np.mean(np.sign(o_s) == np.sign(o_p))
    print(f"  доля совпадающих знаков:          {agree:.6f}")

    # coefficient growth -- T159's Theta(phi^k), measured here
    print()
    print("  рост координат по слоям (то, что T159 называет ценой точности):")
    a, b = xa.copy(), xb.copy()
    for k in range(1, 7):
        c = rng.integers(-2, 3, (a.shape[1], a.shape[1]))
        a, b = layer_pairs(a, b, c)
        m = max(np.abs(a).max(), np.abs(b).max())
        print(f"    слой {k}: max|coeff| = {m:.3e}   бит = {np.log2(m+1):.1f}")

if __name__ == "__main__":
    main()
