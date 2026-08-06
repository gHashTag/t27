# Plan — Wave Loop 593

**Issue:** #1564 — Module-scope `[5][2]^15 Pt` array-of-struct variable with non-power-of-two outer dimension, initialized from a function call, with indexed signed field writes.

**Branch:** `wave-loop-593`
**Previous:** `wave-loop-592` (#1563)

## Goal

Demonstrate that a module-scope `[5][2]^15 Pt` mutable packed `reg` (5,242,880 bits, 163,840 elements, non-power-of-two outer dimension 5) can be initialized from a function call and exercised with indexed signed field writes, without new compiler support.

## Background

- W592 validated a module-scope `[3][2]^15 Pt` (3,145,728-bit) mutable packed reg
  initialized from a function call, with indexed signed field writes, with zero
  compiler changes.
- W589 fixed `gen_verilog_var`/`gen_verilog_const` to emit wholesale packed
  assignment for multi-D scalar-struct arrays initialized by `ExprCall`.
- Existing compiler paths (`parse_array_type`, `packed_width`,
  `emit_packed_array_literal_concat`, `try_emit_struct_array_access`) and the
  cocotb reference model multiply/stride by actual dimension sizes, not by
  power-of-two assumptions.

## Chosen cooperation variant

**Variant B — `[5][2]^15 Pt` module-scope `var` initialized from a call, with indexed signed field writes and read-back.**

Why:
- Reaches 5,242,880 bits (≈5.0 MiBit), slightly past the 4-MiBit cliff that
  W591/W592 validated, while testing a non-power-of-two outer dimension.
- Exercises a larger module-scope non-power-of-two outer dimension (5).
- Builds directly on W589/W592 paths; no compiler change expected.

## Plan

1. **Witness spec.** Write `specs/scratch/w593_bench_module_5x2p15_aos_var_call_write.t27`:
   - `pub struct Pt { x : i16, y : i16 }`
   - `pub fn make_grid(offset : u16) -> [5][2]^15 Pt` returning a 5,242,880-bit packed literal.
   - `pub const expected : [5][2]^15 Pt = make_grid(0);`
   - `pub var dst : [5][2]^15 Pt = make_grid(0);`
   - `test module_var_5x2p15_call_write`: initial state equals `expected` plus corner indexed reads.
   - `bench module_bench_5x2p15_call_write`: read before writes, signed indexed field writes, read-back, frame-condition checks.
2. **Leaf-value schedule.** Use `(2*e + offset) % 32768` to keep all `i16` leaf values in `[-32768, 32767]` for all 163,840 elements.
3. **Multi-line brace style.** Use W584 multi-line brace style for the 16-D literal to avoid single-line parser truncation.
4. **Integration test.** Add `accepts_w593_bench_module_5x2p15_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
5. **Seal and baseline.** Generate seal and Icarus baseline for the witness.
6. **Local verification.**
   - `cargo build --release -p t27c`
   - `cargo test -p t27c --bin t27c`
   - `cargo test -p tri`
   - `cargo test -p t27c --test icarus_lowerable`
   - `./scripts/tri test --fast`
7. **Full Icarus/cocotb tri pipeline.** `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`.
8. **Documentation.** Update `.trinity/current-issue.md`, `.trinity/experience.md`, write closeout report, and update persistent memory.

## Variants considered

- Variant A: `[2]^18 Pt` module var. 8,388,608-bit packed vector; crosses 4-MiBit cliff and likely exceeds interactive Icarus/Yosys budget. Deferred.
- Variant B: `[5][2]^15 Pt` module var. Chosen.
- Variant C: `[2]^17 Pt` conditional reassignment inside `if`. Useful follow-up but lower coverage than Variant B; deferred to W594 or later.

## Risk mitigations

- Keep leaf values inside signed i16 with modulo schedules.
- Use multi-line W584 brace style for the 16-D literal.
- If whole-array `$display` fails, use the local-`expected` workaround.
- Avoid adding a second giant literal to keep file size and wall-clock manageable.
