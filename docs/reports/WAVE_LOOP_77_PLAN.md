# Wave Loop 77 Plan

**Scope:** Parser postfix array notation, CKM CP archive, Lean 4 bridge revival, and arXiv endorsement push.

## Health Gates (daily)

- [ ] `t27c suite --repo-root .` → 549/549
- [ ] `cargo test --workspace` → 534/534
- [ ] `cd proofs/trinity && make` → 0 errors, 0 Admitted
- [ ] `cargo clippy --workspace` → 0 warnings

---

## Track A — Core Engineering

### A1: Parser Postfix Array Type Notation
**Goal:** Fix `parse_type_annotation` to handle `i64[]` and `[N]i64` postfix array notation.
**Current behavior:** `let arr: i64[] = [1i64, 2, 3]` parses type as `"i64"`, dropping `[]`.
**Acceptance:** Parsed `extra_type` contains `"i64[]"` or equivalent array marker.

### A2: Conformance Spec for Array Literals
**Goal:** Create `specs/test_array_literal_inline.t27` with ≥3 test cases.
**Cases:** inferred array, explicit `i64[]` type (if parser fixed), nested arrays.
**Acceptance:** Suite passes, all backends generate valid code.

---

## Track B — Formal Phenomenology & Preprint

### B1: CKM CP-Violation Archive Conjecture (δ_CK = e/2)
**Goal:** Add `delta_CK_e_2_ansatz` (`e / 2 = 77.9°`) as `Conjecture` in `Archive_Conjectural.v`.
**Note:** W57 reconciliation established canonical δ_CP = e/2 = 77.9°. Document as phenomenological anchor with PDG falsifiability band.

### B2: Lean 4 Bridge — Recreate + 5 Lemmas
**Goal:** Recreate `lean4_bridge/` directory and translate 5 `CorePhi.v` lemmas.
**Lemmas:** `phi_algebraic`, `phi_golden`, `phi_reciprocal`, `phi_quartic`, `phi_spectral_norm`.
**Acceptance:** `lake build` passes; proofs via `by ring` / `by field_simp`.

---

## Track C — Competitive Intelligence

### C1: arXiv Endorsement Push
**Goal:** Draft and send endorsement request for `trinity_arxiv.tex` to 2 potential endorsers.
**Categories:** hep-th, math-ph.
**Acceptance:** Request sent (not necessarily approved).

### C2: Monthly Competitor Monitoring
**Goal:** Check Washburn repo, Ω-Theory repo for new commits.
**Acceptance:** Log entry if activity detected.

---

## Track D — Cooperation Variants (W77 Edition)

Produce three concrete cooperation proposals. See [WAVE_LOOP_77_COOPERATION.md](WAVE_LOOP_77_COOPERATION.md).

---

## Definition of Done

- [ ] Suite 549/549, cargo 534/534, Coq 0 Admitted.
- [ ] W76 report published.
- [ ] W77 plan published.
- [ ] At least 1 of Track A1–A2 or Track B1–B2 merged.
- [ ] 3 cooperation variants written.
- [ ] Memory + skills saved.
