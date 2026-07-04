# Wave Loop 72 Plan — Coq Normalization + Lagrangian Sprint + AST Port Fix

**Date:** 2026-06-17

## Mission Statement

Resolve the Coq/Rocq toolchain duality blocking proof work, complete the Lagrangian open-problems sprint (#1167), fix the remaining AST path port collision (#965.2 Strategy B), and expand competitive monitoring to the Lean 4 arXiv channel.

---

## Track A — Compiler / CRITICAL

### A1. #965.2 Strategy B — AST path reserved-name scan
- **Objective:** Sanitize `gen_verilog` AST path to skip hardcoded ports (`clk`, `rst_n`, `en`, `ready`) when a top-level const/var already uses that name.
- **Approach:**
  1. In `gen_verilog` (line ~3906), before emitting the ANSI header, build a `HashSet<&str>` of all top-level declaration names.
  2. Elide any hardcoded port whose name is in the reserved set.
- **Complexity:** LOW (localized, single function).
- **Owner:** Agent C (Compiler).

### A2. C backend variable declaration types
- **Objective:** Fix `let arr = [3]f64{...}` still emitting `int arr = (f64[]){...}`.
- **Approach:** Add local type inference in `gen_c_local_var` by inspecting the RHS `ExprArrayLiteral`.
- **Complexity:** LOW–MEDIUM.
- **Owner:** Agent C.

---

## Track B — Proofs / Lagrangian

### B1. Coq toolchain normalization
- **Objective:** Eliminate Rocq 9.1.1 ↔ Coq 8.20 `.vo` version collision.
- **Approach (pick one):**
  1. **Upgrade path:** Migrate all proofs to Rocq 9.1.1 (rebuild all `.vo` files, update CI to use `rocq c`).
  2. **Pin path:** Enforce `COQBIN` in Makefile, regenerate all `.vo` files with Coq 8.20, and add a CI gate preventing Rocq contamination.
- **Recommendation:** Pin path — Coq 8.20 is battle-tested; Rocq 9.1.1 is new and may introduce breaking changes in `lra`/`nra` behavior.
- **Owner:** Agent L (Learner / Toolchain).

### B2. #1167 — Open problems sprint (3 new .v files)
- **Objective:** Create `CKMCPViolation.v`, `DarkMatterPhi.v`, `CosmologicalConstant.v` shells.
- **Prerequisite:** B1 (toolchain normalized).
- **Approach:**
  1. Each file: 5–10 `Theorem`/`Lemma` with `Admitted` (honest math).
  2. `Require Import` chains matching existing files.
  3. ASCII-only identifiers, no actionable TODOs.
  4. Literature citations in comments.
- **Owner:** Agent C (Coq proofs).

### B3. #1168 — Strengthen Higgs + SM proofs
- **Objective:** +5 Qed lemmas in `HiggsPotentialH4.v` and `SMLagrangian.v`.
- **Prerequisite:** B1.
- **Owner:** Agent C.

---

## Track C — Competitive Intelligence

### C1. September 2026 arXiv sweep
- **Objective:** Reschedule competitive sweep for early September 2026, after the 2607/2608/2609 arXiv batches are indexed.
- **Channels:** arXiv hep-th/hep-ph, Zenodo, viXra, Google Scholar alerts for known competitors.
- **Owner:** Agent E.

### C2. Lean 4 ecosystem maturity check
- **Objective:** Assess whether Washburn/GIFT/de la Fournière have published updated versions (v2, v3) or GitHub activity.
- **Owner:** Agent E.

---

## Cooperation Variants for Wave Loop 73

### Variant A — Coq/Rocq Contractor
Hire a freelance Coq specialist (e.g., via Upwork or academic network) to:
- Normalize the toolchain duality (2–4 hours).
- Deliver the 3 open-problem `.v` shells with syntactically valid `Admitted` theorems.
- Cost estimate: $800–$1,200 for a 2-day focused contract.
- Deliverable: Clean build, 3 new modules, updated `_CoqProject`.

### Variant B — Lean 4 Collaboration Letter
Draft and send a formal collaboration inquiry to **Jonathan Washburn** (arXiv:2506.12859v3) or the **GIFT** team, proposing:
- Joint paper on φ-based fermion mass formalization.
- Exchange of Coq ↔ Lean 4 translation methodology.
- Shared benchmark dataset for numeric verification.
- Goal: Convert a competitor into a collaborator, leveraging Trinity’s hardware/CORDIC/FPGA angle as differentiator.

### Variant C — FPGA + Conformance Grant
Apply for a small R&D grant (e.g., NLNet or CERN Openlab) focused on:
- GF14 hardware accelerator design (ASIC/FPGA).
- Bit-exact conformance certification for the full GoldenFloat format family.
- Open-source delivery with Trinity branding.
- Aligns with Trinity’s numeric-first differentiator vs. purely theoretical competitors.

---

## Success Criteria

| Criterion | Target |
|-----------|--------|
| Suite pass rate | 548/548 |
| Cargo tests | 534/534 |
| New issues closed | ≥3 (#965.2 full, #1146 already closed, #1167 deferred → closed in W72) |
| Coq toolchain duality | Resolved (one toolchain only) |
| New .v modules | ≥3 shells created |
| New Qed count | +5 minimum |
| Clippy warnings | 0 |
| Seal mismatches | 0 |
| Competitors discovered | ≥0 (stable acceptable) |

---

*End of Wave Loop 72 Plan*
