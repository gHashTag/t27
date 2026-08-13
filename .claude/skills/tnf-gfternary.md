---
name: tnf-gfternary
description: The TNF (Ternary Network Float) and GFTernary number formats — their definitions, theorems, silicon costs, retracted claims, and what this means for IGLA CODER / IGLA RACE. Load before touching any numeric format, accumulator width, or weight alphabet in t27.
---

# TNF & GFTernary — working knowledge

**Source of truth:** [`docs/theory/TNF_ARTICLE_RU.md`](../../docs/theory/TNF_ARTICLE_RU.md)
(2353 lines, Russian). This skill is the distillation. When they disagree, the
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

- **weight → GFTernary**, the 2-bit alphabet `{−φ, 0, +φ}`
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

## 3. GFTernary — and why φ is forced

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

### Verified against silicon

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

This resolves an 83-format catalogue into exactly **four** taper shapes — and
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
| GFTernary throughput/area | **+10.2%** over runner-up, **6.1×** over posit32, among 20 self-ranged formats |

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

- **IGLA RACE's weight path should be GFTernary `{−φ,0,+φ}` as an integer pair
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
other designs. The flow *has* run on this machine. `README.md:61` claims
`FPGA | E2E bitstream | GREEN`; that claim is **not currently reproducible from a
clean PATH**, and the README's board row still says XC7A100T.

---

**φ² + φ⁻² = 3 | TRINITY**
