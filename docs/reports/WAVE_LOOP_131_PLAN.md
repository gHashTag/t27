# Wave Loop 131 — Decomposed Plan

**Date:** 2026-06-16
**Issue:** #1055
**Branch:** `trinity-rust-rings`
**Target:** 564 specs | 564/564 PASS | L1-L7 compliance

---

## 1. Executive Summary

Wave Loop 131 continues the proportional-growth strategy: identify weakest IGLA CODER / IGLA RACE specs via test+bench count sort, add exactly 2 tests per weak spec, and track 2 new competitors. Zero regressions, zero TODOs, zero broken tri stubs.

---

## 2. Weakness Audit Results

Sorted ascending by `^test ` + `^bench ` line count across `specs/igla/**/*.t27`, excluding specs already strengthened in W128–W130:

| Spec | Previous | After W131 |
|------|----------|------------|
| `rtl.t27` | 15 | **17** |
| `eda.t27` | 16 | **18** |
| `cordic_fixed.t27` | 16 | **18** |
| `bram_weights.t27` | 16 | **18** |
| `cordic.t27` | 16 | **18** |
| `cordic_top.t27` | 16 | **18** |
| `formal.t27` | 16 | **18** |
| `gemm.t27` | 16 | **18** |

**Total new tests:** +16 across 8 specs.

---

## 3. Competitive Intelligence Targets

Two new competitors identified during literature sweep:

1. **BiKA** — arXiv:2602.23455v1 (February 2026): Systolic FPGA multiply-free KAN accelerator. **HIGH** hardware threat.
2. **GIFT** — GitHub gift-framework/GIFT (2026): Lean 4, 33 SM predictions, 0 sorry. **EXTREME** formal-verification threat.

---

## 4. Tracks

### Track A: Test Expansion (8 specs)
- `rtl.t27`: `emit_verilog_contains_assign`, `count_mul_ops_nested_paren`
- `eda.t27`: `compute_ppa_score_zero_area`, `parse_synthesis_log_no_area_keyword`
- `cordic_fixed.t27`: `cordic_fixed_sin_zero`, `cordic_fixed_cos_zero`
- `bram_weights.t27`: `read_weight_first_cell`, `flatten_addr_oob_returns_max`
- `cordic.t27`: `cordic_sin_zero`, `cordic_cos_zero`
- `cordic_top.t27`: `cordic_top_reset_state`, `cordic_top_input_max_q15`
- `formal.t27`: `prove_equivalence_different_ports`, `generate_report_full_coverage`
- `gemm.t27`: `booth_mul_u32_zero_lhs`, `gemm_2x2_identity`

### Track B: Competitor Tracking
- Add `bika_competitor()` and `gift_competitor()` to `specs/igla/coder/benchmark.t27`
- Update `docs/COMPETITIVE_POSITIONING.md`

### Track C: Verification
- Regenerate seals for 9 modified specs
- Run `./target/release/t27c suite --repo-root .` → 564/564 PASS
- Zero cascade seal mismatches

### Track D: Documentation
- Write `WAVE_LOOP_131_REPORT.md`
- Write `WAVE_LOOP_131_COOPERATION.md` (3 variants)
- Save memory to `wave-loop-131.md`

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
