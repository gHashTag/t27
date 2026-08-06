# Wave Loop 103 --- Three Cooperation Variants

**Focus:** IGLA CODER x IGLA RACE --- KV-cache runtime, dataset scale, hybrid tokenizer, Yosys subprocess.
**Date:** 2026-06-17
**Competitive Alert:** StepPRM-RTL (IBM) remains EXTREME threat. No new July 2026 competitors detected.

---

## Variant A: Runtime Engineer / Compiler Specialist (Immediate Priority)

**Context:** t27c generates code for 6 backends but lacks runtime primitives needed for model inference and hardware synthesis.

**What we offer:**
- Bounty: $3000--$5000 per merged runtime primitive PR.
- Authorship on technical blog post: "Implementing LLM Inference in a Spec-First Compiler".
- Direct mentorship from core t27c compiler team (Rust + Zig).

**What we need:**
- **Task 1: 2D array append primitive** --- Implement `append_2d_row(matrix: [][]T, row: []T) -> [][]T` in t27c Zig backend. This unblocks KV-cache incremental update.
- **Task 2: Subprocess spawn** --- Implement `spawn_process(cmd: string, args: []string) -> ProcessHandle` using `std.process.Child` in Zig backend. This unblocks real Yosys CLI integration.
- **Task 3: String replace** --- Implement `string_replace(haystack: string, needle: string, replacement: string) -> string` in t27c C/Zig backend. This unblocks port-name mutation in dataset engine.

**Deliverable:** Merged PRs with tests + seal regen + documentation.
**Timeline:** 1--2 months per task (parallelizable).
**Risk:** Low --- scoped, well-defined tasks.

---

## Variant B: ML Research Engineer (Dataset + Training Pipeline)

**Context:** Dataset mutation engine generates ~320 samples. Real training needs 10K+ with parameter permutation, port renaming, comment insertion, and synthetic Verilog generation.

**What we offer:**
- Co-authorship on arXiv preprint: "IGLA: Sacred-Constraint RTL Generation at Scale".
- Access to full t27 spec codebase and Yosys synthesis toolchain.
- Integration with existing VerilogEval benchmark infrastructure.

**What we need:**
- Build template parameter permutation engine (clock polarity, reset level, signed/unsigned) expanding dataset to 2000+ samples.
- Implement synthetic Verilog generator using Trinity templates as building blocks (combinational + sequential composition).
- Run small-scale training experiment (sub-100M params) on expanded dataset with R-SI-1 reward shaping.
- Evaluate on VerilogEval v2 benchmark and report Pass@1.

**Deliverable:** Dataset generation pipeline + training run results + benchmark report.
**Timeline:** 3--4 months.
**Risk:** Medium --- depends on GPU access; benchmark may underperform vs IBM baseline.

---

## Variant C: Academic Formal Verification Partner (Coq/Lean Bridge)

**Context:** Trinity has 78 Coq Qed proofs and a Lean 4 bridge under construction. No competitor combines LLM code generation with formal verification.

**What we offer:**
- Co-authorship on paper bridging formal verification and neural code generation.
- Authorship on Trinity Lean 4 library contributions.
- Access to Trinity's Coq proof corpus (neutrino masses, spectral action, CKM matrices).

**What we need:**
- Complete Lean 4 translation of CorePhi.v (16 lemmas already started; remaining ~40 lemmas).
- Implement Coq proof checker as reward signal for PRM: `reward_formal(step) -> RewardSignal`.
- Prove R-SI-1 compliance property for generated RTL: "no multiplication operator in assignments" as Coq theorem.
- Integrate Lean 4 proof state into IGLA Coder training loop.

**Deliverable:** Lean 4 library + Coq proof integration + joint paper.
**Timeline:** 4--6 months.
**Risk:** Low --- academic scope; high publishability.

---

## Recommended Priority

1. **Variant A** (Runtime Engineer) --- Immediate unblock of KV-cache + subprocess. Highest engineering ROI.
2. **Variant B** (ML Research Engineer) --- Parallel with Variant A; builds dataset and training pipeline.
3. **Variant C** (Formal Verification) --- Long-term academic differentiation. Can start immediately but delivers over 4--6 months.

---

## Competitive Strategy Update

IBM StepPRM-RTL (arXiv:2606.04246) achieved 0.857 Pass@1 on VerilogEval-human. Trinity IGLA Coder does not yet have a trained model, so direct benchmark comparison is impossible.

**Trinity's path to competitiveness:**
1. Complete runtime primitives (Variant A) --- 1--2 months
2. Scale dataset to 10K+ samples (Variant B) --- 2--3 months
3. Train sub-1B model with R-SI-1 reward shaping (Variant B) --- 3--4 months
4. Publish benchmark results + formal verification integration (Variant C) --- 4--6 months

**Realistic timeline to working demo:** 6--8 months with 2--3 parallel collaborators.
**Realistic timeline to benchmark competitiveness:** 12--18 months with GPU cluster access.

---

phi^2 + 1/phi^2 = 3 | TRINITY
