# FPGA Loop Closeout — Wave Loop 583

**Date:** 2026-07-07  
**Branch:** `wave-loop-583`  
**Issue:** #1554  
**Branch head:** `wave-loop-583`  
**FROZEN_HASH:** `8db163435fb06702b62c266e951da7e92ae151cfc4db7a8e7870a7ff4f460c02`

## Goal

Close the module-scope 2-D/3-D array-of-struct boundary gap by creating a
3-D module-level constant `[2][2][2]Pt` and proving it can be compared
whole-array against a function-returned 3-D AoS whose fields are computed
expressions (`offset + N`).

## What changed

- `bootstrap/src/compiler.rs`
  - `emit_packed_scalar_value`: non-literal expressions inside packed
    concatenations are now emitted as `width'(expr)` (unsigned) or
    `$signed(width'(expr))` (signed). This eliminates Icarus 12.0
    "Concatenation operand ... has indefinite width" errors when struct/array
    literals contain arithmetic expressions.
- `bootstrap/stage0/FROZEN_HASH`
  - Updated to SHA-256 of the changed `bootstrap/src/compiler.rs`.
- `specs/scratch/w583_bench_module_3d_aos_call_dedup.t27`
  - New deterministic witness:
    - `pub const expected : [2][2][2]Pt` module-level packed parameter.
    - `make_cube(offset : u16) -> [2][2][2]Pt` returns a 3-D AoS with computed
      fields.
    - `test` block directly compares `make_cube(0)` to `expected`.
    - `bench` block stores the call result in a local `actual` and asserts
      `assert_eq(actual, expected)` for VCD cross-check.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w583_bench_module_3d_aos_call_dedup`.
- Resealed 71 affected specs whose generated Verilog changed due to the
  width-cast fix.
- Added Icarus baseline
  `.trinity/icarus-baselines/specs/scratch/w583_bench_module_3d_aos_call_dedup.json`.

## What did not change

- No predicate or proof changes in `proofs/lean4/Trinity.IcarusLowerable`;
  only generated expression syntax changed.
- No change to Zig, Rust, or C generation.
- Yosys synthesis-smoke width-warning count remains 24 pre-existing
  failures.

## Scientific / engineering background

- IEEE Std 1800-2017 §7.4.1 guarantees at least 65,536-bit packed arrays.
  t27's rank-agnostic lowering already exceeds this; W583 shifted focus from
  width scaling to scope coverage.
- Icarus Verilog 12.0 accepts SystemVerilog `width'(expr)` casts in
  `-g2012` mode, which is the same mode required for packed-array support.
- Yosys 0.63 synthesizes `width'(expr)` without errors (verified on a minimal
  reproduction).
- CIRCT `HWLegalizeModules` recursively peels one packed-array dimension at a
  time; t27's expression-level width cast is a complementary frontend fix that
  keeps the emitted Verilog self-contained.

## Verification matrix

| Check | Result |
|---|---|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 43 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke failures |
| Direct `t27c icarus-simulate` on W583 witness | PASS |
| Direct `t27c icarus-cocotb` on W583 witness | PASS |

## Weak spot addressed

Function-local/call-return AoS had been scaled to 16-D, but module-scope
3-D AoS with computed-field initializers failed Icarus elaboration. The root
cause was that non-literal scalar expressions inside packed concatenations
lacked an explicit width context. The W583 fix closes this gap with a
minimal, tool-compatible SystemVerilog cast.

## Risks accepted

- Resealing 71 specs is a large surface-area change, but every affected seal
  was recomputed deterministically by `t27c seal --save` and verified by the
  second `./scripts/tri test` run.
- `width'(expr)` is SystemVerilog-2012 syntax; t27 already requires `-g2012`
  for packed arrays, so no new host dependency is introduced.

## Next wave cooperation variants (Wave Loop 584)

### Variant A — Extreme rank scaling (recommended if CI budget allows)
17-D `[2]^17 Pt` (4,194,304 bits, 131,072 elements). Validates whether the
function-local/call-return path accepts a 4-MiBit vector and whether the
Icarus local-`expected` workaround scales. Risk: witness ~22 MB / ~1.2 M
lines; direct simulation may approach 8–10 min.

### Variant B — Non-power-of-two at the boundary
16-D with outer dimension 3: `[3][2]^16 Pt` (3,932,160 bits, 196,608
elements). Tests non-power-of-two outer dimension at the edge of practical
file size. Witness ~34 MB / ~1.8 M lines.

### Variant C — Module-scope multi-site / variable initialization
Extend W583 to a module-level 3-D AoS **variable** initialized from a
function-call return, or multiple call sites reading the module const, to
exercise CSE/deduplication across module-scope and function-return
boundaries. Keeps file size small while probing W557 call-array CSE
interaction with module-level constants.
