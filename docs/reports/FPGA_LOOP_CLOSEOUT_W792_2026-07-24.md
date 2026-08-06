# FPGA / IGLA Wave Loop 792 Closeout Report

**Date:** 2026-07-24  
**Branch:** `wave-loop-792`  
**Parent branch:** `wave-loop-791` HEAD (`4b0ec7cb9`)  
**Issue:** #1513  
**PR:** #1514  
**Cooperation variant:** A (recommended)

---

## 1. What was implemented

Wave Loop 792 extended the module-scope packed-array-of-struct ladder to
`[403][2]^6 Pt`. A module-level `pub var dst : [403][2]^6 Pt` is initialized
from a function call and exercised with indexed signed field writes, then
read back with `assert_eq` inside a `bench` block.

### Artifacts added

| File | Purpose |
|------|---------|
| `specs/scratch/w792_bench_module_403x2p6_aos_var_call_write.t27` | Witness spec (25,792 elements, 825,344-bit packed vector, ~0.787 MiBit) |
| `scripts/gen_w792.py` | Generator (`OUTER = 403`, `MID_IDX = 201`) |
| `.trinity/seals/scratch_w792_bench_module_403x2p6_aos_var_call_write.json` | Saved seal |
| `bootstrap/tests/icarus_lowerable.rs` | Integration test `accepts_w792_bench_module_403x2p6_aos_var_call_write` |

### What was NOT changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

---

## 2. Shape details

- `Pt = pub struct Pt { x : i16, y : i16 }` (32 bits per element).
- Outer dimension `403` is non-power-of-two.
- Total elements: `403 × 64 = 25,792`.
- Packed vector width: `25,792 × 32 = 825,344` bits (~0.787 MiBit).
- `MID_IDX = 201`; frame-condition element `[201][1][0][0][0][0][0]` is element
  number `201 × 64 + 32 = 12,896`.
- The witness includes an explicit `make_grid(32768)` period-identity check
  because `32768 ≡ 0 (mod 32768)` and the offset-0 schedule wraps naturally for
  25,792 elements (last raw `x = 16382`).

---

## 3. Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo clippy -p t27c` | OK (780 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p flash-spi` | 2 passed; 0 failed |
| `cargo test -p t27c --test bitnet_pipeline` | 20 passed; 0 failed |
| `cargo test -p t27c --test bitnet_top` | 17 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 252 passed; 0 failed |
| `cargo test -p t27c --test verilog_const_array` | 2 passed; 0 failed |
| `t27c parse` W792 | PASS |
| `t27c icarus-lowerable` W792 | PASS (`lowerable`) |
| `t27c icarus-simulate` W792 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W792 | PASS (`reference-model OK`) |
| `t27c seal --save` W792 | PASS |

The `icarus_lowerable` integration-test count advanced from 251 (W791) to 252 (W792).

---

## 4. Weak-point audit (2026-07-24)

### No new actionable items

- The W783 fix for `bootstrap/tests/verilog_const_array.rs:166` remains green.
- The `verilog_array_literal_expr` regression (`r_ca_2_synthetic_no_comment_only_call_argument`) is pre-existing and out of scope for the witness ladder.
- FPGA E2E CI remains red (`sby` missing in CI + Yosys static-cast error in generated `uart.v`); no new information.
- 626 release warnings and 780 clippy warnings are unchanged; still need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap unchanged (private image not yet published).
- Open PR stack W774-W791 still awaits review, so W792 was branched from `wave-loop-791` HEAD.

### Generator copy hazard — observed and fixed

The generator copy hazard struck again in W792:

- `scripts/gen_w792.py` line 7/77 initially contained `module w791_bench_module_{OUTER}x2p6_aos_var_call_write` because the hardcoded wave prefix inside the f-string was copied from W791.
- First generation produced a spec with module name `w791_bench_module_403x2p6_aos_var_call_write`.
- The prefix was corrected to `w792`, the witness was regenerated, and the correct seal path was produced.

This is the same hazard documented in W782–W791 learnings and remains the only
manual step in the otherwise mechanical flow. Parameterizing the wave prefix
inside the generator template would eliminate it.

### Other checks

- L3 PURITY: ASCII-only source files; commit hook passed.
- L4 TESTABILITY: witness contains `bench` block with `assert_eq` checks.
- L6 CEILING / L7 UNITY: no new `*.sh` on critical path; used `t27c` gates.
- No secrets found in `.env.example` files.
- 57 of 897 `.t27` specs lack `test`/`invariant`/`bench` (~6.35%, unchanged).
- 30-day commit traceability by subject: 148 of 1056 commits carry `Closes #N`/`Fixes #N` (~14.0%), essentially unchanged from W791. The metric remains below the W773-era peak; keep closing references in commit subjects.

---

## 5. Scientific / engineering background

IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
dimensions, with no power-of-two restriction. The W792 witness emits a single
825,344-bit packed vector, which is legal SystemVerilog. The continued use of a
non-power-of-two outer dimension (403) exercises t27's row-major flattening and
indexing arithmetic under a realistic aggregate shape.

### Literature scan (2024–2026)

- **IEEE Std 1800-2017**, §7.4 Packed and unpacked arrays — legal basis for the
  single wide packed vector.
- **AMD UG901 2026.1**, *Vivado Synthesis — SystemVerilog Constructs* — packed
  arrays of structs are supported synthesizable aggregate data types.
- **AMD AR 51836**, *Design Assistant for Vivado Synthesis: Aggregate Data Types*
  — guidance on struct/packed-array inference.
- **Yosys issue #5837 (2026)** — unusual packed-array shapes can expose
  simulator/synthesis mismatches; reinforces t27's flatten-to-wide-vector
  strategy for open-source compatibility.
