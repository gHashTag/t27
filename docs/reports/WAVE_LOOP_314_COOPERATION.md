# Wave Loop 314 — Three Cooperation Variants for W315

**Date:** 2026-06-23
**Prepared for:** Next loop (W315) and external collaboration
**Competitive baseline:** Sparkle HDL (~200 theorems, 0 generic ∀), LeanMarathon (Jun 2026, multi-agent autoformalization), Theory-Level Autoformalization (ICML 2026), TENET/TerEffic (sparsity-aware ternary inference)

---

## Variant A: Sprint to 35 Generic ∀ Theorems (RECOMMENDED)

### Thesis
With 32 generic ∀ theorems, t27 is **3 theorems away from 35** — the mid-30s milestone that creates a perception of overwhelming dominance. Sparkle HDL has ~200 total theorems but zero generic ∀. Reaching 35 makes the gap mathematically unassailable in the near term.

### W315 Targets

| Objective | Current | W315 Target | W316 Stretch |
|-----------|---------|-------------|--------------|
| Generic ∀ theorems | 32 | **34** | **35** |
| Total Lean 4 theorems | 64 | **66** | **67** |
| Pool A floor | 56 | **57** (uniform) | **58** |
| CODER floor | 47 | **48** | **49** |

### Tactics
1. **N-times scaling theorem**: Prove `ternaryMacNPlusGeneric` using Lean 4's `induction` tactic to show that N consecutive plus-weight MAC operations on the same activation yield `N*a` for all natural N. This would be the first **induction-based** generic theorem in t27's suite, demonstrating that the scaling pattern holds for arbitrary systolic depth.
2. **Commutativity for zero-psum**: Prove `ternaryMacZeroPsumCommutesGeneric` showing that `mac(0, a, w) = mac(0, w, a)` when interpreted as weighted activation (but keeping ternary weight as selector). Requires careful handling of types.
3. **Proof automation library**: Package the growing collection of `by simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode] <;> try omega` patterns into a reusable `ternary_tac` tactic library.
4. **arXiv preprint v5**: "32 Generic Ternary MAC Theorems in Lean 4 — A Verified Foundation for Sparsity-Aware Edge Inference" — position t27 as the formal reference for the emerging sparsity-ternary convergence.

### Resource Needs
- 1 senior Lean 4 proof engineer (40% time)
- CI compute unchanged
- arXiv v5 update (~3 hours)

### Risk: LOW | Reward: VERY HIGH
Proven path. 34→35 generic ∀ by W316 is the most defensible trajectory.

---

## Variant B: LeanMarathon Defense — Multi-Agent Proof Ecosystem

### Thesis
**LeanMarathon** (arXiv:2606.05400, June 2026) demonstrates that **multi-agent autoformalization** can achieve zero-sorry proofs for research-level mathematics. Its four-agent harness (Blueprinter, Target-Reviewer, Worker, Refiner) with CI-gated proof discharge could be adapted to hardware verification. t27 must **preempt this threat** by creating a multi-agent proof ecosystem that is too domain-specific for general-purpose autoformalization to replicate.

### W315 Targets

| Objective | Deliverable |
|-----------|-------------|
| Multi-agent proof spec | `.trinity/agents/proof_agents.md` defining Blueprinter/Worker/Refiner roles for ternary proof generation |
| Public proof challenge | `benchmarks/leanmarathon_challenge/` — 10 ternary theorems where multi-agent AF is tested against human+t27 pipeline |
| Adversarial dataset | 5 theorems with "trap" properties that look simple but require ternary-specific algebraic insight |
| Workshop proposal | Submit "Multi-Agent Hardware Verification: Human vs. Machine" to CHDL 2027 or FMCAD 2026 |

### Tactics
1. **Proof role specification**: Document the ternary proof pipeline as a multi-agent workflow:
   - **Blueprinter**: Identifies algebraic pattern (e.g., "zero-activation identity")
   - **Target-Reviewer**: Validates that the pattern applies to all three weight codes
   - **Worker**: Generates the `simp` + `omega` proof script
   - **Refiner**: Optimizes tactic sequence and adds docstring/hardware mapping
2. **Trap theorems**: Design 3 theorems that are formally simple but semantically deep:
   - `ternaryMacDoubleMinusEqualsNegateDoublePlus` — requires understanding that `-2*a = -(2*a)`
   - `ternaryMacZeroWeightNopVsMulZero` — requires distinguishing `mac(psum, a, .zero)` from `mul(a, .zero)`
   - `ternaryMacPlusMinusVsMinusPlus` — requires proving cancellation order independence
3. **Benchmark**: Invite LeanMarathon authors to attempt the trap theorems. Measure:
   - Proof correctness (`lake build`)
   - Time to correct proof
   - Semantic depth (does the autoformalized proof explain hardware intuition?)
4. **Publication**: arXiv paper "32 Ternary Proofs That Multi-Agent Autoformalization Cannot Yet Generate" — documenting where domain-specific insight outperforms general-purpose agents.

### Resource Needs
- 1 proof engineer for trap theorem design (30% time)
- 1 systems engineer for multi-agent workflow spec (20% time)
- Academic outreach to LeanMarathon authors (~5 hours)

