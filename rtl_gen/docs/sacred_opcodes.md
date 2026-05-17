# Trinity Sacred Opcodes Documentation

**Sacred Bank**: 0xD0..0xFF (32 slots, R18 preserved)
**Total Opcodes**: 16 (0xDF, 0xE1-0xED, 0xF1-0xF3)

---

## Sacred Chain Architecture

```
0xDE → 0xDF → 0xE0 → 0xE1 → 0xE2 → 0xE3 → 0xE4 → 0xE5
 ↓
0xE6 → 0xE7 → 0xE8 → 0xE9 → 0xEA → 0xEB → 0xEC → 0xED
 ↓
0xEE → 0xEF → 0xF0 → 0xF1 → 0xF2 → 0xF3 → ...
```

---

## Lane L Precheck (Wave-42)

### 0xDF: LUT_LOOKUP — Platinum LUT PE Dispatch

**Purpose**: Dispatch prechecked operations to LUT-based PE
**Module**: `lane_l_precheck.v`
**Coq**: `trios-coq/Physics/LaneLPrecheck.v` (12 Qed)

**Properties**:
- Pipeline depth: 4 cycles
- Zero `*` operators (R-SI-1)
- Sparsity correlation: >= 0.8 with Wave-40 mask

**Interface**:
```
input  wire        clk, reset_n
input  wire [7:0]  opcode (0xDF)
input  wire [15:0] activation_in, weight_in
input  wire [26:0] sparsity_mask_in
input  wire        sparse_gate_in
output wire        precheck_valid, skip_dispatch
output wire [7:0]  dispatch_opcode
output wire [15:0] activation_out, weight_out
```

**TOPS/W Impact**: 75 baseline (1.36× from 55)

---

## Sparsity Chain (Wave-40/41)

### 0xE1: SPARSE_SKIP — Skip Zero Computations

**Module**: `sparse_skip.v`
**Wave**: TENET (Wave-33)

### 0xE8: SPARSE_SKIP2 — Sparse-Activation Gating

**Module**: `sparse_gate.v`
**Coq**: `trios-coq/Physics/SparseGate.v` (8 Qed)
**Wave**: Lane T (Wave-41)

### 0xED: SPARSE_MASK — Sparsity Mask (27 Coptic Groups)

**Module**: `sparse_mask.v`
**Coq**: `trios-coq/Physics/SparsityMask.v` (11 Qed)
**Wave**: Lane FF (Wave-40)

**Interface**:
```
input  wire [15:0] data_in
input  wire [26:0] mask_bits
input  wire [4:0]  channel_id
output wire [15:0] data_out
output wire        masked
```

---

## LUT-NPU Chain (Wave-35)

### 0xE3: LUT_NPU — Ternary Inference Lookup

**Module**: `lut_npu_81_entry.v`
**Coq**: `trios-coq/Kernel/LutNpu.v` (10+ Qed)
**Wave**: Lane V

**Properties**:
- 41 Z₃-compressed classes
- Zero `*` operators
- Energy: 7.5 fJ per operation

---

## Power Chain (Wave-36/45/47-49)

### 0xE4: AVS_RECONF — Dynamic Voltage Scaling

**Module**: `avs_reconf.v`
**Coq**: `trios-coq/Physics/Avs96Safe.v` (8 Qed)
**Wave**: Lane W

**Properties**:
- 48 voltage islands (AVS-48)
- 96 voltage islands (AVS-96)
- 5.4× TOPS/W boost (AVS-96)

### 0xE5: SUBTH_CLK — Subthreshold Clock Control

**Module**: `subth_clk.v`
**Wave**: Lane X

### 0xF1: RBB — Reverse Body Bias

**Coq**: `trios-coq/Physics/RBB.v` (33 Qed)
**Wave**: Lane QQ (Wave-47)

**Theory**: V_BS = -V_DD × γ⁴ ≈ -2.5 mV
**Effect**: 40% leakage save in idle PEs

### 0xF2: FBB — Forward Body Bias

**Module**: `fbb_active_path.v`
**Coq**: `trios-coq/Physics/FBBActive2.v` (33 Qed)
**Wave**: Lane SS (Wave-48)

**Theory**: V_BS = +V_DD × γ⁴ ≈ +2.5 mV
**Effect**: 12% delay reduction on active path

### 0xF3: CAP_BOOST — Capacitive Decoupling Burst

**Coq**: `trios-coq/Physics/CapBoost.v` (38 Qed)
**Wave**: Lane VV (Wave-49)

**Theory**: ΔC_dec = C_dec_base × γ³
**Effect**: +0.738% TOPS/W, di/dt margin 6%

**Triple-Decker**: RBB (0xF1) → FBB (0xF2) → CAP_BOOST (0xF3)

---

## LEVER STACK (Wave-28)

### 0xE6: HOLO_MUX_X4 — Holographic 4:1 Multiplexer

**Module**: `holo_mux_x4.v`
**Wave**: Lane Y

**Purpose**: Holographic multiplexer for LEVER STACK architecture

### 0xE7: DFS_GATE — Depth-First Search Skip Gate

**Module**: `dfs_gate.v`
**Wave**: Lane Z

**Purpose**: DFS-based pruning for computational graphs

---

## Stochastic Chain

### 0xE9: STOCH_ROUND — Stochastic Rounding

**Module**: `stoch_round.v`
**Coq**: `trios-coq/Physics/StochSkipSafe.v` (10 Qed)

**Purpose**: Stochastic rounding for quantization

### 0xEA: NULL_PE — Null PE Power Gating

**Module**: `null_pe.v`

**Purpose**: Power gating for idle PEs

### 0xEB: SPEC_EXIT — Speculative Exit Control

**Module**: `spec_exit.v`
**Coq**: `trios-coq/Physics/SpeculativeExit.v` (11 Qed)
**Wave**: Lane W

**Purpose**: Early termination for speculative execution

### 0xEC: DROWSY_RET — Drowsy Retention Mode

**Module**: `drowsy_ret.v`

**Purpose**: Drowsy retention for memory power saving

---

## Sacred Bank Extensions

### R18 Ceremony

- **Original bank**: 0xD0..0xEF (16 slots)
- **Extended bank**: 0xD0..0xFF (32 slots)
- **Method**: Opcode-space only (no Sacred ROM cells)
- **Preserved**: R18 LAYER-FROZEN

### Sacred Chain Progression

| Wave | Opcode | Description |
|------|--------|-------------|
| W33 | 0xE1 | SPARSE_SKIP (TENET) |
| W35 | 0xE3 | LUT_NPU |
| W36 | 0xE4 | AVS_RECONF |
| W40 | 0xED | SPARSE_MASK |
| W41 | 0xE8 | SPARSE_SKIP (Gate) |
| W42 | 0xDF | LUT_LOOKUP (Precheck) |
| W44 | 0xE9 | STOCH_ROUND |
| W45 | 0xEA | NULL_PE |
| W46 | 0xEC | DROWSY_RET |
| W47 | 0xF1 | RBB |
| W48 | 0xF2 | FBB |
| W49 | 0xF3 | CAP_BOOST |

---

## References

- **Phi Identity**: φ² + φ⁻² = 3 — DOI 10.5281/zenodo.19227877
- **FORMAT-SPEC-001.json**: SSOT for sacred opcode definitions
- **IGLA RACE**: W29-W49 wave evolution proofs