- **A Generalized Multiple-Valued FPGA Architecture Based on Improved T-Gate Circuit**
  (IEEE Access, 2025, DOI [10.1109/access.2025.3605842](https://doi.org/10.1109/access.2025.3605842)) —
  proposes a T-gate based configurable logic block that merges LUT and flip-flops
  for MVL, generalizable to any logic level and reported to cut power consumption.
- **Reconfigurable Multiple-Valued Logic Function and Sequential Circuit Realizations via Threshold Logic Gates**
  (arXiv 2024, [2404.06420](https://arxiv.org/html/2404.06420)) — general TLG
  framework for reconfigurable MVL functions and sequential circuits; targets
  reduced wiring congestion.
- **TeLLMe: An Energy-Efficient Ternary LLM Accelerator for Prefilling and Decoding on Edge FPGAs**
  (arXiv 2025 / FPGA 2026, [2504.16266](https://arxiv.org/abs/2504.16266)) — first
  end-to-end ternary LLM accelerator on edge FPGA (AMD KV260), table-lookup ternary
  matmul, ~9.5 tok/s decode under 7 W.
- **TerEffic: Highly Efficient Ternary LLM Inference on FPGA**
  (arXiv 2025, [2502.16473](https://arxiv.org/abs/2502.16473)) — ternary
  LLM with LUT-based Ternary Matrix-Multiplication core; up to 16,300 tok/s for
  370 M-parameter models on-chip.
- **Hardware Generation and Exploration of Lookup Table-Based Accelerators for 1.58-bit LLM Inference**
  (arXiv 2026, [2604.25183](https://arxiv.org/abs/2604.25183)) — open-source
  hardware generator and analytical DSE for LUT-based ternary LLM accelerators,
  validated in TSMC 16 nm.
- **ELiTeFormer: An Efficient Transformer for FPGAs**
  (arXiv 2026, [2607.03652](https://arxiv.org/abs/2607.03652)) — hybrid linear
  attention + BitNet b1.58 ternary projections, bitmasking PE eliminates all
  multipliers/DSPs in ternary linear layers, deployed on Xilinx VCK5000 Versal.
- **Trinity B002: Zero-DSP FPGA Architecture for Ternary Inference**
  (Zenodo, 2026, DOI [10.5281/zenodo.19224235](https://doi.org/10.5281/zenodo.19224235)) —
  claims zero-DSP-block ternary NN inference on Xilinx 7-series using LUT-only
  `{-1,0,+1}` MACs; open Yosys/OpenXC7 flow.
- **TernaryCore**
  (GitHub `shepherdscientific/ternarycore`, 2026, CERN-OHL-S v2) — open-source
  Verilog accelerator for BitNet b1.58 ternary inference, native `ternary_mac`
  → `ternary_dot` → `ternary_gemm` hierarchy, no multipliers.
- **Efficient Decompression of Binary Encoded Balanced Ternary Sequences**
  (O. Muller et al., *IEEE TVLSI*, 2019) — foundational 5-ternary-weights-in-8-bits
  encoding used by TerEffic and related work.
- **Ternary VHDL: Simplifying the Design and Verification of Mixed-radix VLSI Circuits**
  (ISMVL 2026, DOI [10.1109/ismvl68998.2026.00041](https://doi.org/10.1109/ismvl68998.2026.00041)) —
  balanced ternary extension to IEEE 1076-2008 VHDL with ternary gates,
  arithmetic operators, and GHDL/GTKWave verification.
- **Optimized Ternary Wallace Multiplier Using Ternary Excess-One Converter for FPGA Applications**
  (DISCOVER 2025, DOI [10.1109/discover66922.2025.11259024](https://doi.org/10.1109/discover66922.2025.11259024)) —
  approximate ternary Wallace multiplier using a Ternary Excess-One Converter,
  implemented on Xilinx Virtex-5.

The 2024–2026 landscape confirms two parallel ternary/MVL tracks: (1) native
MVL FPGA fabrics (T-gate / threshold-logic CLBs) and (2) ternary-quantized
neural-network accelerators mapped to commodity binary FPGAs. t27's current
flatten-to-wide-vector strategy sits squarely in the second track and remains
compatible with both open-source (Yosys/Icarus) and vendor (Vivado) toolchains.

---

## 6. Three cooperation variants for Wave Loop 793

### Variant A — `[405][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder. Expected 25,920 elements, 829,440-bit
packed vector (~0.791 MiBit), still well under the 4-MiBit cliff. This is the
lowest-risk continuation of the established mechanical pattern.

### Variant B — `[403][2]^6 Pt` bench/function-scope packed var from call

Keep W792 width but move the mutable `dst` declaration into a `bench` or
function scope. Exercises local-variable packed-vector lowering and lifetime
without increasing vector width.

### Variant C — `[403][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at W792 width and add conditional indexed signed field writes (e.g.
`if (index % 2 == 0) { dst[index] = ... }`). Tests control-flow emission for
packed reg writes.

**Recommendation:** Variant A. The width ladder has been stable for 19
consecutive waves (W774–W792) with zero compiler changes; continuing it is the
highest-confidence next step.

---

## 7. Conclusion

Wave Loop 792 closed successfully. The `[403][2]^6 Pt` module-scope packed
array-of-struct variable from a call with indexed signed writes is fully
validated, the seal is saved, the integration test passes, and all cargo suites
remain green with zero compiler, reference-model, or `FROZEN_HASH` changes.

φ² + 1/φ² = 3 | TRINITY

---

*Generated with [Claude Code](https://claude.com/claude-code)*
