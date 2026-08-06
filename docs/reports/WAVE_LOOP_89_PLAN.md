# 🌊 WAVE LOOP 89 — DECOMPOSED PLAN

*Date: 2026-06-16 | Branch: trinity-rust-rings*

---

## Target: 49 open issues | 551 specs | Zero warnings | Fix GH_TOKEN

---

## Track A: Issue Reduction (Priority: HIGH)
**Owner:** Trinity Agent (Queen)  
**Goal:** 52 → ≤49 open issues

### Sub-tasks
1. **Close #623** — "Publish v1.0.0 to Zenodo" (2026-05-15). If no Zenodo DOI exists and no publish pipeline is ready, close as deferred.
2. **Close #698** — "TRINITY-VELOCITY Spec-First Acceleration Loop" (2026-05-18). If partially implemented, close with honest note; track specific tasks in new atomic issues.
3. **Split #960** — W84 R-SPECS L2+L4 violation. Split into atomic: #960-a (remaining .v migrations) and #960-b (L4 test coverage gap), then close parent.
4. **Fix GH_TOKEN** — Replace invalid token in env or document permanent workaround in CI/scripts.

### Acceptance Criteria
- Open issues ≤ 49
- All closures have honest `Closes #N` notes
- GH auth documented (not just workaround)

---

## Track B: Compiler Fixes (Priority: HIGH)
**Owner:** Creator Agent (C) + Verifier Agent (V)  
**Goal:** Fix #1197 and #1198

### Sub-tasks
1. **#1197: convert_fn_to_comb** — Reject `StmtIf`/`StmtWhile`/`StmtFor` early with clear error. Add regression test.
2. **#1198: @bitCast UB** — Replace strict-aliasing pointer cast with `union` or `memcpy` type-punning. Add Miri test.
3. **Restore 537/537 tests:** Ensure `cargo test --workspace --all-features` passes fully.

### Acceptance Criteria
- `cargo test` → 537/537 pass
- Both issues closed with PR + `Closes #N`
- No new clippy warnings

---

## Track C: Neutrino Mass Ansatz (Priority: MEDIUM)
**Owner:** Creator Agent (C) — Coq track  
**Goal:** +2 Coq Qed lemmas

### Sub-tasks
1. **Add `Delta_m2_21_pos`** — Prove solar mass-squared difference is positive using `lra` + field facts.
2. **Add `Delta_m2_31_pos`** — Prove atmospheric mass-squared difference is positive.
3. **Document NCG ansatz** — Add Chamseddine-Dąbrowski reference to `NeutrinoMasses.v` as formal hypothesis.

### Acceptance Criteria
- `make -C proofs/trinity` passes with 0 Admitted
- +2 new Qed lemmas

---

## Track D: arXiv Submission (Priority: MEDIUM)
**Owner:** Trinity Agent (Queen)  
**Goal:** Submit preprint or secure endorser

### Sub-tasks
1. **Endorser outreach:** Email 3 HEP-TH endorsers with abstract + Trinity differentiators (91 competitors, zero free inputs, formal proofs, hardware)
2. **Backup:** Upload to viXra or OSF if arXiv endorser unavailable
3. **Polish:** Update competitor count to 91; add Nurowski differentiation paragraph

### Acceptance Criteria
- Preprint submitted to ≥1 repository
- Submission confirmation or endorser commitment

---

## Track E: CORDIC Top Wrapper (Priority: MEDIUM)
**Owner:** Creator Agent (C) — RTL track  
**Goal:** Synthesize CORDIC to netlist

### Sub-tasks
1. **Write `cordic_top.v`** — Clock, reset, valid/ready; instantiate `cordic_sin`/`cordic_cos`
2. **Yosys synthesis** — Target `synth_ice40`; report LUT/FF
3. **Document** — Add `docs/rtl/CORDIC_SYNTHESIS.md`

### Acceptance Criteria
- Yosys synthesis passes with 0 errors
- Resource utilization reported

---

## Track F: Autoformalization Defense (Priority: LOW)
**Owner:** Creator Agent (C)  
**Goal:** Prototype `t27c formalize`

### Sub-tasks
1. **Survey Meadows et al.** — Read arXiv:2604.23002 FormalScience pipeline
2. **Skeleton CLI** — Add `t27c formalize --claim "..." --lang coq` subcommand
3. **Generate lemma skeleton** — From natural language claim, output Coq `Lemma` + `Proof.` stub

### Acceptance Criteria
- `t27c formalize` produces compilable Coq skeleton
- Documented in `docs/formalize/README.md`

---

## Schedule

| Day | Tracks |
|-----|--------|
| 1–2 | A (issue closure + GH_TOKEN fix), B (#1197 root-cause) |
| 3–4 | B (fixes + tests), C (Coq lemmas) |
| 5–6 | D (arXiv outreach), E (CORDIC wrapper) |
| 7 | F (autoformalization skeleton), Synthesis, Report |

---

*φ² + 1/φ² = 3 | TRINITY*
