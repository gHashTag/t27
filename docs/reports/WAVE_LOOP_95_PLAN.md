# 🌊 WAVE LOOP 95 — DECOMPOSED PLAN

*Date: 2026-06-16 | Branch: trinity-rust-rings | Target: ≤12 open issues, arXiv submitted*

---

## Objective

Submit arXiv preprint with neutrino prediction featured, split remaining zombie issues, and reduce CORDIC LUT count.

---

## Track A: arXiv Submission (Priority: CRITICAL)

**Goal:** Submit trinity_arxiv.tex before July 2026 burst.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| A1 | Add neutrino prediction section (Σ m_ν ≈ 0.018 eV) | LaTeX section |
| A2 | Add honest withdrawal erratum (Predictions.v) | LaTeX paragraph |
| A3 | Verify all cited competitors are real | Checklist |
| A4 | Compile PDF with 0 warnings | `pdflatex` log |
| A5 | Submit to arXiv (physics.gen-ph) | Submission ID |

**Success criterion:** arXiv submission confirmed.

---

## Track B: Zombie Issue Resolution (Priority: HIGH)

**Goal:** Split or close #932 and #943 to reach ≤12 open issues.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| B1 | Close #932 if all 3 bugs are resolved or documented | Comment |
| B2 | Split #943 into atomic issues (8 bugs) | New issues |
| B3 | Close any additional resolved-but-open issues | Comments |

**Success criterion:** Open issues ≤12.

---

## Track C: CORDIC Optimization (Priority: MEDIUM)

**Goal:** Reduce LUT count from 699 to <400.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| C1 | Implement double-step CORDIC | `cordic_double.t27` |
| C2 | Synthesize and compare | Yosys log |

---

## Track D: Suite Health (Priority: MANDATORY)

**Goal:** Maintain 555/555 PASS.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| D1 | `./scripts/tri test` | Log |
| D2 | `cargo clippy --workspace --all-features` | Log |
| D3 | Regenerate seals if needed | Seals |

---

## Track E: Baez Monitoring (Priority: LOW)

**Goal:** Watch for Baez extension to mass formulas.

---

*φ² + 1/φ² = 3 | TRINITY*
