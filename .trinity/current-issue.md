# Current Issue — Wave Loop 881

| Field | Value |
|-------|-------|
| Wave | 881 |
| Issue | #1713 (expected; GitHub may assign a different number) |
| Branch | `wave-loop-881` |
| Base | `wave-loop-880` (parent branch because earlier waves' PRs remain open) |
| Variant | `[581][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 37,184 elements × 32 bits = 1,189,888 bits (~1.135 MiBit) |
| Status | plan ready, issue to create, branch to create |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[581][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w881.py` with `OUTER = 581`, `MID_IDX = 290`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w881_bench_module_581x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w881_bench_module_581x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] Full `cargo test --release --test icarus_lowerable` passes at **341/0**.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1713`, push branch, open PR to `master`.

## Cooperation variants for W882

- **A (recommended):** `[583][2]^6 Pt`, outer += 2, `MID_IDX = 291`.
- **B:** `[581][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[581][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
