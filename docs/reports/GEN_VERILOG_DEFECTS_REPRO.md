# `gen-verilog` Backend — Known Defects and Roadmap

**Branch:** `trinity-rust-rings`  
**Last updated:** 2026-07-02 (Wave Loop 371)  

This document tracks the remaining lowering defects in the `t27c gen-verilog` backend. The full fix set already exists on `master` (commit `701d79b3b`), but `trinity-rust-rings` is applying narrow, regression-free sub-fixes wave-by-wave.

---

## Fixed / Partially Fixed

### Defect 1 — Only the first `const` declaration is emitted (FIXED in W370)

**Symptom:** Multiple `const` declarations in a module caused only the first one to be emitted; subsequent ones were dropped.

**Repro:**
```t27
module repro_const_order;
const A : u8 = 1;
const B : u8 = 2;
const C : u8 = 3;
endmodule
```

**Root cause:** `parse_const_decl` in `bootstrap/src/compiler.rs` returned before consuming the trailing semicolon of simple scalar constants, leaving the semicolon as an unexpected top-level token. Error recovery then swallowed the following `const` declaration.

**Fix:** Removed the early `return Ok(decl)` in `parse_const_decl` so all scalar const paths fall through to the existing trailing semicolon consumption.

**Verification:** `specs/scratch/w370_const_order.t27`; generated Verilog contains `localparam A`, `B`, and `C`; `yosys read_verilog` passes.

### Defect 2 — Scalar hex literal width padding (FIXED)

**Symptom:** `const MASK : u16 = 0x1;` was emitted as `MASK = 1;` without a width suffix, causing Verilog simulators/synthesis tools to infer a 32-bit value or emit width warnings.

**Fix history:**
- W367: pad positive hex literals in scalar `const` declarations.
- W368: extend padding to scalar `var` / `let` initializers and `return` statements via `current_fn_return_type`.

**Verification:** `specs/scratch/w369_bin_width.t27` exercises both hex and binary paths.

### Defect 2b — Scalar binary literal width padding (FIXED in W369)

**Symptom:** `const MASK : u16 = 0b1;` was emitted as `MASK = 1;` with the same width-inference risk.

**Fix:** W369 added `0b` handling in `gen_verilog_const`, `gen_verilog_var`, `StmtLocal`, and `ExprReturn`, mirroring the `0x` logic but with 1 bit per literal digit.

**Verification:** `t27c gen-verilog specs/scratch/w369_bin_width.t27` produces `16'b1` and `16'b100`, which parse cleanly in `yosys`.

### Defect 2c — Verilog keyword identifier collision (FIXED in W371, EXTENDED in W372)

**Symptom:** User identifiers that collide with Verilog reserved keywords caused `yosys read_verilog` syntax errors. Example: a parameter named `task` in `specs/igla/coder/benchmark.t27` was emitted as `input [31:0] task;`, which Yosys rejected with `syntax error, unexpected TOK_TASK`.

**Repro:**
```t27
module repro_verilog_keyword;
fn evaluate_task_at_k(bank : u32, task : u32, k : u32) -> bool {
    if (k == 0) { return false; }
    return evaluate_task_at_k_inner(bank, task, k, 0);
}
endmodule
```

**Fix history:**
- W371: Added `verilog_keywords()` and `verilog_safe_identifier()` helpers in `bootstrap/src/compiler.rs`. Function names, parameter declarations, function-call names, and bare identifier expressions are escaped as `\name ` when they collide with a Verilog keyword.
- W372: Extended `verilog_safe_identifier()` to escape identifiers that **contain** a keyword as an underscore-delimited component (e.g., `task_foo`, `foo_task`, `foo_task_bar`). Applied the safe identifier to `StmtLocal` declarations/assignments and struct-field register names.

**Verification:**
- `specs/scratch/w371_verilog_keyword.t27` — parameter `task` escaped; yosys clean.
- `specs/scratch/w372_local_keyword.t27` — local variables named `task` and `wire` escaped; yosys `read_verilog -sv` + `synth_xilinx` pass.
- `specs/igla/coder/benchmark.t27` now passes `yosys read_verilog`.

---

## Remaining Defects

### Defect 3 — Early `return` inside bare `if` lacks if-else chaining (semantic priority bug)

