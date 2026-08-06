# Plan — Wave Loop 594

**Issue:** #1565 — Module-scope `[7][2]^14 Pt` array-of-struct variable with a
non-power-of-two outer dimension, initialized from a function call, with
indexed signed field writes.

**Branch:** `wave-loop-594`
**Previous:** `wave-loop-593` (#1564)

## Goal

Demonstrate that a module-scope `[7][2]^14 Pt` mutable packed `reg`
(3,670,016 bits, 114,688 elements, non-power-of-two outer dimension 7) can be
initialized from a function call and exercised with indexed signed field
writes, without new compiler support.

## Background

- W593 validated a module-scope `[5][2]^15 Pt` (5,242,880-bit) mutable packed reg
  initialized from a function call, with indexed signed field writes, with zero
  compiler changes.
- W592 validated a module-scope `[3][2]^15 Pt` (3,145,728-bit) mutable packed reg
  with the same pattern.
- W589 fixed `gen_verilog_var`/`gen_verilog_const` to emit wholesale packed
  assignment for multi-D scalar-struct arrays initialized by `ExprCall`.
- Existing compiler paths (`parse_array_type`, `packed_width`,
  `emit_packed_array_literal_concat`, `try_emit_struct_array_access`) and the
  cocotb reference model multiply/stride by actual dimension sizes, not by
  power-of-two assumptions.

## Chosen cooperation variant

**Variant B — `[7][2]^14 Pt` module-scope `var` initialized from a call, with
indexed signed field writes and read-back.**

Why:
- Tests a larger non-power-of-two outer dimension (7) while staying
  comfortably under the validated 4-MiBit cliff (3,670,016 bits ≈ 3.5 MiBit).
- Builds directly on W589/W592/W593 paths; no compiler change expected.
- Lower risk than Variant A (`[2]^18`, 8 MiBit) and broader layout coverage
  than Variant C (conditional reassignment at 4 MiBit).

## Plan

1. **Witness spec.** Write
   `specs/scratch/w594_bench_module_7x2p14_aos_var_call_write.t27`:
   - `pub struct Pt { x : i16, y : i16 }`
   - `pub fn make_grid(offset : u16) -> [7][2]^14 Pt` returning a 3,670,016-bit
     packed literal.
   - `pub const expected : [7][2]^14 Pt = make_grid(0);`
   - `pub var dst : [7][2]^14 Pt = make_grid(0);`
   - `test module_var_7x2p14_call_write`: initial state equals `expected` plus
     corner indexed reads.
   - `bench module_bench_7x2p14_call_write`: read before writes, signed indexed
     field writes, read-back, frame-condition checks.
2. **Leaf-value schedule.** Use `(2*e + offset) % 32768` to keep all `i16` leaf
   values in `[-32768, 32767]` for all 114,688 elements.
3. **Multi-line brace style.** Use W584 multi-line brace style for the 15-D
   literal to avoid single-line parser truncation.
4. **Integration test.** Add
   `accepts_w594_bench_module_7x2p14_aos_var_call_write` to
   `bootstrap/tests/icarus_lowerable.rs`.
5. **Seal and baseline.** Generate seal and Icarus baseline for the witness.
6. **Local verification.**
   - `cargo build --release -p t27c`
   - `cargo test -p t27c --bin t27c`
   - `cargo test -p tri`
   - `cargo test -p t27c --test icarus_lowerable`
   - `./scripts/tri test --fast`
7. **Full Icarus/cocotb tri pipeline.**
   `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`.
8. **Documentation.** Update `.trinity/current-issue.md`,
   `.trinity/experience.md`, write closeout report, and update persistent
   memory.

## Variants considered

- Variant A: `[2]^18 Pt` module var. 8,388,608-bit packed vector; crosses the
  4-MiBit cliff and likely exceeds interactive Icarus/Yosys budget. Deferred.
- Variant B: `[7][2]^14 Pt` module var. Chosen.
- Variant C: `[2]^17 Pt` conditional reassignment inside `if`. Useful follow-up
  but lower coverage than Variant B; deferred to W595 or later.

## Risk mitigations

- Keep leaf values inside signed i16 with modulo schedules.
- Use multi-line W584 brace style for the 15-D literal.
- If whole-array `$display` fails, use the local-`expected` workaround.
- Avoid adding a second giant literal to keep file size and wall-clock
  manageable.
- Verify Yosys smoke still passes; t27 flattening avoids the unsupported
  “array of packed struct” construct.
