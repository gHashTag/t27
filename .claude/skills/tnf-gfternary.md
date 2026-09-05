---
name: tnf-gfternary
description: The TNF (Ternary Network Float) and GA-T number formats — their definitions, theorems, silicon costs, retracted claims, and what this means for IGLA CODER / IGLA RACE. Load before touching any numeric format, accumulator width, or weight alphabet in t27.
---

# TNF & GA-T — working knowledge

## THE GOLDEN SIEVE — the format derived from the theorems

**SIX** filters, each a measured theorem. A candidate passes only if it survives
every one. **Ten of sixteen measured candidates are removed; eight fall to S3
alone. S6 removes the remaining ladder family entirely.**

| filter | rule | theorem |
|---|---|---|
| **S1 packing** | `|A| = 3^k` — powers of three waste nothing in trits | T367 |
| **S2 ceiling** | `k ≤ 2` — 27 levels measured +0.15 / −0.13 pp, ns | T288/T369 |
| **S3 single lane** | **commensurable**: all ratios rational, so one accumulator. A common scale factors out, so `{0,±φ}` passes and `{0,±1,±φ}` does not | T293/T406 |
| **S4 three-trit** | fan-in × bits ≤ 6 — 2.00 LUT/neuron, else 39–54 | T368b |
| **S5 no bad primitive** | no DSP48E1 / SRL16E — wrong bitstream, correct netlist | T246/T342 |
| **S6 non-dominance** | top magnitude ≤ sum of the others, else the neuron reads one input | **T408/T409** |

**THE SIEVE IS TWO FILTERS WEARING SIX (T441).** Over the 16-candidate top, only
**S3** (5 unique kills) and **S6** (3 unique kills) remove anything no other filter
removes. **S1 and S2 are strictly subsumed by S6**; **S4 and S5 never fire**,
because every candidate is evaluated at fan-in 3 / two bits / no DSP — they are
held constant, not tested.

**⚠ CARDINALITY IS FREE IN A TABLE DATAPATH (T448).** A truth-table neuron is
enumerated over `2^(fanin × bits)` codes — **64 rows whatever the alphabet.** The
alphabet decides *which output each row gets*, not how many rows exist.
Post-route, `L=4`, `H=16`: **ternary `{0,±1}` costs 137 SLICE_LUTX, dyadic 9 costs
137, linear 9 costs 128.** Three levels are **not cheaper**. Cardinality costs
area only in an **adder** datapath (T398: 203 vs 103).

**⚠ AND THE ACCURACY COST OF THREE LEVELS IS 0.27 pp, NOT 0.84 (T447).** T286's
`+0.844` was measured on `train_ladder.py`, which has **no normalisation**;
re-measured on a normalised stand the 3→9 gain is **+0.26 (UNSW) / +0.28
(Fashion)** — the fixed threshold was penalising the *narrow* alphabet, which
gained +0.90 from normalisation against nine levels' +0.21. **Alphabet SIZE drops
from +0.844 to +0.27, beside alphabet SHAPE (+0.149) rather than tenfold above
it.** T288's Nine-Rung ceiling holds on Fashion and **breaks on UNSW at 13
levels** (+0.14 pp) — **downgraded from a law to a measurement.**

**SO THERE IS NO TRADE IN A TABLE DATAPATH: three levels cost 0.27 pp and save
nothing. Take the nine.**

**JUNTA DEGREE DOES NOT CROSS CARDINALITIES (T449).** Ternary's junta is **1.778**,
*below* dyadic's 2.189, yet its area is equal — because `{0,±1}` puts 1/3 of its
mass on zero against nine levels' 1/9, and a zero weight removes a *wire* while
domination removes a *distinction*. T410's `r = +0.991` was over five nine-level
alphabets: **it holds at fixed cardinality only.**

**THE OPTIMUM IS ENUMERATED, NOT PICKED (T444).** All **1156** admissible
nine-level integer alphabets `{0,±1,±b,±c,±d}` with `d ≤ 1+b+c`, `d ≤ 24` were
enumerated over all 729 weight triples. **`linear 9 = {0,±1,±2,±3,±4}` is rank 1
of 1156** at junta degree **2.551**, runner-up `{1,3,4,5}` at 2.519. The bound is
**not load-bearing** — maximum junta degree is non-increasing in spread and
plateaus at 2.453 beyond `d/a = 8` — so the result is global. **Dyadic and base 3
are not ranked low here; they are not in the space at all**, failing S6.

**AND THE FORMULA RETURNS BALANCED TERNARY (T442).** For
`A = {0} ∪ {±bⁱ : i < k}`, `b ∈ ℤ`, `b ≥ 2`, `k ≥ 2`:

    top = b^(k−1),   rest = (b^(k−1) − 1)/(b − 1) ≤ b^(k−1) − 1 < top

so **S6 fails for every integer ladder of two or more magnitudes, at every size.**
Exhaustive over `b ∈ [2,12]`, `k ∈ [2,12]`: zero counterexamples; at `b = 2` the
margin is exactly **+1** for every `k` and never closes. **`TNF(k,b)` has exactly
one admissible shape — one magnitude, `{0, ±c}`, three levels, one trit.** The
escape is not a different base: it is a **non-ladder integer alphabet**, and
`linear 9 = {0,±1,±2,±3,±4}` is the measured example (top 4 < rest 6).

**S3 IS COMPUTED, NOT SUPPLIED (T406).** It used to take `lanes` as an argument —
the answer typed in by hand, and typed wrong it kills our own format.

**THE ONE FORMULA, AND ITS REFUTATION:**

    TNF(k, b) = {0} ∪ { ±b^i : 0 ≤ i < (3^k − 1)/2 },   k ∈ {1,2},  b ∈ ℤ, b ≥ 2

- `k = 1` → **3 levels, one trit** — `{0, ±1}`
- `k = 2` → **9 levels, two trits** — `{0, ±b⁰, ±b¹, ±b², ±b³}`

**T409: no integer base clears S6 at `k = 2`.** For the ladder `{b⁰..b³}` the top
weight exceeds the sum of the others exactly when `b³ > b² + b + 1`, whose root is
the **tribonacci constant 1.8392867552** — and S3 demands `b ∈ ℤ, b ≥ 2`, so
`8 > 7` already. **The ladder was never the requirement; integrality was.** The
formula must widen to any integer `A` with `|A| = 3^k` whose largest magnitude
does not exceed the sum of the others: linear 9 at `4 < 6`, fib at `3 < 4`.

**EFFECTIVE FAN-IN — enumerated over all 9³ = 729 weight triples, exhaustive:**

