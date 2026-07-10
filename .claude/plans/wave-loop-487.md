# Wave Loop 487 Implementation Plan

## Direction summary

Variant B is the default: continue hardening the Icarus/Verilog backend for the
three remaining lowering gaps after W486:

- module-scope wildcard struct-literal bindings (`let _ = Pt{...};`),
- module-scope wildcard array aliases (`let _ = existing_array;`),
- 2-D / struct bench-local arrays crossing function boundaries.

The work is bounded to anonymous emission for module-scope wildcard
struct-literal bindings, anonymous copy-memory emission for module-scope wildcard
array aliases, packed-vector handling for bench-local 2-D and array-of-struct
parameters, and small correctness fixes discovered while validating the new paths.
The zero-`UNSUPPORTED_ICARUS` contract from W486 must be preserved.

**Note on syntax scope:** the existing t27 struct-literal field separator is
`=` (e.g. `Pt { x = 1, y = 2 }`). We keep that separator; the wildcard binding
work does not require a new parser change and therefore avoids destabilizing the
large body of existing specs.

## Subtasks

### 1. `gen_verilog_const`: emit anonymous scalar-struct registers for wildcard struct literals

`let _ = Pt{...};` parses as a `ConstDecl` named `_` with an `ExprStructLit`
child. `gen_verilog_const` previously only handled `ExprArrayLiteral` and
`ExprIdentifier` wildcards; `ExprStructLit` fell through to a discard comment.

Changes:

- In the wildcard guard inside `gen_verilog_const`, add a branch for
  `NodeKind::ExprStructLit` before the unsupported fallback.
- Build an anonymous node:
  - name = `_wildcard_struct_{counter}`
  - `extra_type = init.name` (the struct type from the literal)
  - children = the struct-literal initializer
- Increment the counter and recursively call `self.gen_verilog_const(&anon_node)`
  so the existing scalar-struct constant lowering path emits one `reg` per
  flattened leaf field and an `initial` block that initializes them.
- No named `_` identifier must appear in the generated Verilog.

Acceptance criteria:

- New witness `/Users/playra/t27/specs/scratch/w487_wildcard_module_literal.t27`
  contains:
  ```t27
  struct Pt { x: u32, y: u32 }
  let _ = Pt { x = 1, y = 2 };
  test wildcard_struct_literal_pass { assert_eq(1, 1); }
  ```
- `t27c gen-verilog` for this spec emits `_wildcard_struct_0_x` and
  `_wildcard_struct_0_y` registers, no `reg _`, and passes yosys and Icarus smoke.

### 2. `gen_verilog_const`: extend wildcard array aliases to scalar multi-dimensional arrays and arrays of structs

`let _ = src;` only emitted an anonymous copy for 1-D scalar arrays that were
recorded in `module_scalar_array_dims`. Two gaps remained:

- `const src : [2][3]u32 = ...` was not recorded in `module_scalar_array_dims`
  by the const array path.
- Arrays of structs were recorded in `module_struct_array_dims` /
  `module_struct_array_fields` but the wildcard alias branch did not look them
  up.

Changes:

- In the const scalar-array path of `gen_verilog_const`, after emitting the
  source memory, insert `module_scalar_array_dims.insert(...)` so later aliases
  can resolve it. Multi-dimensional scalar arrays emit an unpacked memory with
  the full dimensions and are initialized with nested `initial` loops via
  `flatten_array_literal_values` and `index_combinations`.
- In the wildcard `ExprIdentifier` branch, add a second lookup against
  `module_struct_array_dims` and `module_struct_array_fields`. When the source
  is an array of structs, emit one anonymous per-field memory per flattened scalar
  leaf and copy element-by-element in an `initial` block.
- If an alias shape is too risky to lower correctly, fall back to a comment-only
  alias that names the source and the reason.

Acceptance criteria:

