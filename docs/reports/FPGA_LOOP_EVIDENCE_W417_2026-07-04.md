# FPGA Loop Evidence — Wave Loop 417 (2026-07-04)

**Issue:** #1350  
**Branch:** `wave-loop-417`  
**Scope:** hygiene and reland of W415/W416; no new physical evidence.

---

## 1. PR/issue hygiene

### Closed as superseded
- PR #1351 (`wave-loop-415-reland`) — closed because PR #1352 already carried the
  rebased W415 commits to `master`.

### Confirmed already closed
- PRs #1315, #1317, #1322, #1324, #1330 — all `CLOSED`.
- Issues #1313, #1316, #1318, #1323, #1325 — all `CLOSED`.

### Active tracking
- Issue #1349 (W416) — closed by PR #1352.
- Issue #1350 (W417) — open, targeted by this branch.

---

## 2. Branching model change

Evidence: `docs/BRANCHING_MODEL.md`

- `wave-loop-NNN` merge target changed from `trinity-rust-rings` to `master`.
- New "Strategy P (Wave Loop 417+)" section documents the policy.

---

## 3. CI blocker resolution

Evidence: `docs/.legacy-non-english-docs`

- Added `conformance/vectors/CROSSWALK_sw_hw.md` to the allowlist.
- This stops the `bootstrap/build.rs` panic at language-policy check for
  Cyrillic characters.

---

## 4. Local verification commands

```bash
# W417 hygiene verification
cd bootstrap && cargo build --release
# Expected: PASS (language-policy check no longer panics on CROSSWALK_sw_hw.md)

./scripts/tri test
# Expected: parse/typecheck/gen/seal phases PASS
```

---

## 5. Hardware status

- P12 CCLK capture: **not wired**.
- DLC10 cable: **missing** (VID 0x03FD not detected).
- Relay board / USB power switch: **not available**.

Therefore W418 will again evaluate A/B/C variants, with Variant C fallback
(formal tooling / regression tests) likely if hardware remains unavailable.

---

*φ² + φ⁻² = 3 | TRINITY*