| alphabet | top : rest | mean eff. fan-in | LUT (L=8) | Fashion Δ | UNSW Δ |
|---|---|---:|---:|---:|---:|
| **linear 9** `{0,±1,±2,±3,±4}` | 4 < 6 | **2.55** | 271 | — | — |
| fib `{0,±1,±1,±2,±3}` | 3 < 4 | 2.52 | 246 | −0.03 ns | −0.43 ns |
| dyadic `b=2` | 8 > 7 | 2.19 | 209 | **−0.52** | −0.45 ns |
| base 3 | 27 > 13 | 1.49 | 116 | **−1.55** | −0.92 ns |
| base 4 | 64 > 21 | **1.03** | **45** | **−2.93** | −1.60 ns |

**Effective fan-in predicts LUT at r = +0.991 and accuracy at r = +0.956 (UNSW) /
+0.987 (Fashion).** So the alphabet's skew is ONE knob moving both: **6× area for
under 3 pp.** That is a trade to make deliberately, not a ranking.

**WHAT THIS CORRECTED (T410, T413a).** "Base 3 is 46% cheaper in table layers
because a faster-growing base compresses better" — **withdrawn**. It does not
compress; it lowers effective fan-in from 2.19 to 1.49. And the follow-on claim
that base 4 is *Pareto-dominant on UNSW* — **also withdrawn**: that was a stand
with no normalisation, which flipped the correlation from −0.971 to +0.956.

**THE BASE TOP CARRIES SIGNIFICANCE NOW, OR IT IS NOT A TOP (T403).** W763's
eleven bases, re-tested paired on the saved per-seed data, n = 5:

- **UNSW: 0 of 11 differ significantly from dyadic.** φ's +0.39 pp is `t = +2.27`, ns.
- **Fashion: 2 of 11** — √2 at +0.09 pp (significant only because its sd is 0.05)
  and `b = 1.0` at −0.25 pp, the degenerate three-level control.
- **The ordering printed in T365 and T395 is withdrawn.** The rows stay as
  measurements. **Accuracy does not choose the base; area does.**

**THE 27-MATRIX IS THE NEURON, NOT THE WEIGHT**: three ternary inputs → `3³ = 27`
reachable rows, zero waste. The same neuron in a binary LUT6 has 64 rows of which
27 are reachable — **42% used, 58% lost to the substrate.**

**THE CATALOGUE THROUGH THE SIEVE (T405, run at W778 over the 83 formats then
registered; the catalogue is 109 at v3, Sep 2026, and the 26 rows added since are
not covered by this run): 1 admissible.** 12 unsievable
(no width, or decimal), 71 sieved, **70 killed by S1 alone** — every one is a
power-of-two code space and `2^n` is a power of three for no `n ≥ 1`. The
survivor is `gfternary`. **This is a category result before a quality one**: the
catalogue answers the *accumulator* question, the sieve asks the *weight*
question, and `int8` being cut says nothing against `int8`. Nearest miss: `gf4` /
`mxgf4`, span `[8,16]`, the only catalogued formats that *could* hold nine levels
under a different convention — they hold seven.

**THE FAN-IN LAW (T482) — predictive, validated out of sample.** Mean per-feature
mutual information predicts the fan-in 3→6 gain **before training**:

| dataset | mean MI | gain | |
|---|---:|---:|---|
| `0v1` | 0.0823 | **+0.14** | **predicted** |
| Fashion | 0.0561 | +0.91 | fitted |
| UNSW | 0.0269 | +1.73 | fitted |
| `4v9` | 0.0096 | **+2.77** | **predicted** |
| MNIST | 0.0058 | +4.51 | fitted |

Monotone across five datasets, extremes predicted in advance. **Compute mean MI
first; it tells you whether fan-in 6 is worth 3.75× the area (T454) on your task.**
Functional form unmeasured — ordering only.

**WHAT SURVIVES THREE TASKS (T480).** Measured on UNSW-NB15, Fashion-MNIST and
MNIST, 8 seeds, shuffled splits, sparse fan-in networks:

| intervention | UNSW | Fashion | MNIST | |
|---|---:|---:|---:|---|
| **normalisation (per-layer BN)** | +17.85 | +6.36 | +20.48 | **sign ×3, significant ×3** |
| **fan-in 6 over fan-in 3** | +1.73 | +0.91 | +4.51 | **sign ×3, significant ×3** |
| **ternary activations** | −2.37 | −0.35 | −1.83 | **costs, on all three** |
| balanced coverage | −0.77 | +0.54 | +0.01 | **task-specific — do not claim** |
| depth `L=5` | −0.27 | +0.46 | +2.80 | **task-specific — do not claim** |

**Only the first three may be stated as properties.** Never pool them: the same
intervention differs fourfold between tasks, and UNSW is the harder one.

**Fan-in matters most where inputs are spatially related** (+4.51 MNIST, +0.91
Fashion) — a mechanism S4's derivation does not contain, and the strongest
evidence that the six-bit rule costs accuracy.

**CALIBRATION AGAINST THE FIELD — read this before quoting any gap (W779).**

| source | task | sparse | dense | gap |
|---|---|---:|---:|---:|
| SparseLUT Tab. IV, fan-in 6, **random** mask | MNIST HDR(D=1) | 93.76 % | 98.55 % | **4.79 pp** |
| SparseLUT, **learned** connectivity | MNIST HDR(D=1) | 95.89 % | 98.55 % | 2.66 pp |
| NeuraLUT-Assemble | NID | — | — | **≤ 1 pp**, NID **+0.5** |
| PolyLUT-Add / PolyLUT / LogicNets | **UNSW-NB15** | **92.0 / 92.2 / 91.0 %** | — | — |
| **this project, best** | UNSW-NB15 | **80.23 %** | 89.62 % | **9.39 pp** |

**The field reaches 91–93 % where we reach 80.** Our stand's gap is ~2× the
comparable random-mask figure. The baselines batch-normalise **every layer's
inputs and outputs** and use **quantised activations with learned scaling
factors** (Brevitas); we do the first now (+2.56 pp, T422) and **not the second**.

**PRIOR ART — cite these, do not re-derive them.**

- **S4 (six-bit rule)** is **LogicNets**, arXiv:2004.03021. Their own NID configs
  run at **14 input bits**. Six is *our* choice, not a law.
- **S6's mechanism** is the **critical-index / head-tail decomposition of linear
  threshold functions**, Servedio, *Comput. Complexity* 2007, arXiv:0902.3757.
- **The names**: a neuron reading one of three inputs is a **1-junta** or
  **dictator**; the neuron is an **LTF**. O'Donnell, *Analysis of Boolean
  Functions*. **Say "junta degree", not "effective fan-in"** — the latter is
  ODIN's (arXiv:1804.07858) for accumulator depth.
- **The tribonacci constant** is **OEIS A058265** by definition.
- **"PoT has rigid resolution"** is **APoT**, arXiv:1909.13144, verbatim. Our
  mechanism differs (their argument is per-weight projection error; ours is
  functional dependence) and that difference is machine-verified: their full text
  contains fan-in 0, junta 0, dominat\* 0, "truth table" 0, Boolean 0.
