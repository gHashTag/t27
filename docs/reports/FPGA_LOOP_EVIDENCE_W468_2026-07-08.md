# FPGA Loop Evidence — Wave Loop 468 (2026-07-08)

**Issue:** #1446  
**Branch:** `wave-loop-468`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was executed

Wave Loop 468 selected **Variant B** from the W468 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line.

The wave extended the W467 struct lowering to:

1. Struct-return function call assignment (`let p : Pt = make_pt()`).
2. Two-dimensional scalar local arrays (`var m : [3][3]u8`).
3. RAM-style pragma propagation into local arrays.

It also added a scratch spec that exposes the still-unsupported 2D array
parameter path; that spec is held green by the existing `-DSIMULATION` yosys
smoke guard.

---

## Evidence

### Compiler-backend changes

- `bootstrap/src/compiler.rs`
  - Added `struct_return_width` / `struct_field_widths` and updated
    `gen_verilog_fn_internal` so that functions declared `-> Pt` emit a packed
    Verilog function whose width matches the struct layout.
  - Extended `ExprStructLit` to emit a packed concatenation `{y, x}` in
    expression context (e.g. a `return Pt{...}` statement).
  - Added `gen_verilog_struct_return_slicing` to unpack a packed struct-return
    call into per-field local registers.
  - Extended struct-variable initialization / assignment paths to accept an
    `ExprCall` RHS.
  - Added zero-parameter function tracking and dummy `_unused` inputs / `0`
    placeholder arguments so t27 functions with no parameters remain legal
    Verilog functions.
  - Added multi-dimensional scalar array lowering: `[M][N]T` local arrays
    become per-leaf scalar registers with literal-index and variable-index
    read/write rewriting.
  - Extended the parser to accept `pragma name = "value";` inside function
    bodies and to attach the pragma to the next local declaration.
  - Emitted `(* {pragma} *)` attributes before local array register
    declarations.
  - Added loop-variable `integer` declarations to prevent yosys simplifier
    assertions on complex function bodies with undeclared loop variables.

### Regression specs

- `specs/scratch/w468_struct_return_assign.t27`
- `specs/scratch/w468_local_2d_scalar_array.t27`
- `specs/scratch/w468_local_ram_style.t27`
- `specs/scratch/w468_2d_array.t27` (unsupported-path exposure spec)

### Suite result

- `./scripts/tri test --fast` reports **610/610 non-smoke PASS**, **90/90 yosys
  smoke PASS**, FPGA smoke gate OK, 0 baseline failures, 0 seal mismatches,
  **TOTAL FAILURES: 0**.
- `cargo test -p t27c --bin t27c` reports **1524 passed, 0 failed, 2 ignored**.

### Generated Verilog samples

`w468_struct_return_assign.t27` lowers `make_pt()` to a packed `[31:0]` return
and `sum_made()` slices it into `p_x` / `p_y`:

```verilog
// function: make_pt
function [31:0] make_pt; // -> Pt
    input _unused;
    begin : make_pt_body
        make_pt = {3, 4};
    end
endfunction

// function: sum_made
function [31:0] sum_made; // -> u32
    input _unused;
    begin : sum_made_body
        reg [15:0]  p_x;
        reg [15:0]  p_y;
        reg [31:0] _struct_tmp_0; // packed struct return temporary
        _struct_tmp_0 = make_pt(0);
        p_x = _struct_tmp_0[31:16];
        p_y = _struct_tmp_0[15:0];
        sum_made = ((p_x & {32{1'b1}}) + (p_y & {32{1'b1}}));
    end
endfunction
```

`w468_local_2d_scalar_array.t27` flattens a `[3][3]u8` to scalar registers and
rewrites variable-index writes into an if-else chain:

```verilog
reg [7:0]  m_0_0;
reg [7:0]  m_0_1;
reg [7:0]  m_0_2;
reg [7:0]  m_1_0;
reg [7:0]  m_1_1;
reg [7:0]  m_1_2;
reg [7:0]  m_2_0;
reg [7:0]  m_2_1;
reg [7:0]  m_2_2;
m_0_0 = 1;
m_0_1 = 2;
...
if ((i == 0) && (j == 0)) begin
    m_0_0 = v;
end
else if ((i == 0) && (j == 1)) begin
    m_0_1 = v;
end
...
```

`w468_local_ram_style.t27` propagates the pragma attribute to the flattened local
array registers:

```verilog
// function: sum_diag
function [31:0] sum_diag; // -> u32
    input _unused;
    begin : sum_diag_body
        (* ram_style = "block" *)
        reg [7:0]  m_0_0;
        reg [7:0]  m_0_1;
        reg [7:0]  m_0_2;
        ...
```

---

## What remains blocked

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Live-capture `XADC_LIVE_W468_OPERATING_POINT` — bench unavailable.
- Master-merge of the `gen-verilog` fix set from `master` (`701d79b3b`) — still
  rejected as too risky for a single wave.
- Multi-dimensional arrays of structs (`[M][N]Pt`), module-level scalar struct
  variables/consts, scalar struct parameters, and whole-struct comparison remain
  queued for W469.

---

*φ² + φ⁻² = 3 | TRINITY*
