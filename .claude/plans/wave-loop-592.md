# Plan — Wave Loop 592

**Issue:** #1563 — Module-scope `[3][2]^15 Pt` array-of-struct variable with non-power-of-two outer dimension, initialized from a function call, with indexed signed field writes.

**Branch:** `wave-loop-592`
**Previous:** `wave-loop-591` (#1562)

## Goal

Demonstrate that a module-scope `[3][2]^15 Pt` mutable packed `reg` (3,145,728 bits, 98,304 elements, non-power-of-two outer dimension 3) can be initialized from a function call and exercised with indexed signed field writes, without new compiler support.

## Background

- W589 fixed `gen_verilog_var`/`gen_verilog_const` to emit wholesale packed assignment for multi-D scalar-struct arrays initialized by `ExprCall`.
- W590/W591 validated whole-array reassignment of module-scope `[2]^17 Pt` from a second call and from a packed array literal, both at the 4-MiBit cliff.
- Agent E and the literature survey confirm that non-power-of-two packed dimensions are legal SystemVerilog and that t27's existing compiler paths (`parse_array_type`, `packed_width`, `emit_packed_array_literal_concat`, `try_emit_struct_array_access`) and the cocotb reference model are dimension-agnostic.
- Existing function-local non-p2 witnesses (`w569`, `w571`) already validate the same layout path.

## Chosen cooperation variant

**Variant B — `[3][2]^15 Pt` module-scope `var` initialized from a call, with indexed signed field writes and read-back.**

Why:
- Stays under the validated 4-MiBit cliff (3.1 MiBit vs. 4.2 MiBit), avoiding interactive time/memory risk.
- Exercises the first module-scope non-power-of-two outer dimension.
- Builds directly on W589/W591 paths; no compiler change expected.

## Plan

1. **Smoke test.** Verify that a small non-p2 module-scope witness (e.g. `[3][2]Pt`) works end-to-end with `./scripts/tri test`.
2. **Witness spec.** Write `specs/scratch/w592_bench_module_3x2p15_aos_var_call_write.t27`:
   - `pub struct Pt { x : i16, y : i16 }`
   - `pub fn make_grid(offset : u16) -> [3][2]^15 Pt` returning a 3,145,728-bit packed literal.
   - `pub const expected : [3][2]^15 Pt = make_grid(0);`
   - `pub var dst : [3][2]^15 Pt = make_grid(0);`
   - `test module_var_3x2p15_call_write`: initial state equals `expected` plus corner indexed reads.
   - `bench module_bench_3x2p15_call_write`: read before writes, signed indexed field writes, read-back, frame-condition checks.
3. **Leaf-value schedule.** Use `(2*e + offset) % 32768` to keep all `i16` leaf values in `[-32768, 32767]` for all 98,304 elements.
4. **Multi-line brace style.** Use W584 multi-line brace style for the 16-D literal to avoid single-line parser truncation.
5. **Integration test.** Add `accepts_w592_bench_module_3x2p15_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
6. **Seal and baseline.** Generate seal and Icarus baseline for the witness.
7. **Local verification.**
   - `cargo build --release -p t27c`
   - `cargo test -p t27c --bin t27c`
   - `cargo test -p tri`
   - `cargo test -p t27c --test icarus_lowerable`
   - `./scripts/tri test --fast`
8. **Full Icarus/cocotb tri pipeline.** `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`.
9. **Documentation.** Update `.trinity/current-issue.md`, `.trinity/experience.md`, write closeout report, and update persistent memory.

## Variants considered

- Variant A: `[2]^18 Pt` module var. 8,388,608-bit packed vector; crosses 4-MiBit cliff and likely exceeds interactive Icarus/Yosys budget. Deferred.
- Variant B: `[3][2]^15 Pt` module var. Chosen.
- Variant C: `[2]^17 Pt` conditional reassignment inside `if`. Useful follow-up but lower coverage than Variant B; deferred to W593 or later.

## Risk mitigations

- Keep leaf values inside signed i16 with modulo schedules.
- Use multi-line W584 brace style for the 16-D literal.
- If whole-array `$display` fails, use the local-`expected` workaround (bind literal to local variable before `assert_eq`).
- Avoid adding a second giant literal to keep file size and wall-clock manageable.
