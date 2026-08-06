# 🌊 WAVE LOOP 93 — DECOMPOSED PLAN

*Date: 2026-06-16 | Branch: trinity-rust-rings | Target: ≤27 open issues*

---

## Objective

Drive open issues below 27, close the neutrino mass-gap weakness, and begin Baez & Schwahn differentiation ahead of potential arXiv submission.

---

## Track A: Issue Reduction (Target: ≤27 open issues)

**Goal:** Close 2+ additional resolved-but-open issues to reach ≤27.

| Sub-task | Action | Issue |
|----------|--------|-------|
| A1 | Audit zombie pattern in #1195 (run_asm) | #1195 |
| A2 | Audit zombie pattern in #1196 (run_sort) | #1196 |
| A3 | Close if superseded by W92 fixes | #1195 / #1196 |
| A4 | Verify #1197 (convert_fn_to_comb) still blocked by test-only | #1197 |
| A5 | If fixed, close; else, document remaining scope | #1197 |

**Success criterion:** Open issues ≤27.

---

## Track B: Neutrino Mass Gap Closure

**Goal:** Produce first validated absolute neutrino mass prediction.

| Sub-task | Action | Deliverable |
|----------|--------|-------------|
| B1 | Implement Type-I seesaw mass formula in Coq | `NeutrinoMasses.v` |
| B2 | Derive m_nu_1, m_nu_2, m_nu_3 from φ-seesaw + Koide | Lemma |
| B3 | Compute Σ m_ν numeric value (target: ~0.06–0.1 eV range) | Coq `compute` |
| B4 | Compare with cosmological bound (Σ m_ν < 0.12 eV) | Analysis |

**Success criterion:** At least one `Qed` lemma giving numeric neutrino mass.

---

## Track C: Baez & Schwahn Differentiation

**Goal:** Document why Trinity is NOT subsumed by arXiv:2606.15235.

| Sub-task | Action | Deliverable |
|----------|--------|-------------|
| C1 | Read arXiv:2606.15235 abstract + skim sections | Notes |
| C2 | Identify overlap (Jordan algebra → SM gauge group) | Gap analysis |
| C3 | Document Trinity differentiators (600-cell, spectral action, testability) | `docs/COMPETITIVE_POSITIONING.md` |
| C4 | Add Baez response to arXiv LaTeX if appropriate | `paper/` |

**Success criterion:** Baez row in COMPETITIVE_POSITIONING.md contains ≥3 Trinity-specific differentiators.

---

## Track D: Compiler Fixes (Blocked Issues)

**Goal:** Reduce #1197 and #1198 severity.

| Sub-task | Action | Deliverable |
|----------|--------|-------------|
| D1 | Reproduce #1197 (convert_fn_to_comb) in isolation | Test case |
| D2 | If test-only fix possible, implement; else document | Comment |
| D3 | Investigate #1198 (@bitCast UB) scope | Analysis |

**Success criterion:** At least 1 of #1197/#1198 moved to "documented limitation" status.

---

## Track E: Suite Health Maintenance

**Goal:** Keep 555/555 PASS and zero clippy warnings.

| Sub-task | Action | Deliverable |
|----------|--------|-------------|
| E1 | Run `./scripts/tri test` | Log |
| E2 | Run `cargo clippy --workspace --all-features` | Log |
| E3 | Regenerate any seal mismatches | Seals |
| E4 | Add tests for new W93 features | Specs |

**Success criterion:** 555/555 PASS, 0 clippy warnings.

---

## Track F: CORDIC Optimization (Deferred if time-constrained)

**Goal:** Reduce CORDIC LUT count from 699 to <400.

| Sub-task | Action | Deliverable |
|----------|--------|-------------|
| F1 | Implement double-step CORDIC (2 angles per iteration) | `cordic_double.t27` |
| F2 | Synthesize and compare LUT count | Yosys log |

**Success criterion:** Synthesis reports <400 LUTs.

---

## Resource Allocation

| Track | Priority | Estimated Effort | Owner |
|-------|----------|------------------|-------|
| A | CRITICAL | 2h | Queen |
| B | CRITICAL | 4h | Coq Agent |
| C | HIGH | 2h | Research Agent |
| D | HIGH | 3h | Compiler Agent |
| E | MANDATORY | 1h | CI |
| F | LOW | 3h | Hardware Agent |

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| #1197/#1198 deeper than expected | Medium | High | Document as limitation, close issue |
| Coq neutrino formula numerically unstable | Medium | High | Use `lra` with wide bounds |
| No new arXiv competitors to analyze | Low | Low | Skip Track C gracefully |

---

*φ² + 1/φ² = 3 | TRINITY*
