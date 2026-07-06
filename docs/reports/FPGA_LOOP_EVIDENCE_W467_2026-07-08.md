# FPGA Loop Evidence — Wave Loop 467 (2026-07-08)

**Issue:** #1445  
**Branch:** `wave-loop-467`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was executed

Wave Loop 467 selected **Variant B** from the W467 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line.

The wave extended the W466 struct-array lowering to whole-struct assignment by
value, whole-element assignment into struct arrays, struct fields that are
fixed-size arrays, and a keyword-field clone-path regression spec. It also fixed
the if-else code shape for variable-index writes on local struct arrays so that
multi-field branches are grouped with `begin ... end`.

---

## Evidence

### Compiler-backend changes

- `bootstrap/src/compiler.rs`
  - Added `local_struct_var_types` registry to track function-local struct
    variable types.
  - Added helpers to declare, initialize, and assign struct variables by
    recursively expanding fields into scalar registers and memories:
    `gen_verilog_struct_field_decl`, `gen_verilog_local_struct_var_decl`,
    `gen_verilog_scalar_assign`, `gen_verilog_struct_field_assign`,
    `gen_verilog_local_struct_var_init`, `gen_verilog_try_struct_var_assign`,
    `gen_verilog_try_struct_array_element_assign`.
  - Updated `StmtLocal` to emit per-field declarations for local struct
    variables and to initialize each leaf from a struct literal or a source
    struct variable.
  - Updated `StmtAssign` to try whole-struct and whole-element decomposition
    before the existing field-wise and scalar paths.
  - Wrapped each arm of the variable-index local-struct-array write path in
    `begin ... end` so multiple field assignments share the same condition.

### Regression specs

- `specs/scratch/w467_struct_assign.t27`
- `specs/scratch/w467_struct_array_element_assign.t27`
- `specs/scratch/w467_struct_field_array.t27`
- `specs/scratch/w467_keyword_field_struct_array_clone.t27`

### Suite result

- `./scripts/tri test --fast` reports **606/606 non-smoke PASS**, **86/86 yosys
  smoke PASS**, FPGA smoke gate OK, 0 baseline failures, 0 seal mismatches,
  **TOTAL FAILURES: 0**.
- `cargo test -p t27c --bin t27c` reports **1524 passed, 0 failed, 2 ignored**.

### Generated Verilog samples

`w467_struct_assign.t27` lowers whole-struct copy and literal initialization
into per-field assignments:

```verilog
// function: copy_var
function [31:0] copy_var;
    input _unused;
    begin : copy_var_body
        reg [15:0]  a_x;
        reg [15:0]  a_y;
        a_x = 1;
        a_y = 2;
        reg [15:0]  b_x;
        reg [15:0]  b_y;
        b_x = 3;
        b_y = 4;
        a_x = b_x;
        a_y = b_y;
        copy_var = ((a_x & {32{1'b1}}) + (a_y & {32{1'b1}}));
    end
endfunction
```

`w467_struct_array_element_assign.t27` lowers whole-element assignment into a
local struct array with properly grouped if-else branches:

```verilog
if (idx == 0) begin
    tmp_0_x = 7;
    tmp_0_y = 8;
end
else if (idx == 1) begin
    tmp_1_x = 7;
    tmp_1_y = 8;
end
else begin
    tmp_2_x = 7;
    tmp_2_y = 8;
end
```

`w467_struct_field_array.t27` flattens an array field into a Verilog memory:

```verilog
// function: set_coords
function [31:0] set_coords;
    input [31:0] i;
    input [7:0] v;
    begin : set_coords_body
        reg [7:0]  p_coords [0:2];
        reg [7:0]  p_tag;
        p_coords[0] = 1;
        p_coords[1] = 2;
        p_coords[2] = 3;
        p_tag = 4;
        p_coords[i] = v;
        set_coords = ((((p_coords[0] & {32{1'b1}}) + (p_coords[1] & {32{1'b1}})) + (p_coords[2] & {32{1'b1}})) + (p_tag & {32{1'b1}}));
    end
endfunction
```

`w467_keyword_field_struct_array_clone.t27` shows keyword-safe bound memories
after the clone path:

```verilog
// LUT: data [3] Word (struct)
reg [15:0]  data_reg [0:2];
reg [15:0]  data_wire [0:2];
initial begin
    data_reg[0] = 1;
    data_wire[0] = 2;
    data_reg[1] = 3;
    data_wire[1] = 4;
    data_reg[2] = 5;
    data_wire[2] = 6;
end

// function: sum_x_indirect
function [15:0] sum_x_indirect;
    begin : sum_x_indirect_body
        sum_x_indirect = sum_x_direct();
    end
endfunction
```

---

*φ² + φ⁻² = 3 | TRINITY*
