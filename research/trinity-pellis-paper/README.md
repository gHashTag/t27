# Trinity x Pellis hybrid — research scaffold (issue #277)

This directory holds the **IMRaD-style scaffold** for the central question:

> Why does phi-scaled structure appear near electroweak / fine-structure numerology, and can Trinity monomials be obtained as limits of Pell-type polynomial maps?

## Hypothesis (falsifiable)

**Pellis (thin-structure proxies) -> Trinity (effective scaling law):** Trinity monomials may behave as stable fixed points of a renormalized Pell-weighted map. The CLI diagnostic `tri math compare --hybrid` prints **hybrid v1** (H_5^(v1), L1+max Pell map) only; **hybrid v2** (L2 cosine, variable N) is specified in `hybrid-conjecture.md` and tracked in [#287](https://github.com/gHashTag/t27/issues/287). Neither proves H1 by itself.

## Commands (SSOT path)

```bash
./scripts/tri math compare
./scripts/tri math compare --pellis
./scripts/tri math compare --pellis --pellis-extended --hybrid --sensitivity
```

Each run appends one JSON line to `.trinity/experience/math_compare.jsonl` (proof chain).

## Contents

| File | Purpose |
|------|---------|
| [`FORMULA_TABLE.md`](FORMULA_TABLE.md) | Placeholder catalog toward 152 formulas (IDs + category + status). |
| [`hybrid-conjecture.md`](hybrid-conjecture.md) | Formal sketch of the hybrid hypothesis, falsifiers, sensitivity scope, open work. |
| [`ROADMAP.md`](ROADMAP.md) | Post-merge priorities (formula table, outreach, preprint). |
| `README.md` | Hypothesis, scope, and CLI pointers. |

## Project impact (summary)

- **SSOT:** `pellis-formulas.t27` places a **Pell ladder** next to the existing **Trinity / \(\phi\)** layer in one verifiable spec (issue #277).
- **CLI:** `tri math compare` exposes Pellis-style contrasts, SM reference constants, hybrid scalar, and \(\phi\)-sensitivity — all in Rust via `t27c`.
- **Traceability:** experience JSONL lines now include **`pellis_spec_seal_hash`** when the seal file is present, linking runs to the sealed spec revision.

## Specs

- `specs/physics/pellis-formulas.t27` — L5 anchor, Pell block, alpha^-1 reference, TDD blocks.

## Status

Scaffold only: extend the table as specs and `tri math compare` gain real mappings.
