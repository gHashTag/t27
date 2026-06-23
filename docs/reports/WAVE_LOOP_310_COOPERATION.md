# Wave Loop 310 — Three Cooperation Variants for W311

**Date:** 2026-06-23
**Prepared for:** Next loop (W311) and external collaboration
**Competitive baseline:** VitaLLM (ASIC 16nm), CktFormalizer v3 (Lean 4 HDL autoformalization), Hesper (verified GPU BitNet), Sparkle HDL (191+ theorems, 0 generic ∀)

---

## Variant A: Deepening — Generic ∀ Theorem Sprint to 25 (RECOMMENDED)

### Thesis
With 22 generic ∀ theorems, t27 is 3 theorems away from the **quarter-century milestone** (25 generic ∀). This psychological threshold is significant: Sparkle HDL has ~200 total theorems but zero generic ∀. Reaching 25 creates a **perception gap** that competitors cannot close quickly.

### W311 Targets

| Objective | Current | W311 Target | W312 Stretch |
|-----------|---------|-------------|--------------|
| Generic ∀ theorems | 22 | **24** | **25** |
| Total Lean 4 theorems | 55 | **57** | **58** |
| Pool A floor | 51 | **52** | **53** |
| CODER floor | 42 | **43** | **44** |

### Tactics
1. **Commutativity-Symmetry Pair**: Prove `ternaryMacCommutativityActivationPsumGeneric` (mac(psum, a, w) = mac(a, psum, w) for symmetric cases) and `ternaryMacAssociativityGeneric` (tiled decomposition preserves result) — but limit to zero-psum/zero-activation cases where `native_decide` handles open terms.
2. **Tiled GEMM Base Case**: Prove that 2×2 ternary GEMM tile equals reference scalar GEMM for generic inputs. This directly maps to VitaLLM TINT core and BNRV SIMD units.
3. **Proof automation macro**: Create `by_ternary_simp` Lean 4 tactic macro that chains `simp [ternaryMul, ternaryDecode, ternaryMac_eq_acc_plus_mul]` with `native_decide`/`omega` selection based on goal shape.
4. **arXiv preprint v2**: Update "Generic Ternary MAC Verification in Lean 4" to include Zero-Activation Identity Trinity (W310) and commutativity theorems (W311).

### Resource Needs
- 1 senior Lean 4 proof engineer (25% time)
- CI compute unchanged
- arXiv v2 update (~2 hours)

### Risk: LOW | Reward: HIGH
Proven path. 24 generic ∀ by W311 is achievable with zero-activation/zero-psum strategies.

---

## Variant B: ASIC Bridge — From Verified Algorithm to Silicon

### Thesis
**VitaLLM** (arXiv:2605.00320v1) is the first ternary ASIC prototype but has **zero formal verification**. t27 can bridge the gap by:
1. Formalizing VitaLLM's TINT core datapath in `.t27`
2. Proving equivalence between t27's generic ternary GEMM and VitaLLM's tile-level operations
3. Generating golden reference vectors for VitaLLM's testbench

### W311 Targets

| Objective | Deliverable |
|-----------|-------------|
| TINT core formalization | `specs/igla/race/vitallm_tint.t27` with PE array invariants |
| Equivalence proof | Lean 4 theorem: `ternaryGemm2x2EqualsTINTGeneric` |
| Golden vectors | `gen/vitallm_golden.json` with 1024 reference outputs for BitNet b1.58 layers |
| Publication | arXiv note: "Formalizing the VitaLLM TINT Core in Lean 4" |

### Tactics
1. **TINT PE spec**: Model VitaLLM's `8×8` multiplier-free PE array as a `.t27` systolic array with ternary weight select and INT8 activation addition.
2. **Cycle-accurate invariants**: Add timing invariants (1 cycle per MAC, 8 cycles per dot product) to match VitaLLM's 1 GHz @ 16nm claims.
3. **Equivalence proof**: Show that t27's `ternary_gemm_2x2` (algorithm level) produces identical outputs to TINT's tile decomposition (hardware level) for all generic inputs.
4. **Cross-validation**: Compare generated Verilog against VitaLLM's published Verilog (if open-sourced) or against their performance claims (72.46 tok/s decode).

### Resource Needs
- 1 RTL engineer for TINT spec (40% time)
- 1 proof engineer for equivalence theorem (40% time)
- Access to VitaLLM paper and supplementary materials (public)

### Risk: MEDIUM | Reward: VERY HIGH
If successful, t27 becomes the **formal reference model** for the first ternary ASIC. VitaLLM authors gain credibility; t27 gains citation.

---

## Variant C: Autoformalization Offensive — Beat CktFormalizer at Its Own Game

