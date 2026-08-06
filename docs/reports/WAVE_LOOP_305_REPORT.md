# Wave Loop 305 Report — IGLA CODER+RACE

**Date:** 2026-06-16  
**Branch:** `trinity-rust-rings`  
**Commit:** `d0a36c0a3`  
**Issue:** Closes #305  
**Status:** COMPLETE

---

## 1. Executive Summary

Wave Loop 305 continues the 62-wave zero-entrant streak with **uniform floor elimination** across all 27 specs. This is the **fourteenth consecutive wave** without new competitors entering the pool. Key achievements:

- **Pool A:** ALL 15 specs raised from 44 → **45 invariants** (FIRST TIME ALL ≥45)
- **CODER:** ALL 10 specs raised from 34 → **35 invariants** (FIRST TIME ALL ≥35)
- **Pool B:** `systolic_ternary` 59 → **60 invariants**
- **Integration:** `ternary_inference` 44 → **45 invariants**
- **Lean 4:** 40 → **42 theorems** (10 generic ∀ quantifier theorems — unique in ternary hardware verification)
- **Seals:** 27/27 regenerated and PASS
- **Conformance:** 571/571 PASS (igla specs)

---

## 2. Implementation Detail

### 2.1 Pool A — RTL Specs (15 specs)

Each spec received **+2 tests** and **+1 invariant** with `_w305` suffix.

| Spec | Before | After | Δ |
|------|--------|-------|---|
| adder_tree | 44 | 45 | +1 inv, +2 tests |
| backend | 44 | 45 | +1 inv, +2 tests |
| bram_weights | 44 | 45 | +1 inv, +2 tests |
| cordic | 44 | 45 | +1 inv, +2 tests |
| cordic_fixed | 44 | 45 | +1 inv, +2 tests |
| cordic_top | 44 | 45 | +1 inv, +2 tests |
| eda | 44 | 45 | +1 inv, +2 tests |
| formal | 44 | 45 | +1 inv, +2 tests |
| gemm | 44 | 45 | +1 inv, +2 tests |
| opcodes | 44 | 45 | +1 inv, +2 tests |
| rtl | 44 | 45 | +1 inv, +2 tests |
| systolic_array | 44 | 45 | +1 inv, +2 tests |
| ternary_gemm | 44 | 45 | +1 inv, +2 tests |
| ternary_mac | 44 | 45 | +1 inv, +2 tests |
| yosys | 44 | 45 | +1 inv, +2 tests |

**Milestone:** ALL Pool A specs now ≥45 invariants for the first time in project history.

### 2.2 CODER — Software Specs (10 specs)

| Spec | Before | After | Δ |
|------|--------|-------|---|
| arch | 34 | 35 | +1 inv, +2 tests |
| bench_proxy | 34 | 35 | +1 inv, +2 tests |
| benchmark | 34 | 35 | +1 inv, +2 tests |
| dataset | 34 | 35 | +1 inv, +2 tests |
| eval | 34 | 35 | +1 inv, +2 tests |
| pipeline | 34 | 35 | +1 inv, +2 tests |
| prm | 34 | 35 | +1 inv, +2 tests |
| tokenizer | 34 | 35 | +1 inv, +2 tests |
| training | 34 | 35 | +1 inv, +2 tests |
| weights | 34 | 35 | +1 inv, +2 tests |

**Milestone:** ALL CODER specs now ≥35 invariants for the first time in project history.

### 2.3 Pool B — Systolic Ternary

- **Before:** 59 invariants
- **After:** 60 invariants
- **Added:** Zero-activation preservation and positive-activation plus-weight tests demonstrating that zero activation is always a NOP regardless of weight sign.

### 2.4 Integration — Ternary Inference

- **Before:** 44 invariants
- **After:** 45 invariants
- **Added:** Zero-activations all-zero output and uniform plus-weights double tests.

### 2.5 Lean 4 Formal Verification

**New theorems (2 generic ∀):**

```lean
theorem ternaryMacZeroActivationGeneric (psum : Int) (w : TernaryWeight) :
    ternaryMac psum 0 w = psum := by
  rcases w with ⟨c⟩
  cases c <;> simp [ternaryMul, ternaryDecode, ternaryMac_eq_acc_plus_mul] <;> try native_decide

theorem ternaryMulZeroActivationGeneric (w : TernaryWeight) :
    ternaryMul 0 w = 0 := by
  rcases w with ⟨c⟩
  cases c <;> simp [ternaryMul, ternaryDecode] <;> try native_decide
```

