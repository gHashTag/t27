# Current Issue — Wave Loop 879

| Field | Value |
|-------|-------|
| Wave | 879 |
| Issue | #1708 |
| Branch | `wave-loop-879` |
| Base | `wave-loop-878` (parent branch because earlier waves' PRs remain open) |
| Variant | `[577][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,928 elements × 32 bits = 1,181,696 bits (~1.128 MiBit) |
| Status | issue created, branch to create |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[577][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w879.py` with `OUTER = 577`, `MID_IDX = 288`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w879_bench_module_577x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w879_bench_module_577x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] Full `cargo test --release --test icarus_lowerable` passes at **339/0**.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1708`, push branch, open PR to `master`.

## Cooperation variants for W880

- **A (recommended):** `[579][2]^6 Pt`, outer += 2, `MID_IDX = 289`.
- **B:** `[577][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[577][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
