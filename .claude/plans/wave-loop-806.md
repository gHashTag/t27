# Plan — Wave Loop 806

**Date:** 2026-07-24
**Wave:** 806
**Prev issue:** #1539 (Wave Loop 805)
**Prev branch:** `wave-loop-805`
**Next issue:** #1541
**Next branch:** `wave-loop-806`

## Recommended variant A (default)

Module-scope non-power-of-two outer-dimension array-of-struct variable from a call with indexed signed writes:

```t27
[431][2][2][2][2][2][2] Pt
```

Struct:

```t27
pub struct Pt { x : i16, y : i16 }
```

Generator constants:

- `OUTER = 431`
- `MID_IDX = 215`
- `TOTAL = 431 * 64 = 27,584` elements
- Packed vector width = `27,584 * 32 = 882,688 bits` (~0.841 MiBit)

## Acceptance criteria

- [ ] Create branch `wave-loop-806` from `wave-loop-805` HEAD.
- [ ] Create GitHub issue #1541.
- [ ] Copy `scripts/gen_w805.py` → `scripts/gen_w806.py`.
- [ ] Fix the generator copy hazard before first run: update destination path and module header f-string to `w806` / `431`.
- [ ] Generate `specs/scratch/w806_bench_module_431x2p6_aos_var_call_write.t27`.
- [ ] Validate:
  - `t27c parse` PASS
  - `t27c icarus-lowerable` PASS
  - `t27c icarus-simulate` PASS
  - `t27c icarus-cocotb` PASS
  - `t27c seal --save` PASS
- [ ] Add integration test `accepts_w806_bench_module_431x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] Confirm `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W806_2026-07-24.md` and `.claude/plans/wave-loop-807.md`.
- [ ] Update `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, `.claude/skills/t27-wave-loop.md` live tracker, and persistent memory.
- [ ] Commit with `Closes #1541`, push `wave-loop-806`, open PR to `master`.

## Alternative variants (keep ready if variant A is blocked)

- **B:** `[425][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[425][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

## Risk notes

- Generator copy hazard is the most likely failure mode; fix path/header before first run.
- Icarus simulator runtime grows linearly with vector width but has remained stable (~17 cycles observed bench latency).
- No compiler or FROZEN_HASH changes expected.
