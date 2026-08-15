"""W778: the layer mix as a VARIABLE (T402a), and the claim it puts at risk.

T398 measured a trained base-3 network against a dyadic one on three dice:

    784 -> 16  tables      83  vs  89     base 3 cheaper by 6
     16 -> 16  tables      62  vs  60     base 3 DEARER by 2
     16 ->  1  adder      203  vs 103     dyadic half the cost

and the sieve's spec turned the first row into a general rule:
`prefers_base(b, LAYER_TABLE) = b >= 3`. But the two table rows DISAGREE, and
the rule was written from the one with the bigger number. So the mix arithmetic
for a net with L middle table layers is not what T402a assumed:

    base3(L)  = 83 + 62 L + 203
    dyadic(L) = 89 + 60 L + 103
    gap(L)    = 94 + 2 L        <- INCREASING. Base 3 never catches up.

The earlier reading -- that stacking table layers must eventually favour base 3,
crossing over near L = 17 -- silently used the 784->16 row's slope for a layer of
the 16->16 kind. It is the wide INPUT layer that was cheaper, and that is one
measurement, not a law.

REGISTERED FORECAST, before the run (T44):

  H1  base 3 is genuinely cheaper in table layers. Then its per-layer slope is
      BELOW dyadic's and the gap shrinks with L.
  H0  the 83-vs-89 row was the wide input layer, or noise. Then the base-3 slope
      is at or above dyadic's and the gap never closes.

  I forecast H0, from the mechanism: in a TABLE layer the weights are baked into
  a case statement, so the base changes WHICH truth table, never the structure.
  The area difference is truth-table entropy -- small, and signed at random.
  Quantitatively: |slope(base3) - slope(dyadic)| < 4 LUT per 16-neuron layer,
  and the sign of that difference is not stable across widths.

  REFUTATION CONSEQUENCE, stated now so it cannot be renegotiated: if the base-3
  slope is at or above dyadic's, "base 3 is cheaper in table layers" is WITHDRAWN
  as a general claim, re-scoped to the wide input layer alone, and
  `prefers_base` in specs/numeric/golden_sieve.t27 is wrong and must be
  re-scoped with it.

The stand: L stacked table layers of H neurons, each neuron reading fan-in 3
ternary symbols (6 bits, the six-bit rule) from the layer before and emitting one
ternary symbol. Uniform layers, so the slope is clean. Weights are drawn once
from a fixed seed and shared across bases by INDEX, so the two arms differ only
in the alphabet's values -- never in which weight went where.
"""
import argparse
import random

FANIN = 3          # three ternary symbols = six bits = the six-bit rule
SYM_BITS = 2       # a ternary symbol on a binary substrate


NAMED = {
    # W778: alphabets that are NOT geometric ladders. The ladder family cannot
    # avoid weight domination over Z (T409), so the comparison needs members that
    # can. Magnitudes only; 0 and the negatives are added by alphabet().
    "linear": [1., 2., 3., 4.],
    "fib":    [1., 1., 2., 3.],
    "1235":   [1., 2., 3., 5.],
}


def alphabet(base, n_pos=4):
    """{0} u {+-base^i} NORMALISED to unit mean magnitude: the k=2 rung.

    W778 DEFECT AND FIX. The first version returned the raw ladder and set the
    threshold to its mean magnitude -- 3.75 for dyadic, 10.0 for base 3. That is
    a CONFOUND, not a control: relative to its own spread, a base-3 neuron with
    small weights (1,1,3) cannot reach 10 under any input and emits a CONSTANT.
    Constants propagate, the chain collapses, and the area measurement becomes a
    measurement of how much of the network died. It read base 3 at 8 LUT for
    EIGHT layers, below its own 34 at one.

    Training never had this problem because it uses BitNet absmean scaling
    (train_ladder.py): s = mean|W| / mean|lv|, so the alphabet's SCALE is
    normalised away and only its SHAPE differs between bases. This does the same,
    once, here.
    """
    lv = list(NAMED[base]) if isinstance(base, str) else \
         [+float(base) ** i for i in range(n_pos)]
    g = sum(lv) / len(lv)
    lv = [x / g for x in lv]
    return [0.0] + lv + [-x for x in lv]


def decode(sym):
    """Two-bit code -> trit value. 0b00=0, 0b01=+1, 0b10=-1; 0b11 unused."""
    return {0: 0, 1: 1, 2: -1, 3: 0}[sym]


def truth_table(weights, thr):
    """Enumerate the neuron over all 2^(FANIN*SYM_BITS) input codes.

    This IS the datapath (T331): a neuron small enough to enumerate costs 2.00
    LUT, and the weights never appear as arithmetic. Which is exactly why the
    base can only change the table's CONTENT.
    """
    rows = []
    for code in range(1 << (FANIN * SYM_BITS)):
        acc = 0.0
        for i in range(FANIN):
            acc += weights[i] * decode((code >> (SYM_BITS * i)) & 3)
        out = 1 if acc > thr else (2 if acc < -thr else 0)
        rows.append((code, out))
    return rows


def live_count(H, wts, thr):
    """Neurons whose truth table is NOT constant.

    The guard the first run lacked. A constant neuron is not a cheap neuron, it
    is an absent one, and a layer of them makes the whole stand read low. Any
    area comparison between two arms is void unless both are equally alive.
    """
    n = 0
    for w in wts:
        outs = {o for _, o in truth_table(w, thr)}
        if len(outs) > 1:
            n += 1
    return n