- **The threshold/normalisation identity**: **FINN** §4.2.2 is titled
  *"Batchnorm-activation as Threshold"*; **TWN** arXiv:1605.04711 gives the dead
  zone as `Δ* ≈ 0.75·E|W|`; **Sari et al.** arXiv:1909.09139 quantify BN removal
  at **88.8 → 31.7**. **On hardware, "remove the fixed threshold" and "add
  normalisation" are one object.**
- **Functional input count is already priced**: **Logic Shrinkage**,
  10.1145/3583075, learns per-LUT input counts for **1.54× area**.

**PRIOR ART ADDED W805 — the ternary-FPGA field itself, which this list omitted
for four months (T491a). Every citation above is a *quantisation* paper; none of
them is a ternary accelerator. Search the field the artefact is IN.**

- **THE ZERO-DSP RESULT IS NO LONGER OURS ALONE.** **ELiTeFormer**,
  arXiv:2607.03652 (2026-07): a PE that *"eliminates all multiplications in
  ternary linear projections through bitmasking operations … completely avoiding
  dedicated DSP blocks."* **Cite it.** Claiming zero-DSP as novel after 2026-07
  is a provenance error. And state the other half (T487): zero DSP is true of the
  ternary **core** and false of a full Q2_0 pipeline, whose one FP16 scale per
  128 weights costs ~1 DSP48E1 of 740.
- **A DIRECT COMPETITOR TO THE GOLDEN SIEVE**: arXiv:2604.25183 (2026-04),
  *Hardware Generation and Exploration of LUT-Based Accelerators for 1.58-bit LLM
  Inference* — formalises the ternary-LUT design space with an analytical cost
  model **and an open-source generator**, ASIC-validated. Its finding cuts
  against our regime: *"LUT-based reuse offers significant gains for high-cost
  arithmetic (e.g. FP16), it yields diminishing returns for small integer
  types"* — and our activations are ternary. Our counter (their LUT is an ASIC
  precompute table; ours is a fabricated, idle LUT6) is **unverified — the paper
  is unread**. **The sieve may not be called novel until this is answered (T491).**
- **The ternary-LLM-on-FPGA baseline the article does not acknowledge**:
  **TerEffic** arXiv:2502.16473 (370 M fully on-chip, multi-FPGA, 16,300 tok/s;
  2.7 B with HBM at 727 tok/s / 46 W); **TeLLMe** arXiv:2504.16266 (AMD KV260,
  9 tok/s @ 1024 ctx, 7 W) and **v2** arXiv:2510.15926 (25 tok/s, 5 W);
  **TENET** arXiv:2509.13765 (sparsity-aware LUT-centric, 4.3× energy vs A100).
- **Scale honesty (T488/T489)**: three XC7A200T dice hold **25.5 M** ternary
  parameters on-chip (5.05 MB of BRAM at 1.58 b/w). TerEffic's 370 M needs 43
  dice of this part. **No published ternary LLM accelerator targets an Artix-7** —
  every one uses HBM or a Zynq UltraScale+ with a hardened DDR controller.

**THE NUMBERS TO QUOTE (T455), slope with bootstrap 95 % CI, not `r`:**

| relation | n | slope | 95 % CI |
|---|---:|---:|---|
| junta → LUT, **L=8** | 5 | **+151 LUT/junta** | [+139, +189] |
| junta → LUT, **L=4** | 5 | +31 LUT/junta | **[−25, +46] — includes 0** |
| junta → accuracy, UNSW | 7 | **+1.0 pp/junta** | [+0.1, +1.3] |
| junta → accuracy, Fashion | 7 | **+1.9 pp/junta** | [+1.1, +2.1] |

**The area relation is CONDITIONAL ON DEPTH** — it separates at `L=8` (LUT range
6×) and not at `L=4` (range 1.6×). **The accuracy relation is unconditional on
both tasks.** Lead with accuracy; quote area with its depth attached.

**DO NOT REPORT `r` FOR THESE SWEEPS.** Every correlation in the alphabet line is
over 5–7 arms *constructed* to vary monotonically in the predictor, with no
confidence interval. **Report the pairs and the slope.** T418 showed the failure
from inside: Pearson `+0.916` against Spearman `+0.607` on the same seven points.

**AND THE NULL IS A LIMIT, NOT A LAW.** The defensible sentence is *"on two tasks,
at this scale, with five seeds we could not resolve alphabet shape at fixed
cardinality"* — **not** *"shape is worth ≤ 0.25 pp"*. Counter-evidence exists:
**AdaMX** (arXiv:2608.03867) removes **83 %** of MXFP4's loss by adapting the
element representation; **GSQ** (arXiv:2604.18556) works at **3–8 levels**, our
exact regime.

**NEVER PUT THESE IN A TRAINING CATALOGUE:** φ, √2, plastic, silver, e,
tribonacci, ψ₄, supergolden. All fail S3, and all **teach a false hardware
cost** — multiplier-free in the weight application, not multiplier-free once the
`a + b·φ` pair is resolved. A model trained on them learns a datapath that cannot
be built. They belong in the *history*, with their refutations attached, never in
the *catalogue*.

Runnable: `experiments/gfternary-line/golden_sieve.py`,
`sieve_catalog.py` (run at W778 over the then-83-format catalogue; re-run against
the current 109), `gen_mix.py` (effective fan-in + area),
`fanin_accuracy.py` (accuracy, with the `scale` control).
Spec: `specs/numeric/golden_sieve.t27` — 3 tests, 7 invariants proved comptime.

---

## MEASURE IN TRITS — Dmitrii's standing correction

**Every LUT figure in this repository is in the wrong units for the target.** A
LUT is a *binary* primitive of the FPGA we happen to own. The line targets
ternary silicon, and in its own units the picture changes. Restate before
quoting.

**Radix economy** [classical — Knuth, *TAOCP* v2]. Cost of representing `N`
values in base `b` is `E = b·log_b(N)`. At `N = 10⁶`: base 2 → 39.86, **base 3 →
37.73**, base 4 → 39.86, `e` → 37.55. **Base 3 is the nearest integer to `e` and
the most economical integer radix, beating binary by 5.7%.** Not our discovery —
but it is our substrate, and no measurement here had been stated against it.

**Alphabet packing.** Sizes that waste nothing in ternary storage are exactly the
**powers of three**:

| levels | bits | waste | **trits** | **waste** |
|---:|---:|---:|---:|---:|
| **3** | 2 | 21% | **1** | **0%** |
| 5 | 3 | 41% | 2 | 44% |
| 7 | 3 | 14% | 2 | 22% |
| **9** | 4 | **21%** | **2** | **0%** |
| 27 | 5 | 5% | **3** | **0%** |

