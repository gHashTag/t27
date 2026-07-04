# Wave Loop 297 → Wave Loop 298 Cooperation Variants

**Date:** 2026-06-23 | Next Cycle: W298

---

## Current State (Post-W297)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥37** (FIRST TIME) — 15 specs @ 37 |
| **CODER** | **ALL ≥27** (FIRST TIME) — 10 specs @ 27 |
| **Pool B** | systolic_ternary @ 52 |
| **Integration** | ternary_inference @ 37 |
| **Lean 4** | 29 ternary theorems / 64 total |
| **Zero-entrant streak** | 62 waves (61st consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥38 + CODER Depth + Lean 4

**Goal:** Raise ALL 15 Pool A specs from 37→38 AND push ALL 10 CODER specs from 27→28.

### Pool A (15 specs × +1 invariant = +15 invariants)
- All Pool A specs currently at 37→38
- No spec already above 38

### CODER Depth (10 specs × +1 invariant = +10 invariants)
- Target: ALL 10 specs 27→28

### Pool B (1 spec)
- systolic_ternary 52→53 (+1 invariant)

### Integration (1 spec)
- ternary_inference 37→38 (+1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceLutPlusWeightIdentityGeneric` — generic plus-weight identity for any concrete activation

### Expected Totals
- +15 Pool A invariants, +30 tests
- +10 CODER invariants, +20 tests
- +1 Pool B invariant, +2 tests
- +1 Integration invariant, +2 tests
- +1 Lean 4 theorem
- **Total: +28 invariants, +54 tests, +1 theorem**

---

## Variant B (Innovation): New `ternary_lut.t27` Spec

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to KU Leuven arXiv:2604.25183 / TernaryCore / TerEffic gap in formal verification.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul
- Responds to Microsoft T-MAC, OpenBitSys vlut.cpp, KU Leuven ternary-lut-dse

### Pool A Depth
- 8 specs 37→38 (+8 invariants)

### Pool B Depth
- systolic_ternary 52→53 (+1 invariant)

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
- `ternaryInferenceLutZeroWeightNopConcrete` — zero weight NOP for any concrete activation (already added in W297; strengthen to generic)
- `ternaryInferenceLutPlusWeightIdentityGeneric` — plus weight preserves any activation
- `ternaryGemmAssociativityConcrete` — GEMM associativity for 2x2 concrete case

### Pool A (8 specs 37→38)
- +8 invariants

### CODER (3 specs 27→28)
- +3 invariants

**Total:** +30 tests, +14 invariants, +3 theorems.
**Milestone:** Closing gap with Sparkle HDL (162+ → t27 ~65).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 38 | 8 specs → 38 | 8 specs → 38 |
| CODER target | ALL → 28 | maintain | 3 specs → 28 |
| Pool B target | 52→53 | 52→53 | maintain |
| Integration target | 37→38 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +54 | +22 | +30 |
| Total invariants | +27 | +15 | +14 |
| Historic milestone | Pool A ≥38 + CODER ≥28 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | KU Leuven/TOM/TernaryCore/TerEffic | Sparkle HDL / OpenVM FV |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥38 + CODER depth + Lean 4).**

Rationale:
1. Pool A uniform ≥38 is the natural next step after achieving ≥37
2. CODER has 10 specs at 27 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Sparkle HDL gap (162+ vs 64) is structural — Variant C doesn't close it meaningfully; need new spec modules (Variant B) or sustained depth growth
6. Variant B (LUT) should follow in W299 once Pool A reaches ≥38 and CODER ≥28

---

## Cooperation Protocol

1. **No file overlap:** Each session claims distinct spec files before editing.
2. **Commit before seal:** Seal hashes must be regenerated after any spec change.
3. **Lean 4 gate:** Every W298 variant must include ≥1 new Lean 4 theorem.
4. **Report within 24h:** Post WAVE_LOOP_298_REPORT.md before next cycle starts.

---

## Scientific Context

- **KU Leuven Ternary LUT** (arXiv:2604.25183, ISPASS 2026) — LUT-based accelerator for 1.58-bit LLMs; 2.2× area reduction; Chisel generator; TSMC 16nm; NO formal verification.
- **TernaryCore** (shepherdscientific, Apr 2026) — open-source Verilog FPGA BitNet b1.58; Artix-7 target; simulation only; NO Lean 4.
- **TerEffic** (arXiv:2502.16473v2, Peking+NUS) — 16,300 tok/s; LUT-based TMat Core; NO formal verification.
- **ternfpga** (Neumann-Labs, Jun 2026) — $130 Arty A7-35T; cocotb+Verilator; NO formal verification.
- **NativeTernary** (arXiv:2604.03336, IIT Bombay) — self-delimiting binary encoding for ternary weights; 460× vs GGUF; NO hardware formal verification.
- **Sparkle HDL** 162+ theorems (102 RV32IMA + 60+ BitNet) — HIGH
- **ATOMiK** 92 Lean 4 theorems — HIGH
- **2026 is the year of Lean 4 HDL**

**Key gap identified:** NONE of the 2026 ternary FPGA/ASIC accelerators use Lean 4 (or any formal proof assistant) for HDL verification. t27's `proofs/lean4/Trinity/TernaryInference.lean` with 29 concrete theorems is a unique competitive differentiator.

phi^2 + 1/phi^2 = 3 | TRINITY
