# 🌊 WAVE LOOP 98 — DECOMPOSED PLAN

*Date: 2026-06-17 | Branch: trinity-rust-rings | Target: ≤8 open issues, arXiv submitted*

---

## Objective

Split #943 zombie, submit arXiv preprint, fix Lean 4 proof, and reduce CORDIC LUT count.

---

## Track A: #943 Zombie Split (Priority: CRITICAL)

**Goal:** Split #943 into 8 atomic issues and close the original.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| A1 | Create issue for bug 1: bridge watch URL | #1209 |
| A2 | Create issue for bug 2: GraphQL injection | #1210 |
| A3 | Create issue for bug 3: proxy DoS (unbounded body) | #1211 |
| A4 | Create issue for bug 4: proxy FD exhaustion | #1212 |
| A5 | Create issue for bug 5: audio success count inflation | #1213 |
| A6 | Create issue for bug 6: invalid WAV headers | #1214 |
| A7 | Create issue for bug 7: partial_cmp panic on NaN | #1215 |
| A8 | Create issue for bug 8: division by zero in formula_eval | #1216 |
| A9 | Close #943 as superseded | Comment |

**Success criterion:** Open issues ≤8.

---

## Track B: arXiv Submission (Priority: CRITICAL)

**Goal:** Submit trinity_arxiv.tex with neutrino prediction featured.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| B1 | Locate LaTeX skeleton from W61 | `paper/` directory |
| B2 | Add neutrino section (Σ m_ν ≈ 0.018 eV) | LaTeX |
| B3 | Add honest withdrawal erratum | LaTeX |
| B4 | Compile PDF with 0 warnings | Log |
| B5 | Submit to arXiv (physics.gen-ph) | ID |

---

## Track C: Lean 4 Proof Repair (Priority: HIGH)

**Goal:** Fix `linarith` failure in CorePhi.lean:80.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| C1 | Analyze failing proof at line 80 | Analysis |
| C2 | Replace `linarith` with explicit steps or `nlinarith` | Fix |
| C3 | Verify `lake build` passes | Log |

---

## Track D: CORDIC Optimization (Priority: MEDIUM)

**Goal:** Reduce LUT count from 699 to <400.

### Sub-tasks
| # | Action | Deliverable |
|---|--------|-------------|
| D1 | Implement double-step CORDIC | `cordic_double.t27` |
| D2 | Synthesize with Yosys | Log |

---

## Track E: Suite Health (Priority: MANDATORY)

**Goal:** Maintain 555/555 PASS and 536/0 failed/1 ignored.

---

*φ² + 1/φ² = 3 | TRINITY*
