# FPGA Loop Evidence — Wave Loop 465 (2026-07-08)

**Issue:** #1443  
**Branch:** `wave-loop-465`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was executed

Wave Loop 465 selected **Variant B** from the W465 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line.

The wave extended the W464 struct-array lowering to function-local and
bench-local arrays of structs, verified keyword-safe field-memory names, and
locked in multi-site struct-literal array argument deduplication.

---

## Evidence

### Compiler-backend changes

- `bootstrap/src/compiler.rs`
  - Added `local_array_elem_types` registry and helpers
    `gen_verilog_local_struct_array_decl`,
    `gen_verilog_local_struct_array_init`,
    `local_array_elem_is_struct`.
  - Extended `StmtLocal`, `gen_verilog_local_decl_hoisted`, and
    `gen_verilog_local_assign` for struct element types.
  - Extended `ExprFieldAccess` for indexed field access on local arrays of
    structs.

### Regression specs

- `specs/scratch/w465_local_struct_array.t27`
- `specs/scratch/w465_bench_local_struct_array.t27`
- `specs/scratch/w465_keyword_field_local_struct_array.t27`
- `specs/scratch/w465_keyword_field_struct_array.t27`
- `specs/scratch/w465_multi_site_struct_array_literal.t27`

### Suite result

- `./scripts/tri test --fast` reports **599/599 non-smoke PASS**, **79/79 yosys
  smoke PASS**, FPGA smoke gate OK, 0 baseline failures, 0 seal mismatches,
  **TOTAL FAILURES: 0**.
- `cargo test -p t27c --bin t27c` reports **1524 passed, 0 failed, 2 ignored**.

### Generated Verilog samples

`w465_local_struct_array.t27` emits per-element per-field registers inside the
function:

```verilog
function [15:0] sum_local;
    input _unused;
    begin : sum_local_body
        reg [15:0]  pts_0_x;
        reg [15:0]  pts_0_y;
        reg [15:0]  pts_1_x;
        reg [15:0]  pts_1_y;
        reg [15:0]  pts_2_x;
        reg [15:0]  pts_2_y;
        pts_0_x = 1;
        pts_0_y = 2;
        // ...
        sum_local = ((pts_0_x + pts_1_x) + pts_2_x);
    end
endfunction
```

`w465_keyword_field_local_struct_array.t27` emits single-token names for keyword
fields:

```verilog
reg [15:0]  words_0_reg;
reg [15:0]  words_0_wire;
```

`w465_multi_site_struct_array_literal.t27` emits a single anonymous ROM set for
two call sites:

```verilog
// LUT: _lit_3_Pt_struct_x_1_y_2_struct_x_3_y_4_struct_x_5_y_6 [3] Pt
reg [15:0]  _lit_3_Pt_struct_x_1_y_2_struct_x_3_y_4_struct_x_5_y_6_x [0:2];
reg [15:0]  _lit_3_Pt_struct_x_1_y_2_struct_x_3_y_4_struct_x_5_y_6_y [0:2];
```

---

## Conclusion

W465 closes the function-local/bench-local struct-array lowering gaps and
hardens field-memory name safety. The suite remains fully green and the
competitive boundary is unchanged because no new public Lean-native ternary-FPGA
competitor appeared.

---

*φ² + φ⁻² = 3 | TRINITY*
