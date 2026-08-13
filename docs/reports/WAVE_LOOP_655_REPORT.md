# Wave Loop 655 — the ternary vertical exists, and it has no multiplier

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_654_REPORT.md`](WAVE_LOOP_654_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
Asked directly "do you use GFTernary and TNF?", the answer was measured:
  TNF in .t27 specs                    0 of 1,064
  gfternary.t27 -- who references it   1 file (itself)
  gft_* using the GFT_ alphabet        0 of 12

Four specs later the vertical exists and synthesises:

  numeric/tnf17.t27          first TNF implementation   0 LUT (negate = wiring)
  fpga/ternary_link.t27      3B2T + GFTernary bridge    7 LUT6, bridge FREE
  igla/race/phi_weights.t27  Z[phi] weight application  3 LUT6, ZERO DSP
  igla/race/ternary_node.t27 the composed node          66 LUT + 24 CARRY4, ZERO DSP

135 tests, both backends, zero compile errors.
```

---

## 1. The question that reordered the wave

`ternary_inference.t27:20` — *"Weights are stored as ternary codes (0=zero, 1=+1,
2=-1)"*. `gfternary.t27:20-22` — `GFT_ZERO=0x00 → 0`, `GFT_POS=0x01 → +φ`,
`GFT_NEG=0x02 → −φ`.

**The codes were already identical. Only the interpretation differed** — RACE read
code `1` as `+1`, not `+φ` — **and that difference is the entire golden-alphabet
argument** (T89): with `{−1,0,+1}` the layer gain is `1`, carries no information,
and every published method hangs a learned real `α_ℓ` on each layer whose
application **puts the multiplier back**.

**A correction was needed and made** (T86). It had been stated that `gft_dot2.t27`
"is a binary float whose comment claims balanced-ternary." Too strong: the file
enforces `BIAS=40`, `OFFSET_MAX=80` — exactly the 81 values of four balanced
trits — and the *scale* being `2^e` is **correct** by the article's own radix
theorem. **GF-T16 is implemented faithfully.** What survived: GFTernary had no
consumers, and TNF did not exist.

---

## 2. What was built

| spec | Zig | iverilog+vvp | silicon |
|---|---|---|---|
| `numeric/tnf17.t27` | 34/34 | 34 PASSED | **0 LUT** — negation is a bit flip |
| `fpga/ternary_link.t27` | 46/46 | 46 PASSED | **7 LUT6**, bridge adds nothing |
| `igla/race/phi_weights.t27` | 29/29 | 29 PASSED | **3 LUT6, 0 DSP** |
| `igla/race/ternary_node.t27` | 26/26 | 26 PASSED | **66 LUT + 24 CARRY4, 0 DSP** |

> **T93.** One algebraic fact — `φ² = φ + 1` — removes a stage at **every**
> boundary, and at each the saving is the same shape: **not a stage made cheap
> but a stage that does not exist.** Between nodes, because a wire symbol *is* a
> GFTernary code (T87, measured at zero LUTs). Inside the datapath, because
> `Z[φ]` is a ring. Between layers, because the gain is an integer pair.

**Stated limits.** One MAC step. No fan-in, no depth, no accumulator-width growth,
and **no comparison against a `{−1,0,+1}` node** — so "zero DSP" is one side of a
comparison that has not been run.

---

## 3. T88 — two defects caught by the machine, not by review

1. **An invariant failed at Zig comptime** because `TNF_MINUS_ONE` was written
   `85504` when `20480 + 65536 = 86016`. **An invariant written to document a
   layout functioned as a checker of its author.**
2. **The Zig backend emits a raw `%` on signed integers**, which Zig rejects.

Routing around (2) produced a better design: trits are extracted from the
**biased offset** in unsigned arithmetic via the excess-1 identity
`40 = 1+3+9+27 = (3⁴−1)/2`, so unbiasing decrements every base-3 digit and **no
signed division or remainder appears anywhere**.

> **Corollary.** The bias of a balanced-radix-`r` field of `d` digits is the
> repunit `(r^d−1)/(r−1)`, and *because* it is the repunit, unbiasing is a
> per-digit decrement rather than a borrow. **Bias 40 is not a convention; it is
> the unique value that makes the balanced view free.**

---

## 4. T95 — `f32` decided, with the measurement specified in advance

| | before | after |
|---|---:|---:|
| f32/f64 specs compiling | 17 | **17** |
| tests PASSED | 4 | **5** |
| tests **FAILED** | **2** | **0** |

**Zero regressions. Both failures fixed.** The risk that argued against `real` —
that packed uses would break — **did not materialise on a single spec**, and the
estimate that produced it was a crude proxy.

> **T95.** Of three ways to lower a type the target cannot represent, only one is
> **silent**, and the project held that one for its entire history — because
> **the option that looks like it is working is selected by exactly the property
> that makes it wrong.**

---

## 5. T90/T91 — and the error was mine, twice

```
gh repo list gHashTag --limit 100  -> 100    <- reported as "100 repos"
gh repo list gHashTag --limit 200  -> 200    <- the recon brief I wrote
gh repo list gHashTag --limit 1000 -> 219    <- the answer
```

