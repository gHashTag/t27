# Wave Loop 656 — a ternary network on silicon, and the arithmetic that says what to do with it

**Date:** 2026-08-14 · **Predecessor:** [`WAVE_LOOP_655_REPORT.md`](WAVE_LOOP_655_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
The MVP exists and is on three boards:
  a complete ternary NN layer -- 8 inputs, 3 classes, 24 weights in
  {-phi,0,+phi} -- at 83 LUT, 37 CARRY4 and ZERO DSP.
  31/31 Zig, 31 PASSED Verilog, P&R 0 errors, Done 0->1 on 0:4, 0:7, 0:10.

And the arithmetic that decides what it is worth:
  T97   'zero DSP' does NOT separate the alphabets at inference
  T99   fan-in is logarithmic; DEPTH IS EXPONENTIAL
  T101  the phi alphabet trades ALL its gain freedom, and absorption
        fails below an achieved sum of three
  T107  a forecast registered and SCORED: 313 specs repaired, both
        halves inside the band
  T109  the interconnect is 77x slower than the compute it feeds
```

---

## 1. The MVP — T108

One layer, 8 binary inputs → 3 class scores → argmax. Twenty-four ternary
weights, fifteen non-zero. **The same operation a BitNet layer performs**, at a
size where every expected value was computed independently and written into the
file header *before* the implementation ran.

| | |
|---|---|
| LUT | **83** (4 LUT2, 4 LUT3, 49 LUT4, 2 LUT5, 24 LUT6) |
| CARRY4 | 37 |
| **DSP48E1** | **0** |
| share of XC7A200T | 0.06 % |
| Zig | **31/31** |
| iverilog + vvp | **31 PASSED**, 0 compile errors |
| P&R | 28 warnings, **0 errors**, 5,174 FASM lines |
| loaded | `Done 0x0 → 0x1` on all three boards |

> **T108.** A ternary network needs **no multiplier at any layer**, and the cost
> of proving it is 83 LUT. Every activation is `0` or `1` and enters `Z[φ]` as
> `(x, 0)`, so applying a weight yields `(0, ±x)`: the layer accumulates in the
> `b` component alone and the score is an **exact integer**. **The closure
> argument is visible at 24 weights, not only at scale.**

**Proven: configuration. Not proven: function.** Nothing was read back, and
`Done 0x1` reads identically before and after any load. The three boards carry
the *same* network — **replication, not distribution.**

---

## 2. T97 — the control refuted the headline, and located where the claim lives

T93 stated the condition that would refute it. **It did.** A `{−1,0,+1}` node
with a learned per-layer scale was built as a control:

| | φ node | unit node |
|---|---:|---:|
| LUT2 | 32 | 63 |
| CARRY4 | 24 | 33 |
| IBUF | **115** | 83 |
| **DSP48E1** | **0** | **0** |
| total cells | 246 | 245 |

**Both zero DSP, area within one cell.** Cause: `ALPHA_TRAINED = 352 = 2⁸+2⁶+2⁵`
— **a trained `α` is a constant at inference, and a constant multiply
strength-reduces to shifts.** Probed with `α` as a runtime input: **3 DSP48E1**.

> **T97.** The figure separates the alphabets **wherever `α` varies** — training,
> per-sample or dynamic scaling — at 3 DSP48E1 against 0. It does **not** separate
> them at inference with a frozen `α`. **A cost a compiler can constant-fold is
> not a cost of the architecture; it is a cost of the deployment mode.**

**And the φ node pays in pins:** 115 IBUF against 83, because `(a,b)` is two
integers. **`Z[φ]` exactness is not free**, and no earlier note said so.

---

## 3. T99 / T101 — two limits the article's phrasing blurs

**Fan-in is logarithmic; depth is Fibonacci, hence exponential.**

```
fan-in    8 -> 11 bits    512 -> 17 bits      (8 + ceil(log2 N))
depth  k= 5 -> 11 bits   k=30 -> 28 bits      (8 + 0.694 k)
```

> **T99.** Doubling the fan-in costs **one bit**; adding **fourteen layers costs
> ten**. A design sized from the fan-in figure and then deepened **will
> overflow** — and `Z[φ]` has neither saturation nor rounding, so **the exactness
> that makes the datapath free is exactly what makes the overflow invisible.**

**And the φ alphabet's gain freedom is zero.**

| | gain after `k` layers | degrees of freedom |
|---|---|---:|
| unit + learned `α_ℓ` | any positive real | **k** |
| φ | `φ^k` | **0** |

Snapping an arbitrary gain to the nearest `φ^k` costs `√φ = 1.272` → **+27.2%**.
The next layer's integer sum absorbs it, but only to `1/|m|` where `m` is the
**achieved** sum — and the crossover sits at **`m = 3`**.

> **T101.** A ternary network is **sparse by design**, so achieved sums are small,
> and the regime where the fixed φ scale *cannot* be absorbed is **the operating
> point, not an edge case.** Whether a real network sits above or below `m = 3`
> is empirical and **was not measured.**

---

## 4. T107 — a forecast registered and scored

| | before | forecast | measured | in band |
|---|---:|---:|---:|:---:|
| compiling | 236 | 590 ± 60 | **549** | ✅ |
| undeclared identifier | 488 | 130 ± 60 | **175** | ✅ |
| syntax error | 90 | — | 90 | unchanged |
| duplicate declaration | 16 | — | 16 | unchanged |

**313 specs repaired by one fix** — the largest single repair measured here, and
it repaired **a regression this same session introduced** (T106: the `t27_failed`
flag from T74 was declared in the test emitter and not the bench emitter).

> The forecast worked because it came from a **measured proportion on a random
> sample of fifteen**, not an estimate. The shortfall — 356 forecast, 313 observed
> — **is the multi-defect population**, 12%, exactly what T67 predicts.
>
> **Both untouched classes stayed exactly still. Checking the classes that should
> NOT move is half of scoring a forecast, and it is the half usually skipped.**

---

## 5. T109 — the number that decides the architecture

```
XC7A200T   1.60 MB BRAM -> 6.73 M weights @2 bits
           2,039 MAC units at 66 LUT -> 204 GMAC/s, zero DSP

layer 576x576 = 331,776 MAC                        =    1.6 us
its activations, seq=128, 32 wires                 =  123 us
```

> **T109.** Splitting a model across Artix-7 boards leaves the fabric idle **99%
> of the time**. **A network of FPGAs is bandwidth-bound, not capacity-bound**,
> and the capacity table — 21 boards for SmolLM2-135M — answers a question that
> is not the binding one.

**One regime inverts it:** at `seq = 1`, 576 activations cross in **1.0 µs on 32
wires** against 1.6 µs of compute. **Layer-splitting works for generation and
fails for batch.**

### The economics, computed and unflattering

```
selling compute:  node earns $0.31-1.88/month against $4.60/month cost -- LOSS
GPU:              RTX 4090 beats us 150x on GMAC/$ and 1400x on weights/$
big FPGA:         XCVU19P 0.206 GMAC/$ against our 1.359 -- 6.6x WORSE at 200x the price
```

**Three strategies are dead on our own numbers.** What survives is not raw
compute but **presence**: 5 W, 1.6 µs deterministic latency, and a multiplier
that is absent *by theorem* rather than by optimisation.

---

## 6. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean, seal matches |
| MVP, both backends | **31/31** and **31 PASSED** |
| MVP synthesis | 83 LUT, 37 CARRY4, **0 DSP** |
| bitstream | built locally, SHA `2b2ace34…`, 9,730,896 B |
| three boards | `Done 0x0 → 0x1` each |
| ratchet | **CLEAN 326/326**, rc 0 |
| working tree | clean |

---

## 7. What is NOT done

- **No function readback.** Configuration is proven; what the fabric computes is not.
- **No inter-board link.** The only pin map in the repo is for a **different
  board** (`CSG324` vs our `FGG676`) and contradicts its neighbour on `T14`/`T15`.
- **UART wiring unverified.** All three CP2102 bridges returned silence, which
  proves only that the loaded design cannot speak.
- **175 specs still fail on undeclared identifiers**, undiagnosed.
- **799 of 852 modules have no data ports** (T96 marks them; no default guessed).
- **265 Icarus baselines** record a harness that could not fail.
- **Accuracy never measured** — not for φ vs unit, not for the MVP.
- **Two research workflows were killed by the session restart** and have been
  resumed; their findings are not in this report.

---

## 8. Three ways to continue (pick one for W657)

### Option 1 — **Make the MVP observable**

Configuration is proven and function is not. Give the classifier a channel: a
UART transmitter on a discovered pin, or a BSCANE2 user register readable over
JTAG.

- **Cost:** medium. BSCANE2 needs no pin discovery and no wiring — **it is the
  only readback path that works today.**
- **Pays off in:** turns "a bitstream loaded" into "the silicon computed the
  right class", which is the difference between a demo and a result.
- **Risk:** BSCANE2 support in the open flow is unverified; `nextpnr` claims it
  (`pack_clocking_xc7.cc` maps `id_STARTUPE2`), but that is not the same cell.
- **Confirming measurement:** the class read back over JTAG matches the reference
  table for all ten test inputs.

### Option 2 — **Diagnose the remaining 175**

T107 repaired 313 and left 175. The method is proven: a random sample of fifteen
carried individually to the name of the unresolved identifier.

- **Cost:** low — the sweep script exists and the method scored inside its band.
- **Pays off in:** the second-largest population in the corpus, and the last
  thing between the compiler and a majority-compiling tree.
- **Risk:** T105 — do not generalise from one case. **Sample fifteen.**
- **Confirming measurement:** a cause histogram over the 175 summing to 175, and
  a forecast registered before any fix.

### Option 3 — **Measure accuracy, φ against unit**

T101's stated limit and the one number no measurement in this project has: does
the fixed φ scale cost accuracy at the sparse operating point?

- **Cost:** high — needs a training loop, which does not exist here.
- **Pays off in:** the only remaining claim that could refute the whole argument.
  T97 already showed area does not separate them; **if accuracy does not either,
  the φ alphabet is decoration.**
- **Risk:** an unfair comparison is worse than none. Equal parameter count, equal
  training budget, and the tie rules stated in advance.
- **Confirming measurement:** perplexity or accuracy for both alphabets at
  matched size, with the sparsity histogram reported alongside.

**Recommendation: Option 1.** Everything this wave produced rests on a bitstream
whose *function* was never observed. **A demo that cannot be checked is the exact
shape of every defect this session found** — and BSCANE2 needs no wire, no pin
map, and no human.

**φ² + φ⁻² = 3 | TRINITY**
