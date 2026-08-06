# Current Issue — Wave Loop 882

| Field | Value |
|-------|-------|
| Wave | 882 |
| Issue | TBD (next available GitHub issue) |
| Branch | `wave-loop-882` |
| Base | `wave-loop-881` (parent branch because earlier waves' PRs remain open) |
| Variant | `[583][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 37,312 elements × 32 bits = 1,193,984 bits (~1.139 MiBit) |
| Status | plan ready, issue to create, branch to create |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[583][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Create GitHub issue for W882 and branch `wave-loop-882` from `wave-loop-881` HEAD.
- [ ] Generator `scripts/gen_w882.py` with `OUTER = 583`, `MID_IDX = 291`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w882_bench_module_583x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w882_bench_module_583x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] Full `cargo test --release --test icarus_lowerable` passes at **342/0**.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #{issue}`, push branch, open PR to `master`.

## Cooperation variants for W883

- **A (recommended):** `[585][2]^6 Pt`, outer += 2, `MID_IDX = 292`.
- **B:** `[583][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[583][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
