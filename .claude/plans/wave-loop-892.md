# Wave Loop 892 Plan

**Issue:** #1845 — feat(igla): Wave Loop 892 — module-scope `[603][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

---

## Goal

Continue the mechanical packed-vector array-of-struct ladder one step past W891. The target witness is module-scope `[603][2]^6 Pt`:

- Outer dimension: 603 (non-power-of-two)
- Inner struct: `[2]^6 Pt` = 384 bits per element
- Total elements: 603 × 64 = 38,592
- Packed vector width: 38,592 × 32 = 1,234,944 bits (~1.178 MiBit)

Pattern identical to previous waves: module-scope variable initialized from a pure `make_grid(0)` call, with indexed signed field writes and `assert_eq` read-back inside a `bench` block.

---

## Variants

### A — Mechanical increment (recommended)
- Copy `scripts/gen_w891.py` → `scripts/gen_w892.py`.
- Set `OUTER = 603`, `MID_IDX = 301`.
- Fix destination path, module header, `MID_IDX` comment; verify with `grep`.
- Run gates, add test, seal, commit, push, open PR with `Closes #1845`, auto-merge.
- Expected zero compiler / `FROZEN_HASH` changes.

### B — Increase inner struct width
- Keep outer dimension 603 but expand inner struct to `[2]^8 Pt` or `[4]^6 Pt`.
- Could reveal whether the ceiling is element count vs total packed width.
- Larger generated spec; defer until the ladder hits a hard boundary.

### C — Variable signed-index stress
- Replace constant `MID_IDX` writes with dynamic index expressions.
- Tests Icarus index normalization and cocotb reference-model agreement.
- Risk of compiler/reference-model delta; keep as a side experiment.

---

## Procedure

1. Create and push branch `wave-loop-892` from `wave-loop-891` HEAD.
2. Copy generator and fix three stale-reference locations.
3. `python3 scripts/gen_w892.py`.
4. Run direct gates: parse, lowerable, simulate, cocotb, seal save + verify.
5. Add integration test to `bootstrap/tests/icarus_lowerable.rs`.
6. Run targeted cargo test.
7. Commit with `Closes #1845`, push, open PR, enable auto-merge.
8. Write closeout report and update trackers.

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --verify` → `MATCH`
- [ ] Targeted cargo test PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened referencing #1845

phi^2 + 1/phi^2 = 3 | TRINITY
