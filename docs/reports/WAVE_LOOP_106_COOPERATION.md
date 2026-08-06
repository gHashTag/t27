# Wave Loop 106 — Three Cooperation Variants

**Focus:** Benchmark evaluation integrity, beam-search + PRM pipeline, real-world RTL templates, auto testbench generation.
**Date:** 2026-06-16
**Competitive Alert:** StepPRM-RTL (0.857 Pass@1), EstRTL (0.705), LLM4RTL (0.608/0.667), CASS-RTL (0.487/0.525). RTL-BenchLS sets the dataset scale bar at 10K+.

---

## Variant A: Empirical Pass@K Evaluation (ML Engineer / GPU Access)

**Context:** Wave Loop 105–106 built complete benchmark infrastructure (`benchmark.t27`), competitor presets, and dataset bridge — but Trinity has **never run an actual Pass@K evaluation** on VerilogEval or a comparably sized dataset. All scores are conceptual. StepPRM-RTL reports 0.857; Trinity reports nothing.

**What we offer:**
- Co-authorship on technical report: "Sacred-Constraint RTL Generation: First Empirical Pass@K Baseline".
- Full t27 spec codebase access (564 specs, TDD-first).
- Existing benchmark harness with `BenchmarkTask`, `BenchmarkResult`, `compute_aggregate_report`, and `sacrebleu_precision`.
- Pre-loaded competitor presets for direct comparison.

**What we need:**
- GPU compute or API budget (OpenAI, Anthropic, or local LLM) to generate RTL from prompts at scale.
- Run Pass@1 / Pass@5 / Pass@10 on VerilogEval-human and VerilogEval-machine subsets.
- Temperature sweep (0.2, 0.5, 0.8, 1.0) with sacred-compliance filtering.
- Measure **R-SI-1 violation rate**: % of generated samples containing `*` operator in assign statements.
- Compare generated scores against published competitor baselines.
- Feed failure modes (syntax errors, synthesis failures, functional mismatches, sacred violations) back into `dataset.t27` augmentation.

**Deliverable:** Empirical benchmark report with Trinity Pass@K scores, sacred-compliance rate, synthesis success rate, and failure taxonomy.
**Timeline:** 1–2 months.
**Risk:** Low — well-defined scope. Even modest scores provide honest competitive positioning.

---

## Variant B: Simulator Subprocess Integration (EDA Engineer / CI DevOps)

**Context:** `eval.t27` now has `generate_testbench_for_template` and `run_verilog_simulation_with_testbench`, but both are **conceptual stubs** — they return strings and `SimResult` structs without invoking real `yosys`, `iverilog`, or `verilator`. The synthesis bridge parses hypothetical log formats. Competitors (RTL-BenchLS, CASS-RTL) run real simulation + formal equivalence checking.

**What we offer:**
- Co-authorship on technical report: "Automated Simulation and Synthesis Verification for Sacred-Constraint RTL".
- Full t27 spec codebase and IGLA RACE synthesis pipeline.
- Existing Yosys log parser and testbench templates for adder, counter, fifo, uart_tx, priority_arbiter, barrel_shifter, mac_unit.
- CI integration points (`scripts/tri`, GitHub Actions).

**What we need:**
- Replace conceptual `SimResult` stubs with real subprocess invocations:
  - `iverilog -g2012 -o sim.vvp tb.sv rtl.sv && vvp sim.vvp`
  - `yosys -p "read_verilog rtl.sv; synth_ice40"`
  - `verilator --lint-only -Wall rtl.sv`
- Parse actual Yosys log output for cell count, critical path, and synthesis success/failure.
- Parse actual Icarus/Vilator stdout for `$display("PASS")` / `$display("FAIL")` assertions.
- Integrate into `scripts/tri test --sim` so CI can gate on simulation pass.
- Support all 7 templates (adder, counter, fifo, uart_tx, priority_arbiter, barrel_shifter, mac_unit) with self-checking testbenches.

