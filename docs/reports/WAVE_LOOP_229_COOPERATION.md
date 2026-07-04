# Wave Loop 229 — Three Cooperation Variants

*Date: 2026-06-19 | Variant A | 228 total competitors (+1) | φ² + 1/φ² = 3 | TRINITY*

---

## Executive Summary

Wave Loop 229 delivered a **horizontal coverage lift** across four RACE specs (opcodes, gemm, ternary_mac, adder_tree) and one CODER depth push on pipeline (the most invariant-starved spec in the CODER module). The competitive landscape saw **one new entrant** (TilelliLab/atome-lm, 11 June 2026), bringing total tracked competitors to **228**. The post-disruption pattern stabilizes at approximately **1 new entrant per wave**, indicating active niche exploration but no mass-market rush. Cooperation remains more viable than pure competition.

---

## Variant A — Shared Benchmark + Citation (Low Friction, W230 Target)

**Partners:** shepherdscientific/ternarycore, TilelliLab/atome-lm

**Mechanics:**
1. Trinity authors a **joint benchmark specification** (`docs/cooperation/TRINITY_MICRO_BENCHMARK.md`) defining standardized ternary-accelerator evaluation metrics for three tiers: FPGA (Artix-7/Alveo), MCU (Cortex-M3/ESP32), and ASIC (tinyML).
2. shepherdscientific runs the benchmark on their Artix-7 ternarycore (no-DSP GEMM, 4×4 verified).
3. TilelliLab runs the benchmark on their ESP32 atome-lm (60K params, C99 engine).
4. Trinity integrates results into `docs/reports/JOINT_BENCHMARK_W230.md` with co-authored attribution.
5. Each party cites the joint benchmark in their own publications / repos.

**Value Exchange:**
- Trinity gains hardware validation data across FPGA and MCU tiers.
- Partners gain credibility from Trinity’s formal-proof reputation and standardized methodology.
- TilelliLab specifically benefits from a rigorous benchmark counterbalance to their self-reported perplexity numbers.

**Risk:** Low. No IP transfer; purely measurement sharing. Both partners are academic/open-source. TilelliLab is brand-new (June 2026) and likely eager for visibility.

---

## Variant B — Joint Submission + Dual Attribution (Medium Friction, Q3 2026 Target)

**Partners:** Neumann-Labs/ternfpga, deveworld/bitnet-tt

**Mechanics:**
1. Trinity proposes a **joint workshop paper** (e.g., FPGA 2026 or tinyML Summit) titled *"Ternary LLM Inference: From Formal Specification to Edge Silicon — A Three-Platform Comparative Study."*
2. Trinity contributes:
   - Formal specification framework (t27 specs, L5 identity proofs)
   - Comparative analysis methodology across FPGA / MCU / custom silicon
3. Partners contribute:
   - Neumann-Labs: Arty A7-35T measurements (energy/token, 0-DSP validation, unstructured sparsity skipping)
   - deveworld: Blackhole p150a throughput measurements (73.4 tok/s, BFP2 packing, 3.9× energy reduction)
4. Submission is dual-attributed; all repos link to the pre-print.

**Value Exchange:**
- Trinity gains access to cutting-edge hardware measurements from two distinct platforms (FPGA edge + custom silicon).
- Partners gain theoretical grounding and formal-method credibility from Trinity.
- Joint paper carries more weight than any individual submission; covers 3 of 4 ternary hardware tiers.

**Risk:** Medium. Requires coordinating timelines and agreeing on paper scope. Neumann-Labs is active and responsive; deveworld is a solo developer (higher variance). Mitigate by drafting outline unilaterally and offering co-authorship for data contribution only.

---

## Variant C — Deep Integration / Joint Venture (High Friction, Q4 2026 Target)

**Partner:** ETH Zurich / fpgasystems/ternaryLLM

**Mechanics:**
1. Trinity and ETH HACC cluster establish a **joint ternary-hardware verification pipeline**:
   - Trinity specs define RTL generation targets (`igla::race::rtl` → SystemVerilog)
   - ETH Coyote framework deploys generated RTL on Alveo U55C
   - SymbiYosys formal checks (from `igla::race::yosys`) run pre-synthesis
   - Trinity seal verification validates bit-exact equivalence
2. Trinity contributes formal specs and generated Verilog; ETH contributes FPGA infrastructure and benchmarking harness.
3. Joint output: **open-source reference design** (`trinity-eth-ternary-reference`) with:
   - Complete ternary GEMM accelerator (Trinity spec → ETH implementation)
   - Formal proof of correctness (SymbiYosys + Coq)
   - Reproducible benchmark results on public cloud FPGA (HACC)
   - SSR paper (DATE 2026) extended with formal-methods appendix
4. Revenue model (future): joint consulting for datacenter ternary-accelerator deployments.

**Value Exchange:**
- Trinity gains world-class FPGA infrastructure, academic credibility, and access to DATE 2026 conference network.
- ETH gains a unique formal-methods differentiator for their ternaryLLM project; Trinity specs provide a specification layer their current codebase lacks.
- Both parties gain a reference design that no competitor can replicate without equivalent formal foundations.

**Risk:** High. Requires legal review (ETH IP policy, open-source licensing alignment), quarterly sync meetings, and sustained commitment from both sides. Mitigate by starting with a 3-month pilot (Variant A/B hybrid with a single Alveo benchmark) and evaluating continuation based on pilot deliverables.

---

## Recommendation

**Lead with Variant A in W230.** Approach TilelliLab with a low-friction shared-benchmark invitation — they are brand-new and highly motivated for visibility. Simultaneously re-engage shepherdscientific (already receptive from W228 outreach). Use the resulting dual collaboration as social proof to recruit Neumann-Labs or deveworld into Variant B for a Q3 2026 joint submission. Defer Variant C until after the first joint publication establishes precedent and credibility with ETH.

**Primary blocker:** Trinity’s own arXiv v1 must be submitted **this week** (W229) to establish publication priority before any cooperation proposal is sent. Partners will only take Trinity seriously if it has a live pre-print.
