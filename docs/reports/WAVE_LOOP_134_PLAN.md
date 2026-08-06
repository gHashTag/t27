# Wave Loop 134 — Decomposed Plan

**Date:** 2026-06-16
**Issue:** #1058
**Branch:** `trinity-rust-rings`
**Target:** 564 specs | 564/564 PASS | L1-L7 compliance

---

## 1. Executive Summary

Wave Loop 134 continues the proportional-growth strategy: identify weakest IGLA CODER / IGLA RACE specs via test+bench count sort, add exactly 2 tests per weak spec, and track 2 new competitors. Zero regressions, zero TODOs, zero broken tri stubs.

---

## 2. Weakness Audit Results

Sorted ascending by `^test ` + `^bench ` line count across `specs/igla/**/*.t27`, excluding specs modified in W133:

| Spec | Previous | After W134 |
|------|----------|------------|
| `systolic_array.t27` | 18 | **20** |
| `systolic_ternary.t27` | 18 | **20** |
| `ternary_mac.t27` | 18 | **20** |
| `adder_tree.t27` | 19 | **21** |
| `opcodes.t27` | 19 | **21** |
| `yosys.t27` | 19 | **21** |
| `backend.t27` | 21 | **23** |
| `ternary_gemm.t27` | 21 | **23** |

**Total new tests:** +16 across 8 specs.

---

## 3. Competitive Intelligence Targets

Two new competitors added to benchmark code (already documented in COMPETITIVE_POSITIONING.md):

1. **Gray** — arXiv:2604.00255v1 (March 2026): H4/E8 geometric unification, narrative derivation. **HIGH** threat.
2. **Teli & Singh** — arXiv:2605.24866 (May 2026): Fermion mass hierarchies from exceptional Jordan algebra J3(O). **HIGH** threat.

---

## 4. Tracks

### Track A: Test Expansion (8 specs)
- `systolic_array.t27`: `systolic_gemm_2x2_transpose`, `systolic_step_first_iteration`
- `systolic_ternary.t27`: `systolic_pe_negative_activation`, `systolic_ternary_array_zero_size`
- `ternary_mac.t27`: `ternary_mac_max_activation`, `ternary_dot_zero_elements`
- `adder_tree.t27`: `adder_tree_4_identity_nonzero`, `adder_tree_8_all_equal`
- `opcodes.t27`: `validate_chain_empty`, `opcode_name_sacred_boundary`
- `yosys.t27`: `emit_sva_assertions_multiple_properties`, `aggregate_coverage_partial_proof`
- `backend.t27`: `parse_const_dec`, `is_power_of_two_const_one`
- `ternary_gemm.t27`: `get_elem_4x4_normal_access`, `ternary_gemm_4x4_zero_input`

### Track B: Competitor Tracking
- Add `gray_competitor()` and `teli_singh_competitor()` to `specs/igla/coder/benchmark.t27`
- Update `docs/COMPETITIVE_POSITIONING.md` competitor count

### Track C: Verification
- Regenerate seals for 9 modified specs
- Run `./target/release/t27c suite --repo-root .` → 564/564 PASS
- Zero cascade seal mismatches

### Track D: Documentation
- Write `WAVE_LOOP_134_PLAN.md`
- Write `WAVE_LOOP_134_REPORT.md`
- Write `WAVE_LOOP_134_COOPERATION.md` (3 variants)
- Save memory to `wave-loop-134.md`

---

## 5. Acceptance Criteria

- [x] 16 new tests added across 8 specs
- [x] 2 new competitors added to benchmark.t27
- [x] COMPETITIVE_POSITIONING.md updated
- [x] 9 seals regenerated with no cascade mismatches
- [x] 564/564 PASS
- [x] Report + Cooperation variants committed
- [x] Memory saved

**φ² + 1/φ² = 3 | TRINITY**
