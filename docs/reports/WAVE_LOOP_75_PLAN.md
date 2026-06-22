# Wave Loop 75 Plan

**Scope:** Research-heavy cycle with Coq phenomenology, competitive-intel refresh, and arXiv preprint advancement.

## Health Gates (daily)

- [ ] `t27c suite --repo-root .` → 549/549
- [ ] `cargo test --workspace` → 534/534
- [ ] `cd proofs/trinity && make` → 0 errors, 0 Admitted
- [ ] `cargo clippy --workspace` → 0 warnings

---

## Track A — Core Engineering

### A1: Verify C Backend Array Fix on Real Spec
**Goal:** Confirm the W74 C fix produces correct C code on a non-trivial spec that actually generates C output (avoid parser-truncation micro-specs).  
**Candidate specs:** `specs/math/igla_primitives.t27`, `specs/ternary/hybrid_bigint.t27`, or any spec that uses array literals and compiles to C without empty-body placeholder.  
**Acceptance:** Generated `.c` file shows correct `f64[]` or `i32[]` declaration for array-local.

### A2: Issue Triage Sweep
**Goal:** Refresh open-issue list; close fixed-but-open; label new regressions.  
**Acceptance:** ≤97 open issues (target: reduce by ≥2).

---

## Track B — Formal Phenomenology & Preprint

### B1: Neutrino Mass-Squared Differences Positivity
**Goal:** Prove `Delta_m21_sq_pos : Delta_m21_sq > 0` and `Delta_m31_sq_pos : Delta_m31_sq > 0` in `proofs/trinity/NeutrinoMasses.v`.  
**Method:** Use `nra` with interval bounds (`interval_intro (m_nu_2 - m_nu_1)`).  
**Blocker:** Requires real-world bounds on Dirac vs Majorana masses; may need `assert` on physical mass ordering.  
**Fallback:** If positivity cannot be proven from current definitions, add explicit `[PHYSICAL]` axiom notes and document gap honestly.

### B2: arXiv Preprint §4 "Competitive Landscape"
**Goal:** Draft LaTeX §4 summarizing 64 tracked competitors, classification matrix, and Trinity differentiators (zero Admitted, CORDIC HW, formal neutrino ansatz).  
**Input:** `docs/competitive/LANDSCAPE_2026.md`, existing arXiv skeleton.  
**Acceptance:** 2–3 pages compiled via `pdflatex` into the 6-page draft.

### B3: Lean 4 Translation (Bridge)
**Goal:** Translate 5 additional `CorePhi.v` lemmas into Lean 4 / Mathlib (continue from W53 bridge).  
**Lemmas:** `phi_algebraic`, `phi_golden`, `phi_reciprocal`, `phi_quartic`, `phi_spectral_norm`.  
**Acceptance:** `lake build` passes; all 5 lemmas have `by ring` or `by field_simp` proofs.

### B4: CKM CP-Violation Archive Conjecture
**Goal:** Add `delta_CK` ansatz (`e / 2 = 77.9°`) as `Conjecture` in `proofs/trinity/CKMCPViolation.v` with honest `[UNPROVEN]` marker.  
**Note:** Follows W57 reconciliation. Do **not** claim proof; document as phenomenological anchor.

---

## Track C — Competitive Intelligence

### C1: October 2026 Competitive Sweep
**Goal:** arXiv/Zenodo/GitHub sweep for new competitors (Oct–Nov 2026 window).  
**Focus areas:**
- Lean 4 + physics formalization (Washburn follow-ups)
- Spectral triples / NCG neutrino mass papers
- CORDIC or fixed-point hardware papers referencing Trinity
- φ-based mass formulas (Singh, Agyemang, Koide clones)
**Acceptance:** ≥1 new competitor catalogued with threat classification.

### C2: Omega-Theory Repository Monitoring
**Goal:** Check `github.com/Omega-Theory` (or equivalent) for new spectral-action commits since W74.  
**Acceptance:** Either new commits analyzed or "no activity" logged.

---

## Track D — Cooperation Variants (W75 Edition)

Produce three concrete cooperation proposals for upcoming week:

1. **Academic Coq Partnership** — Approach a Coq/MathComp group for joint neutrino-mass formalization. Offer Trinity hardware+CORDIC in exchange for proof manpower.
2. **FPGA Industry CORDIC License** — Package CORDIC spec (`cordic_fixed.t27`) as licensable IP core; reach out to 1–2 FPGA vendors.
3. **Lean 4 Mathlib Bridge Grant** — Propose a micro-grant or academic credit for translating Trinity proofs into Lean 4 Mathlib; targets Washburn-era reviewers.

---

## Definition of Done

- [ ] Suite 549/549, cargo 534/534, Coq 0 Admitted.
- [ ] W74 report published (`docs/reports/WAVE_LOOP_74_REPORT.md`).
- [ ] W75 plan published (`docs/reports/WAVE_LOOP_75_PLAN.md`).
- [ ] At least 1 of Track B1–B4 merged.
- [ ] At least 1 of Track C1–C2 completed.
- [ ] 3 cooperation variants written to `docs/reports/WAVE_LOOP_75_COOPERATION.md`.
- [ ] Memory + skills saved.
