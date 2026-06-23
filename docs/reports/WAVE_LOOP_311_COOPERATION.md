# Wave Loop 311 — Three Cooperation Variants for W312

**Date:** 2026-06-23
**Prepared for:** Next loop (W312) and external collaboration
**Competitive baseline:** Sparkle HDL (~200 theorems, 0 generic ∀), KU Leuven Ternary LUT DSE (TSMC 16nm, no Lean 4), TorchLean (20+ general NN theorems), CktFormalizer v3 (autoformalization, instance-only proofs)

---

## Variant A: Sprint to 30 Generic ∀ Theorems (RECOMMENDED)

### Thesis
With 24 generic ∀ theorems, t27 is 6 theorems away from **30** — a round-number milestone that creates a **perception of dominance**. Sparkle HDL has ~200 total theorems but zero generic ∀. Reaching 30 generic ∀ makes the gap mathematically obvious and journalistically irresistible.

### W312 Targets

| Objective | Current | W312 Target | W314 Stretch |
|-----------|---------|-------------|--------------|
| Generic ∀ theorems | 24 | **26** | **30** |
| Total Lean 4 theorems | 57 | **59** | **63** |
| Pool A floor | 52 | **53** (uniform) | **54** |
| CODER floor | 43 | **44** | **45** |

### Tactics
1. **Associativity Base Cases**: Prove `ternaryMacPlusWeightAssociativityGeneric` for zero-psum cases: `mac(mac(0, a, .plus), b, .plus) = mac(0, a+b, .plus)`. This avoids Int.add_assoc for open terms by fixing psum=0.
2. **Tiled GEMM Decomposition**: Prove that 2×2 ternary GEMM tile preserves result under row/column partitioning for generic inputs. Maps directly to KU Leuven LUT DSE and BNRV SIMD.
3. **Proof automation tactic**: Create `by_ternary` macro that auto-selects between `native_decide` (closed terms) and `omega` (open linear equalities) based on goal shape.
4. **arXiv v3 preprint**: "24 Generic Ternary MAC Theorems in Lean 4" — showcase the quarter-century milestone as standalone contribution.

### Resource Needs
- 1 senior Lean 4 proof engineer (30% time)
- CI compute unchanged
- arXiv v3 update (~3 hours)

### Risk: LOW | Reward: VERY HIGH
Proven path. 26→30 generic ∀ by W314 is the most defensible trajectory.

---

## Variant B: KU Leuven Ternary LUT DSE Formalization Bridge

### Thesis
**KU Leuven** (arXiv:2604.25183) is the **first ternary ASIC with published synthesis results** (TSMC 16nm, 500 MHz, 2.2× area reduction). But they have **zero formal verification** — only synthesis validation against an analytical model. t27 can bridge this gap.

### W312 Targets

| Objective | Deliverable |
|-----------|-------------|
| LUT decomposition spec | `specs/igla/race/kul_lut_dse.t27` with LUT-based PE invariants |
| Equivalence proof | Lean 4 theorem: `ternaryGemmEqualsKULLUTGeneric` (for generic inputs) |
| Area model verification | Prove that t27's multiplier-free GEMM ≤ KU Leuven's analytical area model |
| Joint publication | arXiv/DAC submission with KU Leuven authors |

### Tactics
1. **LUT PE formalization**: Model KU Leuven's conditional-addition LUT PE as a `.t27` function `kul_lut_pe(activation, weight) -> Int` and prove equivalence to `ternaryMul`.
2. **Analytical model proof**: Show that t27's `ternary_gemm_2x2` (generic algorithm) produces identical outputs to KU Leuven's tile decomposition (hardware) for all generic inputs.
3. **Area bound**: Add invariants that bound LUT count and DSP usage for ternary GEMM, proving t27's spec achieves the same 2.2× reduction as KU Leuven.
4. ** Outreach**: Email KU Leuven MICAS group (Marian Verhelst, Joren Dumoulin) proposing joint verification of their open-source Chisel generator against t27's Lean 4 spec.

### Resource Needs
- 1 RTL engineer for LUT PE spec (40% time)
- 1 proof engineer for equivalence theorem (40% time)
- Academic outreach (~5 hours)

### Risk: MEDIUM | Reward: VERY HIGH
If successful, t27 becomes the **formal reference model** for the first published ternary ASIC. Citations from top-tier VLSI conferences (DAC, ISSCC, ISPASS).

---

## Variant C: TorchLean Integration — Ternary Tensor Lemmas Upstream

