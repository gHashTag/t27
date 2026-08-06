# Plan — Wave Loop 804

**Date:** 2026-07-24
**Wave:** 804
**Prev issue:** #1535 (Wave Loop 803)
**Prev branch:** `wave-loop-803`
**Next issue:** #1537
**Next branch:** `wave-loop-804`

## Recommended variant A (default)

Module-scope non-power-of-two outer-dimension array-of-struct variable from a call with indexed signed writes:

```t27
[427][2][2][2][2][2][2] Pt
```

Struct:

```t27
pub struct Pt { x : i16, y : i16 }
```

Generator constants:

- `OUTER = 427`
- `MID_IDX = 213`
- `TOTAL = 427 * 64 = 27,328` elements
- Packed vector width = `27,328 * 32 = 875,008 bits` (~0.834 MiBit)

## Acceptance criteria

- [ ] Create branch `wave-loop-804` from `wave-loop-803` HEAD.
- [ ] Create GitHub issue #1537.
- [ ] Copy `scripts/gen_w803.py` → `scripts/gen_w804.py`.
- [ ] Fix the generator copy hazard before first run: update destination path and module header f-string to `w804` / `427`.
- [ ] Generate `specs/scratch/w804_bench_module_427x2p6_aos_var_call_write.t27`.
- [ ] Validate:
  - `t27c parse` PASS
  - `t27c icarus-lowerable` PASS
  - `t27c icarus-simulate` PASS
  - `t27c icarus-cocotb` PASS
  - `t27c seal --save` PASS
- [ ] Add integration test `accepts_w804_bench_module_427x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] Confirm `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W804_2026-07-24.md` and `.claude/plans/wave-loop-805.md`.
- [ ] Update `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, `.claude/skills/t27-wave-loop.md` live tracker, and persistent memory.
- [ ] Commit with `Closes #1537`, push `wave-loop-804`, open PR to `master`.

## Alternative variants (keep ready if variant A is blocked)

- **B:** `[425][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[425][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

## Risk notes

- Generator copy hazard is the most likely failure mode; fix path/header before first run.
- Icarus simulator runtime grows linearly with vector width but has remained stable (~17 cycles observed bench latency).
- No compiler or FROZEN_HASH changes expected.