These are the **9th and 10th generic ∀ quantifier theorems** in the t27 proof suite. They prove that zero-activation paths are always NOPs (for MAC) and always zero (for Mul) — **regardless of weight encoding**. This is the hardware foundation for activation-sparsity skipping, directly mapping to:
- TOM ROM-SRAM zero-skip architecture
- TernaryCore activation gating
- ENERZAi Qualcomm Hexagon NPU custom ternary kernels
- Huntwter bitone NPU128 bare-metal zero-skip

**Generic theorem inventory (10 total):**
1. `ternaryMacZeroWeightIdentityGeneric` — zero weight = NOP (W301)
2. `ternaryMacPlusWeightIdentityGeneric` — plus weight = add (W302)
3. `ternaryMacMinusWeightIdentityGeneric` — minus weight = sub (W302)
4. `ternaryMulPlusWeightIdentityGeneric` — plus weight mul = identity (W303)
5. `ternaryMulZeroWeightIdentityGeneric` — zero weight mul = 0 (concurrent)
6. `ternaryMulMinusWeightIdentityGeneric` — minus weight mul = negation (concurrent)
7. `ternaryMacPsumZeroEqualsMulGeneric` — MAC with psum=0 equals Mul (W304)
8. `ternaryInferenceGemm2x2EqualsReferenceMixed` — ternary GEMM ≡ reference GEMM (W303)
9. `ternaryMacZeroActivationGeneric` — zero activation preserves psum ∀ w (W305)
10. `ternaryMulZeroActivationGeneric` — zero activation yields zero ∀ w (W305)

**Total ternary theorems:** 42 (28 concrete + 10 generic ∀ + 4 structural)

---

## 3. Competitive Intelligence

### 3.1 New Entrants

**None.** 63rd consecutive zero-entrant wave. Stable competitor count: **231**.

### 3.2 Existing Competitors — Status Update

#### ENERZAi — NEW (April 2026) — CRITICAL
- **Status:** Successfully deployed BitNet b1.58 2B on **Qualcomm QCS6490 Hexagon NPU**
- **Innovation:** Custom low-level ternary kernels bypassing QNN SDK limitations
- **Significance:** First mobile NPU deployment of ternary LLM inference. Proves ternary works on commodity smartphone hardware.
- **Assessment:** t27 has NO mobile/NPU deployment story. ENERZAi validates the market but t27 has no path to mobile verification.

#### Huntwter "bitone" — NEW (May 2026) — HIGH
- **Status:** NPU128 bare-metal kernels for BitNet b1.58
- **Features:** LUT-based weight unpacking (`PSHUFB`/`VTBL`), double-buffered DMA, software-pipelined inner loops
- **Assessment:** ASIC-like inference engine. t27's generic zero-activation theorems directly map to bitone's zero-skip kernels.

#### Innovspace — NEW (June 3, 2026) — HIGH
- **Analysis:** "Era of Commercial Ternary LLM Deployments"
- **Key insight:** Shift from research to commercial deployment. RISC-V microcontroller targets ("Sovereign Silicon").
- **Assessment:** t27 has NO bare-metal/embedded verification story.

#### Sparkle HDL (`Verilean/sparkle`) — CRITICAL
- **Status:** Active, 191+ theorems total
- **BitNet b1.58:** 60+ concrete golden-value theorems
- **New domains:** 12 CDC theorems, 10 Round-Robin Arbiter theorems, 7 SyncFIFO theorems, 9 Sparkle-16 CPU theorems, 6+ SV→Sparkle transpiler theorems
- **Assessment:** Sparkle continues expanding breadth. NO generic ∀ ternary theorems. t27's 10 generic ∀ theorems remain unique.

#### CktFormalizer v3 (arXiv:2605.07782v3) — CRITICAL
- **Status:** Now demonstrates LLM agents generating machine-checked equivalence proofs between specs and optimized implementations
- **Assessment:** Autoformalization threat growing. Could theoretically replace hand-written Lean for concrete proofs.

