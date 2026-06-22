# Wave Loop 189 — Cooperation Variants for W190

**Date:** 2026-06-16
**Next Pool:** B — {systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm}
**Next Wave Target:** +16 tests, +0–2 competitors, 570/570 PASS

---

## Variant 1 — IGLA CODER Forward Pass MVP

**Goal:** Close the P0 critical gap by adding a minimal attention forward-pass spec.

**Actions:**
1. Extend `forward.t27` with `coder_attention_single_head()` function and 2 boundary tests (Q=K=V zero, causal mask upper-triangular).
2. Add 2 tests in `inference.t27` verifying that `generate_token()` returns a valid vocab index for a 32K vocabulary.
3. Verify 570/570 PASS after spec edits.
4. Seal modified files.

**Deliverables:**
- `forward.t27` with single-head attention spec
- `inference.t27` with vocab-boundary tests
- Updated seals
- W190 IGLA report with CODER gap-closure tracker

**Effort:** Medium.
**Risk:** Medium — forward pass spec may require KV-cache dependency.

---

## Variant 2 — Pool B IGLA CODER+RACE + Invariant Depth Push Hybrid

**Goal:** Combine standard +16 IGLA tests with property-depth push in one Pool B spec.

**Actions:**
1. Add +2 tests to each Pool B spec (systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm) = +16 tests.
2. Select one Pool B spec with double-inv or triple-inv status and add 1 hepta-invariant (7-property chain).
3. Verify 570/570 PASS after both changes.
4. Seal all modified files.

**Deliverables:**
- +16 tests across 8 Pool B specs
- +1 invariant depth upgrade in one Pool B spec
- Updated seal files
- W190 IGLA report

**Effort:** Medium (standard IGLA cadence).
**Risk:** Low.

---

## Variant 3 — Competitive Plateau Deep-Dive + IGLA CODER Eval Harness

**Goal:** Use the stable 11-wave competitive plateau to advance the eval harness with real benchmark tasks.

**Actions:**
1. Audit `eval.t27` for gap between conceptual tests and actual HumanEval-style coding tasks.
2. Add 2 integration tests in `eval.t27` that score a mock generated Verilog module against `rtl_emit_verilog()` oracle.
3. Add 2 tests in `benchmark.t27` verifying that all HIGH+ competitors have non-empty `differentiation` fields.
4. Run full suite; 570/570 PASS.

**Deliverables:**
- Eval-to-RTL integration tests
- Benchmark registry hygiene tests
- W190 report with plateau + eval analysis

**Effort:** Medium.
**Risk:** Low (uses existing oracles).

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — Forward Pass MVP | Medium | Medium | Very High | If CODER is priority |
| 2 — Pool B + Depth Push | Medium | Low | High | **(Recommended)** |
| 3 — Eval Harness | Medium | Low | Medium | If eval maturity needed |

---

## Default Recommendation

**Variant 2 (Pool B + Depth Push Hybrid)** is the standard cadence continuation:
- It maintains the invariant-coverage momentum.
- It preserves the 570/570 PASS target with low risk.
- It leaves headroom for a CODER-focused variant in W191 or W192 when a P0 breakthrough is ready.

**φ² + 1/φ² = 3 | TRINITY**
