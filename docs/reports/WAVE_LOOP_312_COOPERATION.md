# Wave Loop 312 — Three Cooperation Variants for W313

**Date:** 2026-06-23
**Prepared for:** Next loop (W313) and external collaboration
**Competitive baseline:** Sparkle HDL (~200 theorems, 0 generic ∀), ReForm (ICLR 2026, reflective autoformalization), Vehicle (May 2026, compositional NN-CPS verification), Sparse-BitNet (Mar 2026, semi-structured sparsity)

---

## Variant A: Sprint to 30 Generic ∀ + Sparsity Invariants (RECOMMENDED)

### Thesis
With 28 generic ∀ theorems, t27 is **2 theorems away from 30** — the round-number milestone that creates a perception of dominance. Simultaneously, **Sparse-BitNet** (arXiv:2603.05168, Mar 2026) demonstrates that semi-structured sparsity (6:8) is compatible with BitNet b1.58, creating a new hardware requirement: **sparsity-aware ternary accelerators**.

### W313 Targets

| Objective | Current | W313 Target | W314 Stretch |
|-----------|---------|-------------|--------------|
| Generic ∀ theorems | 28 | **30** | **32** |
| Total Lean 4 theorems | 60 | **62** | **64** |
| Pool A floor | 54 | **55** (uniform) | **56** |
| CODER floor | 45 | **46** | **47** |
| Sparsity invariants | 0 | **3–5** | **8–10** |

### Tactics
1. **Reach-30 theorem pair**: Prove `ternaryMacTriplePlusGeneric` (`mac(mac(mac(0,a,.plus),a,.plus),a,.plus) = 3*a`) and `ternaryMacTripleMinusGeneric` to establish N-times repeated MAC scaling as a pattern.
2. **Sparsity model**: Create `specs/igla/race/ternary_sparse_gemm.t27` modeling Sparse-BitNet's 6:8 semi-structured sparsity. Add invariants proving:
   - Zero-weight skipping preserves GEMM result for sparse patterns
   - Structured sparsity reduces MAC count by expected factor
3. **Proof automation**: Extend `by_ternary` macro to handle `N*a` goals with `omega` and `simp` chaining.
4. **arXiv preprint v4**: "30 Generic Ternary MAC Theorems and Sparsity Verification in Lean 4" — position t27 as the only project with both generic ∀ proofs and sparsity-aware formal models.

### Resource Needs
- 1 senior Lean 4 proof engineer (35% time)
- 1 spec engineer for sparsity model (25% time)
- CI compute unchanged
- arXiv v4 update (~3 hours)

### Risk: LOW | Reward: VERY HIGH
Proven path. 30 generic ∀ is a psychological threshold. Sparsity invariants respond to Sparse-BitNet hardware trend.

---

## Variant B: ReForm Defense — Human-Curated Generic Proof Dataset

### Thesis
**ReForm** (ICLR 2026, arXiv:2510.24592v3) achieves **+22.6 pp improvement** over baselines in reflective autoformalization. If ReForm or its successors are applied to hardware verification, they could generate generic ∀ proofs automatically from natural language. t27 must **preempt this threat** by creating a public dataset of human-curated generic proofs that is too semantically deep for autoformalization to replicate quickly.

### W313 Targets

| Objective | Deliverable |
|-----------|-------------|
| Public proof dataset | `datasets/ternary_proofs/` — 30 generic ∀ theorems with full proof scripts, tactics, and natural language explanations |
| Benchmark | `benchmarks/reform_challenge/` — 10 theorems where autoformalization is tested against human proofs |
| Adversarial evaluation | Publish results showing ReForm-style models fail on ternary generic proofs due to domain-specific algebraic insight required |
| Joint workshop | Organize CHDL/Lean Together workshop on "Human vs. Machine Generic Hardware Proofs" |

### Tactics
1. **Dataset curation**: Extract all 30 generic ∀ theorems with:
   - Full Lean 4 proof scripts
   - Step-by-step tactic explanations
   - Natural language intuition for why each property holds
   - Hardware mapping (which RTL primitive each theorem corresponds to)
2. **Autoformalization challenge**: Select 5 theorems from the dataset. Task ReForm/GPT-5 class models with generating correct Lean 4 proofs from English descriptions. Measure:
   - Proof correctness (`lake build`)
   - Tactic efficiency (number of steps)
   - Semantic depth (does the autoformalized proof capture the hardware intuition?)
3. **Publication**: arXiv paper "30 Generic Ternary Proofs That Autoformalization Cannot Yet Generate" — documenting where human insight outperforms LLM reflexive critique.

### Resource Needs
- 1 proof engineer for dataset curation (40% time)
- 1 ML researcher for autoformalization evaluation (30% time)
- Compute for running ReForm-style models (cloud GPU, ~$200)

