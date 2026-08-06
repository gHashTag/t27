# Wave Loop 135 — Decomposed Plan

**Date:** 2026-06-16
**Issue:** #1059
**Branch:** `trinity-rust-rings`
**Target:** 564 specs | 564/564 PASS | L1-L7 compliance

---

## 1. Executive Summary

Wave Loop 135 continues the proportional-growth strategy: identify weakest IGLA CODER / IGLA RACE specs via test+bench count sort, add exactly 2 tests per weak spec, and track 2 new competitors. Zero regressions, zero TODOs, zero broken tri stubs.

---

## 2. Weakness Audit Results

Sorted ascending by `^test ` + `^bench ` line count across `specs/igla/**/*.t27`, excluding specs modified in W134:

| Spec | Previous | After W135 |
|------|----------|------------|
| `rtl.t27` | 19 | **21** |
| `eda.t27` | 20 | **22** |
| `cordic_fixed.t27` | 20 | **22** |
| `bram_weights.t27` | 20 | **22** |
| `cordic.t27` | 20 | **22** |
| `cordic_top.t27` | 20 | **22** |
| `formal.t27` | 20 | **22** |
| `gemm.t27` | 20 | **22** |

**Total new tests:** +16 across 8 specs.

---

## 3. Competitive Intelligence Targets

Two new competitors added to benchmark code (already documented in COMPETITIVE_POSITIONING.md):

1. **McGirl** — Zenodo (2025): 7 SM observables from E₈→H₄ invariants, seeking endorsement. **MEDIUM** threat.
2. **Douglas QFT** — arXiv:2603.15770 (March 2026): Lean 4 formalization of QFT foundations, AI-assisted. **EXTREME** methodological threat.

---

## 4. Tracks

### Track A: Test Expansion (8 specs)
- `rtl.t27`: `emit_verilog_module_name`, `count_mul_ops_no_mul`
- `eda.t27`: `compute_ppa_penalty_positive`, `parse_synthesis_log_negative_area`
- `cordic_fixed.t27`: `cordic_fixed_sin_pi`, `cordic_fixed_cos_pi`
- `bram_weights.t27`: `read_weight_oob_returns_zero`, `weight_bank_dimensions_match`
- `cordic.t27`: `cordic_sin_half_pi`, `cordic_cos_half_pi`
- `cordic_top.t27`: `cordic_top_reset_release`, `cordic_top_batch_empty_list`
- `formal.t27`: `prove_equivalence_swap_ports`, `generate_report_name_matches`
- `gemm.t27`: `booth_mul_u32_one`, `gemm_2x2_zero_matrix`

### Track B: Competitor Tracking
- Add `mcgirl_competitor()` and `douglas_qft_competitor()` to `specs/igla/coder/benchmark.t27`
- Update `docs/COMPETITIVE_POSITIONING.md` competitor count

### Track C: Verification
- Regenerate seals for 9 modified specs
- Run `./target/release/t27c suite --repo-root .` → 564/564 PASS
- Zero cascade seal mismatches

### Track D: Documentation
- Write `WAVE_LOOP_135_PLAN.md`
- Write `WAVE_LOOP_135_REPORT.md`
- Write `WAVE_LOOP_135_COOPERATION.md` (3 variants)
- Save memory to `wave-loop-135.md`

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
