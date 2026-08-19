# The φ alphabet, closed

**Date:** 2026-08-14 · **Waves:** W710–W712 · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This project was founded on a weight alphabet: `{−φ, 0, +φ}` instead of
`{−1, 0, +1}`. **Three waves measured it.** This document states what was
claimed, what was measured, and what survives.

---

## What was claimed

That a ternary network whose nonzero weights are `±φ` gains something over one
whose weights are `±1` — exactness, multiplication-freedom, or expressiveness —
because `φ² = φ + 1` makes weight application the Fibonacci step
`(a,b) ↦ (b, a+b)`: one integer add, no multiplier, no rounding.

**Before W710 there was no trained model in this alphabet.** Not in the
literature — a null verified across arXiv, DBLP, OpenAlex and Semantic Scholar —
and not in this repository. The claim rested on algebra alone for the project's
entire history.

---

## What was measured

### 1. The pure alphabet factors out (T158, algebra)

Every layer computes `Σᵢ(±φ)xᵢ = φ·Σᵢ(±1)xᵢ`. **φ leaves the layer.** A depth-*k*
pure-φ network is exactly `φᵏ` times the corresponding ternary network, verified
by exact simulation at depths 1, 2, 3, 5.

**Consequence, and it is a relief rather than a defeat:** the MVP's `contrib`
returning `±x` had been recorded for many waves as "the MVP does not implement
Z[φ]". **It is not a shortfall.** For the pure alphabet, `±x` *is* correct up to
the global `φᵏ`.

### 2. Thirty seeds say φ adds nothing (T205, T207)

UNSW-NB15 binarised (Zenodo 4519767), 593 binary inputs, 593→64→1, straight-
through estimator, **a fixed integer threshold so φ cannot factor out**, four
arms differing only in the weight alphabet, thirty seeds, paired t-test:

| comparison | Δ (pp) | t | verdict |
|---|---:|---:|---|
| `{−φ,0,+φ}` − `{−1,0,+1}` | **+0.025** | **0.67** | **not significant** |
| `{0,±1,±φ}` − `{−1,0,+1}` | +0.280 | 4.89 | significant |
| `{−2,−1,0,+1}` − `{−1,0,+1}` | +0.222 | 3.32 | significant |
| **`{0,±1,±φ}` − `{−2,−1,0,+1}`** | **+0.059** | **1.00** | **not significant** |

**The φ effect shrank as n grew** — +0.116 at n=4, +0.006 at n=21, +0.025 at
n=30. That is what a null looks like.

**Cardinality above three is real**, and **four levels capture all of it**. The
five-level set costs **2.3333 bits/weight against 2.0 — +16.7%** — and buys
nothing measurable.

### 3. The pair form computes the same function (T208)

The last branch: propagate un-collapsed `(a,b)` coordinates across layers, so
weight application stays exact integer arithmetic. Two layers, 16,384 outputs:

```
max relative difference   2.043e-12      float64 precision
sign agreement            1.000000       every output
```

**A pair is an exact representation of the same real number the scalar path
approximates.** It is not a different model.

---

## What survives

**One thing, and it was never in dispute:**

> Applying a weight from `{−φ, 0, +φ}` to a Z[φ] pair costs **one integer add**
> and is **exact** — no multiplier, no rounding.

That is a **hardware** property. It is not a claim about accuracy, about
expressiveness, or about the alphabet being better. And it has a measured price:

**≈3.3 bits of accumulator width per layer** at fan-in 64 — 6.8 bits after one
layer, **23.2 after six**, where the inputs were 4. T159a predicted
`0.5·log₂N + 0.694 = 3.69`; the measurement is within 11%, from a different
construction.

---

## What is retracted

**T158a**, which named `{0, ±1, ±φ}` as *"the defensible restatement… genuinely
richer than 3-level ternary… That is what the project should claim."*

It **is** richer than ternary. But a plain 2-bit set is richer by the same
margin at less cost, so **φ's presence contributes nothing there either**. The
defensible claim, measured, is *"cardinality 4 beats cardinality 3"* — a
statement about counting, not about the golden ratio, and one the 2-bit
quantisation literature has held since **LQ-Nets (ECCV 2018)**, whose learned
floating-point basis already admits irrational levels: `v = [(1+φ)/2, (φ−1)/2]`
reproduces `{±1, ±φ}` exactly as a 2-bit codebook.

---

## What this does not test

- **Competitive accuracy.** 85.5% mean against a **92–93%** published state of
  the art on this benchmark. The measurement establishes *relative order among
  alphabets under identical conditions* and nothing about competitiveness.
- **Other tasks.** One dataset, one architecture, one training budget.
- **The hardware claim.** The LUT figures in T183 are analytic; none has been
  through yosys and nextpnr.

---

## Where this leaves the project

The alphabet was the founding idea, and it does not survive as an accuracy
claim. **What does survive is a multiplier-free exact datapath with a measured
width cost** — which is an engineering position, not a scientific one, and which
the literature already occupies: FINN measured 3.66 LUT per binary MAC in 2017,
and this project's MVP sits at 3.46 on an easier problem with no accuracy figure
attached (T161).

**The honest next question is not "is φ better" — it is answered — but "what,
on this hardware, is worth building at all."**

Code and every run: [`experiments/phi-alphabet/`](../../experiments/phi-alphabet/).

**φ² + φ⁻² = 3 | TRINITY**