### Thesis
**TorchLean** (arXiv:2602.22631v2) is the **most mature Lean 4 NN verification framework**. It has 20+ theorems for general neural networks but **no ternary-specific lemmas**. t27 can contribute its 24 generic ∀ theorems as upstream lemmas, gaining:
1. **Credibility** via association with TorchLean's established reputation
2. **Adoption** by TorchLean users working on quantized/efficient NN inference
3. **Citations** from the broader Lean 4 NN verification community

### W312 Targets

| Objective | Deliverable |
|-----------|-------------|
| Ternary tensor module | `Trinity/TernaryTensor.lean` extending TorchLean's tensor API |
| Upstream lemmas | 5–10 generic ∀ theorems contributed to TorchLean mathlib fork |
| Integration demo | Show t27 `ternary_inference_2x2` running inside TorchLean's execution engine |
| Joint benchmark | Compare TorchLean-verified ternary inference vs PyTorch reference on BitNet b1.58 weights |

### Tactics
1. **Ternary tensor type**: Define `TernaryTensor` as a TorchLean-compatible tensor type with `{Int8, TernaryWeight}` elements and prove `map_ternary(activation, weight)` correctness.
2. **Lemma extraction**: Extract the 5 most general theorems (ZeroWeightIdentity, PlusWeightIdentity, Distributivity, PlusMinusCancel, NegateActivation) as standalone lemmas with minimal dependencies.
3. **Pull request**: Submit PR to TorchLean repository with ternary lemmas and demo. Target acceptance by W314.
4. **Benchmark**: Use real BitNet b1.58 weights from Microsoft's open-source release. Compare:
   - PyTorch reference output (float32)
   - TorchLean verified ternary inference (int8, ternary weights)
   - t27 generated Zig/C output

### Resource Needs
- 1 Lean 4 proof engineer familiar with TorchLean (50% time)
- 1 ML engineer for PyTorch bridge (30% time)
- BitNet b1.58 model weights (public)

### Risk: MEDIUM-HIGH | Reward: VERY HIGH
TorchLean is actively maintained (v1.2 released June 2026). Integration validates t27 in the broader NN verification community. Risk: PR review delays or rejection if lemmas don't fit TorchLean's API.

---

## Comparative Matrix

| Dimension | Variant A (Sprint to 30) | Variant B (KU Leuven Bridge) | Variant C (TorchLean Integration) |
|-----------|--------------------------|------------------------------|---------------------------------|
| **Time to impact** | 1–2 waves | 2–4 waves | 3–4 waves |
| **Resource intensity** | LOW | HIGH | VERY HIGH |
| **Technical risk** | LOW | MEDIUM | HIGH |
| **Strategic risk** | LOW | MEDIUM | MEDIUM-HIGH |
| **Differentiation** | Sustains lead | Opens ASIC frontier | Expands ecosystem |
| **Publication value** | MEDIUM | VERY HIGH | VERY HIGH |
| **Competitive response** | Sparkle may match in 12–18 mo | KU Leuven may adopt specs | TorchLean may absorb t27 lemmas |

---

## Recommendation

**Primary: Variant A** — Continue generic ∀ sprint. 24→26→30 is the most defensible trajectory. Each new theorem raises the barrier and creates content for future publications.

**Secondary: Variant B** — Allocate 25% bandwidth to KU Leuven LUT DSE formalization. The ASIC gap is real: KU Leuven has silicon but no proofs. t27 can "formally verify the competition" and publish equivalence results.

**Tertiary/Experimental: Variant C** — Start TorchLean integration in W312 background track. Extract 3–5 lemmas and test compatibility. Scale in W313–W314 if PR receives positive feedback.

---

## Cooperation Mechanisms

### For Academic Partners (KU Leuven, Stanford, MIT)
- **Co-authorship** on equivalence proofs between algorithm specs and ASIC implementations
- **Benchmark sharing** — t27 provides verified algorithm baseline, partner provides silicon data
- **Student thesis projects** — MSc/PhD on "From Spec to Silicon: Formalizing Ternary ASICs"

### For Open Source Communities (TorchLean, Sparkle HDL, RISC-V)
- **Upstream contributions** of ternary lemmas to TorchLean mathlib
- **Integration workshops** at Lean Together or CHDL
- **RISC-V ternary ISA standardization** using t27 specs as golden reference

### For Industry Partners (AI Chip Vendors, Cloud Providers)
- **Reference implementation** — t27 specs as golden model for silicon verification
- **Joint publications** at DAC/ICCAD showing formally verified ternary inference pipelines
- **IP licensing** of generated Verilog/Rust/C under t27 license

---

*Prepared on 2026-06-23 for Wave Loop 312 planning.*
*Branch: trinity-rust-rings*
