# Wave Loop 487 Close-Out Report

**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Branch:** `wave-loop-487`  
**Issue:** #1457  
**Direction:** Variant B — IGLA/bench Verilog lowering hardening

---

## 1. What was executed

Wave Loop 487 continued the Icarus/Verilog backend hardening started in W486.
The focus was three remaining lowering gaps:

1. Module-scope wildcard `_` bindings with struct-literal initializers.
2. Module-scope wildcard aliases to existing scalar and array-of-struct memories.
3. Bench-local 2-D scalar arrays and arrays of structs crossing function
   boundaries as packed-vector parameters.

A short exploratory attempt to add colon-style struct-literal field separators
(`field: value`) was rolled back after it surfaced regressions across the large
body of existing `igla/` specs that already use colon in struct field declarations
but not in literals. The witness specs therefore keep the existing `=` separator,
which does not reduce expressiveness for the targeted lowering work.

### 1.1 Delivered changes

- **`bootstrap/src/compiler.rs`**
  - `gen_verilog_const`: added wildcard struct-literal branch that re-emits an
    anonymous scalar-struct constant, reusing the existing per-field register
    lowering path.
  - `gen_verilog_const`: recorded scalar-array dimensions for 2-D const arrays
    and added an AOS wildcard-alias branch that emits per-field anonymous memories
    copied element-by-element.
  - Function declaration collection now de-duplicates top-level function names,
    keeping the first declaration, because re-emitting a Verilog function name is
    illegal.
  - `emit_struct_literal_leaf`: emits width-correct zero placeholders for
    non-synthesizable leaf types (`string`, `f32`) and `1'b1`/`1'b0` for boolean
    literals, fixing previously latent syntax errors in yosys/Icarus.

- **Witness specs** under `specs/scratch/`
  - `w487_wildcard_module_literal.t27`
  - `w487_wildcard_module_scalar_2d_alias.t27`
  - `w487_wildcard_module_aos_alias.t27`
  - `w487_bench_2d_array_param.t27`
  - `w487_bench_aos_array_param.t27`

- **Regression fix in an existing witness**
  - `specs/scratch/w486_wildcard_module_literal.t27`: converted struct-literal
    field separators from `:` to `=` and replaced the inline array-literal
    argument (which the Verilog backend cannot lower) with a typed module-level
    array constant. This removes the only Icarus simulation failure that the
    W487 changes would otherwise have introduced.

- **NMSE reseal**
  - `bootstrap/stage0/FROZEN_HASH`
  - `repro/numerics/nmse_manifest.json`
  - `repro/numerics/nmse_manifest_protocol_v1.json`

- **Seals**
  - Regenerated all 62 `.trinity/seals/*.json` files affected by compiler or
    spec changes, plus the 5 new witness seal files.

---

## 2. Verification results

Run from repo root on branch `wave-loop-487`:

```bash
cd bootstrap && cargo build --release
cd .. && cargo test -p t27c --bin t27c
./scripts/tri test
```

Results:

| Gate | Result |
|------|--------|
| `cargo build --release` | OK |
| `cargo test -p t27c --bin t27c` | **1525 passed; 0 failed; 2 ignored** |
| Non-smoke tests | **672 / 672 PASS** |
| Gen Verilog yosys smoke | **152 / 152 PASS; 0 failures** |
| Gen Verilog Icarus smoke | **152 / 152 PASS; 0 documented baseline failures** |
| Seal verify | **672 / 672 MATCH** |
| Fixed-point divergences | **0** |
| `UNSUPPORTED_ICARUS` placeholders | **0 across all 672 specs** |
| FPGA board-less smoke gate | OK |

`./scripts/tri test` final output:

```text
ALL TESTS PASSED
phi^2 + 1/phi^2 = 3 | TRINITY
```

---

## 3. Decisions and deviations

- **Colon-style struct-literal parser change was reverted.** An initial attempt
  to accept `field: value` in `parse_struct_literal` enabled too many existing
  `igla/` specs to parse into full function bodies that the Verilog backend is
  not yet ready to lower cleanly (enum namespace references, string/float
  struct fields, duplicate local declarations, unsupported array-literal call
  arguments). Rather than expand W487 into an unbounded backend cleanup, the
  parser was restored to the existing `=` separator and all witness specs were
  written with `=`. A future wave can revisit colon syntax once the corresponding
  lowering paths are ready.

- **W486 witness updated.** `w486_wildcard_module_literal.t27` used colon
  struct literals and an inline array-literal function argument. The close-out
  of W487 repaired both so the spec stays green after the new parser/lowerer
  work.

---

## 4. Files changed

```
bootstrap/src/compiler.rs
bootstrap/stage0/FROZEN_HASH
repro/numerics/nmse_manifest.json
repro/numerics/nmse_manifest_protocol_v1.json
specs/scratch/w486_wildcard_module_literal.t27
specs/scratch/w487_bench_2d_array_param.t27      (new)
specs/scratch/w487_bench_aos_array_param.t27    (new)
specs/scratch/w487_wildcard_module_aos_alias.t27 (new)
specs/scratch/w487_wildcard_module_literal.t27   (new)
specs/scratch/w487_wildcard_module_scalar_2d_alias.t27 (new)
.trinity/seals/*                                 (62 updated + 5 new)
.claude/plans/wave-loop-487.md
```

---

## 5. Next step

Select one of the three W488 cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W488_2026-07-07.md`, create branch
`wave-loop-488`, and open issue #1458.
