# 🌊 WAVE LOOP 96 — DECOMPOSED PLAN

*Date: 2026-06-17 | Branch: trinity-rust-rings | Target: ≤10 open issues, CORDIC LUT <400*

---

## Objective

Split remaining zombie issues, reduce CORDIC LUT count, and submit arXiv preprint.

---

## Track A: Zombie Split (Priority: CRITICAL)

**Goal:** Split #932 and #943 into atomic issues.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| A1 | Split #932 bug 2 (missing seal = SKIP) into new issue | #1207 |
| A2 | Split #932 bug 3 (hash "none") into new issue | #1208 |
| A3 | Close #932 as superseded by atomic issues | Comment |
| A4 | Split #943 into 8 atomic issues | #1209-#1216 |
| A5 | Close #943 as superseded | Comment |

**Success criterion:** Open issues ≤10 after split/close cycle.

---

## Track B: CORDIC Optimization (Priority: HIGH)

**Goal:** Reduce LUT count from 699 to <400.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| B1 | Implement double-step CORDIC (2 angles per iteration) | `cordic_double.t27` |
| B2 | Synthesize with Yosys and compare LUT count | Synthesis log |
| B3 | If <400 LUTs, integrate into main spec | PR |

**Success criterion:** Synthesis reports <400 LUTs.

---

## Track C: arXiv Submission (Priority: HIGH)

**Goal:** Submit trinity_arxiv.tex.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| C1 | Add neutrino prediction section (Σ m_ν ≈ 0.018 eV) | LaTeX section |
| C2 | Add honest withdrawal erratum | LaTeX paragraph |
| C3 | Compile PDF with 0 warnings | `pdflatex` log |
| C4 | Submit to arXiv (physics.gen-ph) | Submission ID |

---

## Track D: Suite Health (Priority: MANDATORY)

**Goal:** Maintain 555/555 PASS and 536/0 failed/1 ignored.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| D1 | `./scripts/tri test` | Log |
| D2 | `cargo clippy --workspace --all-features` | Log |
| D3 | `cargo test --workspace --all-features` | Log |
| D4 | Regenerate seals if needed | Seals |

---

## Track E: Competitive Intel (Priority: LOW)

**Goal:** Monitor for new competitors.

---

*φ² + 1/φ² = 3 | TRINITY*
