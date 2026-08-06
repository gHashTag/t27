# 🌊 WAVE LOOP 92 — DECOMPOSED PLAN

*Date: 2026-06-16 | Branch: trinity-rust-rings*

---

## Target: 34 open issues | 552 specs | Zero warnings | Close neutrino mass gap

---

## Track A: Issue Reduction (Priority: HIGH)
**Owner:** Trinity Agent (Queen)
**Goal:** 36 → ≤34 open issues

### Sub-tasks
1. **Close #939** — W67 R-PSLQ-1 (HIGH, 2 bugs, likely zombie)
2. **Close #941** — W69 R-SUITE-1 (HIGH, 3 bugs, likely zombie)
3. **Close #962** — W86 R-CONFORMANCE (MEDIUM, likely stale)
4. **Close #974** — W97 R-ENRICHMENT (HIGH/MEDIUM, zombie)

### Acceptance Criteria
- Open issues ≤ 34
- All closures have honest `Closes #N` notes

---

## Track B: Neutrino Mass Gap Closure (Priority: EXTREME)
**Owner:** Creator Agent (C) — Coq track
**Goal:** First validated absolute neutrino mass prediction

### Sub-tasks
1. **Formalize Σ m_ν bound** — Prove `0.05 eV < Sigma_m_nu < 0.07 eV` using H4 Coxeter-number ansatz
2. **Add `normal_ordering_theorem`** — Prove normal ordering with tighter bounds
3. **Document ansatz** — Add formal Coq Axiom for NCG-derived Majorana mass scale

### Acceptance Criteria
- `make -C proofs/trinity` passes with 0 real Admitted
- +3 new Qed lemmas in `NeutrinoMasses.v`
- Σ m_ν prediction documented with error bars

---

## Track C: Baez Differentiation (Priority: EXTREME)
**Owner:** Trinity Agent (Queen)
**Goal:** Explicitly differentiate from Baez & Schwahn

### Sub-tasks
1. **Add to arXiv preprint:** §4.3 — "Why Trinity's 600-cell spectral triple differs from Jordan-algebra approaches"
2. **Add comparison table:** Formal proofs vs none; numerical predictions vs none; hardware vs none
3. **Cite Baez & Schwahn** — Acknowledge their work while emphasizing Trinity's phenomenological advantage

### Acceptance Criteria
- arXiv LaTeX updated with Baez differentiation
- Comparison table in `docs/COMPETITIVE_POSITIONING.md`

---

## Track D: Compiler Fixes (Priority: HIGH)
**Owner:** Creator Agent (C) + Verifier Agent (V)
**Goal:** Fix #1197 and #1198

### Sub-tasks
1. **#1197: convert_fn_to_comb** — Reject `StmtIf`/`StmtWhile`/`StmtFor` early with clear error
2. **#1198: @bitCast UB** — Replace strict-aliasing pointer cast with `union` or `memcpy`
3. **Restore 537/537 tests**

### Acceptance Criteria
- `cargo test` → 537/537 pass
- Both issues closed with PR + `Closes #N`

---

## Track E: arXiv Submission (Priority: HIGH)
**Owner:** Trinity Agent (Queen)
**Goal:** Submit preprint

### Sub-tasks
1. **Endorser outreach:** Email 3 HEP-TH endorsers with abstract + Trinity differentiators (96 competitors, zero free inputs, formal proofs, hardware)
2. **Backup:** Upload to viXra or OSF
3. **Polish:** Update competitor count to 96; add Baez differentiation paragraph

### Acceptance Criteria
- Preprint submitted to ≥1 repository

---

## Track F: CORDIC Top Wrapper (Priority: MEDIUM)
**Owner:** Creator Agent (C) — RTL track
**Goal:** Synthesize CORDIC to netlist

### Sub-tasks
1. **Write `cordic_top.v`** — Clock, reset, valid/ready
2. **Yosys synthesis** — Target `synth_ice40`
3. **Document** — Add `docs/rtl/CORDIC_SYNTHESIS.md`

### Acceptance Criteria
- Yosys synthesis passes with 0 errors

---

## Schedule

| Day | Tracks |
|-----|--------|
| 1–2 | A (issue closure), C (Baez differentiation) |
| 3–4 | B (neutrino mass gap), D (arXiv outreach) |
| 5–6 | D (arXiv polish), E (CORDIC wrapper) |
| 7 | Synthesis, Report, Commit |

---

*φ² + 1/φ² = 3 | TRINITY*
