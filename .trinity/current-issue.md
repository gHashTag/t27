# Current Issue — Wave Loop 880

| Field | Value |
|-------|-------|
| Wave | 880 |
| Issue | #1712 (expected; GitHub may assign a different number) |
| Branch | `wave-loop-880` |
| Base | `wave-loop-879` (parent branch because earlier waves' PRs remain open) |
| Variant | `[579][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 37,056 elements × 32 bits = 1,185,792 bits (~1.131 MiBit) |
| Status | plan ready, issue to create |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[579][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w880.py` with `OUTER = 579`, `MID_IDX = 289`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w880_bench_module_579x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w880_bench_module_579x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] Full `cargo test --release --test icarus_lowerable` passes at **340/0**.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1712`, push branch, open PR to `master`.

## Cooperation variants for W881

- **A (recommended):** `[581][2]^6 Pt`, outer += 2, `MID_IDX = 290`.
- **B:** `[579][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[579][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
