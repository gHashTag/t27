# Current Issue — Wave Loop 864

| Field | Value |
|-------|-------|
| Wave | 864 |
| Issue | #1672 (expected) |
| Branch | `wave-loop-864` |
| Base | `wave-loop-863` (parent branch because earlier waves' PRs remain open) |
| Variant | `[547][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 35,008 elements × 32 bits = 1,120,256 bits (~1.068 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[547][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w864.py` with `OUTER = 547`, `MID_IDX = 273`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w864_bench_module_547x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w864_bench_module_547x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1672`, push branch, open PR to `master`.

## Cooperation variants for W865

- **A (recommended):** `[549][2]^6 Pt`, outer += 2, `MID_IDX = 274`.
- **B:** `[547][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[547][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
