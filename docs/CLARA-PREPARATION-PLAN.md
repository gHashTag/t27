# CLARA Preparation Plan — DARPA TA1/TA2 Submission

**Ring:** 039 | **Issue:** #134 | **Phase:** HARDEN
**Solicitation:** DARPA CLARA (2026-02-10, proposals due 2026-04-17)
**Last updated:** 2026-04-07

---

## Overview

This document maps T27 Trinity S³AI specifications to DARPA CLARA (Common Learning Repository for AI) TA1 and TA2 deliverables. It serves as the submission checklist for the April 17, 2026 deadline.

**CLARA Focus:**
- **TA1 (Technical Area 1):** Argumentation & Reasoning (AR) formal specifications with bounded proof traces
- **TA2 (Technical Area 2):** Composition library for ML+AR hybrid systems

---

## TA1 Checklist — Argumentation & Reasoning

### CLARA TA1 Requirements vs T27 Mapping

| CLARA Requirement | T27 Spec | Ring | Status | Evidence |
|-------------------|----------|------|--------|----------|
| Ternary logic with K3 semantics | `specs/ar/ternary_logic.t27` | 18 | ✅ Sealed | Kleene K3 truth tables, logical operations (AND, OR, NOT, IMPLIES) |
| Bounded proof traces <=10 steps | `specs/ar/proof_trace.t27` | 19 | ✅ Sealed | `MAX_STEPS = 10`, DerivationStep with step limits |
| Forward-chaining Datalog O(n) | `specs/ar/datalog_engine.t27` | 20 | ✅ Sealed | HornClause, DatalogEngine with linear-time inference |
| Bounded rationality/restraint | `specs/ar/restraint.t27` | 21 | ✅ Sealed | RestraintParams, should_continue() with step bounds |
| XAI explanations <=10 steps | `specs/ar/explainability.t27` | 22 | ✅ Sealed | Explanation format, Summary with MAX_STEPS constraint |
| ASP with NAF | `specs/ar/asp_solver.t27` | 23 | ✅ Sealed | AspRule, StableModel, NAF support via negation |
| ML+AR composition patterns | `specs/ar/composition.t27` | 24 | ✅ Sealed | CompositionPattern enum (CNN_RULES, MLP_BAYESIAN, etc.) |

### TA1 Formal Specs Detail

#### AR-001: Ternary Logic (`ternary_logic.t27`)
- **Kleene K3 Truth Values:** `K_FALSE = .neg`, `K_UNKNOWN = .zero`, `K_TRUE = .pos`
- **Operations:** `k3_and`, `k3_or`, `k3_not`, `k3_implies`, `k3_equiv`
- **Complexity:** O(1) per operation
- **Test Coverage:** 19 test cases in `.test` block

#### AR-002: Proof Trace (`proof_trace.t27`)
- **MAX_STEPS:** 10 (enforced constraint)
- **Data Structures:** `ProofTrace`, `DerivationStep`, `RuleApplication`
- **Verification:** `verify_trace()` ensures step limit compliance
- **Test Coverage:** 8 test cases

#### AR-003: Datalog Engine (`datalog_engine.t27`)
- **Horn Clauses:** `HornClause` struct with head/body
- **Forward Chaining:** `derive()` with O(n) complexity claim
- **Fact/Rule Separation:** Clear distinction in data model
- **Test Coverage:** 12 test cases

#### AR-004: Restraint (`restraint.t27`)
- **Bounded Rationality:** `RestraintParams` with step limits, quality thresholds
- **Decision Function:** `should_continue()` returns K_FALSE when bounds exceeded
- **Quality Metrics:** phi-based scoring for argument quality
- **Test Coverage:** 19 test cases

#### AR-005: Explainability (`explainability.t27`)
- **XAI Formats:** Natural language, Fitch-style, Compact
- **Step Limit:** Explanations capped at MAX_STEPS = 10
- **Summary:** Auto-generated synthesis with key conclusions
- **Test Coverage:** 15 test cases

