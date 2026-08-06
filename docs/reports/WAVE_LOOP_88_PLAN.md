# 🌊 WAVE LOOP 88 — DECOMPOSED PLAN

*Date: 2026-06-16 | Branch: trinity-rust-rings*

---

## Target: 52 open issues | 551 specs | Zero warnings | Autoformalization defense

---

## Track A: Issue Reduction (Priority: HIGH)
**Owner:** Trinity Agent (Queen)  
**Goal:** 55 → ≤52 open issues

### Sub-tasks
1. **Close #582, #583, #590** — Oldest EPICs and BLOCKER-1 catch-all; document honest closure notes
2. **Split #960** — L2 file-location violation (remaining 30 .v migrations) vs L4 test coverage gap → 2 atomic issues
3. **Split #955** — 2 remaining C-backend sub-bugs (Bug 2: @bitCast UB type-pun; Bug 6: extract_names over-collection) → 2 atomic issues
4. **Fix gh auth** — Resolve `GH_TOKEN` invalid status or document keyring-only workflow

### Acceptance Criteria
- Open issues ≤ 52
- All closures/splits have honest justification
- L1 TRACEABILITY: every closure references `Closes #N`

---

## Track B: Autoformalization Defense (Priority: EXTREME)
**Owner:** Creator Agent (C)  
**Goal:** Build or adopt agentic proof-generation pipeline before competitors outpace Trinity

### Sub-tasks
1. **Survey Meadows et al. pipeline** — Read arXiv:2604.23002; identify reusable components (parsing, semantic drift detection, theorem sketching)
2. **Prototype `t27c formalize`** — CLI subcommand that takes a physics claim (natural language) and produces a Coq lemma skeleton + proof hints
3. **Integrate LLM proof suggestion** — Use Claude API to generate `Proof. ... Qed.` attempts from lemma statements; validate with `coqc`
4. **Benchmark:** Compare Trinity's manual proof rate (1 lemma/day) vs agentic rate

### Acceptance Criteria
- `t27c formalize --claim "phi^2 = phi + 1"` produces compilable Coq
- ≥1 auto-generated proof accepted into `proofs/trinity/AutoFormalized.v`
- Document false-positive rate (generated proofs that compile but are mathematically wrong)

---

## Track C: Nurowski Differentiation (Priority: HIGH)
**Owner:** Creator Agent (C) — Coq track  
**Goal:** Explicitly prove why 600-cell is unique vs generic finite geometries

### Sub-tasks
1. **Formalize uniqueness theorem:** Prove that among regular 4-polytopes, only the 600-cell has H₄ symmetry and yields φ-monomial mass formulas
2. **Add to arXiv preprint:** §4.2 — "Why the 600-cell and not Schläfli or Cremona-Richmond?"
3. **Nurowski comparison table:** Formal proofs vs descriptive mapping; numerical predictions vs none

### Acceptance Criteria
- ≥1 new Coq lemma in `H4GaugeEmbedding.v` or new file `Uniqueness600Cell.v`
- arXiv LaTeX updated with Nurowski differentiation

---

## Track D: Compiler Fixes (Priority: HIGH)
**Owner:** Creator Agent (C) + Verifier Agent (V)  
**Goal:** Fix #1197 and #1198

### Sub-tasks
1. **#1197: convert_fn_to_comb** — Reject functions containing `StmtIf`/`StmtWhile`/`StmtFor` early; error message instead of silent drop
2. **#1198: @bitCast UB** — Replace pointer-cast with `union`-based or `memcpy`-based type-punning
3. **Regression tests:** Add t27 specs that trigger both bugs; ensure they fail before fix and pass after

### Acceptance Criteria
- `cargo test --workspace --all-features` → 537/537 pass
- Both issues closed with PR + `Closes #N`

---

## Track E: arXiv Submission (Priority: MEDIUM)
**Owner:** Trinity Agent (Queen)  
**Goal:** Submit preprint or secure endorser

### Sub-tasks
1. **Endorser outreach:** Email 3 HEP-TH endorsers with 1-page abstract + Trinity differentiators
2. **Backup:** Upload to viXra or OSF if arXiv endorser unavailable within 1 week
3. **Polish:** Update competitor count to 91; add Nurowski differentiation paragraph

### Acceptance Criteria
- Preprint submitted to at least one repository
- Submission confirmation received

---

## Track F: CORDIC FPGA Instantiation (Priority: MEDIUM)
**Owner:** Creator Agent (C) — RTL track  
**Goal:** Top-level wrapper for CORDIC core

### Sub-tasks
1. **Write `cordic_top.v`** — Clock, reset, valid/ready handshaking; instantiate `cordic_sin`/`cordic_cos`
2. **Yosys synthesis** — Target `synth_ice40`; report LUT/FF count
3. **Bitstream constraints** — PCF for iCE40-HX8K or similar dev board

### Acceptance Criteria
- Yosys synthesis passes with 0 errors
- Resource utilization documented in `docs/rtl/CORDIC_SYNTHESIS.md`

---

## Schedule

| Day | Tracks |
|-----|--------|
| 1–2 | A (issue closure/split), D (#1197 root-cause) |
| 3–4 | D (fixes + tests), C (Coq uniqueness lemma) |
| 5–6 | B (autoformalization prototype), E (arXiv outreach) |
| 7 | F (CORDIC wrapper), Synthesis, Report |

---

*φ² + 1/φ² = 3 | TRINITY*
