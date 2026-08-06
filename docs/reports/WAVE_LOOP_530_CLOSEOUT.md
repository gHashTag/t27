# Wave Loop 530 Closeout — Icarus Verilog simulation gate

**Date:** 2026-07-07  
**Issue:** #1501  
**Branch:** `wave-loop-530`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was planned

Pick up the recommended Variant A from
`docs/reports/FPGA_LOOP_COOPERATION_W530_2026-07-07.md`:

1. Add `--icarus-lowerable` / `--icarus-simulate` flags to `./scripts/tri test`.
2. For every lowerable spec, generate Verilog, compile with `iverilog`, run with
   `vvp`, and capture `$display` output.
3. Add JSON baselines under `.trinity/icarus-baselines/`.
4. Promote the W493–W529 lowerable scratch witnesses into the first simulation
   regression suite.
5. Keep the 16 pre-existing yosys smoke failures as documented baselines.

---

## What was implemented

### 1. Fixed a latent 2-D packed-vector layout bug

`t27` element `[0][0]` must occupy the LSB of the packed Verilog vector, but
`emit_packed_array_literal_concat_level` was emitting `{e0, e1, ...}`. Verilog
concatenation is MSB-first, so the first t27 element ended up at the MSB.
`bootstrap/src/compiler.rs` now reverses the parts before writing the
concatenation so the t27 index order matches the Verilog bit order used by the
packed slice accessors.

### 2. CLI simulation command

- `VerilogCodegen` gained an `emit_test_assertions` option.
- `Compiler::compile_verilog_for_simulation` emits active `assert_eq` checks
  inside `// synthesis translate_off` regions.
- `t27c icarus-simulate <file.t27>` generates simulation Verilog, runs
  `iverilog -g2012`, runs `vvp`, and reports `[TEST] name : PASSED/FAILED`.
- Zero-argument t27 calls now pass `1'b0` for the dummy `_unused` input so Icarus
  accepts them.

### 3. `tri test` integration

- `./scripts/tri test --icarus-simulate --icarus-lowerable` now runs Phase 3d.
- Phase 3d targets the W493–W529 regression specs (`specs/scratch/w5*.t27`).
- Each spec is classified as lowerable by compiling its generated Verilog with
  `iverilog -g2012 -o /dev/null`; non-lowerable specs are skipped.
- Simulation output is compared against a JSON baseline stored under
  `.trinity/icarus-baselines/`. Missing baselines are recorded on the first
  successful run.

### 4. Baselines and seals

- Added/updated 10 Icarus simulation baselines for W526/W528/W529 witnesses.
- Resealed 125 specs whose `gen_hash_verilog` changed after the packed-vector
  layout fix.

---

## Verification

```
cargo build --release -p t27c          # green
cargo test -p t27c --bin t27c          # 1494 passed; 0 failed; 2 ignored
cargo test -p tri                      # 78 passed; 0 failed
./scripts/tri test --icarus-simulate --icarus-lowerable
```

Final suite summary:

```
Parse failures:           0
Typecheck fails:          0
GF16 conformance:         0
Gen Zig failures:         0
Gen Rust failures:        0
Gen Verilog fails:        0
Gen Verilog smoke fails:  16   # pre-existing documented baselines
FPGA smoke fails:         0
Icarus simulation fails:  0
Gen C failures:           0
Seal mismatches:          0
FP divergences:           0
TOTAL FAILURES:    16
```

Icarus Simulation: **10 passed, 0 failed**.

---

## Residual boundaries

- The 16 pre-existing yosys smoke failures remain; they are unrelated to the
  Icarus lowerable subset and are documented separately.
- Variant B/C boundaries (signed scalar-array struct fields, adversarial
  non-lowerability proofs) are deferred to Wave Loop 531.

---

## Learnings

- The first Icarus simulation run immediately exposed a real semantic bug that
  the yosys syntax-only smoke gate had missed. This validates Variant A's core
  premise: a simulation gate catches value-level regressions that static
  classification cannot.
- Verilog packed-vector layout must be reversed at every concatenation level to
  keep t27 index `[0]` at the LSB.
- Baselines should be scoped to a deliberate regression whitelist (W493–W529) so
  unrelated scratch experiments do not destabilize the gate.

---

*φ² + φ⁻² = 3 | TRINITY*
