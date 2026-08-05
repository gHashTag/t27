# Current Issue — Wave Loop 866

| Field | Value |
|-------|-------|
| Wave | 866 |
| Issue | #1680 (expected) |
| Branch | `wave-loop-866` |
| Base | `wave-loop-865` (parent branch because earlier waves' PRs remain open) |
| Variant | `[551][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 35,264 elements × 32 bits = 1,128,448 bits (~1.076 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[551][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w866.py` with `OUTER = 551`, `MID_IDX = 275`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w866_bench_module_551x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w866_bench_module_551x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1680`, push branch, open PR to `master`.

## Cooperation variants for W867

- **A (recommended):** `[553][2]^6 Pt`, outer += 2, `MID_IDX = 276`.
- **B:** `[551][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[551][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
