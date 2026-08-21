# Wave Loop 897 Plan

**Target issue:** #1857 — feat(igla): Wave Loop 897 — module-scope `[613][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes
**Branch:** `wave-loop-897` from `wave-loop-896` HEAD
**Date:** 2026-08-06

---

## Goal

Add the next mechanical rung to the packed-vector AoS ladder: outer dimension **613**, inner dimension **2^6**, struct `Pt { x : i16, y : i16 }`. Expected packed vector: 613 × 64 = **39,232 elements**, 39,232 × 32 = **1,255,424 bits (~1.198 MiBit)**.

---

## PHI LOOP phases

1. **Issue** — use #1857 (already created).
2. **Spec** — generate `specs/scratch/w897_bench_module_613x2p6_aos_var_call_write.t27` from a copied generator.
3. **TDD** — spec already contains `bench` / `assert_eq` / `test` blocks via generator.
4. **Code/Impl** — generator only; no compiler change expected.
5. **Gen** — `t27c` compiles the spec.
6. **Seal** — `t27c seal --save` and `t27c seal --verify`.
7. **Verify** — targeted and full `icarus_lowerable` suite.
8. **Land** — PR with `Closes #1857`, auto-merge/rebase as earlier waves land.
9. **Learn** — update skills, trackers, memory, experience.

---

## File checklist

- [ ] Copy `scripts/gen_w896.py` → `scripts/gen_w897.py`
- [ ] Update generator constants:
  - `OUTER = 613`
  - `LAST_IDX = 612`
  - `MID_IDX = 306` (comment `306`)
  - destination path → `w897_bench_module_613x2p6_aos_var_call_write.t27`
  - module header f-string with outer 613
- [ ] Run generator and sanity-check with `grep`:
  - `611` must not appear in `gen_w897.py` or generated spec
  - `305` must not appear in comments/bounds
- [ ] Add integration test in `bootstrap/tests/icarus_lowerable.rs`
- [ ] Run all validation gates
- [ ] Create seal JSON
- [ ] Update `.claude/skills/t27-wave-loop.md` with W897 worked example
- [ ] Update `.claude/skills/t27-master-executor.md` merge-queue status
- [ ] Update `.claude/skills/wave-loop-autopilot.md` run-list
- [ ] Update `docs/NOW.md`
- [ ] Update `.trinity/experience.md`
- [ ] Update `.trinity/current-issue.md` to next wave (#1858 or TBD)
- [ ] Write close-out report `docs/reports/FPGA_LOOP_CLOSEOUT_W897_2026-08-06.md`
- [ ] Write persistent memory file

---

## Risk notes

- If 1.198 MiBit finally crosses a hard threshold, `icarus-lowerable` may start failing.
- Watch for generator copy hazards at three locations: destination path, module header, `MID_IDX`.
- Pre-existing full-suite failure is not a blocker.

---

## Success criteria

- `t27c parse` PASS
- `t27c icarus-lowerable` returns `lowerable`
- `t27c icarus-simulate` PASSED
- `t27c seal --verify` MATCH
- Targeted cargo test PASS
- Full suite passes increase by 1 (356/1 expected if no threshold hit)
