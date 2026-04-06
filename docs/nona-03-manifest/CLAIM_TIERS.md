# CLAIM_TIERS — classification of mathematical and physics statements

**Status:** Policy (normative target)  
**Date:** 2026-04-06  
**Language:** English (repository **LANG-EN**)  

**Companions:** **`docs/nona-03-manifest/RESEARCH_CLAIMS.md`** (registry rows), **`docs/nona-03-manifest/T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md`** (architecture), **`docs/nona-02-organism/SACRED-PHYSICS-001.md`**, root **`SOUL.md`**.

---

## Purpose

Every **mathematical** or **physics** statement in product **`specs/math/*`**, **`specs/physics/*`**, and related **`specs/numeric/*`** MUST eventually carry an explicit **`claim_tier`** (or equivalent metadata) so reviewers can distinguish:

- definitions and algebraic theorems,  
- experimentally anchored facts,  
- empirical fits (not derivations),  
- refuted “exact” claims kept for transparency,  
- open conjectures.

**`falsified_as_exact` is not a failure** — it is scientific hygiene (see **PHY-005** / Barbero–Immirzi example in **`conformance/axiom_system.json`**).

---

## Tier vocabulary

| Tier | Meaning | Spec / doc requirements |
|------|---------|---------------------------|
| **`exact_definition`** | Adopted by convention; not falsifiable | `statement`, `proof_type: definition`, no fabricated precision |
| **`exact_algebraic`** | Follows algebraically from definitions | `proof_sketch` or proof link; machine-checkable test where possible |
| **`exact_experimental`** | Established experimentally within stated uncertainty | `experiment_reference`, uncertainty |
| **`empirical_fit`** | Numerically useful; **not** derived from first principles | `error_percent` or tolerance; `codata_source` if comparing to CODATA |
| **`empirical_benchmark`** | Comparative benchmark (e.g. GF16 vs BF16) | Method, corpus, reproducibility pointer |
| **`falsified_as_exact`** | Shown **not** exact; may remain as approximation | `falsification_evidence`, date, recommended tier for use |
| **`conjecture`** | Registered; not proved | `prediction`, `test_window`, falsification path |

---

## Enforcement (target)

1. **`tri lint`** (future) — flag `specs/math/*` and `specs/physics/*` without tier metadata.  
2. **Code review** — block merges that mix **exact** wording with **empirical_fit** evidence.  
3. **`conformance/axiom_system.json`** — machine-readable catalog; grow with each closed ring (see unified system spec).

---

**φ² + 1/φ² = 3 | TRINITY**
