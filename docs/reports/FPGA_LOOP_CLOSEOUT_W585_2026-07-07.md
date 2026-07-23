# FPGA Loop Closeout — Wave Loop 585

**Date:** 2026-07-07  
**Branch:** `wave-loop-585`  
**Issue:** #1556

## Goal

Shift focus away from further rank scaling (W584 already reached 4 MiBit and
~22.5 min direct simulation) and validate a new scope/CSE boundary: a
module-level mutable 7-D array-of-struct variable initialized from a function
call, read at multiple whole-array and indexed sites.

## What changed

- `specs/scratch/w585_bench_module_7d_aos_var_call_dedup.t27`
  - New deterministic witness:
    - `pub struct Pt { x : i16, y : i16 }`.
    - `pub fn make_week(offset : u16) -> [2][2][2][2][2][2][2]Pt` returning a
      deterministic 7-D nested literal with computed fields.
    - `pub const expected : [2]^7 Pt` and `pub var dst : [2]^7 Pt`, both
      initialized from `make_week(10)` to exercise multi-site module-scope CSE.
    - `test` block with whole-array equality and indexed field probes.
    - `bench` block with whole-array, indexed, and local-copy assertions,
      reading `dst` at multiple sites.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w585_bench_module_7d_aos_var_call_dedup`.
- `.trinity/seals/scratch_w585_bench_module_7d_aos_var_call_dedup.json`
  - New seal for the W585 witness.
- `.trinity/icarus-baselines/specs/scratch/w585_bench_module_7d_aos_var_call_dedup.json`
  - New Icarus baseline.
- No compiler changes were required. The W583 `emit_packed_scalar_value`
  width-cast fix already covers the computed-field initializers, and the
  module-scope `var` lowering path accepted a call-return initializer without
  modification.

## What did not change

- `bootstrap/src/compiler.rs` — unchanged.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.
- Yosys synthesis-smoke width-warning count remains 24 pre-existing failures.

## Scientific / engineering background

- **Global CSE and value numbering.** Cocke (1970) extended CSE across basic
  blocks; Kildall (1973) unified redundancy detection via global value
  numbering. Monniaux & Six (LCTES 2021) demonstrated a lightweight, Coq-certified
  global CSE pass for CompCert. t27’s Icarus-lowerable path applies the same
  conservative principle: a pure function-call result is materialized once and
  reused, never re-invoked.
- **CompCert CSE.** The CompCert `backend.CSE` pass conservatively resets
  equations at function calls and memory stores. t27 does not have memory
  stores, but module-scope variables and multi-site reads provide a similar
  cross-site reuse guarantee.
- **SystemVerilog packed variables.** IEEE Std 1800-2017 §7.4 permits packed
  arrays as module-level variables. At 524,288 bits, the W585 register is well
  below the 65,536-bit standard-discussion floor and the 4-MiBit W584 stress
  point, so Icarus 12.0 and Yosys handle it without width-related issues.

Sources:
- J. Cocke, “Global Common Subexpression Elimination,” *Symposium on Compiler
  Construction*, 1970. https://doi.org/10.1145/800028.808480
- G. A. Kildall, “A Unified Approach to Global Program Optimization,” *POPL*,
  1973. https://doi.org/10.1145/512927.512945
- D. Monniaux & C. Six, “Simple, light, yet formally verified, global common
  subexpression elimination and loop-invariant code motion,” *LCTES*, 2021.
  https://doi.org/10.1145/3461648.3463850
- CompCert `backend.CSE`: https://compcert.org/doc/html/compcert.backend.CSE.html

## Verification matrix

| Check | Result |
|---|---|
| `cargo build --release -p t27c` | OK (no rebuild needed) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 45 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke failures |
| Direct `t27c icarus-simulate` W585 | PASS |
| Direct `t27c icarus-cocotb` W585 | PASS |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |

## Weak spot addressed

Validated that a module-scope `var` can be initialized from a function call
returning a 524,288-bit packed 7-D array of scalar structs, and that the same
call result can be shared across a module-scope `const`, the `var`, and multiple
bench/test assertion sites without re-invoking the function or corrupting the
packed layout.

## Risks accepted

- The witness intentionally does not write to the module `var`; indexed field
  writes are deferred to W586 Variant C. This kept W585 within the “zero compiler
  change” boundary.
- Indexed probes are chosen from the lower half of the address space and stay
  comfortably inside the signed `i16` field range.
- The 24 pre-existing Yosys synthesis-smoke failures are unchanged and unrelated
  to W585.

## Next wave cooperation variants (Wave Loop 586)

### Variant A — 18-D rank scaling
`[2]^18 Pt` (8,388,608 bits, 262,144 elements). The next doubling. Risk:
witness ~44 MB / ~2.4 M lines; direct simulation likely 40+ min.

### Variant B — Non-power-of-two at rank 17
`[3][2]^17 Pt` (6,291,456 bits, 393,216 elements). Tests product-based
width/index arithmetic at the boundary. Indexed probes must keep `e ≤ 16383`.

### Variant C — Large module-scope 8-D variable with indexed field writes (recommended)
A module `var dst : [2][2][2][2][2][2][2][2]Pt` (1,048,576 bits, 32,768
elements) initialized from a call, then updated at specific indices in a bench
block and read back at multiple sites. Covers module-scope mutation + CSE while
staying under the 4-MiBit direct-simulation cliff.
