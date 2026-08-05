# Current Issue — Wave Loop 877

| Field | Value |
|-------|-------|
| Wave | 877 |
| Issue | #1702 (expected; GitHub may assign a different number) |
| Branch | `wave-loop-877` |
| Base | `wave-loop-876` (parent branch because earlier waves' PRs remain open) |
| Variant | `[573][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,672 elements × 32 bits = 1,173,504 bits (~1.120 MiBit) |
| Status | branch created, ready to implement |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[573][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w877.py` with `OUTER = 573`, `MID_IDX = 286`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w877_bench_module_573x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w877_bench_module_573x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] Full `cargo test --release --test icarus_lowerable` passes at **337/0**.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1702`, push branch, open PR to `master`.

## Cooperation variants for W878

- **A (recommended):** `[575][2]^6 Pt`, outer += 2, `MID_IDX = 287`.
- **B:** `[573][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[573][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
