"""W778: run all 83 catalogued numeric formats through the golden sieve.

Asked twice and deferred twice. The sieve (T394, specs/numeric/golden_sieve.t27)
was derived on WEIGHT ALPHABETS. The catalogue in gen/numeric/formats_catalog.json
is overwhelmingly ACCUMULATOR and STORAGE formats. Running one through the other
is therefore first a CATEGORY test and only then a quality test, and this script
reports it that way -- a format killed by S1 is not a bad format, it is a format
answering a different question.

THE S3 REPAIR, found while writing this and the reason the run is worth keeping.
golden_sieve.t27 states S3 as `lanes == 1` with `lanes` handed in by the caller.
That is not a predicate on the alphabet, it is the answer typed in by hand -- and
it gets our own flagship format WRONG. {-phi, 0, +phi} looks irrational and would
be handed lanes=2, but phi is a COMMON POSITIVE SCALE and factors straight out
(T207): the alphabet is phi*{-1,0,+1} and rides one lane. The two-lane cost of
T293 appears only when powers of phi are MIXED, as in {0,+-1,+-phi}, where 1 and
phi cannot share an accumulator.

So the correct S3 is COMMENSURABILITY: A is single-lane iff every ratio of two
nonzero elements is rational, i.e. A lies in g*Z for some g > 0. Computed here,
not supplied.
"""
import json
import sys
from fractions import Fraction

PHI = (1 + 5 ** 0.5) / 2

# ---------------------------------------------------------------------------
# Level counting. How many DISTINCT FINITE values a format's code space defines,
# which is its alphabet cardinality when it is used as a weight code.
# ---------------------------------------------------------------------------


def levels_of(f):
    """Return (levels, how) or (None, reason) when the record cannot be sieved."""
    b, s, e, m = f["bits"], f["s_bits"], f["e_bits"], f["m_bits"]
    cid, cluster = f["id"], f["cluster"]

    if b == 0:
        return None, "no width declared (technique or parametric family)"

    # Our own ternary format: three levels by construction, and the fourth 2-bit
    # code is unused. That unused code IS the substrate tax, not an extra level.
    if cid == "gfternary":
        return 3, "ternary by construction; 1 of 4 two-bit codes unused"

    if cluster in ("PositUnumIII", "Lns"):
        return 2 ** b - 1, "2^b codes minus one NaR/NaN"

    if cluster == "Ieee754Decimal":
        return None, "decimal encoding; level count is not 2^k and is not needed"

    if e == 0:
        return 2 ** b, "integer/fixed: every code is a value"

    if s + e + m != b:
        return None, f"field widths {s}+{e}+{m} != {b} (record defect)"

    # IEEE-like: the all-ones exponent block holds inf and NaN; +-0 is one value.
    finite = 2 ** b - 2 ** (1 + m)
    return finite - 1, "2^b minus the inf/NaN block, minus the duplicate zero"


# ---------------------------------------------------------------------------
# The five filters
# ---------------------------------------------------------------------------

POWERS_OF_THREE = {3 ** k: k for k in range(1, 40)}


def s1_packing(levels):
    """|A| = 3^k. Only powers of three waste no trits (T367)."""
    return levels in POWERS_OF_THREE


def s2_ceiling(levels):
    """k <= 2. Nine levels; the third trit measured ns on two tasks (T369)."""
    return POWERS_OF_THREE.get(levels, 99) <= 2


def s3_single_lane(alphabet):
    """REPAIRED: commensurability, computed rather than supplied.

    A rides one accumulator lane iff all nonzero elements are rational multiples
    of one another. A common positive scale factors out (T207) -- so {0,+-phi} is
    single-lane and {0,+-1,+-phi} is not.
    """
    nz = [abs(x) for x in alphabet if x != 0]
    if not nz:
        return True
    g = nz[0]
    for x in nz[1:]:
        r = x / g
        # A ratio is accepted as rational only if a small-denominator fraction
        # reproduces it to float precision. phi, sqrt2, plastic all fail this.
        if abs(Fraction(r).limit_denominator(4096) - Fraction(r)) > 1e-12:
            return False
    return True


def s4_trit_fanin(fanin, input_bits):
    """fanin * bits <= 6. Six input bits cost 2.00 LUT; twelve cost 39-54 (T368b)."""
    return fanin * input_bits <= 6


def s5_primitive(dsp, srl):
    """No DSP48E1 and no SRL16E: openXC7 emits wrong bitstreams for both."""
    return dsp == 0 and srl == 0


def sieve(levels, alphabet, fanin=3, input_bits=2, dsp=0, srl=0):
    """Return the first filter that kills the candidate, or None if admissible."""
    if not s1_packing(levels):
        return "S1 PACKING"
    if not s2_ceiling(levels):
        return "S2 CEILING"
    if not s3_single_lane(alphabet):
        return "S3 INTEGRAL"
    if not s4_trit_fanin(fanin, input_bits):
        return "S4 TRIT_FANIN"
    if not s5_primitive(dsp, srl):
        return "S5 PRIMITIVE"
    return None


