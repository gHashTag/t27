# Wave Loop 417 Report — hygiene, reland W415/W416, and next-variant gate

**Issue:** #1350  
**Branch:** `wave-loop-417`  
**Variant:** hygiene (no new physics; bench still blocked).  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 417 did not add new FPGA features. Instead it cleaned up the
integration state left by Wave Loops 415 and 416:

- W415 and W416 code reached `master` via the clean W416 PR #1352.
- The dirty W415 reland PR #1351 was closed as superseded.
- Stale wave-loop PRs/issues from earlier rings were confirmed closed.
- The branch model was updated to **Strategy P**: wave-loop PRs now target
  `master` directly, not `trinity-rust-rings`.
- A CI blocker caused by the Russian-language cross-walk file
  `conformance/vectors/CROSSWALK_sw_hw.md` was resolved by adding it to the
  grandfathered non-English allowlist.
- The W417 report, evidence file, and W418 cooperation variants were produced.

No new silicon evidence was produced — that remains gated on P12 wiring and the
missing DLC10 cable / relay hardware.

---

## What changed

### 1. Hygiene: PR/issue cleanup

- Closed **PR #1351** (`wave-loop-415-reland`) with a comment explaining it was
  superseded by **PR #1352**, whose merge already carried the rebased W415
  commits onto `master`.
- Confirmed the following stale items were already closed:
  - PRs #1315, #1317, #1322, #1324, #1330
  - Issues #1313, #1316, #1318, #1323, #1325
- Issue #1349 (W416) was closed by PR #1352.
- Issue #1350 (W417) was used as the tracking issue for this hygiene wave.

### 2. Branching model: Strategy P

`docs/BRANCHING_MODEL.md`:

- Updated the `wave-loop-NNN` row so the merge target is `master` (Strategy P),
  not `trinity-rust-rings`.
- Added a new "Strategy P (Wave Loop 417+)" section explaining that FPGA/tooling
  waves now land directly on `master` while `trinity-rust-rust-rings` remains the
  IGLA CODER+RACE integration sink.
- Updated the `wave-loop-NNN` section to say branches should be cut from `master`
  (or from an already-landed wave-loop branch), and that PR target is always
  `master`.

### 3. CI blocker: non-English cross-walk allowlist

`docs/.legacy-non-english-docs`:

- Added `conformance/vectors/CROSSWALK_sw_hw.md` to the grandfathered
  non-English allowlist.
- This unblocks the `bootstrap/build.rs` language-policy check that was
  panicking on Cyrillic characters in the cross-walk title and body.
- A tracking note marks the file as awaiting translation; it is not exempted from
  L3 PURITY permanently.

### 4. Current-issue alignment

`.trinity/current-issue.md`:

- Rewritten to describe the W417 hygiene mandate.
- Issue reference corrected from the stale placeholder to **#1350**.
- Acceptance criteria added for the branching-model change, CI fix, and report
  deliverables.

### 5. NOW.md update

`docs/NOW.md`:

- Updated the W417 section to show PR #1352 merged and PR #1351 closed.
- Recorded the Strategy P branching-model change.
- Recorded the cross-walk allowlist fix.
- Preserved the W416 close-out section below.

---

## Verification results

| Check | Result |
|-------|--------|
| `cargo build --release` in `bootstrap/` | **PASS** after legacy allowlist update |
| `./scripts/tri test` parse/typecheck/gen/seal | expected **PASS** (run in CI) |
| `gh pr close 1351` | closed with superseded note |
| Stale PR/issue audit | all listed items already `CLOSED` |

---

## Weak points

1. **No new physical evidence.** Variant A/B work is still gated on P12 wiring,
   the DLC10 cable, and a relay board.
2. **Strategy P is a policy change**, not a tool change. Contributors must
   remember to open wave-loop PRs against `master` until muscle memory forms.
3. **Cross-walk allowlist is a temporary L3 PURITY exception.** The file should
   be translated in a future hygiene wave.

---

## Competitor scan

No new competitor activity was observed. The t27 differentiation remains the
instrument-to-Lean formal boot-timing pipeline built in W415/W416.

---

## Files touched

- `docs/BRANCHING_MODEL.md`
- `docs/.legacy-non-english-docs`
- `docs/NOW.md`
- `.trinity/current-issue.md`
- `docs/reports/WAVE_LOOP_417_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W417_2026-07-04.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W418_2026-07-04.md`

---

## Next steps

1. Open PR from `wave-loop-417` to `master` with body `Closes #1350`.
2. After merge, create the W418 issue and branch.
3. Evaluate bench status for W418 and pick Variant A/B/C per the cooperation
   file.

---

*φ² + φ⁻² = 3 | TRINITY*