#### AR-006: ASP Solver (`asp_solver.t27`)
- **NAF Support:** Negation as failure via `not` operator
- **Stable Models:** `compute_stable_model()` returns consistent assignments
- **Rule Extensions:** AspRule extends HornClause with NAF
- **Test Coverage:** 11 test cases

#### AR-007: Composition (`composition.t27`)
- **Patterns:** CNN_RULES, MLP_BAYESIAN, TRANSFORMER_XAI, RL_GUARDRAILS
- **ML/AR Abstraction:** `MLComponent`, `ARComponent`, `ComposedPipeline`
- **Execution:** `compose_and_execute()` orchestrates hybrid inference
- **Test Coverage:** 9 test cases

### TA1 Test Vector Summary

| Spec | Test Cases | Invariants | Benchmarks |
|------|------------|------------|------------|
| ternary_logic.t27 | 19 | 3 | 2 |
| proof_trace.t27 | 8 | 2 | 1 |
| datalog_engine.t27 | 12 | 3 | 2 |
| restraint.t27 | 19 | 4 | 3 |
| explainability.t27 | 15 | 3 | 2 |
| asp_solver.t27 | 11 | 2 | 1 |
| composition.t27 | 9 | 2 | 2 |
| **TOTAL** | **93** | **19** | **13** |

---

## TA2 Checklist — Composition Library (VSA)

### CLARA TA2 Requirements vs T27 Mapping

| CLARA Requirement | T27 Spec | Ring | Status | Evidence |
|-------------------|----------|------|--------|----------|
| Hypervector operations (bind/unbind) | `specs/vsa/ops.t27` | N/A | ✅ Sealed | `bind()`, `unbind()`, VSA_DIM = 1024 |
| Similarity metrics | `specs/vsa/ops.t27` | N/A | ✅ Sealed | SIM_COSINE, SIM_HAMMING, SIM_DOT |
| Bundle operations | `specs/vsa/ops.t27` | N/A | ✅ Sealed | `bundle2()`, `bundle3()` |
| ML+AR composition | `specs/ar/composition.t27` | 24 | ✅ Sealed | CompositionPattern, ComposedPipeline |
| Performance benchmarks | `specs/vsa/ops.t27` | N/A | ✅ Sealed | `.bench` block with VSA operations |

### TA2 VSA Operations Detail

#### VSA Core Operations (`vsa/ops.t27`)
- **Dimension:** VSA_DIM = 1024 (configurable)
- **Bind:** XOR-like associative binding for role-value pairs
- **Unbind:** Inverse of bind using XOR properties
- **Bundle:** Superposition (addition) for set representation
- **Similarity:** Cosine, Hamming, and dot product metrics
- **Test Coverage:** 14 test cases + 5 benchmarks

#### Composition Patterns (`ar/composition.t27`)
- **CNN+Rules:** Neural features → AR rule evaluation
- **MLP+Bayesian:** Neural forward pass → Bayesian inference
- **Transformer+XAI:** Self-attention → ≤10 step explanations
- **RL+Guardrails:** Policy network → AR constraint checking

### TA2 Benchmark Targets

| Operation | Target Throughput | Current | Status |
|-----------|-------------------|---------|--------|
| bind (1024-dim) | >1M ops/sec | TBD | 🔬 To measure |
| bundle2 | >500K ops/sec | TBD | 🔬 To measure |
| similarity (cosine) | >200K ops/sec | TBD | 🔬 To measure |

---

## License Compliance

### Current License
- **T27 License:** MIT License
- **File:** `LICENSE` (repository root)
- **Key Provisions:** Permission notice required, no warranty, no liability

### CLARA Requirement
- **Required License:** Apache 2.0
- **Rationale:** CLARA requires patent grants and broader redistribution rights

### License Gap Analysis

| Requirement | MIT | Apache 2.0 | Gap |
|-------------|-----|------------|-----|
| Patent Grant | ❌ No | ✅ Yes | Yes |
| Redistribution | ✅ Yes | ✅ Yes | No |
| Attribution | ✅ Yes | ✅ Yes | No |
| Copyleft | ❌ No | ❌ No | No |
| Warranty Disclaimer | ✅ Yes | ✅ Yes | No |

