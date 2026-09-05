# Positioning: t27 as the Conformance / Registry Layer

**Status:** Strategy note. Implementor-facing. No compliance claim, no
superiority claim. This document fixes WHERE the t27 numeric catalog sits in the
numeric-format ecosystem so that every info-drop, cross-walk, and pack speaks
from the same coordinates.

**Re-anchored:** 2026-06-14 (companion to `P3109_CROSSWALK_4PARAM.md`,
`KAPPA_FP8_E4M3.md`, `ERRATA_2026-06-14.md`).

---

## One sentence

We are the **conformance / registry layer**: a vendor-neutral catalog of numeric
formats with bit-exact, honestly-erroring conformance vectors. We are **not** a
standards body, **not** a formal-proof engine, and **not** a competing format
family that claims to beat anyone on accuracy.

## The layer stack (who does what)

```
LAYER 1  STANDARDS          IEEE SA P3109 WG          defines the format family
         (defines)          (arXiv:2606.04028,        (K, P, Signedness, Domain),
                            interim report v0.9.1)     bias, special values,
                                                       operation semantics
                                  |
                                  v  defers-to / verifies-against
LAYER 2  FORMAL PROOF       FLoPS (Rutgers/UCR,        machine-checked Lean model
         (verifies)         arXiv:2602.15965);         of P3109 semantics; finds +
                            Imandra P3109;             fixes spec bugs; the source
                            Rutgers FP verification    of truth for SEMANTICS
                                  |
                                  v  we defer-to on semantics
LAYER 3  CONFORMANCE        >>> t27 numeric catalog <<<  bit-exact encode/decode
         (measures + lists) (THIS WORK)                 vectors per format, with
                            live-count SSOT,            HONEST abs_error; a
                            6 JSON packs,               vendor-neutral REGISTRY of
                            Corona FPGA oracle          what each format's bits mean
                                  |
                                  v  consumed-by
LAYER 4  IMPLEMENTORS       tt-metal, ml_dtypes,       use our vectors as a
         (consume)          PyTorch, IREE, vLLM,       cross-vendor "does it fit"
                            llama.cpp, ONNX Runtime    ruler when two stacks
                                                       disagree on the same format
```

Two Layer-3 cells, spelled out: the SSOT count is a CI invariant, not a fixed
number (109 formats at v3, Sep 2026; `tools/check_catalog_count.py`), and the
Corona oracle is the Artix-7 (XC7A200T) FPGA prototype -- no Corona (TTGF26a)
die has been fabricated.

Reading the stack:
- **Layer 1 decides what a format IS.** We never argue with the WG; we map to
  their 4-parameter model in their own vocabulary (see `P3109_CROSSWALK_4PARAM.md`).
- **Layer 2 verifies the SEMANTICS** (rounding, projection, FMA, special-value
  handling). Where our decode disagrees with FLoPS or Imandra, **they win** and
  we annotate the divergence. We do not attempt to out-verify a Lean/Imandra
  formalization -- that is their layer, and they are better at it.
- **Layer 3 is ours: a REGISTRY + a RULER.** Given a format, we publish exactly
  which bit pattern decodes to which real value, and we state the error honestly
  where a value is not representable. When P3109 leaves a choice open (e.g. FP8
  E4M3 saturate-vs-NaN on overflow), we do not pick a winner -- we **measure the
  cost of each choice** in the WG's own kappa-approximation (see
  `KAPPA_FP8_E4M3.md`).
- **Layer 4 consumes us.** The value to an implementor is: when two stacks
  disagree on the same nominal format, our pack is the neutral arbiter that says
  which bits mean what, with the divergence quantified.

## Why we do NOT compete with Layer 2

Three concrete reasons, stated plainly so no drop ever drifts into a turf claim:

1. **Different artifact.** FLoPS/Imandra produce machine-checked theorems about
   operation semantics. We produce a table of bit patterns and decoded values.
   A theorem and a vector table answer different questions ("is the rounding
   rule sound?" vs "what does 0x7E decode to in this vendor's E4M3?").
2. **We depend on them.** Our cross-walk explicitly defers to FLoPS where decode
   semantics disagree. A registry that argued with the formalization it cites
   would be incoherent.
3. **Standing.** A WG audience judges a superiority claim harder than any single
   repo. The honest, useful posture -- "here is a cross-reference, please correct
   it" -- is the only one that builds name at Layer 1/2. (See
   `ruler-reputation-method` S1->S5 funnel.)

## Breadth, not per-rung superiority (the takum rule)

Our catalog earns its place through **breadth and toolchain coherence** across
the live SSOT count of formats (109 at v3, Sep 2026 -- run
`python3 tools/check_catalog_count.py`, never quote from memory) -- **not** by
claiming any single format beats a competitor on accuracy. This is a hard rule,
mirrored from the Corona ROM governing sentence.

- **takum** (Hunhold,
  [arXiv:2404.18603](https://arxiv.org/abs/2404.18603),
  [arXiv:2404.18603](https://arxiv.org/abs/2404.18603),
  [arXiv:2504.21197](https://arxiv.org/abs/2504.21197)) is the **standing
  counterexample**: at low precision takum is competitive with or better than
  several of our own rungs, and reportedly meets-or-beats OFP8 at low precision.
  We **ship takum un-suppressed in the catalog / Corona ROM**. We do not hide it,
  down-weight it, or frame our ladder as superior to it.
- The "GoldenFloat ladder is better per-rung" claim (FL-002) stays **[Open
  conjecture]** and is never asserted in any drop. What we assert is coverage +
  bit-exact traceability across the whole family, takum included.

This breadth posture is itself the moat: a registry is only trustworthy if it
lists formats that outperform its author's own. Listing takum honestly is the
credibility we trade on.

## What this means operationally

- **Info-drops** lead with a bit-exact fact + honest error, cite the relevant
  layer above us (P3109 for the format definition, FLoPS for semantics), and ask
  nothing. Never "our format is better"; always "here is the cross-vendor
  divergence, measured."
- **Cross-walks** map to the P3109 4-parameter model and defer on semantics.
- **The kappa hook** is how we report an open WG choice as a number, in the WG's
  own measure, without taking a side.
- **The catalog count** is whatever the live SSOT says (run the count, never
  quote from memory -- see `ERRATA_2026-06-14.md` and the CI gate
  `tools/check_catalog_count.py`). Today (2026-09-05): 109.

## What we explicitly do NOT claim (honesty wall)

- We do not claim P3109 compliance -- only an implementor cross-reference.
- We do not claim our formats beat any competitor on any metric.
- We do not claim to verify operation semantics -- we defer to Layer 2.
- We do not claim "first" or "only" anything (banned-hype rule).
- We do not claim a count from memory -- the live SSOT is the only source.

## References

- IEEE P3109 4-parameter model: [arXiv:2606.04028](https://arxiv.org/abs/2606.04028)
- FLoPS Lean formalization: [arXiv:2602.15965](https://arxiv.org/abs/2602.15965)
- Imandra P3109: https://github.com/imandra-ai/ieee-p3109
- takum: [arXiv:2404.18603](https://arxiv.org/abs/2404.18603)
- GoldenFloat preprint: arXiv:2606.05017
- Catalog paper #3: "Golden Ruler: A Numeric Format Catalog with Bit-Exact Conformance Vectors for FP8, BF16, MXFP4, and Microscaling Formats",
  arXiv:2606.09686 (v3, announced 7 Sep 2026; count history in `ERRATA_2026-06-14.md`)
