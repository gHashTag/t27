# Trinity RTL Generation

GoldenFloat Family + Advanced Power/Performance Modules

## Overview

This directory contains Verilog-2005 compatible RTL for Trinity S³AI chip:

- **GF Formats**: 10 phi-optimized floating point formats (4-256 bits)
- **Testbenches**: Comprehensive verification for all formats
- **Power Modules**: AVS controllers, thermal gates, FBB
- **Sacred Opcodes**: SPARSE_SKIP, LUT_NPU, AVS_RECONF, SUBTH_CLK, FBB

## GF Format Family

| Format | Bits | Exp | Mant | BIAS | phi_dist | TOPS/W | Notes |
|--------|------|-----|------|------|----------|--------|-------|
| GF4    | 4    | 1   | 2    | 0    | 0.118    | 70     | Extreme compression |
| GF8    | 8    | 3   | 4    | 3    | 0.132    | 65     | Ultra-low power |
| GF12   | 12   | 4   | 7    | 7    | 0.047    | 60     | **BEST after GF64** |
| GF16   | 16   | 6   | 9    | 31   | 0.049    | 55     | **PRIMARY format** |
| GF20   | 20   | 7   | 12   | 63   | 0.035    | 52     | Balanced |
| GF24   | 24   | 9   | 14   | 255  | 0.025    | 50     | High precision |
| GF32   | 32   | 12  | 19   | 2047 | 0.014    | 48     | FP32-like |
| GF64   | 64   | 24  | 39   | 8388607 | 0.003 | 45     | **BEST phi_dist** |
| GF128  | 128  | 48  | 79   | 140737488355327 | 0.010 | 42     | Extended range |
| GF256  | 256  | 97  | 158  | 7922816251426433759 | 0.004 | 40     | Ultra-high precision |

## TOPS/W Multipliers

- Baseline GF16: 55 TOPS/W
- With Lane L Precheck: 75 TOPS/W (×1.36) — **NEW for TTSKY26b**
- With AVS-48: 66 TOPS/W (×1.2)
- With AVS-96 + η≥0.93: 405 TOPS/W (×7.4 from baseline, ×5.4 from precheck)

## Module Status

| Module | Status | Notes |
|--------|--------|-------|
| GF Add Units | ✅ Complete | All 10 formats |
| GF Mul Units | ✅ Complete | All 10 formats |
| Testbenches | ✅ Complete | 20 testbenches |
| AVS-48 | ✅ Complete | 48 voltage islands |
| AVS-96 | ✅ Complete | 96 voltage islands (5.4x) |
| Purkinje Thermal | ✅ Complete | W45 Coq proof |
| FBB Active Path | ✅ Complete | Sacred opcode 0xF2 |
| LUT-NPU | ✅ Complete | 81-entry lookup |
| Lane L Precheck | ✅ Spec | Wave-42, 75 TOPS/W baseline |
| Quantizers | ✅ Complete | NF4, Int4/8, FP8, Posit16 |
| Converters | ✅ Complete | GF↔FP, GF↔Posit |
| Sacred Opcodes | ✅ Complete | 0xE1, 0xE3, 0xE4, 0xE5, 0xF2, 0xDF |

## Sacred Opcodes

| Opcode | Name | Module | Description |
|--------|------|--------|-------------|
| 0xDF | LUT_LOOKUP | lane_l_precheck.v | Platinum LUT PE dispatch (Lane L) |
| 0xE1 | SPARSE_SKIP | sparse_skip.v | Skip zero computations |
| 0xE3 | LUT_NPU | lut_npu_81_entry.v | Ternary inference lookup |
| 0xE4 | AVS_RECONF | avs_reconf.v | Dynamic voltage scaling |
| 0xE5 | SUBTH_CLK | subth_clk.v | Subthreshold clock control |
| 0xF2 | FBB | fbb_active_path.v | Forward Body Bias |

## Lane L Precheck (Wave-42)

**Target**: 75 TOPS/W baseline (36% boost) via CGT (-12% dynamic power)

Integration points:
- **Wave-40 SparsityMask.v**: 27-bit sparsity mask
- **Wave-41 SparseGate.v**: Sparse activation gating
- **LEVER STACK**: OP_LUT_LOOKUP (0xDF) dispatch to Platinum LUT PE

Coq proof: `trios-coq/Physics/LaneLPrecheck.v` (12 Qed lemmas)
Spec: `specs/lane_l_precheck.t27` (10 invariants, 6 test vectors)

Key properties:
- R-SI-1: Zero `*` operators (LUT-based dispatch)
- Pipeline depth: 4 cycles
- Sparsity correlation: >= 0.8 with Wave-40 mask
- Sacred opcode chain: ... → 0xDF → 0xE0 → ...

## Synthesis

```bash
# Individual module synthesis
yosys synth.ys

# Batch synthesis for all modules
yosys synth_all.ys

# View synthesis report
cat synth_report.txt
```

## Verification

```bash
# Syntax check with iverilog
iverilog -t null gf16_add.v

# Run testbench
iverilog -o sim tb_gf16_add.v gf16_add.v
vvp sim

# Check all GF modules
iverilog -t null gf*_add.v gf*_mul.v
```

## File Structure

```
rtl_gen/
├── gf*_add.v          # Addition units (10 files)
├── gf*_mul.v          # Multiplication units (10 files)
├── tb_*.v             # Testbenches (18 files)
├── avs_controller_*.v # Voltage controllers
├── purkinje_thermal_gate.v
├── fbb_active_path.v
├── lut_npu_81_entry.v
├── *_quantizer.v      # Quantization units
├── *_to_*.v          # Format converters
├── sparse_skip.v     # Sacred opcode 0xE1
├── avs_reconf.v      # Sacred opcode 0xE4
├── subth_clk.v       # Sacred opcode 0xE5
├── gf_formats.v      # Format definitions
├── synth.ys          # Yosys synthesis script
├── synth_all.ys      # Batch synthesis
└── README.md         # This file
```

## References

- φ² + φ⁻² = 3 (DOI: 10.5281/zenodo.19227877)
- FORMAT-SPEC-001.json v2.0
- IGLA RACE Coq proofs (W29-W49)
- QLoRA paper (NF4 quantization)

## License

Apache-2.0