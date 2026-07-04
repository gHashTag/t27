# Wave Loop 417 — hygiene, reland W415/W416, and next-variant gate

**Issue:** #1350  
**Branch:** `wave-loop-417`  
**Milestone:** Close out the W415/W416 reland mess and set up the next FPGA boot-evidence wave.

---

## Goal

Wave 415 and Wave 416 were stuck on a dirty PR (#1346) and a branch that had
fallen behind `master`. Wave 417 lands the hygiene work: close/replace stale
PRs, fix the branch target to `master` (Strategy P), repair the CI blocker
introduced by the Russian cross-walk file, and produce the W417 report plus
W418 cooperation variants.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | GitHub PRs/issues | Close superseded #1351; confirm stale wave-loop PRs/issues are closed |
| 2 | `docs/BRANCHING_MODEL.md` | Document Strategy P: wave-loop PR target is `master` |
| 3 | `docs/.legacy-non-english-docs` | Allowlist `conformance/vectors/CROSSWALK_sw_hw.md` until translated |
| 4 | `docs/NOW.md` | Update W417 section to in-progress / close-out state |
| 5 | `docs/reports/*` | `WAVE_LOOP_417_REPORT.md`, `FPGA_LOOP_EVIDENCE_W417_2026-07-04.md`, `FPGA_LOOP_COOPERATION_W418_2026-07-04.md` |
| 6 | `.trinity/experience.md` | Capture W417 learnings |
| 7 | git/PR | Open PR from `wave-loop-417` to `master`, close #1350, create #1353 for W418 |

---

## Acceptance criteria

- [ ] AC-1: `docs/BRANCHING_MODEL.md` clearly states wave-loop PRs target `master`.
- [ ] AC-2: `cargo build --release` in `bootstrap/` no longer panics on the Russian cross-walk file.
- [ ] AC-3: `./scripts/tri test` parse/typecheck/gen/seal phases pass.
- [ ] AC-4: W417 report and W418 cooperation files exist and are linked from `docs/NOW.md`.
- [ ] AC-5: PR from `wave-loop-417` to `master` is opened with `Closes #1350`.

---

*φ² + φ⁻² = 3 | TRINITY*
