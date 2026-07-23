# Wave Loop 587 Plan — module-scope 8-D AoS var + call init + writes

## Objective

Close Wave Loop 587 (issue #1558) with Variant C: a module-scope mutable
`[2]^8 Pt` variable initialized from a function call returning an 8-D
array-of-struct, plus indexed signed field writes in a bench block.

## Tasks

1. **Witness**
   - Create `specs/scratch/w587_bench_module_8d_aos_var_call_write.t27`.
   - Explicit `expected` const literal, `make_oct(offset)` return literal,
     module `var dst = make_oct(20)`.
   - `test` block for whole-array/indexed equality.
   - `bench` block with multi-site reads, signed writes, read-back, and
     frame-condition checks.

2. **Validation**
   - `t27c parse` succeeds.
   - `t27c gen-verilog-for-simulation` emits a valid 8-D packed concatenation.
   - `t27c icarus-simulate` passes.
   - `t27c icarus-cocotb` reference-model cross-check passes.

3. **Integration**
   - Add `accepts_w587_bench_module_8d_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.

4. **Seal / Baseline**
   - Create `.trinity/seals/scratch_w587_bench_module_8d_aos_var_call_write.json`.
   - Create
     `.trinity/icarus-baselines/specs/scratch/w587_bench_module_8d_aos_var_call_write.json`.

5. **Gates**
   - `cargo build --release -p t27c` green.
   - `cargo test -p t27c --bin t27c` 1494/0/2.
   - `cargo test -p tri` 78/0.
   - `cargo test -p t27c --test icarus_lowerable` 47/0 (including new test).
   - `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
     73/73 Icarus PASS / 73/73 cocotb PASS / 0 seal mismatches / 24 pre-existing
     yosys smoke baselines.

6. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W587_2026-07-07.md`.
   - Update `.trinity/experience.md`.
   - Save persistent memory `wave-loop-587.md`.

## Dependencies

- No compiler changes required; W586 signed packed-slice and CSE facilities
  already support this scenario.
- The only pitfall is literal syntax: no leading commas, balanced braces.

## Status

All tasks complete once the full tri pipeline finishes with zero new failures.
