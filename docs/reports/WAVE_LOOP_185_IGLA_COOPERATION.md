# Wave Loop 185 — Cooperation Variants for W186

**Date:** 2026-06-18
**Next Pool:** B — {systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm}
**Next Wave Target:** +16 tests, +1–2 competitors, 570/570 PASS

---

## Variant 1 — IGLA CODER Inference Engine MVP

**Goal:** Close the P0 critical gap in IGLA CODER by adding a minimal working inference pipeline spec.

**Actions:**
1. Extend `pipeline.t27` with a concrete `coder_inference_mvp()` function that stitches tokenizer → forward → decode into one callable flow.
2. Add 2 integration tests in `pipeline.t27`: one for empty prompt (boundary), one for sacred-opcode prompt (embedding constraint).
3. Add 2 tests in `tokenizer.t27` testing multi-byte token boundary (simulating BPE merge of two ASCII chars into one ID > 256).
4. Verify 570/570 PASS after spec edits.
5. Seal modified files.

**Deliverables:**
- `pipeline.t27` with MVP inference flow spec
- `tokenizer.t27` with multi-byte token tests
- Updated seals
- W186 IGLA report with IGLA CODER gap-closure tracker

**Effort:** Medium.
**Risk:** Spec-only; still needs runtime backend (Burn/candle) for real execution.

---

## Variant 2 — Pool B IGLA CODER+RACE + Invariant Depth Push Hybrid

**Goal:** Combine standard +16 IGLA tests with property-depth push in one Pool B spec.

**Actions:**
1. Add +2 tests to each Pool B spec (systolic_array, systolic_ternary, ternary_mac, adder_tree, opcodes, yosys, backend, ternary_gemm) = +16 tests.
2. Select one Pool B spec with single-inv or double-inv status and add 1 hepta-invariant (7-property chain) or upgrade a double-inv to triple-inv.
3. Verify 570/570 PASS after both changes.
4. Seal all modified files.

**Deliverables:**
- +16 tests across 8 Pool B specs
- +1 invariant depth upgrade in one Pool B spec
- Updated seal files
- W186 IGLA report

**Effort:** Medium (standard IGLA cadence).
**Risk:** Low.

---

## Variant 3 — Stable Plateau Intelligence + IGLA CODER PRM Integration

**Goal:** The competitive maturation plateau has lasted 7 consecutive IGLA waves (W175–W185). Use this stability window to advance IGLA CODER PRM (Process Reward Model) integration.

**Actions:**
1. Audit `prm.t27` for reward signals that can be validated by existing `rtl.t27` / `formal.t27` oracles.
2. Add 2 integration tests in `prm.t27` that call `reward_syntax()` with Verilog snippets and verify R-SI-1 compliance via `count_mul_ops()`.
3. Add 2 tests in `eval.t27` that score a mock generated RTL module against the benchmark competitor registry.
4. Run full suite; 570/570 PASS.

**Deliverables:**
- PRM-to-R-SI-1 integration tests
- Eval-to-competitor-registry tests
- W186 report with plateau + PRM analysis

**Effort:** Medium.
**Risk:** Low (uses existing oracles).

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — IGLA CODER Inference MVP | Medium | Medium | Very High | If CODER is priority |
| 2 — Pool B + Depth Push | Medium | Low | High | **(Recommended)** |
| 3 — PRM Integration | Medium | Low | High | If PRM maturity needed |

---

*φ² + 1/φ² = 3 | Cooperation over conquest | Verification pending*
