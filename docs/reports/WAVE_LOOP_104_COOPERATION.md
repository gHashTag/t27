# Wave Loop 104 --- Three Cooperation Variants

**Focus:** Advanced sampling, dataset permutation, synthesis feedback, contrastive sacred learning.
**Date:** 2026-06-16
**Competitive Alert:** StepPRM-RTL (IBM, 0.857 Pass@1) leads; Trinity closes gap on beam search + PRM scoring.

---

## Variant A: Template Grammar Expansion (ML Engineer / Verilog Specialist)

**Context:** Dataset has ~1,280 conceptual samples (40 base × 8 mutations × 4 permutations). Sub-1B training needs 10K+. Current templates are fixed; need compositional grammar.

**What we offer:**
- Co-authorship on arXiv preprint: "Scaling Sacred-Constraint RTL Generation through Compositional Template Grammars".
- Full t27 spec codebase access.
- Integration with existing Yosys synthesis pipeline.

**What we need:**
- Implement hierarchical RTL composition: combine counter + shift_register → UART RX; adder_tree + Booth → MAC unit.
- Add 10+ new primitive templates (mux, decoder, encoder, ALU slice, comparator, priority arbiter, barrel shifter).
- Build random parameter generator: port names, wire counts, FSM state counts, pipeline stages.
- Generate cycle-accurate testbenches for each composed template.
- Target: 10,000+ unique (prompt, RTL, testbench, sacred_label) quadruples.

**Deliverable:** Dataset generation pipeline producing 10K+ valid training samples with sacred/non-sacred labels.
**Timeline:** 2--3 months.
**Risk:** Medium — requires both Verilog expertise and generative data engineering.

---

## Variant B: Benchmark & Evaluation Lead (Competitive Intelligence + EDA Integration)

**Context:** IBM StepPRM-RTL (0.857 Pass@1), ACE-RTL (NVIDIA, +41% Agentic Pass Rate), and VeriAgent (Xiamen/Tsinghua/HIT) have published benchmark scores. Trinity has none. No Pass@1 on VerilogEval v2.

**What we offer:**
- Co-authorship on benchmark report: "Sacred-Constraint RTL Generation: Baseline and Competitive Analysis".
- Access to Trinity's t27c compiler, IGLA-Coder architecture spec, and PRM reward model.
- Recognition in project README and competitive positioning docs.

**What we need:**
- Set up VerilogEval v2 benchmark harness (Python + Yosys + Icarus Verilog).
- Evaluate current IGLA-Coder templates against VerilogEval-human and VerilogEval-machine.
- Report Pass@1, Pass@5, Pass@10 metrics with temperature sweep.
- Compare with published competitor scores (StepPRM-RTL, ACE-RTL, VeriAgent, COEVO).
- Identify failure modes (synthesis errors, functional mismatches, sacred-constraint violations) and feed back into dataset augmentation.
- Document the R-SI-1 Pass@K penalty: measure how many generated samples fail sacred compliance.

**Deliverable:** Benchmark report with Pass@K scores, sacred-compliance rate, and failure analysis.
**Timeline:** 1--2 months.
**Risk:** Low — well-defined scope; may reveal low scores but provides honest baseline.

---

## Variant C: Contrastive Training Pipeline Engineer (ML Researcher / Rust Systems)

**Context:** Trinity now generates contrastive pairs (sacred vs non-sacred RTL) and has PRM `preference_loss`. These are not connected. Needs a training pipeline that consumes contrastive pairs and optimizes the PRM.

**What we offer:**
- Bounty: $4000--$6000 per merged training pipeline PR.
- Co-authorship on technical report: "Contrastive Sacred Learning for Hardware Code Generation".
- Direct mentorship on t27c Rust compiler internals and spec-first methodology.

**What we need:**
- **Task 1:** Implement `contrastive_batch_generator` that streams `(positive, negative, label)` triples from `generate_contrastive_pair`.
- **Task 2:** Wire `preference_loss(chosen, rejected)` to consume contrastive pairs and compute gradient direction.
- **Task 3:** Implement `train_prm_step` that updates PRM weights via contrastive preference optimization (policy gradient or DPO-style).
- **Task 4:** Add `sacred_compliance_accuracy` metric tracking % of generated RTL that passes R-SI-1 during training.
- **Task 5:** Implement `checkpoint_save` / `checkpoint_load` for PRM weights in Safetensors format.

**Deliverable:** End-to-end training loop: dataset → contrastive pairs → preference loss → PRM weight update → sacred compliance metric.
**Timeline:** 2--3 months.
**Risk:** Medium — requires ML training expertise + Rust/t27c systems knowledge.

---

## Recommended Priority

1. **Variant B** (Benchmark Baseline) — Immediate competitive need. Even low scores provide honest data and roadmap. Establishes credibility.
2. **Variant A** (Template Grammar Expansion) — Parallel with B. Required for training and benchmark improvement. Unblocks real model training.
3. **Variant C** (Contrastive Training Pipeline) — Research unlock. Connects dataset generation to PRM optimization. Highest long-term impact.

---

## Competitive Strategy Update

With StepPRM-RTL (0.857 Pass@1) leading and ACE-RTL (NVIDIA) leveraging agentic context evolution, Trinity's differentiation is under pressure on:
- **Benchmark scores** — competitors have Pass@1 numbers; Trinity has none.
- **Dataset scale** — ACE-RTL uses 1.7M samples; Trinity has ~1,280.
- **Tool integration** — VeriAgent has closed-loop EDA integration; Trinity has conceptual `rank_rtl_variants`.

**Trinity's sustainable moats remain:**
1. **Sacred-constraint R-SI-1** (zero `*` operators) — no competitor enforces this at architecture level. Contrastive sacred pairs encode this as a trainable signal.
2. **Formal verification bridge** — Coq/Lean proofs as reward signals is unique among RTL generators.
3. **Spec-first methodology** — 562 specs with TDD provides transparency and auditability.
4. **Beam search + PRM integration** — Now matches StepPRM-RTL's core architecture (step-level rewards + search).

**Immediate actions:**
- Establish benchmark baseline (Variant B) within 1 month.
- Scale dataset to 10K+ samples (Variant A) within 2 months.
- Build contrastive training pipeline (Variant C) within 3 months.

---

phi^2 + 1/phi^2 = 3 | TRINITY