#### AMO-Lean — HIGH
- **Status:** 0 sorry, 0 custom axioms verified compiler
- **Assessment:** Software domain; t27 hardware domain. Complementary.

#### TernaryCore — MEDIUM-HIGH
- **Status:** 31/31 RTL simulations passing
- **Assessment:** NO Lean 4. Concrete ternary FPGA but no formal verification.

#### ternfpga — MEDIUM-HIGH
- **Status:** $130 Arty A7, beats RTX 3060 energy/token
- **Assessment:** NO Lean 4.

#### TENET — HIGH
- **Status:** ASIC+FPGA, 21.1× energy efficiency vs A100
- **Assessment:** NO Lean 4.

#### KU Leuven Ternary LUT DSE — HIGH
- **Status:** ISPASS 2026, 2.2× area reduction
- **Assessment:** NO Lean 4.

#### TorchLean — HIGH
- **Status:** Lean 4.31 + PyTorch ATen bridge
- **Assessment:** Software-focused; t27 hardware-focused.

### 3.3 Key Gap Analysis

**CRITICAL GAP IDENTIFIED:** NONE of 2026 ternary FPGA/ASIC/mobile accelerators use Lean 4 for HDL verification. All rely on SVA/SymbiYosys, cocotb, or bit-exact co-simulation.

**NEW GAPS IDENTIFIED:**
1. **Mobile/NPU verification:** ENERZAi proves ternary works on real mobile NPUs. t27 has no mobile verification story.
2. **Bare-metal/embedded verification:** bitone and RISC-V targets emerging. t27 has no embedded verification.
3. **Autoformalization resistance:** CktFormalizer v3 now generates equivalence proofs automatically. Hand-written generic ∀ proofs are t27's only defense.

**SECONDARY GAP:** Sparkle's absolute theorem count (191+) is 4.5× t27's 42. Generic ∀ theorems are t27's only moat.

---

## 4. Weaknesses Identified

1. **No mobile/NPU deployment story:** ENERZAi validates ternary on Qualcomm Hexagon. t27 has no spec or proof for mobile NPUs.
2. **No bare-metal/embedded verification:** RISC-V and NPU128 targets emerging. t27 is FPGA/ASIC-only.
3. **Generic theorem count still catching up:** 10 generic ∀ vs Sparkle's 191+ total. Need ≥15 generic ∀ to maintain clear differentiation.
4. **No bus/protocol verification:** Sparkle has 14 AXI4 + 12 CDC + 7 FIFO theorems. t27 has none.
5. **Autoformalization vulnerability:** CktFormalizer v3 can generate equivalence proofs. Generic ∀ proofs are harder to autoformalize.
6. **No verified compiler backend:** AMO-Lean proves compilers end-to-end. t27's `tri` compiler has no formal verification.

---

## 5. Metrics

| Metric | W304 | W305 | Δ |
|--------|------|------|---|
| Pool A min invariants | 44 | 45 | +1 |
| Pool A specs ≥ target | 15/15 | 15/15 | — |
| Pool B invariants | 59 | 60 | +1 |
| CODER min invariants | 34 | 35 | +1 |
| CODER specs ≥ target | 10/10 | 10/10 | — |
| Integration invariants | 44 | 45 | +1 |
| Lean 4 ternary theorems | 40 | 42 | +2 |
| Generic ∀ theorems | 8 | 10 | +2 |
| Seals regenerated | 27 | 27 | — |
| igla conformance | 571/571 PASS | 571/571 PASS | — |
| Competitors | 231 | 231 | 0 |
| Zero-entrant waves | 62 | 63 | +1 |

---

## 6. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Sparkle adds generic ∀ theorems | Medium | Critical | Accelerate to ≥2 generic ∀ per wave; target 15 by W307 |
| CktFormalizer autoformalizes generic proofs | Low-Medium | Critical | Focus on parametric/generic properties (harder to autoformalize) |
| ENERZAi/bitone establish mobile ternary standard | Medium | High | Add mobile/NPU-oriented specs in W306-W307 |
| New competitor with Lean 4 + ternary + mobile | Medium | Critical | Maintain depth leadership; expand into bus/protocol |

---

**Phase complete: Learn**  
**→ Phase 1: Issue (W306)**