def emit_layer(idx, H, picks, wts, thr):
    """One table layer: H neurons, each a case over its three picked symbols."""
    v = [f"  wire [{2*H-1}:0] s{idx};"]
    for n in range(H):
        sel = " , ".join(f"s{idx-1}[{2*p+1}:{2*p}]" for p in reversed(picks[n]))
        v.append(f"  reg [1:0] n{idx}_{n};")
        v.append(f"  always @* case ({{ {sel} }})")
        for code, out in truth_table(wts[n], thr):
            v.append(f"    6'd{code}: n{idx}_{n} = 2'd{out};")
        v.append(f"    default: n{idx}_{n} = 2'd0;")
        v.append("  endcase")
        v.append(f"  assign s{idx}[{2*n+1}:{2*n}] = n{idx}_{n};")
    return v


def build(L, H, base, seed=7):
    """L table layers, H neurons each, registered in and out so nothing prunes."""
    rng = random.Random(seed)
    lv = alphabet(base)              # already unit mean magnitude
    thr = 1.0                        # so ONE threshold is comparable across bases

    # Draw the structure ONCE, before the base is consulted, so both arms get
    # identical wiring and identical weight INDICES -- only the values differ.
    picks = [[[rng.randrange(H) for _ in range(FANIN)] for _ in range(H)]
             for _ in range(L)]
    # W778: LIVENESS IS THE CONTROL, not a side effect. Drawing weights uniformly
    # leaves base 3 with 12-18% constant neurons against dyadic's 0-5%, because
    # its normalised nine levels span 27:1 against dyadic's 8:1 and the three
    # smallest cannot reach the threshold in any combination. Two arms of unequal
    # liveness have incomparable areas: the deader one merely looks cheaper.
    #
    # So each neuron is REJECTION-SAMPLED until its table is non-constant. The
    # arms no longer share weight indices -- they share the property that every
    # neuron exists, which is the control the area question actually needs. The
    # rejection RATE is reported: it is the finding this run produced.
    widx, rejects, draws = [], 0, 0
    for _ in range(L):
        layer = []
        for _ in range(H):
            for _attempt in range(200):
                w = [rng.randrange(len(lv)) for _ in range(FANIN)]
                draws += 1
                if len({o for _, o in truth_table([lv[i] for i in w], thr)}) > 1:
                    break
                rejects += 1
            layer.append(w)
        widx.append(layer)

    live = []
    for l in range(1, L + 1):
        wl = [[lv[widx[l-1][n][i]] for i in range(FANIN)] for n in range(H)]
        live.append(live_count(H, wl, thr))

    # W780: PAD-FREE TOP. nextpnr aborts on this chipdb at the seventh package
    # pin ("No Bel named 'OPAD_X0Y15/IOB33/INBUF_EN'"), so real place-and-route was
    # unreachable while the stand had 12 ports. Driving `din` from an internal LFSR
    # and `addr` from an internal counter leaves exactly TWO pads, clk and dout,
    # and keeps every layer live -- an LFSR is stateful, so nothing constant-folds.
    v = [f"// W778 layer-mix stand: L={L} table layers, H={H}, alphabet={base}",
         f"// live neurons per layer: {live}  ({sum(live)}/{L*H})",
         f"// constant draws rejected: {rejects}/{draws} = {rejects/draws:.1%}",
         "module mix_top(input clk, output reg dout);",
         "  reg [15:0] lfsr = 16'hACE1;",
         "  always @(posedge clk) lfsr <= {lfsr[14:0],"
         " lfsr[15]^lfsr[13]^lfsr[12]^lfsr[10]};",
         "  reg [7:0] ctr = 0;",
         "  always @(posedge clk) ctr <= ctr + 1'b1;",
         f"  reg [{2*H-1}:0] s0;",
         "  always @(posedge clk) s0 <= {s0[%d:0], lfsr[1:0]};" % (2 * H - 3)]
    for l in range(1, L + 1):
        wts = [[lv[widx[l-1][n][i]] for i in range(FANIN)] for n in range(H)]
        v += emit_layer(l, H, picks[l-1], wts, thr)
    # Register the last layer out through a mux, so the whole chain is live but
    # the OUTPUT costs the same in both arms and never masks the layer slope.
    v.append(f"  always @(posedge clk) dout <= s{L}[ctr[%d:0]];" % (max(0, (2*H-1).bit_length()-1)))
    v.append("endmodule")
    return "\n".join(v)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--layers", type=int, required=True)
    ap.add_argument("--width", type=int, default=16)
    ap.add_argument("--base", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    b = a.base if a.base in NAMED else float(a.base)
    src = build(a.layers, a.width, b)
    open(a.out, "w").write(src)
    live = src.split("live neurons per layer: ")[1].split("\n")[0]
    rej = src.split("constant draws rejected: ")[1].split("\n")[0]
    n, tot = live.split("(")[1].rstrip(")").split("/")
    frac = int(n) / int(tot)
    print(f"  L={a.layers} H={a.width} base={a.base}  live {n}/{tot} = {frac:.0%}"
          f"  rejected {rej}")
    if frac < 1.0:
        raise SystemExit(f"GUARD: {frac:.0%} live after rejection sampling -- "
                         f"the area of this design is not the area of this network")
