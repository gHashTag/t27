# Wave Loop 107 Cooperation Variants
## Real Synthesis Bridge + L4 Coverage + Bench Expansion + Honest Math Audit

**Date:** 2026-06-19
**Status:** Post-W106, Pre-W107

---

## Variant A: EDA Tool Integration Engineer (Yosys/Icarus Lead)

**Role:** Own the real synthesis and simulation pipeline. Make Trinity the first open-source IGLA with *verified* Pass@K metrics backed by actual Yosys synthesis and Icarus simulation.

**Deliverables:**
- `std::process::Command` wrapper in bootstrap/Zig runtime for `yosys` + `iverilog` + `vvp` invocation
- 5 hand-written testbenches (`tb_adder.v`, `tb_counter.v`, `tb_fsm.v`, `tb_uart_rx.v`, `tb_alu_slice.v`) in `data/testbenches/`
- `parse_yosys_json(json)` in `eval.t27` — reads real LUT/FF/MHz metrics from synthesis output
- `run_icarus_sim(rtl, tb)` with 30-second timeout guard
- Integration into `benchmark.t27`: evaluate 20 problems with real tool feedback
- CI gate: GitHub Actions job that installs Yosys + Icarus and runs synthesis on generated RTL

**Why it matters:** All competitors (StepPRM-RTL 85.7%, LLM4RTL 60.8%, CASS-RTL 48.7%) publish Pass@K scores but do not open-source their harness. A reproducible benchmark with sacred-constraint filtering + real synthesis becomes a defensible standard and a publication differentiator.

**Cooperation model:** Remote — PRs to `bootstrap/src/runtime/`, `specs/igla/coder/`, and `data/testbenches/`. Weekly sync on synthesis success rates. Requires local Yosys + Icarus installation.

---

## Variant B: Spec Quality Engineer (L4 + Bench Coverage Lead)

**Role:** Bring Trinity's spec coverage to industry standard. Add tests and benchmarks to the 14 naked specs and top 20 hot primitives.

**Deliverables:**
- Add `test` blocks to 14 specs currently missing ALL L4 coverage
- Add `bench` blocks to 20 hot primitives (ML layers, transformer blocks, ternary ISA, collections)
- Verify all new tests pass with `./target/release/t27c suite`
- Document test patterns in `.claude/skills/spec-testing-patterns.md`
- Add `t27c lint --ascii` CI gate that fails if any spec lacks `test` or `bench`

**Why it matters:** Competitors (RTLScout, CHIPCRAFTBRAIN) do not publish spec-first TDD artifacts. Trinity's 564 passing specs with formal invariants and benchmarks is a unique moat — but only if coverage is comprehensive. Closing the 2.5% L4 gap and adding performance baselines makes this moat defensible.

**Cooperation model:** Remote — PRs to `specs/**/*.t27`. Weekly test-coverage reports. No special tooling required.

---

## Variant C: Honest Math Auditor (Coq/Lean Bridge + Conjecture Documentation)

**Role:** Audit Trinity's mathematical claims for honesty. Document gaps, withdraw overreached formulas, and maintain the zero-fabrication standard.

**Deliverables:**
- Audit `CKMCPViolation.v`: document Jarlskog gap honestly; move to `Archive_Conjectural.v` if needed
- Audit `NeutrinoMasses.v` Axioms: add Derive Levels metadata mapping each Axiom to required prerequisite
- Audit `H4Lagrangian.v`: ensure all CONCEPTUAL/CONJECTURE labels are present
- Translate `Bounds_Masses.v` Q07/Q01/Q02/Q04 theorems to Lean 4 `BoundsMasses.lean`
- Add `lake build` CI check for Lean 4 bridge
- Document translation patterns in `.claude/skills/coq-to-lean-patterns.md`

**Why it matters:** The EXTREME threat from Washburn et al. (Lean 4, 0 sorry) and de la Fourniere (Lean 4 certified) means Trinity must compete on *honesty*, not just volume. A publicly visible audit trail of what's proven vs. conjectured vs. withdrawn builds trust with reviewers and makes collaboration easier.

**Cooperation model:** Academic — co-authorship on arXiv updates, weekly pair-programming on proof translation, shared `proofs/` repository access. Requires Coq 8.20+ and ideally Lean 4/Mathlib familiarity.

---

## Cross-Cutting Commitment

All three variants share:
- **L1 TRACEABILITY:** Every PR closes a GitHub issue or references #1032 sub-task.
- **L4 TESTABILITY:** Every `.t27` file has `test`/`invariant`/`bench`.
- **L3 PURITY:** ASCII-only, English identifiers.
- **L7 UNITY:** No `.sh` on critical path; use `tri` / `t27c`.
- **Honesty Protocol:** No fabricated proofs. Conjectures labeled. Withdrawn formulas removed from verified set.

---

phi^2 + 1/phi^2 = 3 | TRINITY
