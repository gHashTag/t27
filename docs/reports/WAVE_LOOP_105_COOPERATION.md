# Wave Loop 105 --- Three Cooperation Variants

**Focus:** Benchmark infrastructure, compositional dataset scaling, synthesis bridge hardening.
**Date:** 2026-06-16
**Competitive Alert:** RTL-BenchLS (10K+ verified designs), CASS-RTL (+10-20% training-free), LLM4RTL (GPT-4O on 7B) raise the bar.

---

## Variant A: Benchmark & Pass@K Lead (ML Engineer / EDA Integration)

**Context:** Trinity now has `benchmark.t27` with `BenchmarkTask`, `BenchmarkResult`, and `compute_aggregate_report`. But we have **no real VerilogEval harness** — we cannot measure actual Pass@1/5/10 on a standard dataset. StepPRM-RTL reports 0.857; Trinity reports nothing.

**What we offer:**
- Co-authorship on technical report: "Sacred-Constraint RTL Generation: Pass@K Baseline and Competitive Analysis".
- Full t27 spec codebase access (563 specs, TDD-first).
- Integration with existing Yosys synthesis pipeline and Icarus Verilog simulation stubs.

**What we need:**
- Set up VerilogEval v2 or RTL-BenchLS benchmark harness (Python + Yosys + Icarus Verilog + Cocotb).
- Evaluate current IGLA-Coder templates against VerilogEval-human and VerilogEval-machine.
- Report Pass@1, Pass@5, Pass@10 with temperature sweep (0.2, 0.5, 0.8, 1.0).
- Measure **sacred-compliance penalty**: % of generated samples that fail R-SI-1 (zero `*` operators).
- Compare with published competitor scores (StepPRM-RTL 0.857, CASS-RTL, LLM4RTL, EstRTL).
- Feed failure modes (syntax errors, functional mismatches, sacred violations) back into dataset augmentation.

**Deliverable:** Benchmark report with Pass@K scores, sacred-compliance rate, synthesis success rate, and failure analysis.
**Timeline:** 1--2 months.
**Risk:** Low -- well-defined scope. Even low scores provide honest data and credibility.

---

## Variant B: Compositional Dataset Engineer (Verilog Specialist / Generative Data)

**Context:** Trinity's dataset has ~1,280 conceptual samples (40 base × 8 mutations × 4 permutations). RTL-BenchLS has 10,000+ formally verified designs. We have `compose_modules` and `expand_dataset_compositional` but no real population.

**What we offer:**
- Co-authorship on dataset paper: "Scaling RTL Training Data through Hierarchical Template Composition".
- Full t27 spec codebase access and IGLA RACE synthesis pipeline.
- Recognition in project README and competitive positioning docs.

**What we need:**
- Implement 10+ new primitive templates (mux, decoder, encoder, ALU slice, comparator, priority arbiter, barrel shifter, FIFO, LFSR, UART TX).
- Build hierarchical composition rules: counter + shift_register → UART RX; adder + Booth → MAC; FSM + counter → memory controller.
- Generate **cycle-accurate testbenches** for each composed template (Cocotb or Verilator).
- Run Yosys synthesis on all composed modules and verify synthesis success.
- Target: **10,000+ unique (prompt, RTL, testbench, sacred_label, yosys_report) quintuples**.
- Integrate with `benchmark.t27` so every sample is a `BenchmarkTask`.

**Deliverable:** Dataset of 10K+ benchmark-ready training samples with synthesis-verified PPA metrics.
**Timeline:** 2--3 months.
**Risk:** Medium -- requires both Verilog expertise and generative data engineering.

---

## Variant C: Training-Free Correctness Steering (ML Researcher / Activation Engineering)

**Context:** CASS-RTL (arXiv:2606.05680) achieves +10--20% functional correctness **without fine-tuning** by steering LLM internal activations toward correctness during inference. Trinity has no equivalent mechanism. This is a potential leap: no dataset collection, no gradient computation, just inference-time steering.

**What we offer:**
- Bounty: $5000--$8000 per merged steering mechanism PR.
- Co-authorship on technical report: "Sacred-Constraint Steering for Hardware Code Generation".
- Direct mentorship on t27c Rust compiler internals and spec-first methodology.

**What we need:**
- **Task 1:** Implement `sacred_constraint_steering(logits, sacred_embed)` that adjusts logits to penalize tokens leading to `*` operator in RTL output.
- **Task 2:** Implement `correctness_subspace_projection(hidden_states, sacred_compliance_embed)` that projects hidden states onto a 2D sacred-compliance subspace (like CASS-RTL but for R-SI-1).
- **Task 3:** Add `adaptive_temperature_by_sacred_score(score, base_temp)` that lowers temperature when sacred compliance is at risk.
- **Task 4:** Measure Pass@1 improvement on VerilogEval with and without steering.
- **Task 5:** Document the steering vector (which hidden dimensions correspond to sacred vs non-sacred generation).

**Deliverable:** Training-free inference-time steering mechanism that improves sacred-compliance rate by ≥10% on VerilogEval.
**Timeline:** 2--3 months.
**Risk:** High -- research direction; may not yield +10% improvement.

---

## Recommended Priority

1. **Variant A** (Benchmark Baseline) -- Immediate competitive need. Establishes credibility and honest Pass@K numbers.
2. **Variant B** (Compositional Dataset) -- Parallel with A. Unblocks real model training and closes the 10K gap.
3. **Variant C** (Training-Free Steering) -- Research unlock. Highest long-term impact if successful; aligns with CASS-RTL trend.

---

## Competitive Strategy Update

With RTL-BenchLS (10K+ verified designs), LLM4RTL (7B ≈ GPT-4O), and CASS-RTL (training-free +10-20%), the LLM-for-RTL space is maturing rapidly. Trinity's differentiation is under pressure on:
- **Dataset scale** -- RTL-BenchLS has 10K+; Trinity has ~1,280.
- **Model size** -- LLM4RTL shows small models can compete; Trinity's sub-1B is ambitious but unproven.
- **Training-free improvements** -- CASS-RTL makes fine-tuning optional; Trinity has no steering mechanism.

**Trinity's sustainable moats remain:**
1. **Sacred-constraint R-SI-1** (zero `*` operators) -- no competitor enforces this at architecture level.
2. **Formal verification bridge** -- Coq/Lean proofs as reward signals is unique among RTL generators.
3. **Spec-first methodology** -- 563 specs with TDD provides transparency and auditability.
4. **Benchmark infrastructure** -- Now matches competitors on evaluation architecture (Pass@K, SacreBLEU, PPA).

**Immediate actions:**
- Establish benchmark baseline (Variant A) within 1 month.
- Scale dataset to 10K+ samples (Variant B) within 2 months.
- Build training-free steering mechanism (Variant C) within 3 months.

---

phi^2 + 1/phi^2 = 3 | TRINITY
