# Trinity x Pellis hybrid — research scaffold (issue #277)

This directory holds the **IMRaD-style scaffold** for the central question:

> Why does phi-scaled structure appear near electroweak / fine-structure numerology, and can Trinity monomials be obtained as limits of Pell-type polynomial maps?

## Hypothesis (falsifiable)

**Pellis (thin-structure proxies) -> Trinity (effective scaling law):** Trinity monomials may behave as stable fixed points of a renormalized Pell-weighted map. The CLI diagnostic `tri math compare --hybrid` prints a **scalar inner product** only; it does **not** prove physics.

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
| `README.md` | Hypothesis, scope, and CLI pointers. |

## Specs

- `specs/physics/pellis-formulas.t27` — L5 anchor, Pell block, alpha^-1 reference, TDD blocks.

## Status

Scaffold only: extend the table as specs and `tri math compare` gain real mappings.