- New witnesses:
  - `/Users/playra/t27/specs/scratch/w487_wildcard_module_scalar_2d_alias.t27`
  - `/Users/playra/t27/specs/scratch/w487_wildcard_module_aos_alias.t27`
- Both produce valid Verilog, emit no identifier named `_`, pass yosys and Icarus
  smoke with zero `UNSUPPORTED_ICARUS` placeholders.

### 3. Bench-local 2-D / array-of-struct array parameters

The array-parameter binding pass already marks bench-local arrays as
`__local__` when the local declaration has an explicit array type annotation.
The packed-vector call-site and callee paths are exercised for 2-D scalar and
AOS cases.

Acceptance criteria:

- New witnesses:
  - `/Users/playra/t27/specs/scratch/w487_bench_2d_array_param.t27`
  - `/Users/playra/t27/specs/scratch/w487_bench_aos_array_param.t27`
- Generated Verilog shows packed-vector function inputs, packed-vector call-site
  concatenation, and correct slicing for both literal and variable indices.
- Both witnesses pass yosys and Icarus smoke with zero `UNSUPPORTED_ICARUS`
  placeholders.

### 4. Correctness fixes exposed by the new paths

While validating the above changes, two latent issues were fixed:

- `emit_struct_literal_leaf` now emits width-correct zero placeholders for
  non-synthesizable leaf types (`string`, `f32`) and boolean literals (`true`
  → `{width}'b1`, `false` → `{width}'b0`). Previously string/float values were
  emitted as invalid based constants and boolean literals as `1'dtrue`, both of
  which break yosys/Icarus parsing.
- Duplicate top-level function names in the same module are de-duplicated during
  declaration collection, keeping the first declaration. Re-emitting the same
  Verilog function name is illegal and surfaced once previously skipped
  struct-return functions became visible.

### 5. Verification and reseal

Commands and pass criteria:

- `cargo build --release`: must succeed.
- `cargo test -p t27c --bin t27c`: must report 1525 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: must report:
  - 672 / 672 non-smoke PASS.
  - 152 / 152 yosys smoke PASS, 0 failures.
  - 152 / 152 Icarus smoke PASS, 0 documented baseline failures.
  - 672 / 672 seal matches.
  - Total `UNSUPPORTED_ICARUS` placeholders across all 672 specs: 0.
- Because `bootstrap/src/compiler.rs` is modified, run:
  ```bash
  RESEAL_YES=1 ./scripts/reseal-apply.sh
  ```
  and confirm the NMSE seal refresh is deterministic and matches the post-change
  output.

## Risk notes and rollback strategy

- **AOS wildcard alias copy** requires per-field memory emission and nested
  index expansion. If it proves too complex or produces Verilog that Icarus
  rejects, fall back to a documented alias comment and move the full AOS alias
  to a follow-up wave.
- **Bench-local variable-index slicing on packed vectors** can generate large
  priority muxes. The witnesses use small arrays and explicit assert guards so
  the generated mux is fully defined.
- Any subtask that introduces regressions should be immediately reverted and the
  partial work preserved only as a scratch spec with a clear `NOTE:` comment
  explaining the parked limitation.

## Critical Files for Implementation

- `/Users/playra/t27/bootstrap/src/compiler.rs`
  - `gen_verilog_const` (scalar/struct/AOS wildcard lowering)
  - function declaration collection and duplicate-function de-duplication
  - `emit_struct_literal_leaf` (bool/string/f32 placeholder handling)
  - array-parameter binding pass
  - packed-vector slicing helpers
- `/Users/playra/t27/specs/scratch/w487_wildcard_module_literal.t27`
- `/Users/playra/t27/specs/scratch/w487_wildcard_module_scalar_2d_alias.t27`
- `/Users/playra/t27/specs/scratch/w487_wildcard_module_aos_alias.t27`
- `/Users/playra/t27/specs/scratch/w487_bench_2d_array_param.t27`
- `/Users/playra/t27/specs/scratch/w487_bench_aos_array_param.t27`
