# FPGA Loop Closeout — Wave Loop 586

**Date:** 2026-07-07
**Branch:** `wave-loop-586`
**Issue:** #1557

## Goal

Exercise module-scope mutation on a multi-dimensional array-of-struct packed
register: write individual signed fields through indexed access, then read them
back at multiple sites. This closes the gap left by W585, which validated
read-only module-scope reuse of a call-returned 7-D AoS.

## What changed

- `specs/scratch/w586_bench_module_8d_aos_var_write.t27`
  - New deterministic witness:
    - `pub struct Pt { x : i16, y : i16 }`.
    - `pub var dst : [2][2][2][2][2][2][2][2]Pt = [2]^8 Pt{}`.
    - `bench` block writes four indexed signed fields and asserts the updated
      values, exercising both positive and negative `i16` assignments.
- `bootstrap/src/compiler.rs`
  - `expr_width_signed` and `field_scalar_array_info` now walk nested
    `ExprIndex` chains so multi-dimensional array-of-struct field accesses
    resolve their base variable correctly.
  - `emit_packed_struct_element_slice` and related scalar-struct field-access
    paths wrap signed packed slices with `$signed(...)` on reads.
  - Added `in_lvalue` codegen flag, set only while emitting the LHS of
    `StmtAssign`, so signed slices remain plain part-selects in assignment
    targets.
- `bootstrap/stage0/FROZEN_HASH`
  - Updated to `61637d927d4b07f415fbe72348bbdf244a26412860fc9f332d07b81a1e9a9a6f`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w586_bench_module_8d_aos_var_write`.
- `.trinity/seals/scratch_w586_bench_module_8d_aos_var_write.json`
  - New seal for the W586 witness.
- `.trinity/icarus-baselines/specs/scratch/w586_bench_module_8d_aos_var_write.json`
  - New Icarus baseline for the W586 witness.
- Resealed 30 affected scratch specs whose generated Verilog changed due to the
  signed packed-slice wrapping fix.

## What did not change

- `scripts/cocotb_ref_model.py` — unchanged.
- Yosys synthesis-smoke width-warning count remains 24 pre-existing failures.

## Scientific / engineering background

- **SystemVerilog packed slice semantics.** IEEE Std 1800-2017 §11.5.1 treats
  part-selects as unsigned bit ranges. A signed interpretation requires an
  explicit `$signed(...)` cast; without it, equality/relational operators become
  unsigned and negative literals compare incorrectly.
- **Lvalue vs. rvalue signedness.** The same packed-slice expression can appear
  on the left or right of an assignment. The bits are correct either way, but
  only the rvalue needs a signed cast; wrapping an assignment target with
  `$signed(...)` is invalid Verilog.
- **Verified compilation with mutable module state.** Sparkle (Lean 4) and
  Kami/Fe-Si (Coq) model hardware modules with explicit registers and prove
  refinement. The relevant invariant is the frame condition: a field write must
  update exactly the target slice while leaving all other bits unchanged. t27's
  packed-vector lowering preserves this because each indexed field assignment is
  a narrow part-select on the single backing register.

Sources:
- IEEE Std 1800-2017, §7.4 *Packed and unpacked arrays*, §11.5.1 *Vector
  bit-select and part-select addressing*.
  https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf
- Sparkle — formally verifiable HDL compiler in Lean 4.
  https://github.com/Verilean/sparkle
- Kami — modular hardware specification and verification in Coq.
  https://adam.chlipala.net/papers/KamiICFP17/KamiICFP17.pdf

## Verification matrix

| Check | Result |
|---|---|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 46 passed; 0 failed |
| `./scripts/tri test --fast` | 0 seal mismatches, 24 pre-existing yosys smoke failures |
| Direct `t27c icarus-simulate` W586 | PASS |
| Direct `t27c icarus-cocotb` W586 | PASS |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 73 Icarus PASS, 73 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke failures |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |

## Weak spot addressed

Signed `i16` field writes on a module-scope 8-D packed array-of-struct register
now lower correctly: the assignment writes the correct two's-complement bits,
and subsequent reads/comparisons interpret those bits as signed values.
Multi-dimensional index chains are resolved consistently for both code
generation and probe metadata.

## Risks accepted

- The witness uses the empty-array initializer `[2]^8 Pt{}` and only writes the
  fields it reads, so uninitialized elements are not probed. This keeps the
  file small (~1 kB) and focuses the wave on the signed write/read path.
- The 24 pre-existing Yosys synthesis-smoke failures are unchanged and unrelated
  to W586.

## Next wave cooperation variants (Wave Loop 587)

### Variant A — 18-D rank scaling
`[2]^18 Pt` (8,388,608 bits, 262,144 elements). Continue the rank-scaling ladder.
Risk: witness ~44 MB / ~2.4 M lines; direct simulation likely 40+ min. Use the
local-`expected` workaround and keep indexed probes at `e ≤ 16383`.

### Variant B — Non-power-of-two at rank 17
`[3][2]^17 Pt` (6,291,456 bits, 393,216 elements). Tests product-based
width/index arithmetic at the boundary, following the W569/W571 pattern.

### Variant C — Module-scope 8-D variable initialized from a call with indexed writes (recommended)
Combine the W585 call-return CSE path with the W586 indexed-field write path:
`pub var dst : [2]^8 Pt = make_oct(20)`, then write a few indexed fields and
read them back. This validates that the call temporary and the mutable packed
register coexist correctly and that the frame condition holds across the
initializer and subsequent writes.
