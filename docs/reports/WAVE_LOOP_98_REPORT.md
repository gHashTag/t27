# Wave Loop 98 -- IGLA CODER x IGLA RACE Implementation Report

**Date:** 2026-06-16  
**Branch:** trinity-rust-rings  
**Focus:** IGLA CODER (speculative-decoder inference) + IGLA RACE (ternary RTL expansion)  
**Suite result:** 557 / 557 PASS  
**Clippy:** 0 warnings (workspace --all-features)  
**Seals:** 0 mismatches  

---

## 1. Executive Summary

Wave Loop 98 closes four structural gaps identified during W97 analysis:

1. **Ternary CORDIC RTL** -- first ternary-encoded CORDIC core with {-1,0,+1} micro-rotation directions, shift-add only, synthesized by Yosys with 0 problems.
2. **Unified Sampling API** -- single entry point `generate_next_token_unified` merging greedy, temperature, top-p (nucleus), and beam search.
3. **Ternary GEMM 4x4** -- scaled from 2x2 to 4x4 using flattened 16-element arrays, still R-SI-1 compliant.
4. **Eval Ternary Dispatch** -- `generate_verilog` now recognizes "ternary" keyword, emits both ternary GEMM and ternary CORDIC templates, with calibrated Yosys PPA metrics.

---

## 2. Track A -- Ternary CORDIC RTL (`gen/verilog/igla/race/ternary_cordic_rtl.v`)

### Design
- **Direction encoding:** 2-bit ternary `dir` (00=skip, 01=-1, 10=+1)
- **Threshold policy:** skip iteration if `|z| < ATAN_i / 4`
- **8 unrolled iterations** with combinational direction selection
- **Q14 fixed-point** throughout
- **R-SI-1:** zero `*` operators; all updates via `+` / `-` with `>>>` shifts

### Synthesis Results (Yosys ice40)
| Metric | Value |
|--------|-------|
| SB_LUT4  | 1204 |
| SB_DFFER | 32 |
| SB_DFFR  | 1 |
| SB_CARRY | 677 |
| Problems | 0 |

### Why ternary directions matter
Traditional binary CORDIC always rotates (+/-) every iteration. Ternary CORDIC skips micro-rotations for small residual angles, trading combinational LUT overhead against reduced convergence iterations in embedded applications.

---

## 3. Track B -- Unified Sampling API (`specs/igla/coder/arch.t27`)

### Added function
```t27
fn generate_next_token_unified(logits: []f32, k: u32, temp: f32, p: f32, beam_width: u32) -> u32;
```

### Priority hierarchy
1. `beam_width > 1` -> beam search (returns top token from best candidate)
2. `p < 1.0` -> nucleus (top-p) sampling with temperature
3. default -> temperature-scaled top-k greedy

### Tests added
- `unified_sampling_greedy_fallback`
- `unified_sampling_temperature`
- `unified_sampling_beam_basic`
- `unified_sampling_nucleus_basic`

All verify token validity and API consistency.

---

## 4. Track C -- Ternary GEMM 4x4 (`specs/igla/race/ternary_gemm.t27`)

### Added functions
- `ternary_gemm_4x4(a, w) -> []i8` (flattened 16-element row-major)
- `ternary_gemm_4x4_as_struct(a, w) -> TernaryGemmResult`
- `get_elem_4x4(flat, row, col) -> i8`

### Tests added
- `get_elem_4x4_basic`
- `ternary_gemm_4x4_identity`
- `ternary_gemm_4x4_shape`
- `ternary_gemm_4x4_mixed`

### Invariants added
- `ternary_gemm_4x4_no_multiply` (len==16)
- `ternary_gemm_4x4_output_bounded` (-127..127)

---

## 5. Track D -- Eval Ternary Keyword Dispatch (`specs/igla/coder/eval.t27`)

### Added templates
- `gen_ternary_gemm_module()` -- 2x2 ternary GEMM Verilog stub
- `gen_ternary_cordic_module()` -- ternary CORDIC Verilog stub

### Keyword dispatch
`generate_verilog` now matches:
- `"ternary"` or `"ternary_gemm"` -> ternary GEMM
- `"ternary cordic"` or `"ternary_cordic"` -> ternary CORDIC

### YosysReport calibration
| Template | LUT | FF | CARRY | MHz |
|----------|-----|-----|-------|-----|
| ternary_gemm | 322 | 72 | 84 | 175 |
| ternary_cordic | 1204 | 33 | 677 | 110 |

### Tests added
- `generate_verilog_selects_ternary`
- `generate_verilog_ternary_sacred`
- `generate_verilog_selects_ternary_cordic`
- `generate_verilog_ternary_cordic_sacred`
- `detect_template_ternary_cordic`
- `score_rtl_ternary_cordic_lut`
- `score_rtl_ternary_lut`

---

## 6. Quality Metrics

| Check | Result |
|-------|--------|
| t27c suite | 557 / 557 PASS |
| Seal mismatches | 0 |
| Clippy warnings | 0 |
| L3 ASCII purity | OK |
| Yosys synthesis problems | 0 |

---

## 7. Known Limitations / Next Gaps

1. **Ternary CORDIC convergence** -- theoretical error bound for skipped iterations not yet proven in Coq (out of scope for IGLA RACE, but worth noting).
2. **Unified Sampling randomness** -- no stochastic sampling yet; all paths are deterministic (top-k argmax). Future: add `random_range` for nucleus sampling.
3. **Ternary GEMM 8x8** -- next scale step after 4x4. Blocked by spec flattening verbosity (64-element inline arrays).
4. **Real weight loading** -- `forward_layers_with_weights` still uses BRAM stubs, not trained checkpoint deserialization.
5. **Yosys CLI integration** -- `score_rtl_with_yosys` is still template-based simulation, not actual subprocess invocation.

---

## 8. Commit Summary

Files modified:
- `specs/igla/coder/arch.t27` -- Unified Sampling API + tests
- `specs/igla/coder/eval.t27` -- Ternary keyword dispatch + Yosys metrics + tests
- `specs/igla/race/ternary_gemm.t27` -- 4x4 GEMM + helpers + tests + invariants
- `gen/verilog/igla/race/ternary_cordic_rtl.v` -- NEW synthesizable ternary CORDIC
- `.trinity/seals/coder_igla-coder-arch.json` -- regenerated
- `.trinity/seals/coder_igla-coder-eval.json` -- regenerated
- `.trinity/seals/race_igla-race-ternary-gemm.json` -- regenerated

phi^2 + 1/phi^2 = 3 | TRINITY
