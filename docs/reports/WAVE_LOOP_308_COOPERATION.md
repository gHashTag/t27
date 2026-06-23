# Wave Loop 308 — Three Variants of Cooperation

**Date:** 2026-06-16  
**For:** Next loop (W309) planning  
**Status:** Proposed

---

## Variant A: Deepening (Глубинное развитие)

**Premise:** Accelerate generic ∀ quantifier theorem production to reach **20 generic ∀ by W310**, cementing t27's position as the only project with algorithmic (not just instance-based) ternary hardware verification.

**Actions:**
1. **Generic theorem sprint:** Add ≥2 generic ∀ theorems per wave (W309 target: 17 generic ∀; W310 target: 20).
2. **New generic domains:**
   - `ternaryMacAssociativeOverPsumGeneric` — mac(mac(p, a, w), b, w) = mac(p, a+b, w)
   - `ternaryMacCommutativePsumAddGeneric` — mac(p1+p2, a, w) = mac(p1, a, w) + p2
   - `ternaryGemm2x2IdentityPreservesAnyConcreteGeneric` — identity weights preserve ANY concrete input
3. **Proof automation:** Develop reusable `rcases` + `cases` + `simp` + `omega` pattern into a custom tactic macro for ternary weight case analysis.
4. **Pool depth:** Maintain Pool A ≥48, Pool B ≥63, CODER ≥38.

**Expected outcome:** 17 generic ∀ theorems by W309, 20 by W310. Creates a mathematical moat that autoformalization cannot easily replicate.

**Risk:** High complexity; parametric array proofs require dependent type machinery.

---

## Variant B: Mobile + Embedded + Multi-Core Expansion (Расширение на мобильные/встраиваемые/многоядерные)

**Premise:** Respond to ENERZAi (Qualcomm Hexagon), Huntwter bitone (NPU128), BNRV (RISC-V SIMD), and BitNet-RISCV-Multicore by expanding t27's verification scope into mobile NPU, bare-metal embedded, and multi-core SoC targets.

**Actions:**
1. **New spec:** `specs/igla/race/hexagon_ternary.t27` — model Qualcomm Hexagon NPU ternary kernel constraints (vector lanes, HVX width, DMA alignment).
2. **New spec:** `specs/igla/race/npu128_ternary.t27` — model NPU128 bare-metal constraints (128-bit in-order, PSHUFB weight unpacking, double-buffered DMA).
3. **New spec:** `specs/igla/race/riscv_simd_ternary.t27` — model BNRV RISC-V SIMD custom instruction extension for ternary MatMul.
4. **New spec:** `specs/igla/race/multicore_ternary.t27` — model BitNet-RISCV-Multicore CVA6 + Ara + Gemmini ternary PE constraints.
5. **Lean 4 proofs:** Prove that t27's generic zero-activation and distributivity theorems imply correctness on all four datapaths.
6. **Pool depth:** Maintain existing floors while adding 3-4 new specs.

**Expected outcome:** t27 becomes the **first and only** formally verified ternary hardware verification framework covering FPGA, ASIC, mobile NPU, embedded MCU, RISC-V SIMD, and multicore SoC.

**Risk:** Scope expansion may dilute focus; new specs start at low invariant count.

---

## Variant C: Autoformalization Defense + Verified Compiler Ecosystem (Защита от автоформализации + верифицированный компилятор)

**Premise:** CktFormalizer v3 now generates machine-checked equivalence proofs automatically. Partner with TorchLean and AMO-Lean to create a **unified formally verified ternary toolchain** that raises the barrier for competitors and autoformalization tools. Simultaneously, apply AMO-Lean's verified compiler framework to t27's `tri` backend.

**Actions:**
1. **TorchLean bridge:** Export t27's 15 generic ∀ theorems as TorchLean-compatible axioms, enabling verified compilation from PyTorch → ternary hardware.
2. **AMO-Lean collaboration:** Apply AMO-Lean's verified compiler framework to formally verify t27's `tri` code generation backend (Zig/Verilog/C/Rust/Lean), producing a machine-checked proof that `.t27` compilation preserves semantics.
3. **CktFormalizer resistance:** Publish a position paper arguing that generic ∀ proofs (especially parametric over unbounded Int and finite TernaryWeight) are inherently harder to autoformalize than concrete equivalence checks — making t27's theorem style a natural defense.
4. **Sparkle liaison:** Propose shared `TernaryWeight` datatype and lemma library. Sparkle contributes concrete golden-value proofs; t27 contributes generic ∀ proofs.
5. **Open-source publication:** Publish `trinity-lean-ternary` v0.3.0 on Lake package registry with all 15 generic ∀ theorems.

**Expected outcome:** t27 becomes embedded in 3+ external verified toolchains, making its theorems de facto standards. The `tri` compiler gains formal verification, closing the gap with AMO-Lean.

**Risk:** Requires coordination with external maintainers (TorchLean, AMO-Lean, Sparkle); timeline uncertain. Verified compiler backend is high-complexity.

---

## Recommendation

**Primary: Variant A (Deepening)** — t27's 15 generic ∀ theorems are its only true moat. Reaching 20 by W310 creates a mathematical barrier that concrete proofs (Sparkle) and autoformalization (CktFormalizer) cannot easily match.

**Secondary: Variant C (Ecosystem + Defense)** — Begin informal outreach to TorchLean and AMO-Lean. The autoformalization threat is real and growing; embedding t27's theorems in external toolchains raises switching costs. The verified compiler backend is a high-value differentiator.

**Tertiary: Variant B (Mobile/Embedded/Multi-Core)** — Defer to W310-W311. Until generic count reaches 20+, breadth expansion risks diluting the specialization advantage. Monitor ENERZAi, bitone, BNRV, and BitNet-RISCV-Multicore for formal verification gaps.

---

**Phase complete: Learn**  
**→ Phase 1: Issue (W309)**
