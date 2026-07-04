# Wave Loop 147 Plan

## OBSERVE Summary

- **Branch**: trinity-rust-rings
- **Last commit**: W146 avg 2.205 → 2.237, Coq zero-Admitted milestone
- **Invariant metrics**: 146 single-inv, 122 two-inv, 78 three+, avg 2.268
- **Seals**: 0 mismatches
- **Clippy**: 0 warnings
- **Coq Axioms**: 6 total (H4GaugeEmbedding 1, Koide 1, NeutrinoMasses 4)
- **GitHub issues**: auth unavailable (401); known open #1041, #1183, #1184
- **Suite**: 570/570 PASS

## Weaknesses Discovered

1. **Property depth plateau**: 146 specs still single-inv. Avg 2.268. Target 2.35+.
2. **Coq Axiom count**: 6 Axioms remain (4 in NeutrinoMasses.v, 1 Koide.v, 1 H4GaugeEmbedding.v). These are documented but weaken formalization strength.
3. **GitHub auth broken**: `gh` CLI returns 401 despite GH_TOKEN env var being set.
4. **Competitive intel**: No new EXTREME competitors in mid-June 2026. Maturation plateau continues.

## Decomposed Tasks

### Track A: Property Depth Push (+25 second invariants)
- Target 25 single-inv specs from remaining 146
- Domains: tri/collections (3), tri/utils (2), tri/search (2), tri/agent (2), tri/trees (2), tri/sort (2), tri/pipeline (2), tri/encoding (2), tri/math (2), tri/graph (1), ml/activation (3), ml/layers (2), igla/race (2), igla/coder (2), compiler (1), math (1), server (1), brain (1), sandbox (1), storage (1), physics (1), sacred (1)
- Expected: single-inv 146 → 121, two-inv 122 → 147, avg 2.268 → 2.30

### Track B: Coq Axiom Assessment
- Audit 6 Axioms: determine which can be promoted to Qed vs must remain Axioms
- Focus: NeutrinoMasses.v 4 Axioms (experimental-input dependent?)
- H4GaugeEmbedding.v phi_irrational_over_Q: requires Coq irrationality library
- Koide.v: 1 Axiom — check if deducible from existing lemmas

### Track C: GitHub Issue Auth Fix
- Debug `gh auth status` and `GH_TOKEN` behavior
- If fixable, attempt closure of #1183 (wp18 gate) or #1184 (GF rungs)

### Track D: Verification & Reporting
- 570/570 PASS, 0 seal mismatches, 0 clippy
- English-only W147 report/cooperation/skill update
- Update MEMORY.md

## Target Metrics

| Metric | Baseline (W146) | Target (W147) |
|---|---|---|
| Single-inv files | 146 | **121** |
| Two-inv files | 122 | **147** |
| Avg invariants/spec | 2.268 | **~2.30** |
| Coq Axioms | 6 | ≤6 (assess closure feasibility) |
| Suite PASS | 570/570 | 570/570 |
| Open GitHub issues | ~3 | ≤3 |
| Clippy warnings | 0 | 0 |

## Risk Assessment
- **Low**: Property depth — batch script proven reliable
- **Medium**: Coq Axiom promotion — may require new lemmas
- **High**: GitHub auth — may remain blocked
- **Low**: Competitive landscape — stable maturation plateau

---

Phase complete: PLAN
→ Phase 3: DELEGATE
