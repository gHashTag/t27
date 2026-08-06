# Wave Loop 186 — Cooperation Variants for W187

**Date:** 2026-06-16
**Next Pool:** A — {rtl, eda, cordic_fixed, bram_weights, cordic, cordic_top, formal, gemm}
**Next Wave Target:** +16 tests, +0–2 competitors, 570/570 PASS

---

## Variant 1 — IGLA CODER Tokenizer + Embedder Integration

**Goal:** Close the P0 gap by adding a minimal BPE tokenizer spec and wiring it to the sacred-opcode embedder.

**Actions:**
1. Extend `tokenizer.t27` with `bpe_merge_pair()` function and 2 boundary tests (ASCII merge + unknown char fallback).
2. Add 2 tests in `embedder.t27` verifying that sacred opcodes 0xDE–0xE8 map to unique embedding indices.
3. Verify 570/570 PASS after spec edits.
4. Seal modified files.

**Deliverables:**
- `tokenizer.t27` with BPE merge rules
- `embedder.t27` with sacred-opcode index uniqueness tests
- Updated seals
- W187 IGLA report with CODER gap-closure tracker

**Effort:** Medium.
**Risk:** Medium — tokenizer spec may require `utf8.t27` dependency for multi-byte support.

---

## Variant 2 — Pool A IGLA CODER+RACE + Invariant Depth Push Hybrid

**Goal:** Combine standard +16 IGLA tests with property-depth push in one Pool A spec.

**Actions:**
1. Add +2 tests to each Pool A spec (rtl, eda, cordic_fixed, bram_weights, cordic, cordic_top, formal, gemm) = +16 tests.
2. Select one Pool A spec with double-inv or triple-inv status and add 1 hepta-invariant (7-property chain).
3. Verify 570/570 PASS after both changes.
4. Seal all modified files.

**Deliverables:**
- +16 tests across 8 Pool A specs
- +1 invariant depth upgrade in one Pool A spec
- Updated seal files
- W187 IGLA report

**Effort:** Medium (standard IGLA cadence).
**Risk:** Low.

---

## Variant 3 — Competitive Landscape Deep-Dive + IGLA CODER Eval Harness

**Goal:** Use the stable 8-wave competitive plateau to advance the IGLA CODER eval harness with real benchmark tasks.

**Actions:**
1. Audit `eval.t27` for gap between conceptual tests and actual HumanEval-style coding tasks.
2. Add 2 integration tests in `eval.t27` that score a mock Verilog module generation against `rtl_emit_verilog()` oracle.
3. Add 2 tests in `benchmark.t27` verifying that all HIGH+ competitors have non-empty `differentiation` fields.
4. Run full suite; 570/570 PASS.

**Deliverables:**
- Eval-to-RTL integration tests
- Benchmark registry hygiene tests
- W187 report with plateau + eval analysis

**Effort:** Medium.
**Risk:** Low (uses existing oracles).

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — Tokenizer + Embedder | Medium | Medium | Very High | If CODER is priority |
| 2 — Pool A + Depth Push | Medium | Low | High | **(Recommended)** |
| 3 — Eval Harness | Medium | Low | Medium | If eval maturity needed |

---

## Default Recommendation

**Variant 2 (Pool A + Depth Push Hybrid)** is the standard cadence continuation:
- It maintains the invariant-coverage momentum (currently ~100% at hepta layer).
- It preserves the 570/570 PASS target with low risk.
- It leaves headroom for a CODER-focused variant in W188 or W189 when a P0 breakthrough is ready.

**φ² + 1/φ² = 3 | TRINITY**