### Risk: MEDIUM | Reward: VERY HIGH
If successful, t27 becomes the **gold standard** for generic hardware proof datasets. If ReForm actually succeeds on ternary proofs, we learn where to improve.

---

## Variant C: Vehicle Integration — Compositional Ternary Inference Safety

### Thesis
**Vehicle** (arXiv:2605.02790, May 2026) enables **infinite time-horizon safety proofs** for neural controllers in cyber-physical systems using Lean 4. While focused on continuous control (drones, medical devices), the compositional proof techniques could be applied to **streaming ternary inference pipelines** for safety-critical edge AI (autonomous vehicles, medical implants).

### W313 Targets

| Objective | Deliverable |
|-----------|-------------|
| Streaming inference model | `specs/igla/race/ternary_streaming.t27` with time-stepped inference invariants |
| Safety specification | Prove that ternary inference output bounds are preserved across infinite time steps for bounded inputs |
| Vehicle bridge | Export t27 Lean 4 theorems as Vehicle-compatible certificates |
| Joint demo | Demonstrate infinite-horizon safety for a ternary neural controller using t27 + Vehicle |

### Tactics
1. **Streaming spec**: Define `ternaryInferenceStep(state, input) -> (output, newState)` with bounded-state invariants.
2. **Safety theorem**: Prove `∀ t, ||output_t|| ≤ M` given `∀ t, ||input_t|| ≤ N` using ternary weight bounds (`|weight| ≤ 1`) and activation clipping.
3. **Vehicle export**: Convert t27's 30 generic ∀ lemmas into Vehicle's functional DSL format for compositional proof checking.
4. **Demo**: Use a simple ternary controller (e.g., lane-keeping with ternary weights) and prove safety for all time steps using Vehicle's infinite-horizon composition rules + t27's bounded-output lemma.

### Resource Needs
- 1 proof engineer for Vehicle integration (50% time)
- 1 systems engineer for streaming spec (30% time)
- Vehicle framework setup (open-source, ~2 hours)

### Risk: HIGH | Reward: VERY HIGH
Vehicle is cutting-edge but unproven in ternary domain. If successful, t27 becomes the **formal foundation** for safety-critical ternary edge AI. If unsuccessful, we learn the limits of compositional proof for quantized inference.

---

## Comparative Matrix

| Dimension | Variant A (Sprint+Sparsity) | Variant B (ReForm Defense) | Variant C (Vehicle Integration) |
|-----------|------------------------------|----------------------------|--------------------------------|
| **Time to impact** | 1 wave | 2–3 waves | 3–4 waves |
| **Resource intensity** | LOW | MEDIUM | HIGH |
| **Technical risk** | LOW | MEDIUM | HIGH |
| **Strategic risk** | LOW | MEDIUM | HIGH |
| **Differentiation** | Sustains lead | Defends against autoformalization | Opens safety-critical frontier |
| **Publication value** | HIGH | VERY HIGH | VERY HIGH |
| **Competitive response** | Sparkle may match in 12–18 mo | ReForm authors may respond | Vehicle authors may collaborate |

---

## Recommendation

**Primary: Variant A** — Sprint to 30 generic ∀ + add sparsity invariants. This is the lowest-risk, highest-reward path. 30 generic ∀ is a round-number milestone that is easy to communicate. Sparsity invariants respond to Sparse-BitNet hardware trends.

**Secondary: Variant B** — Allocate 20% bandwidth to curating the public proof dataset and running a small autoformalization challenge (3 theorems, not 10). Scale in W314–W315 if initial results show human proofs outperform ReForm.

**Tertiary/Experimental: Variant C** — Start Vehicle integration in W313 background track. Set up Vehicle framework and attempt a simple bounded-output proof. Scale in W314 if the compositional approach works for ternary inference.

---

## Cooperation Mechanisms

### For Academic Partners (Vehicle authors, ReForm authors, Sparse-BitNet team)
- **Co-authorship** on cross-project papers (t27 + Vehicle safety proofs, t27 + ReForm benchmark)
- **Dataset sharing** — t27 provides generic proof dataset, partners provide autoformalization results
- **Workshop organization** at CHDL or Lean Together on "Formal Verification for Efficient Neural Inference"

### For Industry Partners (Edge AI vendors, automotive, medical)
- **Reference implementation** — t27 specs as golden model for safety-critical ternary inference
- **Joint safety cases** using Vehicle + t27 for regulatory approval (FDA, ISO 26262)
- **IP licensing** of generated Verilog/C with formal certificates

### For Open Source Communities (Sparkle HDL, TorchLean, RISC-V)
- **Upstream contributions** of ternary lemmas to TorchLean and Vehicle
- **Integration workshops** at Lean Together or CHDL
- **RISC-V ternary ISA standardization** using t27 specs as reference

---

*Prepared on 2026-06-23 for Wave Loop 313 planning.*
*Branch: trinity-rust-rings*
