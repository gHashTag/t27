# FPGA Loop Evidence — Wave Loop 466 (2026-07-08)

**Issue:** #1444  
**Branch:** `wave-loop-466`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was executed

Wave Loop 466 selected **Variant B** from the W466 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line.

The wave extended the W465 struct-array lowering to nested structs,
variable-index reads/writes on local struct arrays, and mixed direct/indirect
struct-literal array arguments across function boundaries.

---

## Evidence

### Compiler-backend changes

- `bootstrap/src/compiler.rs`
  - Added `flatten_struct_fields` and `module_struct_array_fields` to flatten
    nested struct arrays to scalar leaf memories.
  - Extended `gen_verilog_struct_rom_elem_init` with a `field_prefix` parameter
    for nested struct initialization.
  - Added `flatten_nested_array_field_access` and a dedicated
    `ExprFieldAccess` branch for `arr[i].inner.a` style accesses.
  - Added `gen_verilog_try_struct_array_assign` for field-wise variable-index
    writes on struct arrays.
  - Added `gen_verilog_local_struct_array_varidx_read` for correctly
    parenthesised priority muxes.
  - Updated `test_verilog_struct_field_access_indexed` to expect per-element
    flattened names.

### Regression specs

- `specs/scratch/w466_nested_struct_array.t27`
- `specs/scratch/w466_varidx_struct_array.t27`
- `specs/scratch/w466_mixed_struct_array_call.t27`

### Suite result

- `./scripts/tri test --fast` reports **602/602 non-smoke PASS**, **82/82 yosys
  smoke PASS**, FPGA smoke gate OK, 0 baseline failures, 0 seal mismatches,
  **TOTAL FAILURES: 0**.
- `cargo test -p t27c --bin t27c` reports **1524 passed, 0 failed, 2 ignored**.

### Generated Verilog samples

`w466_nested_struct_array.t27` emits one memory per scalar leaf field:

```verilog
// LUT: data [2] Outer (struct)
reg [7:0]  data_inner_a [0:1];
reg [7:0]  data_inner_b [0:1];
reg [7:0]  data_tag [0:1];
initial begin
    data_inner_a[0] = 1;
    data_inner_b[0] = 2;
    data_tag[0] = 3;
    data_inner_a[1] = 4;
    data_inner_b[1] = 5;
    data_tag[1] = 6;
end
```

`w466_varidx_struct_array.t27` lowers field-wise writes on a bound array
parameter and a local struct array:

```verilog
// function: set_and_sum
function [31:0] set_and_sum;
    input [31:0] idx;
    input [15:0] vx;
    input [15:0] vy;
    begin : set_and_sum_body
        data_x[idx] = vx;
        data_y[idx] = vy;
        set_and_sum = ((data_x[idx] & {32{1'b1}}) + (data_y[idx] & {32{1'b1}}));
    end
endfunction

// function: local_set_and_sum
function [31:0] local_set_and_sum;
    input [31:0] idx;
    input [15:0] vx;
    input [15:0] vy;
    begin : local_set_and_sum_body
        reg [15:0]  tmp_0_x;
        reg [15:0]  tmp_0_y;
        reg [15:0]  tmp_1_x;
        reg [15:0]  tmp_1_y;
        reg [15:0]  tmp_2_x;
        reg [15:0]  tmp_2_y;
        // ... scalar init omitted ...
        if (idx == 0) tmp_0_x = vx;
        else if (idx == 1) tmp_1_x = vx;
        else tmp_2_x = vx;
        if (idx == 0) tmp_0_y = vy;
        else if (idx == 1) tmp_1_y = vy;
        else tmp_2_y = vy;
        local_set_and_sum = ((((idx == 0) ? tmp_0_x : ((idx == 1) ? tmp_1_x : ((idx == 2) ? tmp_2_x : 0))) & {32{1'b1}}) + (((idx == 0) ? tmp_0_y : ((idx == 1) ? tmp_1_y : ((idx == 2) ? tmp_2_y : 0))) & {32{1'b1}}));
    end
endfunction
```

`w466_mixed_struct_array_call.t27` verifies the clone path for struct-literal
array arguments:

```verilog
// LUT: _lit_3_Pt_struct_x_1_y_2_struct_x_3_y_4_struct_x_5_y_6 [3] Pt
reg [15:0]  _lit_3_Pt_struct_x_1_y_2_struct_x_3_y_4_struct_x_5_y_6_x [0:2];
reg [15:0]  _lit_3_Pt_struct_x_1_y_2_struct_x_3_y_4_struct_x_5_y_6_y [0:2];
```

---

## Conclusion

W466 closes the nested-struct-array, variable-index local struct-array read/write,
and mixed struct-literal array-argument gaps. The suite remains fully green and
the competitive boundary is unchanged because no new public Lean-native
ternary-FPGA competitor appeared.

---

*φ² + φ⁻² = 3 | TRINITY*
