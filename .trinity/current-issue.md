# Current Issue — Wave Loop 883

| Field | Value |
|-------|-------|
| Wave | 883 |
| Issue | TBD (next available GitHub issue) |
| Branch | `wave-loop-883` |
| Base | `wave-loop-882` (parent branch because earlier waves' PRs remain open) |
| Variant | `[585][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 37,440 elements × 32 bits = 1,198,080 bits (~1.143 MiBit) |
| Status | plan ready, issue to create, branch to create |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[585][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Create GitHub issue for W883 and branch `wave-loop-883` from `wave-loop-882` HEAD.
- [ ] Generator `scripts/gen_w883.py` with `OUTER = 585`, `MID_IDX = 292`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w883_bench_module_585x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w883_bench_module_585x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] Full `cargo test --release --test icarus_lowerable` passes at **343/0**.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #{issue}`, push branch, open PR to `master`.

## Cooperation variants for W884

- **A (recommended):** `[587][2]^6 Pt`, outer += 2, `MID_IDX = 293`.
- **B:** `[585][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[585][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