**Deliverable:** Real simulator subprocess pipeline integrated into `tri` CLI and CI; synthesis reports per template; simulation PASS/FAIL gating.
**Timeline:** 1–2 months.
**Risk:** Medium — requires EDA toolchain setup (Yosys, Icarus, Verilator) and robust log parsing.

---

## Variant C: PRM Fine-Tuning on RTL Data (ML Researcher / Reward Modeling)

**Context:** Wave Loop 106 wired `beam_search_prm_pipeline` — it calls `prm::score_beam_with_prm`, but the PRM is **static** (hand-written heuristic scores). Process Reward Models in StepPRM-RTL and LLM4RTL are trained on outcome labels (syntax correct? synthesizes? functionally equivalent?). Trinity has no training loop.

**What we offer:**
- Bounty: $6000–$10000 per merged PRM training pipeline.
- Co-authorship on technical report: "Training Process Reward Models for Sacred-Constraint RTL Generation".
- Direct mentorship on t27c Rust compiler internals and spec-first methodology.
- Existing beam-search infrastructure and dataset with golden RTL for supervised reward labeling.

**What we need:**
- **Task 1:** Build RTL-specific reward dataset: `(prompt, candidate_rtl, outcome_label)` where `outcome_label` ∈ {syntax_ok, synthesis_ok, functional_eq, sacred_compliant}.
- **Task 2:** Implement lightweight PRM head (sub-10M parameters) on top of existing IGLA CODER backbone or as standalone classifier.
- **Task 3:** Train PRM with binary cross-entropy on outcome labels; validate on held-out VerilogEval subset.
- **Task 4:** Integrate trained PRM into `prm::score_beam_with_prm` so beam search uses learned, not heuristic, scores.
- **Task 5:** Measure Pass@K improvement (with vs without trained PRM) on benchmark suite.
- **Task 6:** Document reward hacking risks (e.g., PRM overfitting to syntax correctness while missing functional equivalence).

**Deliverable:** Trained PRM checkpoint + training code + benchmark showing ≥5% Pass@1 improvement over heuristic scoring.
**Timeline:** 2–3 months.
**Risk:** High — research direction; may not yield measurable improvement; requires compute budget.

---

## Recommended Priority

1. **Variant B** (Simulator Integration) — Unlocks real verification. Without it, all Pass@K scores remain hypothetical. Highest engineering ROI.
2. **Variant A** (Empirical Pass@K) — Parallel with B. Provides competitive numbers but requires API/GPU budget.
3. **Variant C** (PRM Fine-Tuning) — Long-term research unlock. Most impactful if successful; builds on A and B outputs.

---

## Competitive Strategy Update

With StepPRM-RTL (0.857 Pass@1), EstRTL (0.705), and LLM4RTL (0.608/0.667), the LLM-for-RTL space is now benchmark-saturated. Trinity's differentiation must shift from "having infrastructure" to "proving empirical superiority on sacred constraints":

| Competitor | Pass@1 | Sacred Constraint | Formal Verification |
|------------|--------|-------------------|---------------------|
| StepPRM-RTL | 0.857 | None | None |
| EstRTL | 0.705 | None | None |
| LLM4RTL | 0.608 | None | None |
| CASS-RTL | 0.487 | None | None |
| **Trinity (projected)** | **TBD** | **R-SI-1 (zero `*`)** | **Coq/Lean bridge** |

**Trinity's sustainable moats:**
1. **R-SI-1 sacred invariant** — no competitor enforces zero `*` operators at generation time.
2. **Formal verification bridge** — Coq/Lean proof-as-reward is unique among RTL generators.
3. **Spec-first TDD** — 564 specs provide transparency, auditability, and deterministic regression.
4. **Auto testbench + synthesis verification** — once Variant B lands, Trinity will be the only open-source pipeline with built-in simulation gating.

**Immediate actions:**
- Land simulator subprocess integration (Variant B) within 4 weeks.
- Run first empirical Pass@K sweep (Variant A) within 6 weeks.
- Begin PRM dataset curation (Variant C prep) within 8 weeks.

---

phi² + 1/φ² = 3 | TRINITY
