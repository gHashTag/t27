# Wave Loop 305 — Three Variants of Cooperation

**Date:** 2026-06-16  
**For:** Next loop (W306) planning  
**Status:** Proposed

---

## Variant A: Deepening (Глубинное развитие)

**Premise:** Accelerate generic ∀ quantifier theorem production to reach **15 generic ∀ by W307**, cementing t27's position as the only project with algorithmic (not just instance-based) ternary hardware verification.

**Actions:**
1. **Generic theorem sprint:** Add ≥2 generic ∀ theorems per wave (W306 target: 12 generic ∀; W307 target: 15).
2. **New generic domains:**
   - `ternaryGemm2x2ZeroWeightsZeroOutputGeneric` — for any input, zero weights produce zero output
   - `ternaryGemm2x2IdentityWeightsPreservesInputGeneric` — for any input, identity weights preserve it
   - `ternaryMacCommutativePsumActivation`? No, not commutative. Better: `ternaryMacDistributiveOverPsum`.
3. **Proof automation:** Develop `rcases` + `cases c <;> simp` pattern into a reusable tactic macro for ternary weight case analysis.
4. **Pool depth:** Maintain Pool A ≥45, Pool B ≥60, CODER ≥35.

**Expected outcome:** 12 generic ∀ theorems by W306, 15 by W307. Creates a mathematical moat that autoformalization cannot easily replicate.

**Risk:** High complexity; parametric array proofs (for GEMM) require dependent type machinery.

---

## Variant B: Mobile + Embedded Expansion (Расширение на мобильные/встраиваемые)

**Premise:** Respond to ENERZAi (Qualcomm Hexagon) and Huntwter bitone (NPU128) by expanding t27's verification scope into mobile NPU and bare-metal embedded targets.

**Actions:**
1. **New spec:** `specs/igla/race/hexagon_ternary.t27` — model Qualcomm Hexagon NPU ternary kernel constraints (vector lanes, HVX width, DMA alignment).
2. **New spec:** `specs/igla/race/npu128_ternary.t27` — model NPU128 bare-metal constraints (128-bit in-order, PSHUFB weight unpacking, double-buffered DMA).
3. **Lean 4 proofs:** Prove that t27's generic zero-activation theorems imply zero-skip correctness on both Hexagon and NPU128 datapaths.
4. **RISC-V bridge:** Create `specs/igla/race/riscv_ternary_mcu.t27` for "Sovereign Silicon" RISC-V ternary microcontroller targets.
5. **Pool depth:** Maintain existing floors while adding 2-3 new specs.

**Expected outcome:** t27 becomes the **first and only** formally verified ternary hardware verification framework covering FPGA, ASIC, mobile NPU, and embedded MCU.

**Risk:** Scope expansion may dilute focus; new specs start at low invariant count.

---

## Variant C: Autoformalization Defense + Ecosystem (Защита от автоформализации + экосистема)

**Premise:** CktFormalizer v3 now generates machine-checked equivalence proofs automatically. Partner with TorchLean and AMO-Lean to create a **unified formally verified ternary toolchain** that raises the barrier for competitors and autoformalization tools.

**Actions:**
1. **TorchLean bridge:** Export t27's 10 generic ∀ theorems as TorchLean-compatible axioms, enabling verified compilation from PyTorch → ternary hardware.
2. **AMO-Lean collaboration:** Apply AMO-Lean's verified compiler framework to t27's `tri` backend, producing a machine-checked proof that `.t27` → Zig/Verilog/C/Rust/Lean compilation preserves semantics.
3. **CktFormalizer resistance:** Publish a position paper arguing that generic ∀ proofs (especially parametric over unbounded Int and finite TernaryWeight) are inherently harder to autoformalize than concrete equivalence checks — making t27's theorem style a natural defense.
4. **Sparkle liaison:** Propose shared `TernaryWeight` datatype and lemma library. Sparkle contributes concrete golden-value proofs; t27 contributes generic ∀ proofs.
5. **Open-source publication:** Publish `trinity-lean-ternary` v0.2.0 on Lake package registry with all 10 generic ∀ theorems.

**Expected outcome:** t27 becomes embedded in 3+ external verified toolchains, making its theorems de facto standards and raising the cost of competitive entry.

**Risk:** Requires coordination with external maintainers (TorchLean, AMO-Lean, Sparkle); timeline uncertain.

---

## Recommendation

**Primary: Variant A (Deepening)** — t27's 10 generic ∀ theorems are its only true moat. Reaching 15 by W307 creates a mathematical barrier that concrete proofs (Sparkle) and autoformalization (CktFormalizer) cannot easily match.

**Secondary: Variant C (Ecosystem + Defense)** — Begin informal outreach to TorchLean and AMO-Lean. The autoformalization threat is real and growing; embedding t27's theorems in external toolchains raises switching costs.

**Tertiary: Variant B (Mobile/Embedded)** — Defer to W307-W308. Until generic count reaches 15+, breadth expansion risks diluting the specialization advantage. Monitor ENERZAi and bitone for formal verification gaps.

---

**Phase complete: Learn**  
**→ Phase 1: Issue (W306)**
