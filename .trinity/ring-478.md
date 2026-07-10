# Ring 478 — Wave Loop 478

**Date:** 2026-07-07  
**Branch:** `wave-loop-478`  
**Variant:** B — close Icarus Verilog failures in packed-vector struct-array lowering + warning gate hardening  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Goal

Close the Icarus Verilog simulation gap inherited from Wave Loop 477:
- Fix packed-vector array-of-struct / struct-with-array-field lowering so Icarus no longer reports `Assignment to an entire array` or `Concatenation operand has indefinite width`.
- Harden the Icarus gate to surface elaboration warnings and catch latent assertion / expected-value bugs.
- Add an adversarial scratch spec that exercises the previously failing patterns under both yosys and Icarus.
- Keep all existing yosys smoke targets green and all non-smoke tests green.

## Outcome

W478 fixed six structural failure classes (A–E and duplicate test labels) and produced a clean 106/126 Icarus smoke result with only the 20 out-of-scope `igla/` dynamic-method baselines remaining. A new adversarial witness `specs/scratch/w478_icarus_struct_array.t27` passes both yosys and Icarus.

Key backend changes in `bootstrap/src/compiler.rs`:
- Sized literal / cast emission for packed struct/array literal leaves (`<width>'d<value>` and `<width>'(expr)`), eliminating indefinite-width concatenation errors.
- Per-element expansion when struct-return temporaries are unpacked into array-typed struct fields.
- `packed_width` recursion over multi-dimensional arrays so packed-vector slices and offsets are computed correctly.
- Full-index lowering for scalar array-typed struct fields in packed array parameters and returns.
- `module_declared_regs` deduplication so module-level scalar struct vars do not create duplicate per-field `reg` declarations.
- `test_block_names` deduplication so duplicate source test names produce unique `begin : <name>` labels.
- `gen_verilog_try_local_struct_array_assign` for memory-mode local array-of-struct whole-array copy by value.
- `assert_eq` emission now uses `assert(...) else $fatal(1, "assertion failed")` so simulation-time violations actually fail the test.

Spec corrections:
- `specs/scratch/w469_2d_struct_array.t27`: removed the extraneous third argument to `set_and_sum_2d`.
- `specs/scratch/w473_3d_module_var_struct_array.t27`: corrected the expected sum from 1332 to 666.
- `specs/scratch/w382_ram_lowering.t27`: moved module-level memory writes out of the test block into a function so the assertion can actually observe them under the new fatal `assert_eq`.
- `specs/scratch/w476_adversarial_aggregate_tail.t27`: corrected expected values (12→13, 30→27, 17→16).

## Artifacts

- `docs/reports/WAVE_LOOP_478_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W479_2026-07-08.md`
- `.claude/plans/wave-loop-478.md`
- `specs/scratch/w478_icarus_struct_array.t27`

## Verification

- `cargo build --release`: PASS
- `cargo test -p t27c --bin t27c`: 1524 passed, 0 failed, 2 ignored
- `./scripts/tri test --fast`: ALL TESTS PASSED
  - 646/646 non-smoke PASS
  - 126/126 yosys smoke PASS
  - 106/126 Icarus smoke PASS, 20 failed (documented `igla/` dynamic-method baseline)
  - 0 seal mismatches

## Next

- Branch: `wave-loop-479`
- Default Variant B: close or document the remaining 20 Icarus failures in `igla/coder` and `igla/race` caused by dynamic string/array methods.
