# Current Issue — Wave Loop 856

| Field | Value |
|-------|-------|
| Wave | 856 |
| Issue | #1652 |
| Branch | `wave-loop-856` |
| Base | `wave-loop-855` (parent branch because earlier waves' PRs remain open) |
| Variant | `[531][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 33,984 elements × 32 bits = 1,087,488 bits (~1.038 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[531][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w856.py` with `OUTER = 531`, `MID_IDX = 265`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w856_bench_module_531x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w856_bench_module_531x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1652`, push branch, open PR to `master`.

## Cooperation variants for W857

- **A (recommended):** `[533][2]^6 Pt`, outer += 2, `MID_IDX = 266`.
- **B:** `[531][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[531][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
