# When enumeration beats a prover, and what decides it

Written 2026-08-18 from the measurements in #2198, #2200 and #2202. Everything numeric
below was measured in this repository, not estimated.

## 1. Exhaustive agreement over a finite domain is a decision procedure

Let `f_A, f_B : D → R` be two implementations of the same specified function, and let
`D` be finite. Then

> checking `f_A(x) = f_B(x)` for every `x ∈ D` **decides** whether `f_A ≡ f_B`.

There is no induction, no loop invariant, no SMT encoding and no trusted prover kernel.
A disagreement is a counterexample; agreement is the whole theorem. This is not a
weaker substitute for formal equivalence checking — on a finite domain it is the
strongest statement available, and it is what HECTOR and ACL2 are *approximating* when
the domain is too large to walk.

The only question is `|D|`.

## 2. `|D|` is set by the REPRESENTATION, not by the semantics

A trit carries three values. In these specs a trit is declared `u8`:

```
fn tmul(ta: u8, tb: u8) -> i8
fn full_adder(a: u8, b: u8, cin: u8) -> u8
```

so the enumerated domain is `256^k`, not `3^k`.

**Claim.** For a function of `k` trit arguments declared in a `w`-bit type, exhaustive
enumeration costs `(2^w / 3)^k` times more than the semantic domain requires.

**Proof.** The semantic domain is `3^k`; the representational domain is `(2^w)^k`.
Their ratio is `(2^w / 3)^k`. ∎

**Check against measurement.** For `w = 8, k = 3`: `(256/3)^3 = 621,378.6`. Measured in
this tree: `16,777,216 / 27 = 621,378`. The two agree.

| function | semantic `|D|` | representational `|D|` | ratio |
|---|---:|---:|---:|
| `negate` | 3 | 256 | 85 |
| `tmul`, `pack2` | 9 | 65,536 | 7,281 |
| `maj3`, `full_adder` | 27 | 16,777,216 | **621,378** |

Only **1.61 × 10⁻⁶** of `full_adder`'s enumerated space is a valid trit triple.

## 3. Both readings of that number are true

**It is waste.** 16.7 M inputs are walked to distinguish 27 semantic cases. Declaring a
2-bit trit type would put `full_adder` at `4^3 = 64` inputs — and iverilog, which needs
**94 minutes** for the byte-wide domain at its measured 2,972 inputs/s, would finish
the 2-bit domain in about 0.02 s. **A four-million-fold change in verification cost,
from a type declaration, with no change to the algorithm.**

**It is coverage.** Nothing in the type system stops a caller passing `200`, and the
spec has defined behaviour there — `tmul` returns `1` when `ta == tb` regardless of
whether either is a trit, and `pack2` does not mask its arguments, so a value above 3
spills into the neighbouring lane. That behaviour is part of what the backends must
agree on, and enumerating the byte-wide domain verifies it. A 2-bit type would make
those inputs unrepresentable rather than verified — which is better, but it is a
different guarantee, not the same one made cheaper.

## 4. Why this is a property of the number system, not of cleverness

A binary float adder over two 32-bit operands has `|D| = 2^64 ≈ 1.8 × 10^19`. At the
fastest rate measured here — C at roughly 10^7 inputs/s — that is about **58,000
years**. Exhaustive agreement is not available at any budget, which is precisely why
sequential equivalence checking and theorem proving exist for that regime.

Ternary primitives sit on the other side of the line:

| domain | `|D|` | C, ~10⁷/s | iverilog, measured |
|---|---:|---:|---:|
| trit primitive, 2-bit type | 64 | instant | instant |
| trit primitive, `u8` type, k=2 | 65,536 | instant | 0.2 s |
| trit primitive, `u8` type, k=3 | 16,777,216 | ~1.5 s | 13–94 min |
| binary float add, 2×32-bit | 1.8 × 10¹⁹ | 58,000 years | — |

**The line between "enumerate" and "prove" is crossed by the representation width, and
small alphabets sit below it.** That is the one place where choosing ternary buys a
*verification* advantage rather than an area or energy claim — and unlike the area
claims in this project's history, which were withdrawn twice, this one follows from
counting and can be checked by anyone in a few seconds.

## 5. What follows, in order of value

1. **Declare trits in a 2-bit type where the spec allows it.** By §3 that is a
   ~4 × 10⁶ reduction in enumeration cost for 3-argument functions, which moves the
   Verilog arm from a 94-minute slice to an exhaustive check that fits in a PR gate.
   The cost is that out-of-domain behaviour becomes unrepresentable rather than
   verified; that trade should be made deliberately and written down, not drifted into.
2. **Enumerate wherever `|D| < 2^24`** — the survey in this tree found `pack3`, `xor2`,
   `sign0`, `quantize`, `trit2gft` and the `bitnet_*` family all below that line and
   none of them covered.
3. **Say which regime each result is in.** "Exhaustive over `|D| = 16,777,216`" and
   "sampled 800 of 2.8 × 10¹⁴" are different claims, and a reader cannot tell them
   apart from the word *bit-exact*.

## Sources

- Digests and timings: `tools/verify_exhaustive.py`, `tools/ternary_model.py`, this tree
- The regime where enumeration is unavailable: [Formal Verification of Arithmetic RTL: Translating Verilog to C++ to ACL2](https://arxiv.org/pdf/2009.13761), [Automated Formal Equivalence Verification of Pipelined Nested Loops in Datapath Designs](https://arxiv.org/pdf/1712.09818)
- Positioning against multi-target toolchains: `docs/POSITIONING.md`