### Risk: MEDIUM | Reward: VERY HIGH
If successful, t27 becomes the benchmark for domain-specific hardware proof generation. If LeanMarathon succeeds, we learn where to improve.

---

## Variant C: TENET/TerEffic Sparsity Bridge — Verified Sparsity-Aware Inference

### Thesis
**TENET** (Sep 2025) and **TerEffic** (Feb 2025) are the leading academic works on **sparsity-aware ternary inference**. TENET uses LUT-centric sparsity-gating; TerEffic uses custom TMat units with 1.6-bit weight compression. Both have **zero formal verification**. t27 can bridge this gap by formalizing their sparsity mechanisms and proving correctness.

### W315 Targets

| Objective | Deliverable |
|-----------|-------------|
| Sparsity model spec | `specs/igla/race/ternary_sparse_lut.t27` modeling TENET-style LUT-centric sparsity |
| Sparsity-gating proof | Lean 4 theorem: `sparseTernaryGemmEqualsDenseGeneric` — sparse GEMM with 6:8 structured sparsity equals dense GEMM for zero-masked elements |
| Energy bound proof | Prove that zero-activation skipping reduces MAC count by expected factor for Bernoulli-distributed sparsity |
| Joint citation | arXiv note citing TENET/TerEffic and showing t27 formally verifies their core claims |

### Tactics
1. **Sparsity mask formalization**: Define `SparsePattern` as a bitmask over activations and prove that `ternaryMac(psum, a, w)` with `a = 0` is equivalent to `psum` regardless of `w` (already proven as `ternaryMacZeroActivationGeneric` in W314).
2. **Structured sparsity (6:8)**: Extend to prove that for any block of 8 activations with 6 non-zero elements, the ternary GEMM result equals the dense result for the non-zero subset plus identity for zeros.
3. **Energy bound**: Add invariants bounding expected MAC count: `E[MAC_count] = (1 - sparsity_ratio) * dense_MAC_count`. Prove via linearity of expectation over independent Bernoulli-distributed activation sparsity.
4. **Cross-validation**: Compare t27-generated Verilog against TENET's published RTL (if available) or TerEffic's open-source FPGA design.

### Resource Needs
- 1 RTL engineer for sparsity model (40% time)
- 1 proof engineer for structured sparsity theorem (40% time)
- Access to TENET/TerEffic papers (public)

### Risk: MEDIUM | Reward: VERY HIGH
If successful, t27 becomes the **formal reference** for sparsity-aware ternary inference — a rapidly growing field. Citations from top-tier VLSI conferences.

---

## Comparative Matrix

| Dimension | Variant A (Sprint to 35) | Variant B (LeanMarathon Defense) | Variant C (Sparsity Bridge) |
|-----------|--------------------------|---------------------------------|---------------------------|
| **Time to impact** | 1–2 waves | 2–3 waves | 2–3 waves |
| **Resource intensity** | LOW | MEDIUM | HIGH |
| **Technical risk** | LOW | MEDIUM | MEDIUM |
| **Strategic risk** | LOW | MEDIUM | MEDIUM |
| **Differentiation** | Sustains lead | Defends against AF | Opens new frontier |
| **Publication value** | HIGH | VERY HIGH | VERY HIGH |
| **Competitive response** | Sparkle may match in 18+ mo | LeanMarathon authors may respond | TENET authors may adopt |

---

## Recommendation

**Primary: Variant A** — Continue generic ∀ sprint to 34→35. This is the lowest-risk, highest-reward path. Each new theorem raises the barrier and creates content for future publications.

**Secondary: Variant C** — Allocate 30% bandwidth to TENET/TerEffic sparsity formalization. The sparsity-ternary convergence is the dominant hardware trend for edge AI. t27's `ternaryMacZeroActivationGeneric` theorem (W314) is already the foundation — extend it to structured sparsity.

**Tertiary/Experimental: Variant B** — Start multi-agent proof workflow documentation in W315 background track. Design 2–3 trap theorems and test against LeanMarathon-style models. Scale in W316–W317 if initial results show human proofs outperform multi-agent AF.

---

## Cooperation Mechanisms

### For Academic Partners (LeanMarathon authors, TENET authors, TerEffic team)
- **Co-authorship** on cross-project papers (t27 + TENET sparsity verification, t27 + LeanMarathon benchmark)
- **Benchmark sharing** — t27 provides verified algorithm baseline, partners provide hardware data or autoformalization results
- **Workshop organization** at CHDL or FMCAD on "Formal Verification for Efficient Neural Inference"

### For Industry Partners (Edge AI chip vendors, FPGA vendors)
- **Reference implementation** — t27 specs as golden model for sparsity-aware ternary inference silicon
- **Joint power/area proofs** using t27 invariants to bound expected energy consumption
- **IP licensing** of generated Verilog/Rust/C with formal certificates

### For Open Source Communities (Sparkle HDL, TorchLean, Lean 4 mathlib)
- **Upstream contributions** of ternary lemmas to TorchLean and Lean 4 mathlib
- **Integration workshops** at Lean Together or CHDL
- **RISC-V ternary ISA standardization** using t27 specs as reference

---

*Prepared on 2026-06-23 for Wave Loop 315 planning.*
*Branch: trinity-rust-rings*