### Resolution Path

**Option 1: Re-license to Apache 2.0 (Recommended)**
- Requires consent from all contributors
- Mitigates CLARA review risk
- Aligns with industry standard for ML/AI research

**Option 2: Dual License (MIT + Apache 2.0)**
- Allows existing MIT users to continue
- Provides Apache 2.0 for CLARA compliance
- More complex compliance matrix

**Option 3: CLARA Waiver Request**
- Submit with current MIT license
- Include justification for MIT sufficiency
- Risk: CLARA may reject or request revision

**Recommended Action:** Pursue Option 1 — re-license to Apache 2.0 before April 17, 2026

---

## Submission Timeline

### Milestones (April 2026)

| Date | Milestone | Owner | Status |
|------|-----------|-------|--------|
| Apr 01 | TA1 spec validation complete | T27 Team | ✅ Done |
| Apr 03 | TA2 composition library validated | T27 Team | ✅ Done |
| Apr 05 | License compliance assessment | T27 Team | ✅ Done |
| Apr 08 | Apache 2.0 relicense decision | T27 Team | 🔄 Pending |
| Apr 10 | Draft proposal package assembled | T27 Team | ⏳ TODO |
| Apr 12 | Internal review & revision | T27 Team | ⏳ TODO |
| Apr 14 | Final package preparation | T27 Team | ⏳ TODO |
| Apr 15 | Submission to CLARA portal | T27 Team | ⏳ TODO |
| Apr 17 | **DEADLINE** — Final submission | DARPA | — |

### Critical Path

```
License Decision ──► Package Assembly ──► Internal Review ──► Final Submission
     (Apr 08)              (Apr 10)             (Apr 12)          (Apr 15)
```

---

## Deliverables Checklist

### TA1 Package Contents

- [x] Formal spec for ternary logic (K3 semantics)
- [x] Proof trace specification with ≤10 step bound
- [x] Datalog engine spec with O(n) complexity claim
- [x] Restraint module spec with bounded rationality
- [x] XAI module spec with ≤10 step explanation limit
- [x] ASP solver spec with NAF support
- [x] Composition library spec (ML+AR patterns)
- [ ] Test vectors for all 7 AR specs (ZIP archive)
- [ ] Conformance JSON for TA1 requirements
- [ ] Technical narrative (≤10 pages)

### TA2 Package Contents

- [x] VSA operations spec (bind/unbind/bundle)
- [x] Similarity metrics spec
- [x] Composition patterns spec
- [ ] Performance benchmarks (results + methodology)
- [ ] Example composition scripts (3+ examples)
- [ ] Integration guide (VSA + AR + ML)

### Common Package Contents

- [ ] Abstract (≤250 words)
- [ ] Technical narrative (≤15 pages total)
- [ ] Team qualifications summary
- [ ] Relevant publications (arXiv preprints)
- [ ] Code repository link (public access)
- [ ] License file (Apache 2.0 recommended)

---

## Dependencies

### Hard Dependencies
- Ring 035 (#130) — TECHNOLOGY-TREE.md ✅ Complete
- Rings 18-24 — AR specs (all sealed) ✅ Complete
- VSA specs (sealed) ✅ Complete

### Soft Dependencies
- Ring 041 (#136) — GoldenFloat arXiv paper (strengthening)
- Ring 037 (#132) — Parser enforcement (validation tooling)

---

## References

- **CLARA Solicitation:** DARPA 2026-02-10 (public link TBD)
- **T27 Repository:** https://github.com/gHashTag/t27
- **Trinity S³AI:** https://github.com/gHashTag/trinity
- **Issue Tracker:** #134 (this ring), #130 (dependency)
- **Technology Tree:** docs/TECHNOLOGY-TREE.md (Ring 035)

---

**Document Authority:** L1 TRACEABILITY, L4 TESTABILITY
**φ² + 1/φ² = 3 | TRINITY**
