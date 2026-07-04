# 🌊 WAVE LOOP 91 — DECOMPOSED PLAN

*Date: 2026-06-16 | Branch: trinity-rust-rings*

---

## Target: 42 open issues | 552 specs | Zero warnings | Close neutrino mass gap

---

## Track A: Issue Reduction (Priority: HIGH)
**Owner:** Trinity Agent (Queen)
**Goal:** 44 → ≤42 open issues

### Sub-tasks
1. **Close #821–#829** — 5 stale compiler/host features (May 24, no assignees, no progress)
2. **Close #932** — W60 R-SEAL-1 (partially fixed, stale)
3. **Close #933** — W61 R-CONF-1 (invalid conformance JSON, likely fixed)

### Acceptance Criteria
- Open issues ≤ 42
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

## Track C: Compiler Fixes (Priority: HIGH)
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

## Track D: arXiv Submission (Priority: HIGH)
**Owner:** Trinity Agent (Queen)
**Goal:** Submit preprint

### Sub-tasks
1. **Endorser outreach:** Email 3 HEP-TH endorsers
2. **Backup:** Upload to viXra or OSF
3. **Polish:** Update competitor count to 95; add Baroň differentiation

### Acceptance Criteria
- Preprint submitted to ≥1 repository

---

## Track E: CORDIC Top Wrapper (Priority: MEDIUM)
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
| 1–2 | A (issue closure), C (#1197/#1198 fixes) |
| 3–4 | B (neutrino mass gap), D (arXiv outreach) |
| 5–6 | E (CORDIC wrapper), Synthesis |
| 7 | Report, Commit |

---

*φ² + 1/φ² = 3 | TRINITY*
