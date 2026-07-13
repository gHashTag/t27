# Wave Loop 520 — Closeout Report

**Issue:** #1489
**Branch:** `wave-loop-520`
**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Selected variant

**Variant A (recommended):** extend W517 and W519 to multi-dimensional packed
arrays-of-structs (AOS) parameters with array-typed fields.

### Actual focus discovered during implementation

The headline feature — multi-dimensional AOS parameters — was partially in
place for 1-D and for scalar-struct elements, but three independent gaps
blocked the full 2-D/3-D case:

1. **Module-level multi-dimensional AOS initializers did not handle flat
   struct-literal arrays.** The parser emits `[2][3]Pt{ Pt{...}, ..., Pt{...} }`
   as a single flat leaf list, but `gen_verilog_struct_rom_nested_init` only
   understood nested row literals, so only the first row was initialized.
2. **Function-local/test-block local arrays passed to array parameters were
   bound as if they were module-level memories.** The array-parameter binding
   pass did not recognize variables declared inside `test`/`invariant` blocks,
   so the callee referenced non-existent module memories.
3. **Local memory-mode AOS call-site packing used multi-dimensional outer
   indices instead of the linearized address used by the declaration.** This
   produced illegal references such as `ws_pts[0][0][0]` for a memory declared
   `ws_pts [0:3][0:1]`.

W520 fixed all three gaps and added dedicated witness coverage for register-
mode, packed-element, and memory-mode multi-dimensional AOS parameters.

---

## What changed

### 1. Flat struct-literal array initialization

In `bootstrap/src/compiler.rs`:

- `multi_dim_array_literal_get` now recognizes a flat leaf list (a child that is
  not itself an array literal) and computes the linear row-major element address.
- `gen_verilog_struct_rom_nested_init` now initializes from a flat leaf list by
  generating suffix index combinations within the current literal and adding
  the already-accumulated prefix linear address. Nested row literals continue
  to work through the existing recursion.

This fixes module-level `const`/`var` multi-dimensional AOS initializers for
both scalar-struct and memory-mode element types.

### 2. Test/invariant-block local arrays as packed-vector parameters

The array-parameter binding pass now passes the declaring `TestBlock` or
`InvariantBlock` node as the “containing function” when collecting call sites,
so `is_fn_local_array` correctly identifies array variables local to the test
block. Those arguments are marked `__local__` and passed as packed-vector inputs,
matching the existing function-local/bench-local path.

### 3. Leaf element type for array-typed-field parameter binding

When deciding whether an AOS parameter must be passed as a packed vector, the
binding pass now resolves the **leaf** struct element type through any remaining
array dimensions. Previously a parameter `[2][2]Buf` was inspected at the
intermediate type `[2]Buf`, which is not a struct, so the array-typed `data`
field was missed and the parameter was incorrectly bound to a non-existent
module memory.

### 4. Local memory-mode AOS call-site packing

`gen_verilog_pack_array_of_struct_expr` now linearizes the outer dimensions when
packing a function-local memory-mode AOS, producing references such as
`ws_pts[0][0]`, `ws_pts[0][1]` instead of `ws_pts[0][0][0]`. This matches the
`gen_verilog_local_struct_array_memory_decl` layout.

Files touched: `bootstrap/src/compiler.rs`.

### 5. Regression / integration test

Added `bootstrap/tests/w520_multi_dim_aos_param_verilog.rs`, which compiles a
2-D scalar-struct AOS parameter spec and asserts that:

- the callee accepts a packed 192-bit input `m`;
- field access on the parameter slices `m`;
- the call site packs the local array into a concatenation;
- the generated Verilog contains no `UNSUPPORTED_ICARUS` or `TODO` markers.

### 6. Witness specs

Added four scratch witnesses in `specs/scratch/`:

- `w520_2d_struct_array_param_module.t27` — module-level 2-D array of scalar
  structs passed as a function parameter (register-mode AOS).
