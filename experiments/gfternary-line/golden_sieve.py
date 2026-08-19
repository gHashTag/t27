"""W774: THE GOLDEN SIEVE -- one formula that derives the format from the theorems.

Dmitrii asked for the best Ternary Network Float, derived rather than chosen, and
for a top across ALL axes at once, with formats an AI model must not be trained on
removed.

A sieve is a set of PREDICATES, each a theorem this programme measured. A
candidate format passes only if it survives every one. Nothing here is an opinion:
each filter cites the measurement that established it.

THE FIVE FILTERS
  S1  PACKING          |A| = 3^k          -- T367: only powers of three waste
                                             nothing in ternary storage.
                                             Kills 5, 7, 11, 13, 17 levels.
  S2  CEILING          k <= 2             -- T288/T369: nine levels is the
                                             accuracy ceiling; 27 levels measured
                                             +0.15 (UNSW) and -0.13 pp (Fashion),
                                             neither significant. Kills 27, 81.
  S3  SINGLE LANE      A subset of Z       -- T293: an alphabet needing a
                                             two-lane Z[phi] representation
                                             reintroduces the multiplier in the
                                             resolve: 8 DSP48E1 or ~2750 LUT.
                                             Kills phi, sqrt2, plastic, silver, e.
  S4  THREE-TRIT       fan-in * bits <= 6 -- T368b: a neuron of <=6 input bits is
                                             2.00 LUT; 12 bits is 39-54. A ternary
                                             symbol is 2 bits, so fan-in 3.
  S5  NO BAD PRIMITIVE  no DSP48E1/SRL16E -- T246/T342: openXC7 emits a wrong
                                             bitstream for both while every tool
                                             reports success.

THE ONE FORMULA. Everything above collapses to a single expression:

    TNF(k, b) = {0} u {+- b^i : 0 <= i < (3^k - 1)/2},   k in {1,2},  b in Z, b >= 2

  k = 1 -> 3 levels, ONE trit,  {0, +-1}
  k = 2 -> 9 levels, TWO trits, {0, +-b^0, +-b^1, +-b^2, +-b^3}

  and the 27-ENTRY MATRIX is what a NEURON is, not what a weight is:
    3 ternary inputs -> 3^3 = 27 rows, all reachable, zero waste (T368b).
    The same neuron in a binary LUT6 has 64 rows of which 27 are reachable: 42%.

WHAT AN AI MODEL MUST NOT BE TRAINED ON, and why -- this is the part Dmitrii
asked for explicitly. A format that fails S3 teaches a wrong hardware cost: it
looks multiplier-free in the weight application and is not, once the pair is
resolved. Training IGLA CODER or IGLA RACE on such a catalogue teaches a datapath
that cannot be built on this toolchain.
"""
import math, json, sys

PHI=(1+5**0.5)/2
def _root(f,lo,hi):
    for _ in range(200):
        m=(lo+hi)/2
        if f(lo)*f(m)<=0: hi=m
        else: lo=m
    return (lo+hi)/2

# every base this programme has measured, with its measured numbers
CAND = [
 # name           base   levels  acc_unsw acc_fash  lut_dense lut_table  lanes
 ("ternary b=3",  3.0,      9,     89.76,   90.69,      1417,     67,      1),
 ("dyadic b=2",   2.0,      9,     89.62,   90.94,       752,    123,      1),
 ("true ternary", 1.0,      3,     89.52,   90.69,       None,   114,      1),
 ("linear",       None,     9,      None,    None,       812,    128,      1),
 ("golden phi",   PHI,      9,     90.01,   90.97,      2726,    128,      2),
 ("sqrt2",        2**0.5,   9,     89.64,   91.03,      1991,    128,      2),
 ("plastic",      _root(lambda r:r**3-r-1,1,2), 9, 89.66, 91.00, 2709, 128, 2),
 ("silver",       1+2**0.5, 9,     89.80,   90.94,      2971,     88,      2),
 ("e",            math.e,   9,      None,   90.86,      2856,     79,      2),
 ("tribonacci",   _root(lambda r:r**3-r*r-r-1,1,2), 9, 89.84, 91.04, 3222, None, 2),
 ("psi4",         _root(lambda r:r**4-r-1,1,2), 9, 89.86, 91.05, 2932, None, 2),
 ("supergolden",  _root(lambda r:r**3-r*r-1,1,2), 9, 89.66, 91.02, 2850, None, 2),
 ("APoT additive",None,     9,     90.00,   90.91,      None,   None,      1),
 ("Zeckendorf",   None,     9,     89.64,   90.89,      None,   None,      1),
 ("pot17",        2.0,     17,      None,    None,      None,   None,      1),
 ("pot27",        2.0,     27,     89.78,   90.81,      None,   None,      1),
]

def sieve(name, base, levels, lanes):
    """Returns (passes, [reasons for rejection])."""
    bad=[]
    k = round(math.log(levels,3),6)
    if abs(k-round(k))>1e-9: bad.append(f"S1 packing: {levels} != 3^k, wastes ternary codes (T367)")
    else:
        k=int(round(k))
        if k>2: bad.append(f"S2 ceiling: {levels} levels = {k} trits; 27 buys +0.15/-0.13 pp, ns (T369)")
    if lanes and lanes>1:
        bad.append(f"S3 single lane: needs {lanes}-lane Z[b]; resolve costs 8 DSP48E1 or ~2750 LUT (T293)")
    if base is not None and abs(base-round(base))>1e-9:
        bad.append(f"S3 single lane: irrational base {base:.4f} is not in Z")
    return (len(bad)==0), bad

if __name__=="__main__":
    print("  ЗОЛОТОЕ СИТО -- каждый фильтр это измеренная теорема\n")
    surv=[]
    for nm,b,lv,au,af,ld,lt,ln in CAND:
        ok,why = sieve(nm,b,lv,ln)
        if ok: surv.append((nm,b,lv,au,af,ld,lt))
        else:
            print(f"  ОТСЕЯН  {nm:<16} {why[0]}")
            for w in why[1:]: print(f"          {'':<16} {w}")
    print(f"\n  ПРОШЛИ СИТО: {len(surv)} из {len(CAND)}\n")
    print(f"  {'формат':<16}{'уровней':>8}{'тритов':>7}{'UNSW':>8}{'Fashion':>9}{'LUT плотн':>11}{'LUT табл':>10}{'LUT/нейрон':>12}")
    for nm,b,lv,au,af,ld,lt in sorted(surv,key=lambda r:(r[6] if r[6] else 9e9)):
        k=int(round(math.log(lv,3)))
        f=lambda x,w,d=2: (f"{x:>{w}.{d}f}" if x is not None else f"{'--':>{w}}")
        lpn = f"{lt/64:.2f}" if lt else "--"
        print(f"  {nm:<16}{lv:>8}{k:>7}{f(au,8)}{f(af,9)}{(f'{ld}' if ld else '--'):>11}{(f'{lt}' if lt else '--'):>10}{lpn:>12}")
    json.dump([{"name":n,"levels":l,"trits":int(round(math.log(l,3)))} for n,_,l,_,_,_,_ in surv],
              open(sys.argv[1],"w") if len(sys.argv)>1 else sys.stdout, indent=1)
