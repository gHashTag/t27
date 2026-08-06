# Wave Loop 104 Plan
## Dataset Scale-Up + Competitive Intelligence + L4 Expansion

**Date:** 2026-06-17
**Target:** 562/562 PASS, 0 clippy, L3 clean
**Focus:** Close dataset scale gap; document new RTL competitors; expand L4 coverage.

---

## Track A: Dataset Scale-Up (dataset.t27)

**Problem:** ~320 samples (40 base × 8 mutations). Real training needs 10K+.
**Solution:**
- Add 4 new RTL templates: `counter`, `shift_register`, `divider`, `fsm`
- Add parameter permutation: clock polarity (posedge/negedge), reset level (active-high/low)
- Expand adder to 64-bit
- Add tests for new templates and permutations

---

## Track B: Competitive Intelligence Update

**New competitors discovered:**
- VeriAgent (arXiv:2603.17613) - PPA-aware multi-agent
- RTLScout (arXiv:2606.06530) - agentic code + synthesis optimization
- COEVO (arXiv:2604.15001) - co-evolutionary framework
- NL2GDS (arXiv:2603.05489) - NL to GDSII layout

**Action:** Update `docs/COMPETITIVE_POSITIONING.md` with differentiation analysis.

---

## Track C: L4 Coverage Expansion

**Action:** Verify all modified specs have `test`/`invariant`/`bench` blocks. Add missing invariants.

---

## Verification Checklist

- [ ] cargo build --release (0 errors)
- [ ] ./target/release/t27c suite --repo-root . (562/562 PASS)
- [ ] cargo clippy --workspace --all-features (0 warnings)
- [ ] ./target/release/t27c lint --ascii for all modified specs (clean)
- [ ] Regenerate seals for all modified specs
- [ ] Write WAVE_LOOP_104_REPORT.md
- [ ] Write WAVE_LOOP_104_COOPERATION.md
- [ ] Increment .commit_count

---

phi^2 + 1/phi^2 = 3 | TRINITY