**The Nine-Rung ceiling (T288) is 9 = 3² = exactly two trits.** The two sizes the
programme singled out by measurement — the minimum (3) and the ceiling (9) — are
**both perfect in trits and both wasteful in bits.**

**THE SIX-BIT RULE IS THE THREE-TRIT RULE.** T331 measured 2.00 LUT/neuron at ≤6
input bits. A ternary symbol is 2 bits, so **fan-in 3 = 6 bits = exactly 3
trits**, and a natural three-trit table is **3³ = 27 entries** — held in a binary
LUT6 of 64. **42% used; 58% of every LUT in the datapath is wasted by the
substrate, not by the design.**

**Storage tax:** 21% at both 3 and 9 levels. On the trained 593→16→16→1 network
(9,760 weights): **39,040 bits binary vs 19,520 trits = 30,938 bit-equivalents.**

**Is three levels a strong position? YES — say so plainly.** `{0,±1}` is **one
trit**, the founding premise, and it scores within **0.49 pp** of a nine-level
golden alphabet on UNSW (dense, 5 seeds) and **+0.735 pp** pooled over three
tasks (T317). **Half the weight storage for half to three-quarters of a point.**
In the table architecture it costs **114 LUT against 123–128** for the nine-level
arms. This was under-reported in W763 as "the interesting half of a refutation";
it is a result.

**In the TABLE architecture the base ordering INVERTS** (T366): base 3 costs
**1.05 LUT/neuron**, dyadic 1.92, φ 2.00 — the ternary base is **cheapest**,
because fast-growing bases skew the table so one output symbol dominates and
`yosys` folds the rest into `default`. In a dense adder tree the ordering is the
opposite (dyadic 752 LUT, φ 2,726).

**THE BOUNDARY, never cross it.** All of the above is an **efficiency** case:
radix economy, packing, storage, table skew. **It says nothing about accuracy**,
which varies by 0.49 pp across every base measured, and nothing about the
**10.6-point gap** to TreeLUT. **The ternary case is real and it is not an
accuracy case.**

---

## THE ANSWER, if you were asked "which format for ternary on FPGA?"

Dmitrii has asked this four times. Answer it directly, from these measurements,
without re-deriving it.

> **Weights: a TRUTH TABLE, not a format, at ≤6 input bits per neuron.
> Where a table will not fit: `{0, ±1, ±2, ±4, ±8}`. Accumulator: int12–int16.
> φ does not belong in the datapath.**

**Why "not a format" is the honest answer.** A neuron reading ≤6 input bits is
one LUT6 per output bit — **2.00 LUT/neuron, measured** — and contains **no
arithmetic at all**: no multiplier, no adder, no weight memory. The format
question dissolves because there is nothing left to compute with. Dense 593→64
costs **54,914 LUT**; the same layer as tables costs **128**. That is **429×**,
and 3.6× the Fmax (204 vs 57 MHz).

**THE SIX-BIT RULE — the actual design law.** Cost is set by TOTAL BITS READ:

| input | bits each | fan-in | cost |
|---|---:|---:|---:|
| binary | 1 | **6** | **2.00 LUT/neuron** |
| **ternary** | **2** | **3** | **2.00 LUT/neuron** |
| ternary | 2 | 6 (=12 bits) | **39–54 LUT/neuron** |

A ternary symbol is **two bits**, so hidden layers take fan-in **3**. Violating
this costs 20×, and it nearly shipped once: a depth sweep with fan-in 6
everywhere cost **10,250 LUT** where its headline implied 800.

**When arithmetic is unavoidable** (wide fan-in, final accumulator):
`{0, ±1, ±2, ±4, ±8}` — this is **power-of-two (PoT) quantisation** in the
literature (Li, Dong & Wang, ICLR 2020, arXiv:1909.13144; priority Zhou et al.,
ICLR 2017, arXiv:1702.03044, Eq. 1). **`pot9` is this repo's internal tag, never
a format name — do not print it in prose.** Zero DSP at every rung; nine levels
is the ceiling (nothing above nine was significant on any of eight tasks).

**Why not φ, in one table:**

| | measured |
|---|---:|
| alphabet **size**, 3→9 levels | **+0.735 pp** |
| alphabet **shape** at fixed size | +0.149 pp, significant on 1 of 3 tasks |
| **resolving `a + b·φ` against a threshold** | **8 DSP48E1**, or **~2750 LUT** without them |

**The multiplier φ removes from weight application returns in the pair resolve.**
The algebra stands and is Coq-checked — `Z[φ]` is closed, and degree 2 admits φ
alone as a multiplier-free scale. The practical advantage does not.

**The ranking that makes the format question secondary:**

