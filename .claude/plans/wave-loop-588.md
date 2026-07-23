# Wave Loop 588 Plan — module-scope 9-D AoS var + call init + writes

## Objective

Close Wave Loop 588 (issue #1559) with Variant C: a module-scope mutable
`[2]^9 Pt` variable initialized from a function call returning a 9-D
array-of-struct, plus indexed signed field writes.

## Tasks

1. **Witness**
   - Create `specs/scratch/w588_bench_module_9d_aos_var_call_write.t27`.
   - Explicit `expected` const literal, `make_non(offset)` return literal,
     module `var dst = make_non(20)`.
   - `test` block for whole-array/indexed equality.
   - `bench` block with multi-site reads, signed writes, read-back, and
     frame-condition checks.

2. **Validation**
   - `t27c parse` succeeds.
   - `t27c gen-verilog-for-simulation` emits a valid 9-D packed concatenation.
   - `t27c icarus-simulate` passes.
   - `t27c icarus-cocotb` reference-model cross-check passes.

3. **Integration**
   - Add `accepts_w588_bench_module_9d_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.

4. **Seal / Baseline**
   - Create `.trinity/seals/scratch_w588_bench_module_9d_aos_var_call_write.json`.
   - Create
     `.trinity/icarus-baselines/specs/scratch/w588_bench_module_9d_aos_var_call_write.json`.

5. **Gates**
   - `cargo build --release -p t27c` green.
   - `cargo test -p t27c --bin t27c` 1494/0/2.
   - `cargo test -p tri` 78/0.
   - `cargo test -p t27c --test icarus_lowerable` 48/0 (including new test).
   - `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
     75/75 Icarus PASS / 75/75 cocotb PASS / 0 seal mismatches / 24 pre-existing
     yosys smoke baselines.

6. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W588_2026-07-07.md`.
   - Update `.trinity/experience.md`.
   - Save persistent memory `wave-loop-588.md`.

## Dependencies

- No compiler changes required; W586 signed packed-slice and W587 CSE facilities
  already support this scenario.
- The 9-D literal is 1023 braces/brackets; validate balance programmatically.

## Status

All tasks complete; changes merged to `main`.
