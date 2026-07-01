# gen-verilog Lowering Defects — Reproduction Guide

**Date:** 2026-07-01
**Issue:** #1245
**Status:** 1 of 5 defects fixed (binary literals); 4 remain open pending safe backend refactor.

---

## 1. Only the first `const` emits as `localparam`

### Repro
```sh
cd /Users/playra/t27
./target/release/t27c gen-verilog specs/fpga/uart.t27 | grep -c "localparam"
```

### Actual output
```
1
```

### Expected
At least 9 `localparam` declarations (one per `const` in `uart.t27`).

### Root cause (tentative)
`Parser::is_top_level_start()` intentionally excludes `KwConst`/`KwVar` because those keywords can appear inside keyword-style `test`/`invariant`/`bench` blocks. When a `const` with a complex RHS fails to parse fully, the error-recovery `skip_to_next_top_level()` skips past subsequent `const` declarations as well. A proper fix requires tracking whether the parser is inside a top-level context vs. a nested block.

---

## 2. `0b` / `0x` literals

### `0b` — FIXED in W364
```sh
./target/release/t27c gen-verilog specs/fpga/uart.t27 | grep "0b"
```
No raw `0b` literals should appear; binary constants now emit as `N'b...`.

### `0x` — verify sizing
```sh
./target/release/t27c gen-verilog specs/fpga/uart.t27 | grep "'h"
```
`0xFF` as `u8` correctly emits `8'hFF`. The current emitter computes width as `hex.len()*4`, i.e. the *literal* width, not the *declared* type width. This is safe when the literal has the same width as the declared type, but a `u16` initialized with `0x1` will emit `4'h1` instead of `16'h1`. No known conformance failure yet.

---

## 3. Early `return` inside `if` (no `else`) inverts logic

### Repro spec (scratch)
Save as `/tmp/return_repro.t27`:
```t27
module return_repro;
fn f(c: bool) -> u8 {
    if (c) { return 1; }
    return 0;
}
test t1 { assert f(true) == 1; }
test t2 { assert f(false) == 0; }
```

### Repro
```sh
./target/release/t27c gen-verilog /tmp/return_repro.t27
```

### Actual output (excerpt)
```verilog
function [7:0] f;
    input c;
    begin
        if (c) begin
            f = 8'd1;
        end
        f = 8'd0;
    end
endfunction
```

### Expected
```verilog
function [7:0] f;
    input c;
    begin
        if (c) begin
            f = 8'd1;
        end else begin
            f = 8'd0;
        end
    end
endfunction
```

### Workaround in spec
Use `if/else` with single assignment per branch.

---

## 4. `as` cast + compound bitwise in `return` drops body

### Repro spec (scratch)
Save as `/tmp/cast_repro.t27`:
```t27
module cast_repro;
const MASK : u16 = 0x0FFF;
fn pack(win: u16, bit: u8) -> u16 {
    return ((win << 1) | (bit as u16)) & MASK;
}
test t1 { assert pack(0x0001, 1) == 0x0003; }
```

### Actual output
```verilog
function [15:0] pack;
    input [15:0] win;
    input [7:0] bit;
    begin
        // TODO: implement
    end
endfunction
```

### Workaround
Remove the `as u16` cast and rely on implicit width promotion, if the spec allows it.

---

## 5. Struct-field reg name mismatch

### Repro
```sh
./target/release/t27c gen-verilog specs/fpga/uart.t27 | grep -E "uartstate_|uart_state_"
```

### Actual output
```verilog
// declared:
reg [7:0] uartstate_tx_data;
// referenced:
assign ready = 1'b1;   // no direct field reference in top-level
// inside function bodies:
uart_state_tx_data = data;   // undeclared identifier
```

### Root cause
Struct fields emit as `<structtype_lower>_<field>`, but variable access uses `<varname>_<field>`.

---

## Recommended safe triage order

1. **Do not** modify `is_top_level_start()` until the parser tracks top-level vs. nested context.
2. **Verify** `0x` sizing with a targeted test; pad to declared width if regression-free.
3. **Defer** early-return and cast+bitwise fixes until a dedicated Verilog control-flow / expression lowering pass is designed.
4. **Defer** struct-field name fix until the struct lowering naming convention is unified.

---

Trinity invariant: `phi^2 + 1/phi^2 = 3`
