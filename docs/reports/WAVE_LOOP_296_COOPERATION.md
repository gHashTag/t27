# Wave Loop 296 → Wave Loop 297 Cooperation Variants

**Date:** 2026-06-23 | Next Cycle: W297

---

## Current State (Post-W296)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥36** (FIRST TIME) — 15 specs @ 36 |
| **CODER** | **ALL ≥26** (FIRST TIME) — 10 specs @ 26 |
| **Pool B** | systolic_ternary @ 51 |
| **Integration** | ternary_inference @ 36 |
| **Lean 4** | 28 ternary theorems / 63 total |
| **Zero-entrant streak** | 61 waves (60th consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥37 + CODER Depth + Lean 4

**Goal:** Raise ALL 15 Pool A specs from 36→37 AND push ALL 10 CODER specs from 26→27.

### Pool A (15 specs × +1 invariant = +15 invariants)
- All Pool A specs currently at 36→37
- No spec already above 37

### CODER Depth (10 specs × +1 invariant = +10 invariants)
- Target: ALL 10 specs 26→27

### Pool B (1 spec)
- systolic_ternary 51→52 (+1 invariant)

### Integration (1 spec)
- ternary_inference 36→37 (+1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceLutZeroWeightNopConcrete` — concrete zero-weight NOP for arbitrary activation

### Expected Totals
- +15 Pool A invariants, +30 tests
- +10 CODER invariants, +20 tests
- +1 Pool B invariant, +2 tests
- +1 Integration invariant, +2 tests
- +1 Lean 4 theorem
- **Total: +28 invariants, +54 tests, +1 theorem**

---

## Variant B (Innovation): New `ternary_lut.t27` Spec

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to KU Leuven / TOM / TernaryCore / TerEffic gap in formal verification.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul
- Responds to Microsoft T-MAC, OpenBitSys vlut.cpp, KU Leuven ternary-lut-dse

### Pool A Depth
- 8 specs 36→37 (+8 invariants)

### Pool B Depth
- systolic_ternary 51→52 (+1 invariant)

### Lean 4
- `TernaryLUT.lean` with 2 theorems:
  - `lutTernaryMulEquivDirect` — LUT-based equals direct
  - `lutTernaryMulZeroWeightNop` — zero weight is NOP in LUT

**Total:** +22 tests, +15 invariants, +2 theorems, +1 new spec.
**Milestone:** First LUT-based ternary spec in t27; first hardware-algorithm equivalence for LUT.

---

## Variant C (Lean 4 Focus): Proof Depth + Sparkle HDL Response

**Goal:** Add 3 new Lean 4 theorems AND respond to Sparkle HDL RISC-V dominance.

### Lean 4 (+3 theorems)
- `ternaryInferenceLutZeroWeightNopConcrete` — zero weight NOP for arbitrary activation
- `ternaryInferenceLutPlusWeightIdentity` — plus weight preserves activation (already exists, strengthen to generic)
- `ternaryGemmAssociativityConcrete` — GEMM associativity for 2x2 concrete case

### Pool A (8 specs 36→37)
- +8 invariants

### CODER (3 specs 26→27)
- +3 invariants

**Total:** +30 tests, +14 invariants, +3 theorems.
**Milestone:** Closing gap with Sparkle HDL (162+ → t27 ~64).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 37 | 8 specs → 37 | 8 specs → 37 |
| CODER target | ALL → 27 | maintain | 3 specs → 27 |
| Pool B target | 51→52 | 51→52 | maintain |
| Integration target | 36→37 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +54 | +22 | +30 |
| Total invariants | +27 | +15 | +14 |
| Historic milestone | Pool A ≥37 + CODER ≥27 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | KU Leuven/TOM/TernaryCore/TerEffic | Sparkle HDL / OpenVM FV |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥37 + CODER depth + Lean 4).**

Rationale:
1. Pool A uniform ≥37 is the natural next step after achieving ≥36
2. CODER has 10 specs at 26 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Sparkle HDL gap (162+ vs 63) is structural — Variant C doesn't close it meaningfully; need new spec modules (Variant B) or sustained depth growth
6. Variant B (LUT) should follow in W298 once Pool A reaches ≥37 and CODER ≥27

---

## Cooperation Protocol

1. **No file overlap:** Each session claims distinct spec files before editing.
2. **Commit before seal:** Seal hashes must be regenerated after any spec change.
3. **Lean 4 gate:** Every W297 variant must include ≥1 new Lean 4 theorem.
4. **Report within 24h:** Post WAVE_LOOP_297_REPORT.md before next cycle starts.

---

## Scientific Context

- **TernaryCore** (shepherdscientific, 2026) — open-source Verilog FPGA BitNet b1.58; simulation only; NO Lean 4.
- **TerEffic** (arXiv:2502.16473v2, Peking+NUS) — 16,300 tok/s; LUT-based TMat Core; NO formal verification.
- **ternfpga** (Neumann-Labs, Jun 2026) — $130 Arty A7; cocotb+Verilator; NO formal verification.
- **Ternary Fabric** (t81dev, 2026) — ternary-native co-processor; automated synthesis; NO formal verification.
- **Sparkle HDL** 162+ theorems (102 RV32IMA + 60+ BitNet) — HIGH
- **ATOMiK** 92 Lean 4 theorems — HIGH
- **2026 is the year of Lean 4 HDL**

**Key gap identified:** NONE of the 2026 ternary FPGA accelerators use Lean 4 (or any formal proof assistant) for HDL verification. t27's `proofs/lean4/Trinity/TernaryInference.lean` with 28 concrete theorems is a unique competitive differentiator.

phi^2 + 1/phi^2 = 3 | TRINITY
