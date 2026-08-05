# Current Issue — Wave Loop 878

| Field | Value |
|-------|-------|
| Wave | 878 |
| Issue | #1704 (expected; GitHub may assign a different number) |
| Branch | `wave-loop-878` |
| Base | `wave-loop-877` (parent branch because earlier waves' PRs remain open) |
| Variant | `[575][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,800 elements × 32 bits = 1,177,600 bits (~1.124 MiBit) |
| Status | branch created, ready to implement |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[575][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w878.py` with `OUTER = 575`, `MID_IDX = 287`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w878_bench_module_575x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w878_bench_module_575x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] Full `cargo test --release --test icarus_lowerable` passes at **338/0**.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1704`, push branch, open PR to `master`.

## Cooperation variants for W879

- **A (recommended):** `[577][2]^6 Pt`, outer += 2, `MID_IDX = 288`.
- **B:** `[575][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[575][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
