# FPGA Loop Closeout — Wave Loop 584

**Date:** 2026-07-07  
**Branch:** `wave-loop-584`  
**Issue:** #1555

## Goal

Continue the rank-scaling sequence W566–W582 by validating a 17-D
array-of-struct return call deduplication witness (`[2]^17 Pt`). This is a
stress test of the function-local / call-return rank-agnostic machinery at
4,194,304 packed bits.

## What changed

- `specs/scratch/w584_bench_17d_aos_call_dedup.t27`
  - New deterministic witness:
    - `pub fn make_heptadeca() -> [2]^17 Pt` returning a fully-nested literal
      with 131,072 scalar-struct elements.
    - `test` block with indexed probes and whole-array assertions against a
      local `expected` literal.
    - `bench` block that binds the call result to a local variable and asserts
      equality against `expected`, reusing the W573–W582 local-`expected`
      workaround.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w584_bench_17d_aos_call_dedup`.
- `.trinity/seals/scratch_w584_bench_17d_aos_call_dedup.json`
  - New seal for the W584 witness.
- `.trinity/icarus-baselines/specs/scratch/w584_bench_17d_aos_call_dedup.json`
  - New Icarus baseline.
- No compiler changes were required; the W583 `emit_packed_scalar_value`
  width-cast fix already covers computed-field initializers, and the 17-D
  literal uses only literals, so no new code path was exercised.

## What did not change

- `bootstrap/src/compiler.rs` — unchanged.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.
- Yosys synthesis-smoke width-warning count remains 24 pre-existing failures.

## Scientific / engineering background

- IEEE Std 1800-2017 §7.4.1 packed-array minimum = 65,536 bits. W584 tests a
  4,194,304-bit vector — **64×** the LRM minimum.
- Icarus Verilog 12.0 accepted the 4-MiBit packed vector when the wide literal
  is bound to a local variable before assertion. This confirms the W573–W582
  `$display` VPI workaround scales to rank 17.
- Icarus `vpi/sys_display.c` allocates decimal string buffers proportional to
  vector width (`calc_dec_size`), so direct `$display` of a 4-MiBit value is
  the stress point, not the packed vector itself.
- EDA Playground / Cadence reports show simulator-specific segfaults around
  500 kbit packed vectors; Icarus 12.0 on this host handled 4 MiBit without
  crash, but wall-clock grew to ~22.5 min for direct simulation.

## Verification matrix

| Check | Result |
|---|---|
| `cargo build --release -p t27c` | OK (no rebuild needed) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 44 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke failures |
| Direct `t27c icarus-simulate` W584 | PASS (~22.5 min wall-clock) |
| Direct `t27c icarus-cocotb` W584 | PASS (~23.7 min wall-clock) |

## Weak spot addressed

Validated that the rank-agnostic compiler paths scale from 16-D to 17-D
without modification, and that Icarus 12.0 can simulate a 4-MiBit packed
vector when the assertion-side literal is bound to a local variable. Also
confirmed that at rank 17, indexed probes need three leading zeros to stay
within the signed `i16` field range (`e ≤ 16383`).

## Risks accepted

- Direct simulation wall-clock is now ~22.5 min; 18-D would likely exceed 40 min
  and approach CI timeout limits. The `--fast` tri gate still passes because it
  reuses the saved Icarus baseline for the W584 witness after the first
  successful run.
- The witness file is ~22 MB / ~1.18 M lines, which is acceptable for git but
  doubles the repo size contribution of the AoS rank-scaling series.
- Indexed probes cover only the lower half of the address space due to the
  signed i16 constraint; this is inherent to the chosen `Pt { x: i16, y: i16 }`
  struct.

## Next wave cooperation variants (Wave Loop 585)

### Variant A — 18-D rank scaling
`[2]^18 Pt` (8,388,608 bits, 262,144 elements). The next doubling. Risk:
witness ~44 MB / ~2.4 M lines; direct simulation likely 40+ min, may exceed
practical CI budget.

### Variant B — Non-power-of-two at rank 17
`[3][2]^17 Pt` (6,291,456 bits, 393,216 elements). Tests product-based
width/index arithmetic at the boundary while staying within the same rank
class. Witness ~33 MB / ~1.8 M lines.

### Variant C — Large module-scope multi-D AoS variable (recommended)
A module `var` of type `[2][2][2][2][2][2][2]Pt` (7-D, 16,384 elements,
524,288 bits) initialized from a function call and used in multiple bench/test
sites. Combines W583 module-scope learning with W557 call-array CSE while
keeping file size small and avoiding the 20+ minute simulation wall-clock of
rank 17/18.