- `w520_2d_struct_array_param_local.t27` — test-block local 2-D array of scalar
  structs passed as a packed-vector parameter.
- `w520_2d_packed_aos_param.t27` — 2-D array of packed scalar structs (element
  has fixed-size scalar array fields) passed from both module-level and local
  sources.
- `w520_2d_memory_aos_param.t27` — 2-D array of memory-mode structs (element
  has an array-typed direct field of structs) passed from both module-level and
  local sources.

All four specs were sealed under `.trinity/seals/`.

Existing seals affected by the initializer/declaration changes were resealed:

- `specs/scratch/w469_2d_struct_array_param.t27`
- `specs/scratch/w473_3d_module_var_struct_array.t27`
- `specs/scratch/w473_3d_module_var_struct_array_write.t27`

---

## Validation

| Gate | Result |
|------|--------|
| `cargo build --release` | ✅ |
| `cargo test -p t27c --bin t27c` | 1525 passed, 0 failed, 2 ignored |
| `cargo test -p t27c --tests` | ✅ all integration tests pass |
| `./scripts/tri test --icarus-lowerable --fast` | 0 failures, 0 seal mismatches, 0 yosys/Icarus baseline failures |
| `./scripts/tri verify --lean-lowerable` | ✅ passed (251 lowerable specs) |
| `lake build Trinity.IcarusLowerable.Soundness` | ✅ zero `sorry` |
| Manual Icarus smoke for the four new witnesses and the resealed W469/W473 specs | ✅ all pass |

Suite summary:

```
Parse failures:           0
Typecheck fails:          0
Gen Verilog fails:        0
Gen Verilog smoke fails:  0
Gen Verilog Icarus fails: 0
Seal mismatches:          0
FP divergences:           0
Icarus lowerable:         231
Icarus not lowerable:     0
Icarus disagreements:     0
TOTAL FAILURES:           0
```

---

## Scientific background

- IEEE Std 1800-2017, §7.2.1: packed structures are treated as a single vector
  when used as a primary.
- IEEE Std 1800-2017, §11.2.2 / §11.4.4 / §11.4.5: aggregate expressions may be
  copied and compared with relational and equality operators.
- IEEE Std 1800-2017, §6.22.2: packed arrays, packed structures, packed unions,
  and built-in integral types are equivalent if they share the same total bit
  width, state model, and signedness.
- Sutherland / Mills, SNUG 2013, *“Can My Synthesis Compiler Do That?”*: unpacked
  memories are synthesizable, and multi-dimensional arrays are supported by major
  synthesis tools when dimensions are statically bounded.
- AMD Vitis HLS UG1399/UG902 `array_reshape` and `array_partition` directives are
  the HLS analogue of flattening multi-dimensional arrays into linear packed
  vectors for port and function-parameter passing.

Sources:
- [IEEE 1800-2017 standard (MIT mirror)](https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf)
- [SNUG 2013 “Can My Synthesis Compiler Do That?”](https://lcdm-eng.com/papers/2014-DVCon_ASIC-FPGA_SV_Synthesis_paper.pdf)
- [AMD Vivado Synthesis UG901 — SystemVerilog Constructs](https://docs.amd.com/r/2022.1-English/ug901-vivado-synthesis/SystemVerilog-Constructs)
- [Vitis HLS UG1399 — array partitioning](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Array-Partitioning-and-Reshaping)

---

## Residual boundaries carried forward

- Three-dimensional AOS parameters are covered implicitly by the 2-D fixes and
  by the resealed W473 3-D specs, but no dedicated W520 3-D witness was added.
- Formal Lean 4 proof witnesses for multi-dimensional AOS parameter passing are
  not yet written; the existing `module_value_equiv_proved_sequential` machinery
  should cover the new scalar-struct cases once a witness is added to the
  completeness set.
- The Icarus-lowerable classifier was not hardened in this wave; adversarial
  negative witnesses for struct comparison and AOS parameter shapes remain for
  a future wave.

---

*φ² + φ⁻² = 3 | TRINITY*
