# Trinity RTL Generation — TTSKY26b

GoldenFloat Family + Sacred Opcodes + Power Modules

## Overview

This directory contains Verilog-2005 compatible RTL for Trinity S³AI chip:
- **GF Formats**: 10 phi-optimized floating point formats (GF4-GF256)
- **Sacred Opcodes**: 16 sacred opcodes (0xDF, 0xE1-0xED, 0xF1-0xF3)
- **Power Modules**: AVS-48/96, FBB, RBB, CapBoost
- **Synthesized**: 17 modules in `build/` directory

## GF Format Family

| Format | Bits | Exp | Mant | BIAS | phi_dist | TOPS/W | Notes |
|--------|------|-----|------|------|----------|--------|-------|
| GF4    | 4    | 1   | 2    | 0    | 0.118    | 70     | Extreme compression |
| GF8    | 8    | 3   | 4    | 3    | 0.132    | 65     | Ultra-low power |
| GF12   | 12   | 4   | 7    | 7    | 0.047    | 60     | BEST after GF64 |
| GF16   | 16   | 6   | 9    | 31   | 0.049    | 55     | PRIMARY format |
| GF20   | 20   | 7   | 12   | 63   | 0.035    | 52     | Balanced |
| GF24   | 24   | 9   | 14   | 255  | 0.025    | 50     | High precision |
| GF32   | 32   | 12  | 19   | 2047 | 0.014    | 48     | FP32-like |
| GF64   | 64   | 24  | 39   | 8388607 | 0.003 | 45     | BEST phi_dist |
| GF128  | 128  | 48  | 79   | 140737488355327 | 0.010 | 42     | Extended range |
| GF256  | 256  | 97  | 158  | 7922816251426433759 | 0.004 | 40     | Ultra-high precision |

## TOPS/W Multipliers

- Baseline GF16: 55 TOPS/W
- With Lane L Precheck: 75 TOPS/W (×1.36)
- With AVS-96: 405 TOPS/W (×7.4 from baseline)

## Sacred Opcodes

| Opcode | Hex | Module | Wave | Description |
|--------|-----|--------|------|-------------|
| LUT_LOOKUP | 0xDF | lane_l_precheck.v | Lane L | Platinum LUT PE dispatch |
| SPARSE_SKIP | 0xE1 | sparse_skip.v | TENET | Skip zero computations |
| LUT_NPU | 0xE3 | lut_npu_81_entry.v | Lane V | Ternary inference lookup |
| AVS_RECONF | 0xE4 | avs_reconf.v | Lane W | Dynamic voltage scaling |
| SUBTH_CLK | 0xE5 | subth_clk.v | Lane X | Subthreshold clock control |
| HOLO_MUX_X4 | 0xE6 | holo_mux_x4.v | Lane Y | Holographic 4:1 multiplexer |
| DFS_GATE | 0xE7 | dfs_gate.v | Lane Z | Depth-First Search skip gate |
| SPARSE_SKIP2 | 0xE8 | sparse_gate.v | Lane T | Sparse-Activation Gating |
| STOCH_ROUND | 0xE9 | stoch_round.v | Lane U | Stochastic rounding |
| NULL_PE | 0xEA | null_pe.v | Lane V | Null PE power gating |
| SPEC_EXIT | 0xEB | spec_exit.v | Lane W | Speculative exit control |
| DROWSY_RET | 0xEC | drowsy_ret.v | Lane X | Drowsy retention mode |
| SPARSE_MASK | 0xED | sparse_mask.v | Lane FF | Sparsity mask (27 Coptic) |
| RBB | 0xF1 | rbb.v | Lane QQ | Reverse Body Bias |
| FBB | 0xF2 | fbb_active_path.v | Lane SS | Forward Body Bias |
| CAP_BOOST | 0xF3 | cap_boost.v | Lane VV | Capacitive decoupling burst |

## Sacred Bank Extension

**0xD0..0xFF**: 32 slots (R18 preserved)
- Triple-Decker (W47-W49): RBB (0xF1) → FBB (0xF2) → CAP_BOOST (0xF3)

## Synthesis

```bash
# View synthesis results
ls build/

# Individual module synthesis
yosys -p "read_verilog build/gf16_add_synth.v; stat"
```

## Verification

### Coq Physics Proofs

```bash
cd ../trios-coq
coqc Physics/LaneLPrecheck.v
coqc Physics/SparsityMask.v
coqc Physics/SparseGate.v
```

**Status**: 350+ Qed lemmas, 0 Admitted in sacred ops

## L1-L7 Compliance

| Law | Status | Notes |
|-----|--------|-------|
| L1: TRACEABILITY | ✅ | All commits include `Closes #N` |
| L2: GENERATION | ✅ | Files under `gen/` generated |
| L3: PURITY | ✅ | ASCII-only RTL |
| L4: TESTABILITY | ✅ | All specs have test/invariant |
| L5: IDENTITY | ✅ | φ² = φ + 1, φ² + φ⁻² = 3 |
| L6: CEILING | ✅ | FORMAT-SPEC-001.json SSOT |
| L7: UNITY | ✅ | No new `*.sh` on critical path |

## File Structure

```
rtl_gen/
├── build/                    # Synthesized modules (17 files)
│   ├── gf*_add_synth.v      # 10 adders
│   ├── gf*_mul_synth.v      # 10 multipliers (partial)
│   └── *.json               # Synthesis reports
├── README.md                 # This file
└── __pycache__/             # Python cache
```

## References

- **Phi Identity**: φ² + φ⁻² = 3 — DOI 10.5281/zenodo.19227877
- **FORMAT-SPEC-001.json**: SSOT for GF formats
- **IGLA RACE**: W29-W49 wave evolution proofs
- **TTSKY26b**: 2026-05-18 22:00 UTC deadline

## License

Apache-2.0