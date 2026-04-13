# Research Papers — Trinity Programme

This directory contains research papers and supporting documents for the Trinity S³AI programme.

## Papers

### Trinity γ-Paper (Barbero-Immirzi Parameter)

**Status:** Draft v0.1

**Main Paper:** [`trinity-gamma-paper/GAMMA_PAPER_DRAFT_v0.1.md`](trinity-gamma-paper/GAMMA_PAPER_DRAFT_v0.1.md)

**Key Claim:** γ_φ = φ⁻³ = √5 − 2 differs from standard LQG γ₁ by only **0.63%**, while the internal LQG dispute between γ₁ and γ₂ is **13.9%**.

**Documents:**
- [`GAMMA_PAPER_DRAFT_v0.1.md`](trinity-gamma-paper/GAMMA_PAPER_DRAFT_v0.1.md) — Main paper
- [`PREREGISTRATION.md`](trinity-gamma-paper/PREREGISTRATION.md) — Pre-registered hypotheses H-A, H-B, H-C
- [`README.md`](trinity-gamma-paper/README.md) — Overview and verification instructions

**Spec:** `specs/physics/gamma_conjecture.t27` — Formal definition of Conjecture GI1

**Verification:** (Pending CLI integration) `tri math compare --gamma-conflict`

---

### Trinity Pellis-Paper (Hybrid Theory)

**Status:** Sprint 1-4 Complete

**Main Reference:** [`trinity-pellis-paper/`](trinity-pellis-paper/)

**Key Claim:** Trinity monomials × Pell weights → hybrid scoring for particle physics predictions.

**Documents:**
- [`FORMULA_TABLE.md`](trinity-pellis-paper/FORMULA_TABLE.md) — Formula catalog with trust tiers
- [`hybrid-conjecture.md`](trinity-pellis-paper/hybrid-conjecture.md) — Conjecture H1

**Specs:**
- `specs/physics/pellis-formulas.t27` — Pell numbers P₁…P₅

**Verification:** `tri math compare --pellis --hybrid --sensitivity`

---

## Related Specs

| Spec | Description | Status |
|------|-------------|--------|
| `specs/physics/gamma_conjecture.t27` | GI1: γ = φ⁻³ conjecture | 🟡 CONJECTURAL |
| `specs/physics/pellis-formulas.t27` | Pell numbers P₁…P₅ | ✅ CHECKPOINT |
| `specs/physics/lqg_entropy.t27` | LQG entropy analysis | — |
| `specs/math/sacred_physics.t27` | Sacred physics definitions (includes γ = φ⁻³) | — |

## Citation

When using this research:

```bibtex
@misc{trinity_gamma_2026,
  title={Trinity γ-Paper: Barbero-Immirzi Parameter from Golden Section},
  author={{Trinity Programme Contributors}},
  year={2026},
  note={Draft v0.1},
  url={https://github.com/gHashTag/t27/tree/master/research/trinity-gamma-paper}
}

@misc{trinity_pellis_2026,
  title={Trinity Pellis-Paper: Hybrid Theory for Particle Physics},
  author={{Trinity Programme Contributors}},
  year={2026},
  note={Sprint 1-4 Complete},
  url={https://github.com/gHashTag/t27/tree/master/research/trinity-pellis-paper}
}
```

---

**Last updated:** 2026-04-08