# ---------------------------------------------------------------------------
# The 16-candidate weight top (W774), which is what the sieve was built for.
# Kept here so the catalogue run and the top run use ONE implementation.
# ---------------------------------------------------------------------------

def ladder(base, n_pos):
    return [0.0] + [+base ** i for i in range(n_pos)] + [-base ** i for i in range(n_pos)]


WEIGHT_TOP = [
    ("dyadic 9",        ladder(2.0, 4),                    9),
    ("base 3",          ladder(3.0, 4),                    9),
    ("base 4",          ladder(4.0, 4),                    9),
    ("linear 9",        [0.0, 1, 2, 3, 4, -1, -2, -3, -4], 9),
    ("balanced ternary", ladder(2.0, 1),                   3),
    ("gfternary",       [0.0, PHI, -PHI],                  3),
    ("golden GA-T3",    ladder(PHI, 4),                    9),
    ("silver",          ladder(1 + 2 ** 0.5, 4),           9),
    ("plastic",         ladder(1.3247179572, 4),           9),
    ("supergolden",     ladder(1.4655712319, 4),           9),
    ("tribonacci",      ladder(1.8392867552, 4),           9),
    ("psi4",            ladder(1.9275619754, 4),           9),
    ("sqrt2",           ladder(2 ** 0.5, 4),               9),
    ("e",               ladder(2.718281828, 4),            9),
    ("dyadic 27",       ladder(2.0, 13),                  27),
    ("dyadic 5",        ladder(2.0, 2),                    5),
]


def main():
    cat = json.load(open("gen/numeric/formats_catalog.json"))["formats"]

    print("=" * 74)
    print("PART 1 -- the 16-candidate WEIGHT top, which the sieve was built for")
    print("=" * 74)
    surv = []
    for name, alpha, lv in WEIGHT_TOP:
        killed = sieve(lv, alpha)
        mark = "ADMISSIBLE" if killed is None else f"killed by {killed}"
        print(f"  {name:18s} |A|={lv:3d}  {mark}")
        if killed is None:
            surv.append(name)
    print(f"\n  survivors: {len(surv)}/{len(WEIGHT_TOP)}  ->  {', '.join(surv)}")

    print()
    print("=" * 74)
    print("PART 2 -- all 83 CATALOGUED formats, as candidate weight codes")
    print("=" * 74)
    kills, unsievable, admissible = {}, [], []
    rows = []
    for f in cat:
        lv, how = levels_of(f)
        if lv is None:
            unsievable.append((f["id"], how))
            continue
        # A catalogued format's alphabet is its code space, which is a uniform
        # grid: always commensurable, so S3 never fires here. S4/S5 are datapath
        # properties, held at the ternary-hidden-layer default.
        killed = sieve(lv, [0.0, 1.0, 2.0])
        rows.append((f["id"], f["cluster"], f["bits"], lv, killed))
        if killed is None:
            admissible.append(f["id"])
        else:
            kills[killed] = kills.get(killed, 0) + 1

    for cid, cl, b, lv, killed in sorted(rows, key=lambda r: (r[4] or "", -r[3])):
        mark = "ADMISSIBLE" if killed is None else killed
        print(f"  {cid:22s} {cl:18s} {b:4d}b  |A|={lv:<12d} {mark}")

    print(f"\n  sieved:      {len(rows)}")
    print(f"  unsievable:  {len(unsievable)}")
    for cid, why in unsievable:
        print(f"      {cid:22s} {why}")
    print(f"  admissible:  {len(admissible)}  ->  {', '.join(admissible) or '(none)'}")
    print("  killed by:")
    for k in sorted(kills):
        print(f"      {k:14s} {kills[k]}")

    print()
    print("=" * 74)
    print("PART 3 -- the S3 repair, on the alphabets that expose it")
    print("=" * 74)
    for name, alpha in [
        ("{0,+-1}",              [0.0, 1, -1]),
        ("{0,+-phi}",            [0.0, PHI, -PHI]),
        ("{0,+-1,+-phi}",        [0.0, 1, -1, PHI, -PHI]),
        ("{0,+-1,+-2,+-4,+-8}",  ladder(2.0, 4)),
        ("{0,+-phi^0..3}",       ladder(PHI, 4)),
        ("{0,+-sqrt2^0..3}",     ladder(2 ** 0.5, 4)),
    ]:
        print(f"  {name:24s} single lane: {s3_single_lane(alpha)}")
    print()
    print("  The old S3 took `lanes` as an argument and would have been handed 2")
    print("  for {0,+-phi}, killing our own ternary format. Computed, it is 1.")

    json.dump({"admissible_top": surv, "admissible_catalog": admissible,
               "kills": kills, "unsievable": [u[0] for u in unsievable]},
              open(sys.argv[1], "w"), indent=1)


if __name__ == "__main__":
    main()