**T90 was written after the error and without noticing the error had been made.**
Also corrected: `gHashTag` is a **User**, not an Organization.

> **T91.** A recorded lesson protects only the measurements taken after it, and
> only those the author connects to it. Neither condition held.

**T94 discharges the corollary**: `scripts/check-pagination-truncation.sh` doubles
the limit until the count is strictly below it. Negative control on `openXC7`:
truncated at 10, 19 at 20, type `Organization` — so it **discriminates** rather
than always crying truncation.

> A lesson is a claim about what a future *reader* will remember; a check is a
> claim about what a future *run* will do. **The 329 lessons in
> `t27-wave-loop.md` are a record, not a mechanism** — the ones that stopped a
> recurrence this session are exactly those that became gates, ledgers or scripts.

---

## 6. T96 — 799 modules marked, no default guessed

Every module whose header is only `(clk, rst_n, en, ready)` now carries a
`NO DATA PORTS` marker. **No default `on_comb` was chosen**: picking the last
`pub fn` would silently promote an internal helper to a public boundary, and a
**wrong** boundary is worse than none. Following T52, the *absence* gets a
reserved symbol — loud, greppable, countable.

```
marked NO DATA PORTS   799
with data ports         53
no module emitted      216
```

---

## 7. ⚠ Two items requiring the user, not the agent

**(a) `tnf-publication-readiness` is not on GitHub.** The handoff document's own
first check, run:

```
$ git ls-remote --heads .../trinity-fpga tnf-publication-readiness
(empty)
```

Its instruction for that case is *"nothing to fetch; do not reconstruct from
pieces; ask for confirmation to push."* **264 files, +50,976 lines and a
128-page paper exist only locally.** Nothing was reconstructed.

**(b) `trinity` and `trinity-fpga` are one codebase with two live heads** (T92).
The handoff names base `f4e361a3da1d` — **confirmed as `trinity-fpga`'s HEAD, and
exactly the commit `trinity` answers `HTTP 422 "No commit found"` for.** The
publication work sits on the diverged branch.

**Neither is the agent's to decide.**

---

## 8. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| four new specs, Zig | **135/135** |
| four new specs, iverilog+vvp | **135 PASSED**, 0 compile errors |
| composed node synthesis | 66 LUT + 24 CARRY4, **0 DSP** |
| ratchet after T86–T89 | **CLEAN 326/326** |
| ratchet after T95 | **CLEAN 326/326** |
| ratchet after T96 | **running at time of writing** |
| working tree | clean; seal matches |

---

## 9. Three ways to continue (pick one for W656)

### Option 1 — **Run the comparison T93 says has not been run**

Build a `{−1,0,+1}` node with a per-layer learned scale, synthesise it, and put
its DSP and LUT counts beside the φ node's.

- **Cost:** medium; the φ node is the template and the scale path is the only new part.
- **Pays off in:** turns "zero DSP" from a measurement of one side into a
  **comparison**, which is what the closure argument actually claims. Until this
  runs, T93 is not falsifiable in the direction that matters.
- **Risk:** a fair comparison needs equal accuracy, and accuracy is not measured
  here at all. **State the accuracy caveat before, not after.**
- **Confirming measurement:** both nodes' `synth_xilinx` cell counts side by side,
  with the α-scale path's multiplier visible as DSP48 or LUT-built.

### Option 2 — **Fan-in and depth: does `(a,b)` stay exact?**

T93's stated limit. The article claims component widths grow logarithmically,
reaching 8 bits at fan-in 512. **Nothing here tests that.**

- **Cost:** medium; extend `ternary_node.t27` to a fan-in-N reduction with a
  width invariant.
- **Pays off in:** the exactness claim is the load-bearing one — "zero rounding
  error at any fan-in and any depth" — and it is currently asserted, not tested.
- **Risk:** if the width grows faster than claimed, the accumulator sizing in
  every downstream design is wrong. That is a result worth having either way.
- **Confirming measurement:** measured component width at fan-in 8/32/128/512
  against `⌈log₂(fan-in)⌉ + k`, and zero mismatches against an exact reference.

### Option 3 — **Give the 799 marked modules an `on_comb`, opt-in and in bulk**

T96 made them countable. The next step is to make them *hardware*.

- **Cost:** high by hand, low if the language gains an explicit annotation and the
  marker becomes a gate.
- **Pays off in:** 94% of the corpus currently synthesises to nothing; this is the
  precondition for **any** spec-first hardware beyond the four specs written here.
- **Risk:** the failure mode is a wrong boundary, which is worse than none — so
  the annotation must be **explicit per spec**, never inferred.
- **Confirming measurement:** the 799/53 split moves, and yosys cell counts are
  non-zero for every spec that moved.

**Recommendation: Option 1.** T93 is this wave's headline and it is explicitly
half a comparison. Running the other half either strengthens it into the
project's strongest measured claim or refutes it — and both outcomes are worth
more than another artefact measured on one side.

**φ² + φ⁻² = 3 | TRINITY**
