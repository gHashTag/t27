# 🌊 WAVE LOOP 86 — DECOMPOSED PLAN (IGLA CODER × IGLA RACE)

*Date: 2026-06-16 | Branch: trinity-rust-rings | Next commit target: W86*

---

## I. Context

Wave Loop 85 closed compiler bugs #1195 and #1196, expanded Coq neutrino proofs with `generation_factors_geometric`, and maintained suite health at 550/550 PASS with zero clippy warnings.

**Priority shift:** The IGLA (Integrated Geometric Logic Architecture) program requires simultaneous progress on CODER (compiler toolchain) and RACE (hardware synthesis) tracks. Auth middleware and arXiv submission are now blockers for external credibility.

---

## II. Tracks

### Track A: CODER — Auth Middleware (#1193)
**Owner:** Trinity Agent (Queen)
**Complexity:** MEDIUM
**Goal:** Add JWT auth middleware to compiler/server endpoints

- [ ] Implement `RequireAuth` axum middleware using `jwt::verify_sandbox_token`
- [ ] Apply middleware to `/compile`, `/parse`, `/gen`, `/seal`, `/bench`, `/eval`, `/graph`, `/optimize`, `/typecheck`, `/lint`, `/explain`
- [ ] Add `T27C_NO_AUTH=1` env bypass for local testing/CI
- [ ] Return 401 with `WWW-Authenticate: Bearer` header on missing/invalid token
- [ ] Verify zero clippy warnings
- [ ] Close #1193 with `Closes #1193` trailer

---

### Track B: CODER — Compiler Safety (#1198)
**Owner:** Creator Agent (C)
**Complexity:** HIGH
**Goal:** Fix `@bitCast` strict-aliasing UB

- [ ] Locate all `@bitCast` usages in generated code and compiler
- [ ] Replace pointer-based bitcast with `memcpy` or union-based approach
- [ ] Verify no regression in generated Zig/C/Rust/Verilog
- [ ] Add regression test to suite
- [ ] Close #1198

---

### Track C: CODER — Control Flow (#1197)
**Owner:** Creator Agent (C) + Verifier Agent (V)
**Complexity:** HIGH
**Goal:** Fix `convert_fn_to_comb` dropping control flow

- [ ] Convert `StmtIf`/`StmtWhile`/`StmtFor`/`StmtLocal`/`Return` to HIR correctly
- [ ] Add test cases for each control-flow construct in `convert_fn_to_comb`
- [ ] Verify suite passes
- [ ] Close #1197

---

### Track D: RACE — CORDIC Bitstream Deployment
**Owner:** Trinity Agent (Queen)
**Complexity:** MEDIUM
**Goal:** Synthesize CORDIC core to FPGA bitstream

- [ ] Fix remaining t27c Verilog codegen issues (if any) for CORDIC spec
- [ ] Run Yosys synthesis on `cordic_fixed.t27` generated Verilog
- [ ] Deploy to target FPGA (Lattice ECP5 or Xilinx Artix-7)
- [ ] Verify timing closure and resource utilization
- [ ] Document bitstream generation flow in `docs/fpga/CORDIC_BITSTREAM.md`

---

### Track E: Competitive Acceleration — arXiv Submission
**Owner:** Trinity Agent (Queen)
**Complexity:** MEDIUM
**Goal:** Submit `TRINITY_SYMMETRY_PAPER_arxiv.tex` to physics.gen-ph

- [ ] Update preprint with Tooby-Smith 2026 citation (formal verification catching physics errors)
- [ ] Add paragraph on `generation_factors_geometric` as φ-ladder evidence
- [ ] Contact endorser for physics.gen-ph category
- [ ] Submit via arXiv web interface
- [ ] Record submission ID in `docs/arXiv/SUBMISSION_LOG.md`

---

### Track F: Coq Proof Sprint — Neutrino Bounds
**Owner:** Creator Agent (C)
**Complexity:** MEDIUM
**Goal:** Add 3+ new Qed lemmas

- [ ] `Delta_m2_21_bound` — structural bound on solar mass-squared difference
- [ ] `Delta_m2_31_bound` — structural bound on atmospheric mass-squared difference
- [ ] `typeII_split_product` — `m_nu_tau_typeII_split = phi^2 * m_nu_muon_typeII_split` (division-free)
- [ ] Document any numerical discrepancies honestly

---

### Track G: Issue Triage — Maintain ≤58
**Owner:** Trinity Agent (Queen)
**Complexity:** LOW
**Goal:** Close 2-3 more resolved-but-open issues

- [ ] Scan open issues for resolved/stale items
- [ ] Close with honest notes and `Closes #N` trailers
- [ ] Update issue count target: 58 → ≤55

---

## III. Dependencies

```
Track A (auth) ──────┐
Track B (bitcast) ───┼→ Track G (triage)
Track C (control) ───┤
Track D (CORDIC) ────┤
Track E (arXiv) ─────┘
Track F (Coq) ───────→ Track G (triage)
```

Tracks A-F can run in parallel. Track G depends on at least one other track completing.

---

## IV. Success Criteria

| # | Criterion | Target |
|---|-----------|--------|
| 1 | Open issues | ≤55 |
| 2 | Suite health | 550/550 PASS |
| 3 | Clippy warnings | 0 |
| 4 | New Coq theorems | +3 Qed |
| 5 | Auth middleware | #1193 closed |
| 6 | CORDIC synthesis | Yosys passes or bitstream generated |
| 7 | arXiv status | Submitted or endorser contacted |

---

## V. IGLA Alignment

| IGLA Component | W86 Track | File |
|----------------|-----------|------|
| CODER / arch | Track C | `specs/igla/coder/arch.t27` |
| CODER / eval | Track A | `specs/igla/coder/eval.t27` |
| CODER / prm | Track F | `specs/igla/coder/prm.t27` |
| CODER / training | Tracks B+C | `specs/igla/coder/training.t27` |
| RACE / backend | Track D | `specs/igla/race/backend.t27` |
| RACE / cordic | Track D | `specs/igla/race/cordic_fixed.t27` |
| RACE / formal | Track F | `specs/igla/race/formal.t27` |
| RACE / rtl | Track D | `specs/igla/race/rtl.t27` |

---

*φ² + 1/φ² = 3 | Plan complete → Phase 3: DELEGATE*
