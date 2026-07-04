# Wave Loop 74 Plan — C Backend Parity + Conformance Vectors + arXiv Sprint

**Date:** 2026-06-17  
**Target End:** 2026-06-24  

---

## Mission Statement

Close the last major backend parity gap (C array type inference), generate bit-exact conformance vectors for GF14, advance the arXiv preprint competitive-differentiation section, and maintain the zero-failure suite while monitoring the Lean 4 competitive front.

---

## Track A — Compiler / CRITICAL

### A1. C backend local-array type inference
- **Objective:** Fix `let arr = [3]f64{...}` emitting `int arr = (f64[]){...}` in `gen_c_local_var`.
- **Approach:** Inspect RHS `ExprArrayLiteral`; if element type is `f64`/`f32`/`i64`/`i32`, emit matching C array type instead of default `int`.
- **Complexity:** LOW–MEDIUM (localized to `gen_c.rs`, needs 1–2 pattern matches).
- **Owner:** Agent C (Compiler).
- **Acceptance:** C conformance test for array literal compiles with `gcc -Wall -Werror`.

### A2. Parser body-truncation diagnostic
- **Objective:** Add a compiler warning when `parse_block` truncates a body due to depth/nesting limits.
- **Approach:** In `parser.rs`, when returning early from a deeply-nested block, emit a `Diagnostic::Warning` with file/line.
- **Complexity:** LOW.
- **Owner:** Agent C.

### A3. Seal integrity weekly check
- **Objective:** Run `./scripts/tri seal-check` on CI and auto-regenerate if 3+ consecutive waves show recurring mismatches.
- **Owner:** Agent V (Verifier).

---

## Track B — Proofs / Physics

### B1. Neutrino oscillation mass-squared differences
- **Objective:** Prove `Delta_m21_sq_pos` and `Delta_m31_sq_pos` in `NeutrinoMasses.v` using `interval` and the corrected `M_R ~ Λ` seesaw framework.
- **Approach:**
  1. Expand `Delta_m21_sq` and `Delta_m31_sq` definitions from the light-neutrino mass matrix.
  2. Apply `unfold; field_simplify; interval` with `v_EW`, `Lambda_600`, `Y_e` bounds.
- **Complexity:** MEDIUM (interval bounds may need splitting if `lra` fails on nested `Rpow`).
- **Owner:** Agent C (Coq proofs).
- **Acceptance:** Two new `Qed` lemmas; no new `Admitted`.

### B2. arXiv preprint — competitive differentiation section
- **Objective:** Write §4 "Competitive Landscape" for the arXiv draft, covering:
  - Washburn (RCL, Lean 4, 0 sorry, φ-based).
  - GIFT (E₈, Lean 4, 460+ proofs).
  - McGirl / Gray (H4 observables, 600-cell).
  - Singh (E₆, exceptional Jordan algebra).
  - de la Fournière (φ-mass, Lean 4).
  - **Trinity differentiator:** Hardware-numeric angle (CORDIC, GF16, conformance vectors, zero-Admitted Coq).
- **Owner:** Agent L (Learner / Writer).
- **Acceptance:** Compiled PDF with ≤2 LaTeX warnings; all citations verifiable.

### B3. Lean 4 bridge expansion
- **Objective:** Translate 5 additional lemmas from `CorePhi.v` into the `trinity-lean4` lake project.
- **Target lemmas:** `phi_definition`, `phi_golden_relation`, `phi_irrational` (conjecture), `phi_sq_eq_phi_plus_1`, `phi_pow_recurrence`.
- **Owner:** Agent C (Lean 4).
- **Acceptance:** `lake build` succeeds; `lean4/test_core_phi.lean` passes.

### B4. CKM CP-violation conjecture (Archive)
- **Objective:** Formalize the `delta_CK = PI / phi^2` ansatz as a **conjecture** in `Archive_Conjectural.v`, not a production theorem.
- **Rationale:** Honest math protocol — conjectures belong in the archive, not masquerading as `Admitted` theorems.
- **Owner:** Agent C.

---

## Track C — Competitive Intelligence

### C1. September 2026 arXiv sweep
- **Objective:** Search hep-th/hep-ph/math-ph for new competitors using keywords:
  - `"600-cell"`, `"H4 root system"`, `"golden ratio"`, `"fermion masses"`, `"exceptional Jordan algebra"`.
- **Channels:** arXiv RSS, Zenodo, viXra.
- **Owner:** Agent E.

### C2. Washburn / GITHUB activity check
- **Objective:** Check if Washburn arXiv:2506.12859v3 has a v4 revision, or if GIFT repo has new commits/tags.
- **Owner:** Agent E.

### C3. Lean 4 maturation signal
- **Objective:** Count new Lean 4 physics formalization repos on GitHub (created since June 2026).
- **Owner:** Agent E.

---

## Cooperation Variants for Wave Loop 75

### Variant A — Academic Contractor (Coq + Neutrino)
Hire a senior post-doc or PhD candidate with Coq + high-energy physics background (e.g., via Inria Sophia, IAS, or Nijmegen) for a **2-week focused contract** to:
- Derive `delta_CK` from H4 root-system geometry (or prove impossibility).
- Close `Delta_m31_sq` interval proof using `coq-interval` advanced tactics.
- Deliver: 1–3 new `Qed` theorems + short technical memo.
- **Cost estimate:** €2,500–€4,000 (contractor rate).
- **Risk:** Low; scoped deliverables; Trinity retains IP.

### Variant B — Industry Partnership (FPGA Accelerator)
Engage a small FPGA/IP firm (e.g., Antmicro, Cypress, or a university spin-out) to:
- Synthesize the CORDIC core to a real Xilinx/Intel target.
- Generate bitstream and run GoldenFloat GF16 conformance vectors on hardware.
- Produce a joint white-paper demonstrating bit-exact verification on FPGA.
- **Value:** Converts Trinity from "software spec" to "hardware-certified" — massive differentiator vs. purely theoretical competitors.
- **Cost estimate:** $15,000–$25,000 (small engagement).
- **Risk:** Medium; depends on partner availability and NDAs.

### Variant C — Open-Source Collaboration (Lean 4 ↔ Coq Bridge)
Propose a **formal translation challenge** to the Lean 4 community (e.g., via Zulip or the Mathlib maintainers):
- Trinity translates its 20 most fundamental `CorePhi.v` lemmas into Lean 4.
- In exchange, a Mathlib maintainer reviews the translation and cites Trinity in the `mathlib4` docs.
- Creates cross-community credibility and exposes Trinity’s physics content to Lean 4 users (where the competitor attention is highest).
- **Cost:** Near-zero (volunteer labor + coordination time).
- **Risk:** Low; high reputational upside.

---

## Success Criteria

| Criterion | Target |
|-----------|--------|
| Suite pass rate | 548/548 |
| Cargo tests | 534/534 |
| Coq build | 0 errors, 0 active `Admitted` |
| New `Qed` lemmas | ≥2 (B1) |
| C backend array type fix | 1 PR merged |
| arXiv §4 written | Draft in `docs/arXiv/` |
| Lean 4 lemmas translated | +5 |
| New competitors discovered | ≥0 (stable acceptable) |
| Clippy warnings | 0 |
| Seal mismatches | 0 |

---

*End of Wave Loop 74 Plan*
