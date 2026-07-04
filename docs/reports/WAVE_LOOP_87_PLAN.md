# 🌊 WAVE LOOP 87 — DECOMPOSED PLAN

*Date: 2026-06-16 | Branch: trinity-rust-rings*

---

## Target: 52 open issues | 555 specs | Zero warnings | +3 Coq Qed

---

## Track A: Issue Reduction (Priority: HIGH)
**Owner:** Trinity Agent (Queen)  
**Goal:** 55 → ≤52 open issues

### Sub-tasks
1. **Audit 5 oldest open issues** — identify candidates for honest close (fixed, obsolete, or non-repro)
2. **Batch-close 3+ issues** with honest notes and `Closes #N` traceability
3. **Re-verify audit-wave issues** (#970 area) for zombie detection

### Acceptance Criteria
- `gh issue list --state open | wc -l` ≤ 52
- All closures have honest justification

---

## Track B: Compiler Fixes (Priority: HIGH)
**Owner:** Creator Agent (C) + Verifier Agent (V)  
**Goal:** Fix 2 CRITICAL compiler bugs

### Sub-tasks
1. **#1197: `convert_fn_to_comb` drops control flow**
   - Root-cause: combinational conversion ignores `if`/`while`/`for`
   - Fix: Either reject non-combinational functions or inline control flow correctly
   - Verify: regression test with control-flow-heavy t27 spec

2. **#1198: `@bitCast` strict-aliasing UB**
   - Root-cause: `bitCast` generates pointer-cast violating Rust/C strict aliasing
   - Fix: Use `union` or `memcpy`-based approach for type-punning
   - Verify: Miri test, `miri test --target x86_64-unknown-linux-gnu`

### Acceptance Criteria
- Both issues closed with PR + test
- `cargo clippy --workspace --all-features` remains 0

---

## Track C: Neutrino Phenomenology (Priority: MEDIUM)
**Owner:** Creator Agent (C) — Coq track  
**Goal:** +3 Coq Qed theorems

### Sub-tasks
1. **Mass-squared difference bounds**
   - Prove `Delta_m2_21 < Delta_m2_31` using `lra` + field facts
   - Prove `Delta_m2_21 > 0` (solar splitting is positive)

2. **Chamseddine-Dąbrowski ansatz integration**
   - Document NCG neutrino mass derivation from `NCG_approach.pdf`
   - Add formal hypothesis for seesaw scale matching NCG result

3. **Normal ordering theorem**
   - Strengthen `normal_ordering_theorem` with tighter bounds

### Acceptance Criteria
- `make -C proofs/trinity` passes with zero Admitted
- +3 new Qed lemmas in `NeutrinoMasses.v`

---

## Track D: arXiv Submission (Priority: MEDIUM)
**Owner:** Trinity Agent (Queen)  
**Goal:** Submit preprint or secure endorser

### Sub-tasks
1. **Endorser outreach:** Contact 3 potential endorsers (HEP-TH, GR-QC, MATH-PH)
2. **Backup route:** Prepare viXra or OSF preprint if arXiv endorser unavailable
3. **Polish PDF:** Final LaTeX pass, ensure zero warnings, update competitor count to 84

### Acceptance Criteria
- Preprint submitted to at least one repository
- Submission confirmation or endorser commitment

---

## Track E: CORDIC FPGA Instantiation (Priority: MEDIUM)
**Owner:** Creator Agent (C) — RTL track  
**Goal:** Top-level wrapper for CORDIC core

### Sub-tasks
1. **Write `cordic_top.v` wrapper**
   - Instantiates `cordic_sin` and `cordic_cos` functions
   - Adds clock, reset, valid/ready handshaking
   - Parameterized Q15 fixed-point width

2. **Synthesize with Yosys**
   - Target: `synth_ice40` or `synth_xilinx`
   - Report LUT/FF utilization

3. **Write bitstream constraints** (PCF/XDC)

### Acceptance Criteria
- Yosys synthesis produces netlist with 0 errors
- Resource utilization reported

---

## Track F: Lean 4 Bridge Expansion (Priority: LOW)
**Owner:** Creator Agent (C)  
**Goal:** Translate `Predictions.v` bounds into Lean 4

### Sub-tasks
1. Translate `Bounds_Masses.v` inequalities to `BoundsMasses.lean`
2. Add `interval` tactic equivalents using `Mathlib` approximation tools
3. Verify `lake build` still passes

### Acceptance Criteria
- `lake build` passes (0 errors)
- ≥5 new Lean lemmas covering mass bounds

---

## Track G: Security Hardening (Priority: LOW)
**Owner:** Verifier Agent (V)  
**Goal:** Audit remaining endpoints for bypass vectors

### Sub-tasks
1. **Rate limiting:** Add per-IP rate limiting to `/compile`, `/gen`, `/eval`
2. **Input validation:** Audit `/eval` and `/graph` for injection attacks
3. **Token expiry:** Enforce JWT expiry check (currently missing)

### Acceptance Criteria
- No new clippy warnings
- Security review document updated

---

## Schedule

| Day | Tracks |
|-----|--------|
| 1–2 | A (issue audit + closure), B (#1197 root-cause) |
| 3–4 | B (fixes + tests), C (Coq lemmas) |
| 5–6 | D (arXiv outreach), E (CORDIC wrapper) |
| 7 | F (Lean 4), G (security audit), Synthesis |

---

*φ² + 1/φ² = 3 | TRINITY*