| intervention | effect on accuracy |
|---|---:|
| inter-layer normalisation | **+29.15 pp** |
| training budget | +3.30 pp |
| class-balanced loss | +3.19 pp |
| depth under the six-bit rule | +2.51 pp |
| **alphabet size** | **+0.735 pp** |
| **alphabet shape** | +0.149 pp |
| activation (field's STE vs our tanh) | −0.77 pp |

**The alphabet decides AREA, not accuracy.** That is why the answer is "a table,
not a format".

**Before choosing a task, compute `mi_tot`** — the sum of per-feature mutual
information, one pass over the data. It predicts what the sparse datapath will
cost you: **r = −0.88 within a dataset (45 labellings), −0.68 across three
datasets.** MNIST-bin has the least (3.14) and the worst penalty (+14.85); the
0-vs-1 pair has the most (44.75) and the least (+0.28).

**Toolchain, non-negotiable:** `synth_xilinx -nodsp -nosrl`. openXC7 emits a
**wrong bitstream from a correct netlist** for both `DSP48E1` (live operand) and
`SRL16E`, and both pass the `0→1` acceptance criterion while computing the wrong
answer. `t27c yostat` exits 2 on either. See t27#2173.

---


**Source of truth:** [`docs/theory/TNF_ARTICLE_RU.md`](../../docs/theory/TNF_ARTICLE_RU.md)
(Russian; 3,011 lines at 2026-09-05 by `wc -l`). The English TNF manuscript,
"Ternary Network Floats", was submitted 3 Sep 2026 to Microprocessors and
Microsystems (Elsevier), manuscript MICPRO-D-26-00839, status under review; no
arXiv preprint exists. Revision R1 is prepared in gHashTag/trinity-fpga#742
(open). This skill is the distillation. When they disagree, the
article wins — but re-read the article's own **Ограничения** section first,
because most apparent disagreements are claims the article already retracted.

---

## 1. The one-paragraph version

A ternary node has **three** places a number format could live, and needs one in
exactly **one** of them:

| place | what it needs | why |
|---|---|---|
| **weight** | an *alphabet*, not a float | multiplying by it is a **sign choice** (`+x`, `−x`, `0`) |
| **activation** | nothing | arrives in the ADC's native form and is used as it arrives |
| **accumulator** | a **float** | the only object carrying dynamic range that must be spent |

So the design problem is not "which format" but **which two objects to close**:

- **weight → GA-T**, the 2-bit alphabet `{−φ, 0, +φ}`
- **accumulator → TNF**, `sign · (1 + M/2^(M−2)) · 2^e` with `e` a balanced-ternary exponent

These were derived independently and turned out complementary. Published ternary
methods answer (1) and leave (3) in fp16/int8; published format work answers (3)
and assumes a multiplier in (1).

---

## 2. TNF — the format

```
TNF16 = [ s | E = 4 balanced-ternary trits | M = 11 bits ]
v = (−1)^s · (1 + M/2^9) · 2^e ,   e = Σ tᵢ·3^i ∈ [−40, 40]
1 + E + M = N        ← "the width rule"
```

Three properties follow from the **layout**, not from tuning:

1. **No mode decoding.** The dominant cost in a tapered format (posit, takum) is
   the variable-length regime field: find it, count its length, barrel-shift the
   significand into place. Fixed fields turn that into a **bit slice**.
2. **The exponent is already ternary.** 4 balanced trits = 3⁴ = 81 exponent
   steps; on a ternary fabric exponent addition is native.
3. **Precision does not taper.** 9 significand bits at every magnitude.

### The ladder

| rung | Et | M | note |
|---|---:|---:|---|
| TNF4 | 2 | 1 | |
| TNF8 | 3 | 4 | but the *measured* 8-bit winner is Et=2, M=5 |
| TNF16 | 4 | 9 | the working rung |
| TNF32 | 6 | 25 | **the only rung where GF-T and TNF disagree** |
| TNF64 | 7 | 52 | 7,479 LUT @ 48.20 MHz on XC7A200T |
| TNF128 | 8 | 115 | does not route on Artix-7 |
| TNF256+ | | | exceeds one Artix-7 fabric entirely |

**5 of 9 rungs were measured in hardware.** The spec table and the placed-and-routed
table are deliberately kept apart. Do not quote a spec row as a silicon row.

---

## 3. GA-T — and why φ is forced

> **Theorem (the golden alphabet is unique).** Let the weight alphabet be
> `{−r, 0, +r}`, `r > 1`, and require the product of two weights to be
> expressible in the additive lattice the datapath already sums in, i.e.
> `r² = r + 1`. Then `r = φ`, and it is the only possibility.
> *Proof.* `r² − r − 1 = 0` has one positive root. ∎

> **Theorem (multiplier-free path is exact).** `Z[φ] = {a + bφ : a,b ∈ Z}` is a
> ring and `{−φ,0,+φ} ⊂ Z[φ]`. Therefore for inputs in `Z[φ]` the **entire linear
> part** of a ternary network — every weight application, every accumulation, at
> any fan-in and any depth — stays in `Z[φ]` and is computed with **zero rounding
> error**. ∎

Represent a value as an integer pair `(a,b)` meaning `a + bφ`. Then

```
φ·(a + bφ) = a·φ + b·φ² = a·φ + b·(φ+1) = b + (a+b)·φ
```

so **applying a weight is `(a,b) ↦ (b, a+b)`** — the Fibonacci recurrence, *one
integer addition, no shift*. Negation flips both components; a zero weight is a
skip.

> **Corollary (depth costs no multiplier).** The gain of `k` stacked ternary φ
> layers is exactly `φ^k = F_k·φ + F_{k−1}` — a pair of integers. Inter-layer
> rescaling is therefore shift-and-add, and the zero-multiplier property survives
> to arbitrary depth.

**This is what separates the φ alphabet from `{−1,0,+1}`**, which the entire
ternary-network literature uses. With unit weights the layer gain is 1 and carries
no information, so every published method hangs a **learnable real scale `α_ℓ`**
on each layer — and multiplying by `α_ℓ` **puts the multiplier back**. Storage is
2 bits either way; symbol count is 3 either way. The φ alphabet simply *carries*
the scale that the unit alphabet has to *learn and then pay for*.

### Verified on the Artix-7 FPGA (XC7A200T) prototype

16,000 vectors at fan-in 8/16/32, 8-bit activations, weights drawn uniformly from
the alphabet: **zero mismatches**. Depth checked to `k=30`, where the gain is
exactly `(514229, 832040)` — two integers, no multiply anywhere on the path.
Half of the closure argument is **machine-checked in Coq**.

> **Debugging signature worth memorising.** The first run mismatched on 3,601 of
> 4,000 — but *only in component `b`*, never in `a`. "All wrong in one component"
> = a **convention** difference. "Wrong at the edges" = a missed boundary case.
> "Scattered" = arithmetic. Here the wrong encoding was in the *reference*.

### Why closure, specifically, removes the machinery

> **Corollary (closure removes the conversion stage).** If the additive group
> generated by alphabet `A` is closed under the weight-application operation `s`,
> the datapath needs only that group's addition. If it is not closed, every
> product must be re-expressed in the representable set, which needs a
> **conversion stage and a schedule that routes operands to it**.

Fibonacci integers are **not** closed: `F₄·F₄ = 9 ∉ {1,1,2,3,5,8,13,…}`. The FQP
(Fibonacci quantization processor) therefore carries *two* different arithmetic
blocks plus topological-order routing. That stage measures an order of magnitude
larger than the whole closed accumulation, and pipelining does not save it:

> **Theorem (normalisation threshold).** A normalisation cascade cannot be
> pipelined finer than one compare-and-subtract per stage, so its clock period is
> lower-bounded by that carry delay. Compare-and-subtract is strictly larger than
> addition. Hence the unclosed path's floor lies **above** the closed path's,
> whose step is a single addition.

---

## 3. CLOSED (W743): size beats shape, tenfold

**Three tasks, seven alphabets, 30 seeds each — 630 trained runs.**
Inverse-variance pooled, Cochran's Q for heterogeneity:

```
alphabet SHAPE,  7 levels   +0.085 pp   z 5.67   Q 2.2   homogeneous
alphabet SHAPE,  9 levels   +0.057 pp   z 3.46   Q 6.9   tasks DISAGREE
alphabet SIZE,  3 -> 9      +0.844 pp   z 33.9   Q 59    direction never varies
```

**Quote this.** The shape of the alphabet is worth under a tenth of a point; its
SIZE is worth ten times that. At seven levels the dyadic set is ahead on all
three tasks (the one replicated shape effect); at nine the tasks disagree.

**Engineering rule:** take the cardinality the area allows, then pick the
alphabet by COST — at fixed cardinality accuracy is nearly a wash, and on a
binary fabric the dyadic set is 27% cheaper placed (1103 vs 1509 LUT at K=7).

`{−φ, 0, +φ}` is `φ · GA-T0` and not a rung at all (T209).

---

## 3. CLOSED (W740): what the GA-T line is worth

**Trained, 30 seeds, UNSW-NB15, fixed threshold, paired t-tests:**

```
GA-T0  3 levels  82.378%        GA-T1 - GA-T0  +0.457 pp  t 14.44
GA-T1  5 levels  82.835%        GA-T2 - GA-T1  +0.242 pp  t 10.50
GA-T2  7 levels  83.077%        GA-T3 - GA-T2  +0.175 pp  t  7.28
GA-T3  9 levels  83.252%
```

**Cardinality is the whole effect.** And at EQUAL cardinality:

```
GA-T2 - pot7  -0.104 pp  t -4.85    powers of two are BETTER
GA-T3 - pot9  -0.111 pp  t -3.92    powers of two are BETTER
```

**Quote this, not the representation table.** T215–T217 measured GA-T3 at 91.82%
of the Lloyd–Max optimum against pot9's 69.85% — a 22-point gap that turns into
a tenth of a point AGAINST in trained accuracy. Representation efficiency does
not predict it.

`{−φ, 0, +φ}` is `φ · GA-T0` and not a rung at all (T209).

---

## 3a. The GA-T **line** — the rungs, and their measured price

Named W714. Do not confuse with §8: that ladder indexes *degree* (`r^d = r+1`,
which scale is multiplier-free); this one indexes *power* (which powers of the
one scale φ are in the alphabet).

> **GA-T_n = {0} ∪ {±φ^k : 0 ≤ k ≤ n}**, cardinality **2n+3**.
> Because `φ^k = F(k−1) + F(k)·φ`, weight `φ^k` on input `x` adds `F(k−1)·x` to
> lane A and `F(k)·x` to lane B of a Z[φ] pair. **Integer Fibonacci
> coefficients — no multiplier at any rung.**

| rung | alphabet | levels | bits/weight packed | measured LUT¹ |
|---|---|---:|---:|---:|
| **GA-T0** | `{0,±1}` | 3 | 1.6000 | 1692 |

> **W845 — the GA-T line above is superseded.** T234 corrected these post-route
> LUT counts by a factor of **3.00x**: 1692/1371/1878/2349/2796 become
> **564/457/626/783/932** (`docs/theory/IGLA-FORMAL-RESULTS.md`, T234). This
> file was last committed 2026-08-17, three days AFTER that correction, and
> carried the old numbers forward without mentioning it. Do not quote the
> line above.

| **GA-T1** | `{0,±1,±φ}` | 5 | 2.3333 | **1371** |
| **GA-T2** | `{0,±1,±φ,±φ²}` | 7 | 2.8182 | 1878 |
| **GA-T3** | `… ±φ³` | 9 | 3.1818 | 2349 |
| **GA-T4** | `… ±φ⁴` | 11 | 3.5000 | 2796 |

¹ yosys 0.63 `synth_xilinx -family xc7`, one layer, N=64 binary in, M=8 out,
12-bit accumulator, one shared zero mask. **DSP48 = 0 at every rung.**

**Three things this table settles, and one it kills:**

1. **The historical GA-T `{−φ,0,+φ}` is NOT a rung.** It is `φ·GA-T0`.
   That is the structural reason T158 found φ factors out — not an accident of
   the experiment.
2. **GA-T1 is 19% cheaper in LUT than plain ternary** with five levels instead
   of three: each weight lands in one lane, so two narrow trees beat one wide
   one. This is the only place in the whole programme where φ buys something a
   tool can see.
3. **The seven-level set is prior art** — exactly a 3-bit LQ-Nets (ECCV 2018)
   codebook at `v₁ = φ²/2, v₁ = v₂+v₃`. **The line is not.** Quote the line,
   never the set.
4. **What it kills:** the decision is not free. `sign(A+Bφ)` costs **105 LUT**
   by shift-add (φ≈13/8, 0.43% error) or **9 DSP48** exact, against **0 LUT —
   a wire** for a scalar accumulator (T211). Multiplication is *relocated* from
   N weights to 1 output, not removed.

**Depth (T212/T213).** Propagating pairs widens 3.3 bit/layer and never breaks
even — the break-even quadratic has **no real root**. Applying T160's
`(a,b) ↦ (b−a, a)` each layer holds width constant for a measured **288 LUT** per
8 neurons, moving break-even to **depth 25.5**. At every buildable depth the two
datapaths are within a few percent. **Plan capacity, not advantage.**

---


## 4. The precision law — the single most reusable result

> **Theorem 1 (precision law).** For a format with constant M-bit significand
> under round-to-nearest, with significand `s` and exponent `e` independent,
> `E[|rel. error|] = ½·E[1/s]·2^(−(M+1))`, **independent of `e`**.

The exponent drops out entirely — flatness is *proved*, not observed. The constant
is not universal: `½ln2 = 0.3466` for `s` uniform on `[1,2)`, `0.3607` under
Benford. Our workload has `E[1/s] = 0.7721` → predicts `0.3861`; eight measured
rungs average `0.3756`, spread `0.369–0.390`.

**Practical consequence:** given `M` and the workload's significand distribution,
**you know an unbuilt rung's error before building it.**

Inverted, the same law becomes a *diagnostic instrument*:

> **Theorem 2 (diagnostic).** `M_eff` is constant across bands **iff** the format
> holds constant significand width there and has not exhausted its range. For a
> tapering format `M_eff` decreases with `|e|`, and `dM_eff/d|e|` **is** the taper
> rate in significand bits per binade.

This resolves the format catalogue (83 formats when measured; 109 at v3, Sep
2026) into exactly **four** taper shapes — and
there is no fifth. It reads posit's taper (`k = ⌊|e|/2^es⌋ + 1`) and takum's
(`r = ⌊log₂(|e|+1)⌋`) exactly, from the encoder, not from a fit.

> **Theorem 6 (range–precision dichotomy).** Fix width `N`. If the exponent takes
> unboundedly many values, exponent codewords have unbounded length and
> `M_eff(e) → 0`. **No fixed-width format has both unbounded range and precision
> bounded below by a positive constant.**

> **Corollary 5 (why a ladder).** A format wanting both must get range by varying
> `N`. A ladder of fixed-field rungs does exactly that: each rung flat across its
> own range, the ladder covering any range. Tapering families make the opposite
> choice. **Neither is better in the abstract** — the dichotomy establishes only
> that these are the two options.

---

## 5. The four families — two axes

|  | binary exponent | balanced-ternary exponent |
|---|---|---|
| **golden-ratio rule** `E = round((N−1)/φ²)` | **GF** | **GF-T** |
| **width rule** `1 + E + M = N` | **BNF** | **TNF** |

At 8 bits, SmolLM2-135M / wikitext-2, fp32 baseline `14.4874`:

| family | values | ppl | vs fp32 |
|---|---:|---:|---:|
| GF8 (E=3) | 129 | **14.6130** | 1.009× |
| BNF8 (E=3) | 129 | **14.6130** | 1.009× |
| TNF8 (Et=2) | 73 | **14.7012** | 1.015× |
| GF-T8 (Et=3) | 109 | 15.5147 | 1.071× |

**Two rules name the same format, and neither predicted it.** GF8 and BNF8 give
the *same* number because at 8 bits both derivations pick `E=3, M=4`. That claim
is stronger than either rule alone and was not sought.

**And the width rule's prediction was falsified twice** — it named BNF8 at `E=4`
and TNF8 at `Et=3`; the measured winners were `E=3` and `Et=2`, both one step
narrower. What was wrong was the *range estimate fed into* the rule (measured
from the 0.1st percentile, which counts an energy-free tail), not the rule.

---

## 6. Why ternary — and exactly how much

> **Theorem (radix economy).** Representing `R` values in radix `b` costs
> `b·log_b R` state-cells; `b/ln b` is minimised at `b = e`. Among integers
> `3/ln3 = 2.731` vs `2/ln2 = 2.885` — ternary is **5.3% more economical**.

> **Theorem (no free range on a binary fabric).** An `Et`-trit exponent field spans
> `3^Et` values but is stored in `⌈Et·log₂3⌉` bits, a field spanning `≥ 3^Et`.
> Therefore on a binary fabric a ternary exponent **never** spans more values per
> bit than a binary one; equality only at `Et = 0`. Utilisation
> `ρ = 3^Et / 2^⌈Et·log₂3⌉ ∈ (½, 1]`, and `ρ does not converge`.

> **Theorem (scale radix — why 2 beats 3).** Relative error is `κ(r)·2^(−M)`; a
> radix-3 scale would oscillate by `log₂3 = 1.585` bits vs binary's `1`.

**Read this honestly:** the ternary *exponent* claim is **architectural**, about a
ternary fabric that cannot be bought. Every hardware number in the article was
measured on a **binary** FPGA where the ternary exponent neither wins nor loses.
The claims that survive on purchasable silicon are the **fixed-field** claims: no
mode codec, no exponent to compute. Ternary loses on a binary fabric and has now
done so in **three independent measurements** (BNF16 vs TNF16 within 1% in placed
silicon; GF8 vs GF-T8; MXFP4 vs TNF4 on the block axis).

---

## 7. Silicon — measured on **XC7A200T**, our exact part

The article's hardware campaign used the **same chip we have three of**, on a fully
open flow: **Yosys 0.65, nextpnr-xilinx 1743d0f, Icarus Verilog 13.0, Python 3.14**.

| fact | value |
|---|---|
| ternary neuron | **28 LUT per weight, ZERO DSPs at any fan-in** |
| TNF64 | 7,479 LUT @ 48.20 MHz |
| BNF | 97 LUT @ 388.35 MHz (isolated-decoder bench) |
| ~~GA-T throughput/area~~ | **RETRACTED 2026-08-13 — see §9a** |

### 9a. The throughput-per-area claim is RETRACTED (erratum, W652)

**Do not quote "+10.2% over the runner-up, 6.1× over posit32, first among 20
self-ranged formats." It is withdrawn.** Three independent problems, all
provable from the article's own table:

| claimed | table says |
|---|---|
| +10.2% over next | `0.1584/0.1429 = 1.1085` → **+10.85%** (no row yields 10.2%) |
| 6.1× over posit32 | `0.1584/0.0302 = ` **5.245×** (no row yields 6.1×) |
| 20 formats, 8 ours | **24** data rows, **12** bolded as ours |

**And the decisive one: `int8` on the same bench is 0.1736 MHz/LUT against
GA-T's 0.1584 — `int8` wins by 9.60% — and it is excluded from the
ranking.** The stated grounds for exclusion (int8 needs an external learnable
scale) concern the format's *role in a system*, not the quantity the column
measures, which is MHz per LUT.

**The metric is confounded with input width, and the article proves it before it
presents the ranking.** Its own Truncation Proposition: a decoder with `n` input
bits has at most `2^n` distinct outputs, downstream logic specialises to that
image, so "comparison across different `n` mixes format design with format width;
only comparison within one `n` isolates design." **GA-T is `n = 2`. Every
competitor is `n = 8…32`.** The article even reports GA-T decoding at 66 LUT
where a bare wire costs 112, conceding "a format cannot be cheaper than a wire."

**Say this instead** — it is a design claim, checkable within one `n`, and `int8`
does not refute it:

> Among formats of equal input width, GA-T is the only one whose lattice is
> closed under weight application, and therefore the only one whose linear path
> is exact without a normalisation stage.

**Everything in §3–§8 stands.** Nothing there rests on the ranking: the golden-
alphabet uniqueness theorem, the `Z[φ]` exactness theorem (16,000 vectors, 30
depths, zero mismatches, half machine-checked in Coq), the closure corollary, the
normalisation-threshold theorem, the precision law and its taper diagnostics, the
multiplier-free scale hierarchy, and the 28 LUT/weight zero-DSP datapath
measurement — which is a datapath measurement, not a format ranking.

**This is the eleventh retraction in this work and the first not found by its
authors.**

> **Theorem (area and precision are commensurable).** With `λ = dA/dM` the marginal
> area of a significand bit, a mode codec costing `C` LUT equals `C/λ` significand
> bits of silicon — and by the precision law, a factor `2^(C/λ)` in accuracy at
> equal area.

> **Theorem (decode cost is set by scanning, not by taper shape).** A regime whose
> codeword length is bounded by a run must scan up to `N−1` positions: `Θ(N)`. A
> regime whose length sits in a fixed `b`-bit field needs no scan: `Θ(1)` in `N`.
> Both are independent of taper shape.

> **Theorem (shifter cost).** A run-time variable shift on `W` bits costs
> `Θ(W log W)` LUT; a fixed permutation costs **zero**. Hence an n-term APoT
> applier is `Θ(nW log W)` and a Fibonacci step is `Θ(W)`.

> **Theorem (no single winner).** In (area, delay, error) no applier dominates:
> multiplier is exact and largest, `φ^k` smallest and least accurate, APoT-2
> between and dominated by neither. All three are frontier vertices.

---

## 8. The ladder hierarchy — how fine can you get without a multiplier

> **Theorem (multiplier-free scales, by degree).** A scale is applicable without a
> multiplier **iff** it is an algebraic integer whose companion matrix has entries
> in `{0, ±1}`. Degree 1 gives the shift. **Degree 2 gives φ and nothing else.**
> Degree 3 gives three more, finer.

**Between the shift and φ there is nothing.** φ is the *unique* multiplier-free
refinement of the powers-of-two ladder available with two registers. The family
`r^d = r + 1` reaches any granularity in **one addition** and `d` registers:
**fineness costs registers, not adders.**

> **Theorem (Siegel's floor bounds the family).** The smallest Pisot–Vijayaraghavan
> number is `1.3247179572…`, the root of `r³ = r + 1` (Siegel 1944). That root is
> the degree-3 member, so the family is Pisot at `d = 2` and `d = 3` and at **no
> higher degree**.

Which rung to take follows from the bit budget: **φ at 4 bits, `r⁵=r³+1` at 5,
`r⁶=r+1` at 6** — and at 6 bits with a single adder you reach **3.7% of fp32**.

> **Theorem (ladder law).** Among multiplier-free ladders at fixed code budget,
> the one minimising weight MSE also minimises perplexity — up to pairs whose
> error differs by less than the estimator resolves.

> **Theorem (curvature correction).** Minimising `Σ gᵢⱼ²·δwᵢⱼ²` predicts the
> perplexity ranking where unweighted error does **not**, on both tested models.
> The correction shifts the choice toward a **coarser** ladder.

---

## 9. Retracted claims — read these before quoting a number

The article retracts **ten** of its own claims *in place*. The four that matter most:

1. **`φ^k` grid beats powers of two — RETRACTED.** The measurement (2.44% vs 4.86%
   excess reconstruction error over 210 layers) reproduces and is correct, but it
   was compared against the wrong baseline. The deployed state of the art for
   multiplier-free scales is **APoT**, not bare powers of two. Measured the same
   way, APoT-2 is 0.1651% and APoT-3 is 0.0054%, both in one cycle against our
   `k`. **The φ grid loses by 15×.**
2. **"440 vs 895 LUT, 5.1×" — RETRACTED.** TNF was given pre-expanded fields while
   the competitors unpacked theirs. A later "3.1× / 5.6×" was also wrong — those
   TNF modules **did not implement the format they were labelled with** (six
   modules in the work turned out mislabelled). The real advantage is smaller.
3. **The block axis belongs to MXFP4.** On a short block with a shared scale, the
   best achievable 8-level codebook beats the deployed one by **less than 1%**.
   That axis is not an element-format problem at all. Stated as a competitor's
   result, with the bound that made a fourth attempt unjustified.
4. **The reference oracle carried a sign defect.** Round-tripping 1.5 returned
   −1.5 at M = 21 and 25. The ladder's check was commutativity — and **an inverted
   sign survives both sides of `a + b = b + a`**. The check was real and blind to
   the class. A round-trip claim sees it, costs one line, and now holds on all
   nine rungs.

> The article's own generalisation: **the measurement was almost always right and
> the comparison around it was wrong.** Sixteen ways that can happen are
> enumerated in its self-check section.

---

## 10. What this means for IGLA CODER / IGLA RACE

Direct, actionable consequences:

- **IGLA RACE's weight path should be GA-T `{−φ,0,+φ}` as an integer pair
  `(a,b)`** — not `{−1,0,+1}` with a learned per-layer scale, because that scale
  reintroduces the multiplier the whole design exists to remove.
- **The accumulator is where a format belongs**, and TNF16 (`s | 4 trits | 11 bits`)
  is the working rung. Do **not** put a format on the activation path.
- **Zero DSPs at any fan-in, 28 LUT/weight on XC7A200T** — this is the budget to
  size a systolic array against on our boards.
- **Verification must compare integer pairs, not floats.** The datapath's `(a,b)`
  against the reference's `(a,b)`, as integers. There is nothing in the datapath
  for a float comparison to be imprecise about.
- **Add a round-trip check to every format claim.** Commutativity is blind to sign
  conventions; round-trip is not, and costs one line.
- **Never quote a spec-table row as a silicon row.** Only TNF4–TNF64 were placed
  and routed.

---

## 11. Reproducibility, and the gap on this machine

The article's flow: **Yosys 0.65, nextpnr-xilinx 1743d0f, Icarus Verilog 13.0,
Python 3.14** — all open, nothing licensed. Measured on Artix-7, one die family,
open flow; **not** a multi-corner characterisation, and an ASIC mapping differs.

**Measured on this Mac (2026-08-13):**

| tool | article | here | status |
|---|---|---|---|
| Yosys | 0.65 | **0.63** | close, probably fine |
| Icarus | 13.0 | 13.0 | ✅ |
| Python | 3.14 | 3.14.6 | ✅ |
| nextpnr-xilinx | 1743d0f | **absent from PATH** | ❌ |
| nextpnr-himbaechel | — | present, `--list-uarch` → **xilinx** | ⚠️ |
| himbaechel chipdb | — | `chipdb-xc7a100t.bin` only — **wrong part** | ❌ |
| **openXC7 chipdb** | xc7a200t | `build/fpga/openxc7/xc7a200tfbg676-1.bin` — **332 MB, exists** | ⚠️ |
| prjxray-db | xc7a200t | `build/fpga/openxc7/prjxray-db/artix7/xc7a200tfbg676-1` — present | ✅ |

**The blocker is the binary, not the database.** A first pass looked only in
`/opt/homebrew/share/himbaechel/`, found `chipdb-xc7a100t.bin`, and concluded no
200T database existed anywhere. That was a **single-route measurement published as
a totality claim** — the repo's own build tree carries a 332 MB
`xc7a200tfbg676-1.bin` (and a 980 MB `.bba`) built 2026-08-09, plus the matching
prjxray-db part.

But it is in the **old `nextpnr-xilinx` format**, and the installed engine rejects it:

```
$ nextpnr-himbaechel --device xc7a200tfbg676-1 \
      --chipdb build/fpga/openxc7/xc7a200tfbg676-1.bin --test
Info: Using uarch 'xilinx' for device 'xc7a200tfbg676-1'
ERROR: chipdb ... does not look like a valid himbächel database!
```

So there are two routes back to a buildable bitstream, and they are not equal cost:

- **(a) install `nextpnr-xilinx` (openXC7 fork)** → consumes the existing 332 MB
  database directly. The expensive artefact is already built.
- **(b) regenerate a himbaechel-format chipdb for xc7a200t** from the prjxray-db
  that is present → uses the engine already installed, but repeats the generation
  that produced a 980 MB `.bba`.

**(a) is the cheap route.** Do not plan around (b) without measuring the generation
cost first.

Per the SSOT: `xc7a200tfbg676-1` and `xc7a200tfgg676-1` share die and BGA-676
pinout, so the prjxray-db `fbg676` entry is pinout-correct for our board.

**Also on disk already:** `fpga/verilog/ternary_mac_demo_top_200t.bit` (9.7 MB — a
200T-sized bitstream, vs ~3.8 MB for 100T), with `.fasm` and `.frames` beside
other designs. The flow *has* run on this machine. `README.md:97` claims
`FPGA | E2E bitstream | GREEN`; that claim is **not currently reproducible from a
clean PATH**, and the README's board row (`README.md:98`) still says XC7A100T.

---

**φ² + φ⁻² = 3 | TRINITY**
