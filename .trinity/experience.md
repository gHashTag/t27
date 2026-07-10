## 2026-07-07 — Wave Loop 488 (gen-verilog backend hardening: wildcard array-of-struct aliases with array-typed fields; colon struct-literal prototype rolled back)

### What worked
- Extending the W487 anonymous AOS alias branch to handle array-typed element
  fields is a bounded, single-path change: emit a multi-dimensional `reg` with
  the outer struct index as the first dimension and copy every inner element in
  an `initial` block.
- Reusing `index_combinations` for the inner field dimensions keeps the copy
  loop consistent with existing local-array and module-array initialization.
- The W488 AOS alias witness (`w488_wildcard_aos_array_field_alias.t27) passes
  yosys and Icarus smoke without new `UNSUPPORTED_ICARUS` placeholders.
- Rolling back a parser broadening that surfaces regressions in existing specs
  is the correct conservative move; the colon struct-literal work is now a
  well-scoped W489 target with known failure modes.

### What changed behavior
- `bootstrap/src/compiler.rs`
  - `gen_verilog_const` AOS alias branch now emits multi-dimensional per-field
    memories and element-by-element copies for array-typed element fields.
- New witness spec:
  - `specs/scratch/w488_wildcard_aos_array_field_alias.t27`
- Global reseal of `.trinity/seals/*.json`, `bootstrap/stage0/FROZEN_HASH`, and
  `repro/numerics/nmse_manifest*.json` because `bootstrap/src/compiler.rs`
  changed.
- Added `docs/reports/WAVE_LOOP_488_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W489_2026-07-07.md`.
- Updated `.trinity/current-issue.md`, `docs/NOW.md`, `.trinity/experience.md`,
  and persistent memory.

### Verification
- `cargo build --release`: PASS.
- `cargo test -p t27c --bin t27c`: 1525 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: ALL TESTS PASSED
  - 673/673 non-smoke PASS.
  - 153/153 yosys smoke PASS.
  - 153/153 Icarus smoke PASS, 0 documented baseline failures.
  - 673/673 seal matches.
  - 0 fixed-point divergences.
  - FPGA board-less smoke gate: OK.
  - FPGA standalone lake-package build: OK.
  - FPGA smoke gate replay: OK.
- **Total `UNSUPPORTED_ICARUS` placeholders across all 673 specs: 0.**

### Patterns to reuse
- When extending wildcard lowering, emit anonymous uniquely named nodes and
  reuse existing per-field memory emission paths rather than inlining new code.
- For array-typed struct fields, preserve the source memory's dimension order:
  `[outer_count][inner_dims...]` so that direct indexing stays compatible with
  both yosys and Icarus.

### Anti-patterns to avoid
- Do not enable a parser change that exposes existing specs to backend paths
  that lack local-variable deduplication, keyword-name escaping, or array-typed
  packed-struct field handling.
- Do not emit test-block struct locals without a full hoisting/deduplication
  plan; half-emitted declarations leave the first field commented and the rest
  real, which breaks Icarus.

---

## 2026-07-07 — Wave Loop 487 (gen-verilog Icarus soft-failure hardening: module-scope wildcard struct-literal bindings, wildcard array aliases, 2-D/struct bench-local arrays crossing function boundaries)

### What worked
- Reusing the existing scalar-struct constant lowering path by re-emitting an
  anonymous node (`_wildcard_struct_{n}`) keeps the change small and avoids
  inventing a named `_` reg.
- Recording scalar-array dimensions in the const-array path lets module-scope
  wildcard aliases copy 2-D scalar memories element-by-element.
- Arrays of structs already publish their flattened fields and dimensions; a
  second wildcard alias lookup emits per-field anonymous memories and copies them
  linearly.
- Bench-local 2-D scalar arrays and arrays of structs cross function boundaries
  through the existing `__local__` packed-vector clone and element-width slicing
  inside the callee; the only missing piece was fresh witness specs.
- Non-synthesizable struct-literal leaves (`string`, `f32`, `bool`) must be
  emitted as width-correct placeholders (`'b0`, `'b1`/`'b0`) so the surrounding
  packed concatenation remains legal Verilog.
- A parser change that exposes too many existing specs to new backend paths should
  be rolled back and parked for a dedicated wave with full lowerer support.

### What changed behavior
- `bootstrap/src/compiler.rs`
  - `gen_verilog_const` wildcard struct-literal branch and AOS alias branch.
  - `gen_verilog_const` scalar-array dimension recording for 2-D const arrays.
  - Function declaration collection de-duplicates top-level function names.
  - `emit_struct_literal_leaf` handles `string`/`f32` zero placeholders and boolean
    literals.
- New witness specs:
  - `specs/scratch/w487_wildcard_module_literal.t27`
  - `specs/scratch/w487_wildcard_module_scalar_2d_alias.t27`
  - `specs/scratch/w487_wildcard_module_aos_alias.t27`
  - `specs/scratch/w487_bench_2d_array_param.t27`
  - `specs/scratch/w487_bench_aos_array_param.t27`
- Repaired existing witness:
  - `specs/scratch/w486_wildcard_module_literal.t27` (field separator and typed
    module-level array constant so the Verilog backend can lower the call).
- Global reseal of `.trinity/seals/*.json`, `bootstrap/stage0/FROZEN_HASH`, and
  `repro/numerics/nmse_manifest.json` because `bootstrap/src/compiler.rs`
  changed.
- Added `docs/reports/WAVE_LOOP_487_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W488_2026-07-07.md`.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, and memory.

### Verification
- `cargo build --release`: PASS.
- `cargo test -p t27c --bin t27c`: 1525 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: ALL TESTS PASSED
  - 672/672 non-smoke PASS.
  - 152/152 yosys smoke PASS.
  - 152/152 Icarus smoke PASS, 0 documented baseline failures.
  - 672/672 seal matches.
  - 0 fixed-point divergences.
  - FPGA board-less smoke gate: OK.
  - FPGA standalone lake-package build: OK.
  - FPGA smoke gate replay: OK.
- **Total `UNSUPPORTED_ICARUS` placeholders across all 673 specs: 0.**

### Patterns to reuse
- Anonymous re-emission is the safest way to extend wildcard lowering: create a
  uniquely named node and call the existing path rather than inlining new
  emission logic.
- Multi-dimensional scalar array lowering should flatten initialization with
  `flatten_array_literal_values` and `index_combinations` to avoid nested-loop
  bugs.
- When a parser broadening surfaces regressions in existing specs, revert it and
  keep the work scoped to specs that already parse correctly.

### Anti-patterns to avoid
- Do not emit `string` or `f32` values as Verilog based constants; they are not
  synthesizable.
- Do not emit boolean literals as `{width}'dtrue`; use `{width}'b1`.
- Do not rely on a global `emitted_functions` set for per-module de-duplication;
  collect a local `seen` set during declaration scanning.

---

## 2026-07-07 — Wave Loop 486 (gen-verilog Icarus soft-failure hardening: bench-local arrays crossing function boundaries, namespace helper erasure, module-scope wildcard array literals)

### What worked
- Carrying the containing bench name through the array-parameter binding pass
  lets bench-local arrays reuse the same `__local__` packed-vector clone that
  function-local arrays already used.
- Splitting `emitted_bench_names` into separate counter and initial-block sets
  fixed a long-standing bug where every bench `initial` block was skipped.
- Scalar packed-vector array parameters must be sliced by element width inside
  the callee; treating the packed input as an unpacked memory selects single
  bits and corrupts arithmetic.
- Namespace-qualified calls are cleanly erased when they are dead to
  synthesizable contexts; scanning const/var declarations as synthesizable
  contexts avoids misclassifying values used in module-level initializers.
- Module-scope wildcard array literals can be emitted as anonymous ROMs by
  reconstructing the array type from the literal and re-entering the normal
  const-array emission path with an anonymous name.

### What changed behavior
- `bootstrap/src/compiler.rs`
  - Bench-local array name pre-collection and bench-name tuple in the
    array-parameter binding pass.
  - Scalar packed-vector packing in `gen_verilog_pack_array_of_struct_expr`.
  - Packed-vector element-width slicing for scalar array parameters in
