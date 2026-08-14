#!/usr/bin/env python3
"""Emit Verilog for one dense binary-input layer per alphabet of the phi-ladder.

The ladder: phi^k = F(k-1) + F(k)*phi, so a weight phi^k applied to x adds
F(k-1)*x to the A lane and F(k)*x to the B lane of a Z[phi] pair (A,B).

    phi^0 = 1        -> (1, 0)
    phi^1 = phi      -> (0, 1)
    phi^2 = 1 + phi  -> (1, 1)
    phi^3 = 1 + 2phi -> (1, 2)     first coefficient above 1: one shift
    phi^4 = 2 + 3phi -> (2, 3)     3 is not a power of two: one add

So the ladder's hardware cost is Fibonacci, and phiT2 -- the seven-level set --
is the LAST rung whose coefficients are all in {0,1}: pure wiring, no shift,
no multiplier. This script measures whether the tool agrees.

Every arm uses the SAME zero pattern and the SAME seed; only the distribution
of nonzero levels changes. Anything else would confound alphabet with sparsity.
"""
import argparse, random, sys

# alphabet -> list of (A_coeff, B_coeff) for the POSITIVE levels; sign is added
# separately so every arm has the same sign statistics.
LADDER = {
    "gft0":  [(1, 0)],                                  # {0,+-1}          3 levels
    "q4":  [(1, 0), (2, 0)],                          # {0,+-1,+-2}      5 levels, dyadic
    "gft1": [(1, 0), (0, 1)],                          # {0,+-1,+-phi}    5 levels
    "gft2": [(1, 0), (0, 1), (1, 1)],                  # +-phi^2          7 levels
    "gft3": [(1, 0), (0, 1), (1, 1), (1, 2)],          # +-phi^3          9 levels
    "gft4": [(1, 0), (0, 1), (1, 1), (1, 2), (2, 3)],  # +-phi^4         11 levels
}
PAIRED = {"gft1", "gft2", "gft3", "gft4"}


def emit_layer(arm, n, m, seed, acc_w):
    rnd = random.Random(seed)
    levels = LADDER[arm]
    paired = arm in PAIRED
    # ONE zero pattern shared by every arm: drawn from a seed that does not
    # depend on the arm, so sparsity is held fixed across the comparison.
    zrnd = random.Random(seed ^ 0x5EED)
    lines = []
    ports = ["input wire [%d:0] x" % (n - 1)]
    if paired:
        ports += ["output wire signed [%d:0] accA [0:%d]" % (acc_w - 1, m - 1)]
    lines.append("// arm=%s levels=%d paired=%s" % (arm, 2 * len(levels) + 1, paired))
    lines.append("module layer_%s (" % arm)
    lines.append("    input  wire [%d:0] x," % (n - 1))
    outs = []
    for j in range(m):
        outs.append("    output wire signed [%d:0] a%d" % (acc_w - 1, j))
        if paired:
            outs.append("    output wire signed [%d:0] b%d" % (acc_w - 1, j))
    lines.append(",\n".join(outs))
    lines.append(");")
    for j in range(m):
        termsA, termsB = [], []
        for i in range(n):
            if zrnd.random() < 0.5:            # identical zero mask for all arms
                continue
            ca, cb = levels[rnd.randrange(len(levels))]
            sgn = -1 if rnd.random() < 0.5 else 1
            for c, bucket in ((ca, termsA), (cb, termsB)):
                if c == 0:
                    continue
                op = "-" if sgn < 0 else "+"
                if c == 1:
                    bucket.append("%s $signed({{%d{1'b0}}, x[%d]})" % (op, acc_w - 1, i))
                else:
                    bucket.append("%s $signed({{%d{1'b0}}, x[%d]}) * %d"
                                  % (op, acc_w - 1, i, c))
        def fold(ts):
            if not ts:
                return "%d'sd0" % acc_w
            s = " ".join(ts)
            return s[2:] if s.startswith("+ ") else s
        lines.append("    assign a%d = %s;" % (j, fold(termsA)))
        if paired:
            lines.append("    assign b%d = %s;" % (j, fold(termsB)))
    lines.append("endmodule")
    return "\n".join(lines)


def emit_cmp(kind, acc_w):
    """Sign of A + B*phi.  Two honest variants, measured apart from the layer."""
    if kind == "scalar":
        return ("module cmp_scalar (input wire signed [%d:0] a, output wire y);\n"
                "    assign y = ~a[%d];\nendmodule" % (acc_w - 1, acc_w - 1))
    if kind == "q8":
        # phi ~ 414/256 (Q8).  Approximate: reintroduces the rounding the pair
        # form exists to avoid, and that is the point of measuring it.
        return ("module cmp_q8 (input wire signed [%d:0] a, input wire signed [%d:0] b,\n"
                "                output wire y);\n"
                "    wire signed [%d:0] s = ($signed(a) <<< 8) + $signed(b) * 414;\n"
                "    assign y = ~s[%d];\nendmodule" % (acc_w - 1, acc_w - 1, acc_w + 9, acc_w + 9))
    if kind == "exact":
        # 2(A + B phi) = (2A+B) + B*sqrt5.  Sign is exact via u^2 vs 5v^2 when
        # u and v disagree in sign.  Squares are real multipliers -- that cost
        # is the finding, not a flaw in the encoding.
        w = acc_w + 2
        return ("module cmp_exact (input wire signed [%d:0] a, input wire signed [%d:0] b,\n"
                "                   output wire y);\n"
                "    wire signed [%d:0] u = ($signed(a) <<< 1) + $signed(b);\n"
                "    wire signed [%d:0] v = $signed(b);\n"
                "    wire signed [%d:0] uu = u * u;\n"
                "    wire signed [%d:0] vv = v * v * 5;\n"
                "    assign y = (~u[%d] & ~v[%d]) ? 1'b1 :\n"
                "               ( u[%d] &  v[%d]) ? 1'b0 :\n"
                "               (~u[%d]) ? (uu > vv) : (uu < vv);\n"
                "endmodule" % (acc_w - 1, acc_w - 1, w, w, 2 * w + 3, 2 * w + 3,
                               w, w, w, w, w))
    raise SystemExit("unknown cmp " + kind)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", required=True)
    ap.add_argument("-n", type=int, default=64)
    ap.add_argument("-m", type=int, default=8)
    ap.add_argument("--seed", type=int, default=27)
    ap.add_argument("--acc", type=int, default=12)
    ap.add_argument("--cmp", default=None)
    a = ap.parse_args()
    if a.cmp:
        print(emit_cmp(a.cmp, a.acc))
    else:
        print(emit_layer(a.arm, a.n, a.m, a.seed, a.acc))
