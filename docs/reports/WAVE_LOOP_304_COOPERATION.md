# Wave Loop 304 — Three Variants of Cooperation

**Date:** 2026-06-16  
**For:** Next loop (W305) planning  
**Status:** Proposed

---

## Variant A: Deepening (Глубинное развитие)

**Premise:** Double down on t27's unique differentiator — **generic ∀ quantifier theorems for ternary hardware** — and accelerate production.

**Actions:**
1. **Generic theorem sprint:** Add ≥2 generic ∀ theorems per wave (W305 target: 10 generic ∀).
2. **New generic domains:** Expand beyond MAC/Mul into ternary GEMM (e.g., `ternaryGemm2x2AssociativeGeneric`, `ternaryGemm2x2DistributiveGeneric`).
3. **Bus-level formalization:** Add AXI4-Lite verified interface to `ternary_inference.t27` (responding to Sparkle's 14 AXI4 theorems).
4. **Proof automation:** Develop Lean 4 tactics/macros for repetitive ternary proofs (`by ternary_simp` custom tactic).
5. **Pool depth:** Maintain Pool A ≥44, Pool B ≥59, CODER ≥34.

**Expected outcome:** 10 generic ∀ theorems by W305, closing the gap with Sparkle's depth while maintaining uniqueness.

**Risk:** High complexity; generic proofs may require manual `omega`/`ring` assistance.

---

## Variant B: Broadening (Расширение охвата)

**Premise:** Respond to Sparkle's breadth (191+ theorems across RV32IMA, BitNet, YOLO, H.264, AXI4) by expanding t27's verification scope beyond ternary inference.

**Actions:**
1. **New spec domain:** Create `specs/igla/race/axi4_lite.t27` with formal AXI4-Lite protocol properties (deadlock-freedom, burst alignment, response ordering).
2. **New spec domain:** Create `specs/igla/race/yolo_ternary.t27` — ternary-quantized YOLO object detection accelerator spec with bounding-box invariants.
3. **Lean 4 integration:** Prove AXI4-Lite properties in Lean 4 (e.g., `axi4LiteNoDeadlockGeneric`).
4. **Cross-domain invariants:** Link ternary inference to AXI4 bus (e.g., `ternaryInferenceAxi4NoStall`).
5. **Pool depth:** Maintain existing floors while adding new specs.

**Expected outcome:** 2 new specs with ≥20 invariants each by W305, expanding t27's addressable proof space.

**Risk:** Scope creep; may dilute ternary specialization advantage.

---

## Variant C: Ecosystem (Экосистемное сотрудничество)

**Premise:** Partner with complementary projects (TorchLean, AMO-Lean, Sparkle) to create a **unified ternary verification toolchain** rather than competing in isolation.

**Actions:**
1. **TorchLean bridge:** Export t27's Lean 4 ternary theorems as TorchLean-compatible lemmas, enabling PyTorch → ternary hardware verified compilation.
2. **AMO-Lean collaboration:** Use AMO-Lean's verified compiler infrastructure to formally verify t27's `tri` code generation backend (Zig/Verilog/C/Rust/Lean).
3. **Sparkle HDL liaison:** Propose a joint standard for ternary-weight formal verification (shared `TernaryWeight` datatype, shared lemma library). Sparkle contributes concrete golden-value proofs; t27 contributes generic ∀ proofs.
4. **KU Leuven integration:** Feed t27's generic LUT DSE proofs (zero=wire, plus=add, minus=sub) into KU Leuven's Chisel DSE tool for automatic hardware generation with proof certificates.
5. **Open-source publication:** Publish t27's Lean 4 theorem library as standalone package (`trinity-lean-ternary`) on Lake package registry.

**Expected outcome:** t27 becomes the **de facto standard** for ternary hardware verification, embedded in 3+ external toolchains.

**Risk:** Requires coordination with external maintainers; intellectual property considerations for `.t27` format.

---

## Recommendation

**Primary: Variant A (Deepening)** — t27's 8 generic ∀ theorems are its only true moat against Sparkle's 191+ concrete proofs. Accelerating to 10+ generic theorems in W305 is the highest-ROI defensive move.

**Secondary: Variant C (Ecosystem)** — Begin informal outreach to TorchLean and Sparkle maintainers. A unified lemma library benefits all parties and raises the barrier for new competitors.

**Avoid: Variant B (Broadening)** — Until generic theorem count reaches 15+, expanding breadth risks diluting the specialization that keeps competitors out.

---

**Phase complete: Learn**  
**→ Phase 1: Issue (W305)**
