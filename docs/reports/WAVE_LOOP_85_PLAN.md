# 🌊 WAVE LOOP 85 — DECOMPOSED PLAN

*Date: 2026-06-16 | Branch: trinity-rust-rings*

---

## I. Context

Wave Loop 84 achieved the ≤60 open issues target (66 → 60), implemented SSRF guards, documented 3 new Lean 4 competitors, and expanded Coq neutrino proofs. The suite remains at 550/550 PASS with zero clippy warnings.

**Priority shift:** Lean 4 competitors (Tooby-Smith, Krippendorf) are now the dominant threat axis. The first non-trivial physics error found via formalization (arXiv:2603.08139) validates the entire formal-verification-for-physics research program. Trinity must accelerate its proof base and arXiv presence.

---

## II. Tracks

### Track A: Security — Auth Middleware (#1193)
**Owner:** Trinity Agent (Queen)
**Complexity:** MEDIUM
**Goal:** Add JWT auth middleware to all compiler/server endpoints

- [ ] Read current route definitions in `main.rs`
- [ ] Add `RequireAuth` middleware layer using existing `jwt::verify_sandbox_token`
- [ ] Apply middleware to `/compile`, `/parse`, `/gen`, `/seal`, `/bench`, `/eval`, `/graph`, `/optimize`, `/typecheck`, `/lint`, `/explain`
- [ ] Add CI bypass for local testing (env var `T27C_NO_AUTH=1`)
- [ ] Verify zero clippy warnings
- [ ] Close #1193 with `Closes #1193` trailer

---

### Track B: Quality — Compiler Bug Fixes (#1195-#1198)
**Owner:** Creator Agent (C) + Verifier Agent (V)
**Complexity:** HIGH
**Goal:** Close 2 of 4 atomic compiler bugs

**#1195:** `run_asm` hardcodes expected instruction bytes
- [ ] Identify hardcoded bytes in `run_asm` function
- [ ] Replace with AST-driven instruction generation
- [ ] Add regression test

**#1196:** `run_sort` prints original source instead of sorted AST
- [ ] Serialize sorted AST before printing
- [ ] Add regression test

**#1197:** `convert_fn_to_comb` drops control flow
- [ ] Convert StmtIf/StmtWhile/StmtFor/StmtLocal/Return to HIR
- [ ] Add test cases for each control-flow construct

**#1198:** `@bitCast` strict-aliasing UB
- [ ] Replace pointer cast with memcpy or union-based approach
- [ ] Verify no regression in generated code

---

### Track C: Competitive Acceleration — arXiv Submission
**Owner:** Trinity Agent (Queen)
**Complexity:** MEDIUM
**Goal:** Submit `trinity_arxiv.tex` to physics.gen-ph

- [ ] Update arXiv preprint with new competitor citations (Tooby-Smith 2026)
- [ ] Add paragraph on formal verification catching real physics errors
- [ ] Find endorser for physics.gen-ph
- [ ] Submit via arXiv web interface
- [ ] Record submission ID

---

### Track D: Coq Proof Sprint
**Owner:** Creator Agent (C)
**Complexity:** MEDIUM
**Goal:** Add 3-5 new Qed lemmas to close gap with Lean 4 competitors

- [ ] Add `Sum_m2_nu_pos` (mass-squared sum positivity)
- [ ] Add `Delta_m2_21_bound` (solar mass-squared difference)
- [ ] Add `Delta_m2_31_bound` (atmospheric mass-squared difference)
- [ ] Add `seesaw_consistency_check` (type-I seesaw consistency)
- [ ] Document any numerical discrepancies honestly

---

### Track E: Issue Triage — Maintain ≤60
**Owner:** Trinity Agent (Queen)
**Complexity:** LOW
**Goal:** Close 2-3 more resolved-but-open issues

- [ ] Scan open issues for resolved/stale items
- [ ] Close with honest notes and `Closes #N` trailers
- [ ] Update issue count target: 60 → ≤57

---

## III. Dependencies

```
Track A (security) ─┐
Track B (compiler) ─┼→ Track E (triage)
Track C (arXiv)    ─┘
Track D (Coq) ───────→ Track E (triage)
```

Tracks A-D can run in parallel. Track E depends on completion of at least one other track.

---

## IV. Success Criteria

| # | Criterion | Target |
|---|-----------|--------|
| 1 | Open issues | ≤57 |
| 2 | Suite health | 550/550 PASS |
| 3 | Clippy warnings | 0 |
| 4 | New Coq theorems | +3 Qed |
| 5 | Auth middleware | #1193 closed |
| 6 | arXiv status | Submitted or endorser contacted |

---

*φ² + 1/φ² = 3 | Plan complete → Phase 3: DELEGATE*
