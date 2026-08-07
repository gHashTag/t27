# Wave Loop 891 Plan

**Issue:** #1843 — feat(igla): Wave Loop 891 — module-scope `[601][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

---

## Goal

Continue the mechanical packed-vector array-of-struct ladder one step past W890. The target witness is module-scope `[601][2]^6 Pt`:

- Outer dimension: 601 (non-power-of-two)
- Inner struct: `[2]^6 Pt` = 2 fields × 6 trits × 32 bits = 384 bits per element
- Total elements: 601 × 64 = 38,464
- Packed vector width: 38,464 × 32 = 1,230,848 bits (~1.174 MiBit)

The pattern is identical to W881–W890: a module-scope variable initialized from a pure `make_grid(0)` call, with indexed signed field writes and `assert_eq` read-back inside a `bench` block.

---

## Variants

### A — Mechanical increment (recommended)
- Copy `scripts/gen_w890.py` → `scripts/gen_w891.py`.
- Update copy-hazard checklist: destination path, module header, `MID_IDX` comment.
- Set `OUTER = 601`, `MID_IDX = 300`.
- Generate spec, run gates, add integration test, seal, open PR #TBD with `Closes #1843`.
- Expected zero compiler / `FROZEN_HASH` changes.

### B — Increase inner struct width
- Keep outer dimension 601 but expand the inner struct to `[2]^8 Pt` or `[4]^6 Pt`.
- Same total-element probe but larger per-element footprint.
- Could reveal whether the ceiling is element count vs total packed width.
- Larger generated spec and longer CI; skip unless Variant A is uneventful and we want a second data point.

### C — Variable signed-index stress
- Within the W891 spec, replace the constant `MID_IDX` writes with a loop variable or offset expression.
- Tests Icarus index normalization and cocotb reference-model agreement under dynamic signed indices.
- Risk of compiler/reference-model delta; run only as a separate scratch experiment, not the main wave.

---

## Procedure

1. Create and push branch `wave-loop-891` from `wave-loop-890` HEAD (earlier waves' PRs still open).
2. Copy generator, fix three stale-reference locations, verify with `grep`.
3. `python3 scripts/gen_w891.py`.
4. Run direct gates:
   - `t27c parse`
   - `t27c icarus-lowerable`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save` + `seal --verify`
5. Add `accepts_w891_bench_module_601x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
6. Run targeted cargo test.
7. Commit with `Closes #1843`, push, open PR, enable auto-merge.
8. Write closeout report, update trackers, skills, and persistent memory.

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --verify` → `MATCH`
- [ ] Targeted cargo test PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened referencing #1843

phi^2 + 1/phi^2 = 3 | TRINITY
