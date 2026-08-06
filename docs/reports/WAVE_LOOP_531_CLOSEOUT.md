# Wave Loop 531 Closeout — Extend Icarus simulation regression suite to primitive arrays

**Date:** 2026-07-07  
**Issue:** #1502  
**Branch:** `wave-loop-531`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was planned

Pick up the recommended Variant A from
`docs/reports/FPGA_LOOP_COOPERATION_W531_2026-07-07.md`:

1. Add any new W531 lowerable scratch specs to the Icarus simulation regression
   suite in `./scripts/tri test --icarus-simulate`.
2. Record JSON baselines under `.trinity/icarus-baselines/` for the new
   witnesses.
3. Refine the classifier / lowering so only lowerable specs enter the gate and
   primitive arrays simulate correctly.
4. Maintain 0 Icarus simulation failures and keep the 16 documented yosys smoke
   baselines flat.

---

## What was implemented

### 1. Lowered function-local primitive arrays as unpacked Verilog arrays

The legacy scalar-reg fallback emitted `reg [N-1:0] arr;` and treated
`arr[i]` as a 1-bit bit-select. This broke both signed values (e.g. `i32`
arrays) and variable-index writes (Icarus rejects packed part-selects as
l-values).

`bootstrap/src/compiler.rs` now detects 1-D and multi-D arrays of primitive
scalars in `StmtLocal` and emits unpacked arrays:

```verilog
reg signed [15:0] temps [0:3];
```

Element access is emitted as `temps[i]` so signed widths, 2-D indexing, and
variable indices all work with Icarus Verilog.

### 2. Lowered module-level primitive arrays as unpacked Verilog arrays

The same scalar-reg-per-element fallback was used for module-level `var`
declarations (e.g. `var mem : [4]u16;`). W382 exposed that this produced
1-bit selects for whole-word reads and writes.

`gen_verilog_var` now also emits unpacked arrays for primitive array types and
initializes them with the same `emit_unpacked_primitive_array_init` helper used
for function-local declarations. Module-level reads/writes inside functions are
routed through `try_emit_primitive_array_access`, which resolves the array type
from `module_types`.

### 3. Extended the Icarus regression whitelist

`bootstrap/src/suite.rs::icarus_regression_specs` now includes both the original
W493–W529 witnesses (`w5*`) and the W3xx primitive-array witnesses (`w3*`).
The existing `--icarus-lowerable` classifier still filters out non-lowerable
specs before simulation, so the gate remains scoped.

### 4. Baselines and seals

- Recorded new/updated Icarus JSON baselines for the lowerable W3xx witnesses.
- Resealed specs whose generated Verilog changed due to the unpacked-array
  lowering:
  - `specs/scratch/w382_ram_lowering.t27`
  - `specs/scratch/w383_rom_array.t27`
  - `specs/scratch/w384_variable_index.t27`
  - `specs/scratch/w385_*_local_array*.t27`
  - `specs/scratch/w386_for_local_array*.t27`
  - `specs/scratch/w387_2d_local_array*.t27`
  - `specs/scratch/w388_2d_local_array_init.t27`
  - `specs/queen/lotus.t27`
  - `specs/fpga/testbench/top_tb.t27`
  - and the other specs whose `gen_hash_verilog` changed in this wave.
- Updated `bootstrap/stage0/FROZEN_HASH` after the compiler changes.

---

## Verification

```
cargo build --release -p t27c          # green
cargo test -p t27c --bin t27c          # 1494 passed; 0 failed; 2 ignored
cargo test -p tri                      # 78 passed; 0 failed
./scripts/tri test --icarus-simulate --icarus-lowerable
./scripts/tri test --icarus-lowerable --fast
```

Final suite summary (`--icarus-lowerable --fast`):

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

Icarus Simulation: **24 passed, 0 failed** (up from 10 in W530).

---

## Residual boundaries

- The 16 pre-existing yosys smoke failures remain; they are unrelated to the
  Icarus lowerable subset and are documented separately.
- Signed scalar-array struct fields (Variant B) and adversarial non-lowerability
  proofs (Variant C) are deferred to Wave Loop 532.

---

## Learnings

- Unpacked Verilog arrays (`reg [W-1:0] arr [0:N-1];`) are the correct lowering
  for primitive t27 arrays: they preserve signed element widths and allow
  variable indices as both r-values and l-values.
- The same broken fallback existed in two places (function-local `StmtLocal` and
  module-level `gen_verilog_var`); fixing only one left module-level RAM
  witnesses broken. Always check both declaration sites when changing array
  lowering.
- Extending a regression whitelist is safe when a lowerability classifier
  filters the list before the gate runs; the gate can grow without adding noise.

---

*φ² + φ⁻² = 3 | TRINITY*