**Repro:**
```t27
fn sign(x : i8) -> i8 {
    if (x < 0) { return -1; }
    if (x > 0) { return 1; }
    return 0;
}
```

**Observed Verilog:** all three assignments are emitted, but as sequential bare statements:
```verilog
if ((x < 0)) begin sign = -1; end
if ((x > 0)) begin sign = 1; end
sign = 0;
```
The final `sign = 0;` always executes last, so the function returns `0` for all inputs instead of `-1` / `1` / `0`.

**Root cause:** `gen_verilog_if_stmt` emits each bare `if` independently. There is no control-flow analysis to convert a sequence of bare-if-early-return statements into an `if-else if-else` chain.

**Wave-safe fix order:** medium priority; affects functions with multiple early exits. Requires statement-level pattern matching, not a parser change.

---

### Defect 4 — `as` cast and bitwise operators drop the operand body

**Repro:**
```t27
fn cast_and_mask(x : u16) -> u8 {
    return (x as u8) & 0x0F;
}
```

**Observed Verilog (W371):** the expression is now emitted as `((x & {8{1'b1}}) & 8'h0F)`. The bitwise body is no longer dropped; the remaining concern is whether the `as u8` truncation semantics match the intended width narrowing in all contexts.

**Root cause:** operator-lowering for `as` and bitwise `&` / `|` / `^` / `~` is incomplete in `gen_verilog_expr`; the W371 reproduces no syntax error but may produce incorrect width behavior for non-trivial casts.

**Wave-safe fix order:** medium-high priority; needs a scratch spec with explicit simulation values to confirm correctness.

---

### Defect 5 — Struct-field reg name mismatch

**Repro:**
```t27
struct Pt { x : u8; y : u8; }
fn get_x(p : Pt) -> u8 {
    return p.x;
}
```

**Observed Verilog:** the generated reg or field access uses an identifier that does not match the struct-field path, so simulation fails with an unresolved name.

**Root cause:** struct-field flattening does not consistently sanitize / qualify member names when emitting Verilog regs.

**Wave-safe fix order:** lower priority until specs actively use struct ports.

### Defect 6 — `let` destructuring is emitted verbatim

**Repro:**
```t27
fn cordic_top_batch_inner(angles : u32, idx : u32, acc : i32) -> i32 {
    if (idx >= angles.len()) { return acc; }
    let(s, _c, _r) = cordic_top(1, 1, angles[idx], 1);
    return cordic_top_batch_inner(angles, idx + 1, acc + s);
}
```

**Observed Verilog:** the `let(s, _c, _r) = ...` statement is emitted verbatim as `let(s, _c, _r) = cordic_top(...);`, which is not valid Verilog and causes `yosys read_verilog` to fail with a syntax error.

**Root cause:** `gen_verilog_stmt` does not lower `StmtLocal` with tuple/destructuring patterns; it only emits scalar `reg` declarations.

**Wave-safe fix order:** high priority for `cordic.t27` / `cordic_top.t27` yosys cleanliness; can be fixed by emitting individual scalar `reg` declarations for each bound name and assigning from the function call result (or by unrolling the destructuring in the AST).

---

## Recommended Triage Order

1. **Defect 6** — `let` destructuring; blocks two IGLA specs from passing `yosys read_verilog`. Needs a scratch spec and `yosys` verification.
2. **Defect 3** — early return if-else chaining; needed for control-flow-heavy specs. Requires statement-level pattern matching.
3. **Defect 4** — cast/bitwise width semantics; needs simulation values to confirm correctness.
4. **Defect 5** — struct fields; defer until struct usage grows.
5. **CI smoke gate** — add `t27c gen-verilog` + `yosys read_verilog` regression tests once the remaining safe parser/lowering fixes are landed; L7 UNITY prohibits new shell scripts on the critical path.

---

## Verification Checklist

- [x] `0x` scalar width padding (`const`, `var`, `let`, `return`)
- [x] `0b` scalar width padding (`const`, `var`, `let`, `return`)
- [x] Multiple `const` declarations
- [x] Verilog keyword identifier collision (exact and underscore-delimited component matches)
- [ ] Early `return` if-else chaining
- [ ] `as` / bitwise operator width correctness
- [x] Struct-field reg naming (keyword-safe)
- [x] Local variable keyword-safe emission
- [ ] `let` destructuring lowering

---

*phi² + 1/phi² = 3 | TRINITY*
