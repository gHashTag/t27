# Research Papers — Trinity Programme

This directory contains research papers and supporting documents for the Trinity S³AI programme.

## Papers

### Trinity γ-Paper (Barbero-Immirzi Parameter)

**Status:** Draft v0.1 — **core identification REJECTED in-corpus, see note below**

**Main Paper:** [`trinity-gamma-paper/GAMMA_PAPER_DRAFT_v0.1.md`](trinity-gamma-paper/GAMMA_PAPER_DRAFT_v0.1.md)

**Key Claim:** γ_φ = φ⁻³ = √5 − 2 differs from standard LQG γ₁ by only **0.63%**, while the internal LQG dispute between γ₁ and γ₂ is **13.9%**.

**Note, 2026-08-12:** the identification of φ⁻³ with the Barbero-Immirzi
parameter was rejected in `DELTA-001.md` (sibling `trinity` repo,
`docs/DELTA-001.md`) on 2026-03-28, status FALSIFIED: 0.236068 vs canonical
0.237533, **+0.617%**, source Rovelli & Vidotto, *Covariant Loop Quantum
Gravity* (2014). φ⁻³ is retained as **notation**; the physical identification is
withdrawn. Two things the Key Claim above leaves out: (i) the rejection predates
this page (dated 2026-04-08) and was not carried across; (ii) 0.617% still
exceeds the <0.1% tolerance DELTA-001 names for a fundamental-constant
prediction, so "differs by only 0.63%" is a statement about proximity, not about
agreement. The arithmetic in the Key Claim is otherwise sound — φ⁻³ = √5 − 2
exactly.

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
| `specs/physics/gamma_conjecture.t27` | GI1: γ = φ⁻³ conjecture | ~~🟡 CONJECTURAL~~ ❌ **REJECTED** — DELTA-001, 2026-03-28 |
| `specs/physics/pellis-formulas.t27` | Pell numbers P₁…P₅ | ✅ CHECKPOINT |
| `specs/physics/lqg_entropy.t27` | LQG entropy analysis | — |
| `specs/math/sacred_physics.t27` | Sacred physics definitions (includes γ = φ⁻³ — notation retained, physical identification withdrawn) | — |

## Citation

When using this research:

```bibtex
@misc{trinity_gamma_2026,
  title={Trinity γ-Paper: Barbero-Immirzi Parameter from Golden Section},
  author={{Trinity Programme Contributors}},
  year={2026},
  note={Draft v0.1. Core identification REJECTED in-corpus: DELTA-001.md
        (sibling trinity repo, docs/DELTA-001.md), 2026-03-28, status
        FALSIFIED -- 0.236068 vs canonical 0.237533, +0.617\%, source
        Rovelli \& Vidotto, Covariant Loop Quantum Gravity (2014).
        The title above is the draft's own title and is retained as
        notation; the physical identification is withdrawn.},
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
