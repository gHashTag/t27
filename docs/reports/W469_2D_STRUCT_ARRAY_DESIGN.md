# Design: W469 2-D Array-of-Struct Verilog Lowering

**Status:** design — implementation scheduled for Wave Loop 527  
**Issue:** #1497 (W526 boundary), follow-up #1498 (W527 implementation)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Problem statement

A t27 declaration such as

```t27
pub struct Pt { x : u16, y : u16 }

pub fn sum() -> u32 {
    var m : [2][3]Pt = [2][3]Pt{
        [3]Pt{ Pt{.x=1, .y=2}, Pt{.x=3, .y=4}, Pt{.x=5, .y=6} },
        [3]Pt{ Pt{.x=7, .y=8}, Pt{.x=9, .y=10}, Pt{.x=11, .y=12} },
    };
    return (m[0][0].x as u32) + (m[1][2].y as u32);
}
```

currently produces silent broken output on `master`:

- The parser interprets `[2][3]Pt{...}` as an indexing expression over a bare `[2]` literal and drops the rest of the initializer, leaving the function body truncated.
- The Verilog backend then emits either incomplete placeholder code or synthesizable-but-wrong modules that pass yosys smoke but would fail simulation.

Wave Loop 526 added an explicit compile-time diagnostic in `Compiler::compile_verilog` so the failure is no longer silent. Wave Loop 527 must implement the complete lowering.

---

## Goals

1. Parse `[N][M]Struct{...}` and preserve the full nested literal in the AST.
2. Typecheck the multi-dimensional array type and infer per-dimension sizes.
3. Lower to synthesizable Verilog using a deterministic layout policy.
4. Prove or at least witness value preservation for the Icarus-lowerable subset.

---

## Parser changes

Location: `bootstrap/src/compiler.rs`, `Parser::parse_array_literal`.

Current behavior

```text
[2] -> ExprArrayLiteral(extra_size="2")
[2][3] -> ExprIndex(base=[2], index=3)
```

Required behavior

```text
[2][3]Pt{ ... } -> ExprArrayLiteral(extra_size="2][3", extra_type="Pt", children = inner literals)
[3]Pt{ ... }    -> ExprArrayLiteral(extra_size="3", extra_type="Pt", children = struct literals)
```

Implementation sketch

- After consuming the first `[N]` in `parse_array_literal`, look ahead for another `[`.
- If found, consume each subsequent `[M]` and append `][M` to `extra_size`.
- Then read the element type identifier (possibly namespace-qualified) into `extra_type`.
- Then read `{ ... }` elements as today, each parsed recursively by `parse_expr`.

Edge cases

- `[_]Type{...}` (inferred outer dimension) — keep current single-dim path.
- Mixed scalar and struct elements — reject at typecheck.
- Trailing comma in literals — keep existing handling.

---

## Typechecker changes

Location: `typecheck_ast` and helpers.

- Validate that the number of dimensions in `extra_type` matches the number of bracket pairs in `extra_size`.
- Validate that each dimension size is a positive compile-time constant.
- Resolve `extra_type` to a `StructDecl` or primitive and reject unsupported element types (e.g. arrays of arrays of arrays beyond policy).
- Compute total flattened width for packed-vector lowering.

---

## Verilog lowering policy

We choose **packed-vector array-of-structs (AoS)** for scalar structs because t27 already uses packed-vector lowering for 1-D arrays of scalar structs and for fixed-size scalar array fields inside structs.

For

```t27
var m : [2][3]Pt
```

with `Pt = { x: u16, y: u16 }`, emit one flattened register:

```verilog
reg [2*3*32-1:0] m;   // 2 rows * 3 cols * 32 bits per Pt
```

Bit layout (row-major, field-major inside each element):

```
row 0, col 0 : m[31:16] = y, m[15:0]  = x
row 0, col 1 : m[63:48] = y, m[47:32] = x
...
row i, col j : m[(i*3+j+1)*32-1 : (i*3+j)*32]
```

Access helpers to add to `VerilogCodegen`:

- `emit_2d_struct_array_field_slice(name, row, col, field, row_count, col_count, field_width)`.
- `emit_2d_struct_array_element_range(name, row, col, row_count, col_count, element_width)`.
- Whole-array assignment: copy the packed vector directly (`m_dst = m_src;`).

Initializers become procedural assignments inside `initial begin ... end`, or a generate block for static localparams where legal.

### Alternative considered: struct-of-arrays (SoA)

SoA would emit separate registers per field:

```verilog
reg [2*3*16-1:0] m_x;
reg [2*3*16-1:0] m_y;
```

SoA is better for streaming/vectorized access but worse for random element copies. Because t27 already uses AoS for 1-D packed struct arrays, AoS keeps the layout uniform and simplifies the proof.

---

## Formal soundness

Once the IcarusLowerable Lean 4 stack is on `master`, the W527 implementation must:

1. Add a lowerability predicate case `P_2DArrayOfScalarStruct`.
2. Extend `module_value_equiv_proved_sequential` to cover packed 2-D arrays.
3. Add a scratch witness for read, whole-array copy, and field write.
4. Import the corpus witness into `Completeness.lean` and prove value preservation via `native_decide` or the sequential theorem.

---

## Reseal strategy

Full implementation will change generated Verilog for any spec containing 2-D struct arrays. Before landing W527:

1. Run `./scripts/tri seal --refresh` for affected specs (expected to be only new scratch witnesses and possibly `specs/igla/` AOS specs if they use 2-D struct arrays).
2. Confirm `./scripts/tri test` reports zero seal mismatches.
3. Confirm yosys and Icarus smoke counts match the new baseline.

---

## W526 interim boundary

`Compiler::compile_verilog` now calls `detect_unsupported_verilog_locals`, which returns:

```text
unsupported multi-dimensional array of aggregate type `[2][3]Pt` for local variable `m` at line 12:
2-D array-of-struct lowering is not yet implemented (see docs/reports/W469_2D_STRUCT_ARRAY_DESIGN.md)
```

This prevents silent bad-code emission and gives W527 a clear starting point.

---

*φ² + φ⁻² = 3 | TRINITY*
