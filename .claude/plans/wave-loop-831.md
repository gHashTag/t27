# Wave Loop 831 Cooperation Plan

| Field | Value |
|-------|-------|
| Wave | 831 |
| Issue | #1603 (expected) |
| Branch | `wave-loop-831` |
| Parent branch | `wave-loop-830` HEAD because earlier waves' PRs remain open |
| Status | planned |

## Goal

Continue the non-power-of-two outer-dimension module-scope packed array-of-struct
(AoS) ladder. The default Variant A extends the established `[N][2]^6 Pt`
pattern. Variants B and C explore shape and addressing stress.

## Variants

### A — Recommended: `[481][2]^6 Pt` module-scope AoS variable from call with indexed signed writes

- Outer dimension += 2 (479 → 481), keeping the inner `2^6` and `Pt { x : i16, y : i16 }` pattern.
- Expected 30,784 elements, 985,088-bit packed vector (~0.939 MiBit).
- Still well under the 4-MiBit packed-vector cliff.
- Expected zero compiler / reference-model / `FROZEN_HASH` changes.
- Mechanical generator copy-hazard fix required (`w831` / `481` / `MID_IDX = 240`).

### B — Implementation-heavy: `[479][3]^6 Pt` stride scaling

- Keep outer dimension at 479 but grow the second inner dimension from `2` to `3`.
- Changes the flattened stride and element count (479 × 3^6 = 349,551 elements, ~11.18 MiBit), which may cross simulation wall-clock or width limits.
- Risk: may hit the 4-MiBit cliff or expose a backend width/stride bug.
- If it fails, convert into a negative boundary witness and align the classifier.

### C — Process/tooling: `[479][2]^6 Pt` with negative-index writes

- Same shape as W830 but add signed negative indices to exercise wrap-around addressing.
- Tests that the signed-index lowering path handles negative offsets correctly.
- Risk: if negative indices are not lowerable, add a classifier rule and negative witness.

## Recommended path

Choose **Variant A** for the mechanical ladder; it is the smallest, safest width
increment and preserves the existing pattern. Reserve Variants B and C for when
Variant A becomes blocked or when a deliberate stress/negative-boundary witness
is needed.

## Acceptance criteria

- [ ] Generator `scripts/gen_w831.py` with correct `OUTER` and `MID_IDX`.
- [ ] Witness generated and passes `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`.
- [ ] Integration test added; full `icarus_lowerable` suite expected to be 291/0.
- [ ] `FROZEN_HASH` unchanged unless a compiler change is required.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot, and persistent memory updated.
- [ ] Commit with `Closes #{issue}`, push branch, open PR.

---

*φ² + φ⁻² = 3 | TRINITY*