### Thesis
**CktFormalizer v3** uses Lean 4 as a backend for LLM-generated hardware. It achieves 95–100% backend realizability but:
- Cannot generate **generic algorithmic proofs** (∀ quantifiers)
- Is not **ternary-specific**
- Uses natural language as input (ambiguous)

t27 can **weaponize its structured spec advantage** by creating a public benchmark: "Can CktFormalizer autoformalize t27 specs as well as humans write them?"

### W311 Targets

| Objective | Deliverable |
|-----------|-------------|
| Public benchmark | `benchmarks/autoformalization_challenge/` with 10 ternary theorems |
| CktFormalizer evaluation | Head-to-head: CktFormalizer NL → Lean 4 vs t27 spec → Lean 4 |
| Metaprogram bridge | `t27-to-lean4` metaprogram that consumes `.t27` AST and emits proof scripts automatically |
| Publication | arXiv: "Structured Specifications vs. Natural Language for Hardware Verification" |

### Tactics
1. **Benchmark design**: Select 10 theorems from `TernaryInference.lean` (5 concrete, 5 generic). For each:
   - Write English description (input to CktFormalizer)
   - Provide `.t27` spec (input to t27 pipeline)
   - Measure: time to correct Lean 4, proof correctness (`lake build`), line count, tactic complexity
2. **Metaprogram automation**: Implement Lean 4 `elabT27` tactic that reads JSON AST from `t27c --emit-ast` and auto-generates `by simp [ ... ] <;> native_decide` proofs. Target: 80% of concrete theorems generated automatically.
3. **Community challenge**: Publish benchmark on GitHub with CI runner. Invite CktFormalizer authors and Sparkle HDL team to participate.

### Resource Needs
- 1 compiler engineer (t27c AST emitter) (50% time)
- 1 Lean 4 metaprogramming expert (50% time)
- 1 technical writer for benchmark documentation (20% time)

### Risk: HIGH | Reward: VERY HIGH
If t27 wins the benchmark, it becomes the **gold standard** for hardware formalization. If CktFormalizer wins, t27 learns where structured specs are insufficient and can improve.

---

## Comparative Matrix

| Dimension | Variant A (Deepening) | Variant B (ASIC Bridge) | Variant C (Autoformalization Offensive) |
|-----------|----------------------|-------------------------|----------------------------------------|
| **Time to impact** | 1 wave | 2–3 waves | 2–3 waves |
| **Resource intensity** | LOW | HIGH | VERY HIGH |
| **Technical risk** | LOW | MEDIUM | HIGH |
| **Strategic risk** | LOW | MEDIUM | HIGH |
| **Differentiation** | Sustains lead | Opens new frontier | Defends against disruption |
| **Publication value** | MEDIUM | VERY HIGH | VERY HIGH |
| **Competitive response** | Sparkle may match in 12–18 mo | VitaLLM may adopt | CktFormalizer may adapt |

---

## Recommendation

**Primary: Variant A** — Continue generic ∀ theorem sprint. 22 → 24 → 25 is the most defensible trajectory. Each new generic theorem raises the barrier for Sparkle, Hesper, and CktFormalizer.

**Secondary: Variant B** — Allocate 30% bandwidth to VitaLLM TINT formalization. The ASIC gap is real: VitaLLM has silicon but no proofs. t27 can "formally verify the competition" and publish equivalence results.

**Tertiary/Experimental: Variant C** — Start benchmark design in W311 background track. Run head-to-head on 3 theorems (not 10). Scale in W312–W313 if initial results favor structured specs.

---

## Cooperation Mechanisms

### For Academic Partners (Universities, Research Labs)
- **Co-authorship** on arXiv preprints (generic ∀ theorem series, VitaLLM formalization)
- **Benchmark sharing** — t27 provides verified algorithm baseline, partner provides novel hardware data
- **Student thesis projects** — MSc/PhD on ternary inference verification using t27 pipeline

### For Industry Partners (ASIC/FPGA Vendors, AI Accelerator Startups)
- **Reference implementation** — t27 specs as golden model for silicon verification
- **Joint publications** at DAC, ICCAD, or ISPASS showing "from spec to silicon" flow
- **IP licensing** of generated Verilog/Rust/Zig (under t27 license)

### For Open Source Communities (Lean 4, Sparkle HDL, RISC-V)
- **Upstream contributions** to Lean 4 mathlib (ternary arithmetic lemmas)
- **Integration** with Sparkle HDL or Hesper (export t27 theorems as axioms)
- **Standardization workshops** at RISC-V Summit or CHDL on ternary ISA extensions

---

*Prepared on 2026-06-23 for Wave Loop 311 planning.*
*Branch: trinity-rust-rings*
