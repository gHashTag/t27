# Wave Loop 76 Plan

**Scope:** Parser-critical fix, neutrino phenomenology advance, Lean 4 bridge momentum, and GitHub auth hardening.

## Health Gates (daily)

- [ ] `t27c suite --repo-root .` → 549/549
- [ ] `cargo test --workspace` → 534/534
- [ ] `cd proofs/trinity && make` → 0 errors, 0 Admitted
- [ ] `cargo clippy --workspace` → 0 warnings
- [ ] `gh auth status` → valid token (new gate)

---

## Track A — Core Engineering

### A1: Parser `ExprArrayLiteral.children` Population Fix
**Goal:** Fix the root cause where inline element lists (`[0i64, 1, ...]`) produce empty `children` in `ExprArrayLiteral`, causing empty C compound literals `(int32_t[]){ }`.
**Method:** The `parse_array_literal` rewrite in W74/W75 already attempts element-list parsing with state rewind. Debug why `node.children` remains empty in generated code despite AST population.
**Acceptance:** A conformance spec using `[1i64, 2, 3]` array literal generates valid C/Zig/Rust with non-empty initializer.

### A2: GitHub CLI Auth Hardening
**Goal:** Eliminate recurring `HTTP 401` failures.
**Options:**
- Rotate `GH_TOKEN` env var (short-term).
- Switch to GitHub App authentication with installation token (long-term).
- Add `gh auth status` to daily health gate.
**Acceptance:** `gh issue list` returns open issues without 401 errors.

---

## Track B — Formal Phenomenology & Preprint

### B1: Neutrino Mass-Squared Differences Positivity
**Goal:** Prove `Delta_m21_sq_pos : Delta_m21_sq > 0` and `Delta_m31_sq_pos : Delta_m31_sq > 0` in `proofs/trinity/NeutrinoMasses.v`.
**Method:** Use `nra` with interval bounds (`interval_intro (m_nu_2 - m_nu_1)`). May need explicit physical mass ordering assertion.
**Fallback:** If unprovable from current definitions, add `[PHYSICAL]` axiom note and document gap honestly.

### B2: CKM CP-Violation Archive Conjecture (δ_CK = e/2)
**Goal:** Add `delta_CK` ansatz (`e / 2 = 77.9°`) as `Conjecture` in `proofs/trinity/CKMCPViolation.v` with `[UNPROVEN]` marker.
**Note:** Follows W57 reconciliation. Do not claim proof; document as phenomenological anchor with PDG falsifiability band.

### B3: Lean 4 Bridge — 5 Lemmas
**Goal:** Translate `phi_algebraic`, `phi_golden`, `phi_reciprocal`, `phi_quartic`, `phi_spectral_norm` from `CorePhi.v` into Lean 4 / Mathlib.
**Acceptance:** `lake build` passes; all 5 lemmas proved via `by ring` / `by field_simp`.

---

## Track C — Competitive Intelligence

### C1: Continuous Monitoring
**Goal:** Weekly check of Washburn, sct-theory, Omega-Theory repos for new commits/releases.
**Acceptance:** Log entry in competitive file if activity detected.

### C2: arXiv Endorsement
**Goal:** Submit endorsement request for trinity_arxiv.tex to appropriate arXiv categories (hep-th, math-ph).
**Blocker:** Requires institutional affiliation or endorser.
**Acceptance:** Endorsement request sent to 2 potential endorsers.

---

## Track D — Cooperation Variants (W76 Edition)

Produce three concrete cooperation proposals for upcoming week. See [WAVE_LOOP_76_COOPERATION.md](WAVE_LOOP_76_COOPERATION.md).

---

## Definition of Done

- [ ] Suite 549/549, cargo 534/534, Coq 0 Admitted.
- [ ] W75 report published (`docs/reports/WAVE_LOOP_75_REPORT.md`).
- [ ] W76 plan published (`docs/reports/WAVE_LOOP_76_PLAN.md`).
- [ ] At least 1 of Track A1–A2 or Track B1–B3 merged.
- [ ] 3 cooperation variants written to `docs/reports/WAVE_LOOP_76_COOPERATION.md`.
- [ ] Memory + skills saved.
