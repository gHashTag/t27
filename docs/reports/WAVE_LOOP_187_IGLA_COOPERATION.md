# Wave Loop 187 — Cooperation Variants for W188

**Date:** 2026-06-16
**Next Pool:** B — {systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm}
**Next Wave Target:** +16 tests, +0–2 competitors, 570/570 PASS

---

## Variant 1 — IGLA CODER Tokenizer MVP

**Goal:** Close the P0 critical gap by adding a minimal BPE tokenizer spec with merge rules.

**Actions:**
1. Extend `tokenizer.t27` with `bpe_merge_pair()` function and 2 boundary tests (ASCII merge + unknown char fallback).
2. Add 2 tests in `embedder.t27` verifying that sacred opcodes 0xDE–0xE8 map to unique embedding indices.
3. Verify 570/570 PASS after spec edits.
4. Seal modified files.

**Deliverables:**
- `tokenizer.t27` with BPE merge rules
- `embedder.t27` with sacred-opcode index uniqueness tests
- Updated seals
- W188 IGLA report with CODER gap-closure tracker

**Effort:** Medium.
**Risk:** Medium — tokenizer spec may require `utf8.t27` dependency for multi-byte support.

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
- W188 IGLA report

**Effort:** Medium (standard IGLA cadence).
**Risk:** Low.

---

## Variant 3 — Competitive Plateau Intelligence + Benchmark Registry Hygiene

**Goal:** Use the stable 9-wave competitive plateau to audit benchmark registry completeness.

**Actions:**
1. Audit `benchmark.t27` for missing `differentiation` fields on MEDIUM+ competitors.
2. Add 2 tests in `benchmark.t27` verifying that all HIGH+ competitors have non-empty `differentiation` and `threat_level` fields.
3. Add 2 tests in `eval.t27` scoring a mock generated RTL module against `rtl_emit_verilog()` oracle.
4. Run full suite; 570/570 PASS.

**Deliverables:**
- Benchmark registry hygiene tests
- Eval-to-RTL integration tests
- W188 report with plateau + registry analysis

**Effort:** Medium.
**Risk:** Low (uses existing oracles).

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — Tokenizer MVP | Medium | Medium | Very High | If CODER is priority |
| 2 — Pool B + Depth Push | Medium | Low | High | **(Recommended)** |
| 3 — Registry Hygiene | Medium | Low | Medium | If benchmark audit needed |

---

## Default Recommendation

**Variant 2 (Pool B + Depth Push Hybrid)** is the standard cadence continuation:
- It maintains the invariant-coverage momentum.
- It preserves the 570/570 PASS target with low risk.
- It leaves headroom for a CODER-focused variant in W189 or W190 when a P0 breakthrough is ready.

**φ² + 1/φ² = 3 | TRINITY**
