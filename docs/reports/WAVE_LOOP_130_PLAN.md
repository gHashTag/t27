# Wave Loop 130 — Decomposed Plan

**Date:** 2026-06-16  
**Issue:** #1054  
**Branch:** `trinity-rust-rings`  
**Target:** 564 specs | 564/564 PASS | L1-L7 compliance

---

## 1. Executive Summary

Wave Loop 130 continues the proportional-growth strategy: identify weakest IGLA CODER / IGLA RACE specs via test+bench count sort, add exactly 2 tests per weak spec, and track 2 new competitors. Zero regressions, zero TODOs, zero broken tri stubs.

---

## 2. Weakness Audit Results

Sorted ascending by `^test ` + `^bench ` line count across `specs/igla/**/*.t27`:

| Spec | Previous | After W130 |
|------|----------|------------|
| `systolic_array.t27` | 14 | **16** |
| `systolic_ternary.t27` | 14 | **16** |
| `ternary_mac.t27` | 14 | **16** |
| `adder_tree.t27` | 15 | **17** |
| `opcodes.t27` | 15 | **17** |
| `yosys.t27` | 15 | **17** |
| `ternary_gemm.t27` | 16 | **18** |
| `backend.t27` | 17 | **19** |

**Total new tests:** +16 across 8 specs.

---

## 3. Competitive Intelligence Targets

Two new competitors identified during literature sweep:

1. **bitSMM** — arXiv:2603.14988v1 (March 2026): Bit-serial systolic MM, ASAP7, 73.22 GOPS. **HIGH** hardware threat.
2. **Abraxas1010 / asymptotic-safety-lean** — GitHub (2026): Lean 4 formalization of asymptotic safety with SM predictions, 0 sorry. **EXTREME** formal-verification threat.

---

## 4. Tracks

### Track A: Test Expansion (8 specs)
- `systolic_array.t27`: `systolic_gemm_2x2_negative_activation`, `systolic_gemm_2x2_zero_row`
- `systolic_ternary.t27`: `systolic_pe_max_accumulate`, `systolic_pe_zero_weight_update`
- `ternary_mac.t27`: `ternary_dot_all_minus_one`, `ternary_mac_large_negative`
- `adder_tree.t27`: `adder_tree_8_all_negative`, `adder_tree_4_mixed_large`
- `opcodes.t27`: `opcode_name_unknown`, `get_opcode_cycles_unknown`
- `yosys.t27`: `emit_sva_assertions_empty`, `aggregate_coverage_only_admitted`
- `ternary_gemm.t27`: `get_elem_4x4_oob`, `ternary_gemm_4x4_zero_row`
- `backend.t27`: `parse_const_binary`, `is_power_of_two_const_zero`

### Track B: Competitor Tracking
- Add `bitsmm_competitor()` and `abraxas1010_competitor()` to `specs/igla/coder/benchmark.t27`
- Update `docs/COMPETITIVE_POSITIONING.md`

### Track C: Verification
- Regenerate seals for 9 modified specs
- Run `./target/release/t27c suite --repo-root .` → 564/564 PASS
- Zero cascade seal mismatches

### Track D: Documentation
- Write `WAVE_LOOP_130_REPORT.md`
- Write `WAVE_LOOP_130_COOPERATION.md` (3 variants)
- Save memory to `wave-loop-130.md`

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
