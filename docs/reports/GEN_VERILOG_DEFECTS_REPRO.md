# `gen-verilog` Backend — Known Defects and Roadmap

**Branch:** `trinity-rust-rings`  
**Last updated:** 2026-07-03 (Wave Loop 377)  

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

### Defect 2c — Verilog keyword identifier collision (FIXED in W371, EXTENDED in W372, STRUCT-FIELD TOKENIZATION CORRECTED in W373)

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
- W373: Corrected a tokenization bug in the W372 struct-field path. The W372 implementation escaped the field name in isolation (`\reg `) and then prepended the struct type name, producing `word_\reg `, which Verilog tokenizes as the separate identifiers `word_` and `\reg `. W373 now builds the full flattened name first (`word_reg`) and escapes the entire token as `\word_reg ` when needed. The same full-token escaping is applied to `ExprFieldAccess` in `gen_verilog_expr`.

**Fix history:**
- W374: Applied `verilog_safe_identifier()` to module-level `const` and `var` declarations. Top-level names like `wire` or `reg` are now emitted as escaped identifiers (`\wire ` / `\reg `) in `localparam`, `reg`, and initializer statements. Array var elements use the escaped base name (`\wire_0`, `\reg_0`).

**Verification:**
- `specs/scratch/w371_verilog_keyword.t27` — parameter `task` escaped; yosys clean.
- `specs/scratch/w372_local_keyword.t27` — local variables named `task` and `wire` escaped; yosys `read_verilog -sv` + `synth_xilinx` pass.
- `specs/scratch/w373_struct_field_keyword.t27` — struct fields named `reg` and `wire`; generated regs are `\word_reg ` / `\word_wire ` and parse cleanly through `yosys read_verilog -sv` + `synth_xilinx`.
- `specs/scratch/w374_module_keyword.t27` — top-level const `wire` and var `reg` escaped; `t27c gen-verilog` + `yosys read_verilog -sv` + `synth_xilinx` pass.
- `specs/igla/coder/benchmark.t27` now passes `yosys read_verilog`.

### Defect 3 — Early `return` inside bare `if` lacks if-else chaining (FIXED in W375)

**Repro:**
```t27
fn sign(x : i8) -> i8 {
    if (x < 0) { return -1; }
    if (x > 0) { return 1; }
    return 0;
}
```

**Observed Verilog (before W375):** all three assignments were emitted as sequential bare statements:
```verilog
if ((x < 0)) begin sign = -1; end
if ((x > 0)) begin sign = 1; end
sign = 0;
```
The final `sign = 0;` always executed last, so the function returned `0` for all inputs.

**Fix:** `bootstrap/src/compiler.rs` `gen_verilog_fn` now walks the function body and collapses contiguous bare-if early-return statements into a single Verilog `if ... else if ... else` chain, with each branch assigning to the function-name register. Statements that do not match the chain pattern remain on the original code path.

**Verification:** `specs/scratch/w375_early_return.t27` passes `t27c gen-verilog` and `yosys read_verilog -sv`; generated `sign` function emits:
```verilog
if ((x < 0)) begin sign = -1; end
else if ((x > 0)) begin sign = 1; end
else begin sign = 0; end
```

---

## Remaining Defects

### Defect 4 — `as` cast and bitwise operator width correctness (VERIFIED FIXED in W376)

**Repro:**
```t27
fn cast_and_mask(x : u16) -> u8 {
    return (x as u8) & 0x0F;
}
```

**Observed Verilog (W376):** the expression is emitted as `((x & {8{1'b1}}) & 8'h0F)`. The `as u8` narrowing is implemented as a bitwise mask `{8{1'b1}}`, and the bitwise body is preserved. Generated Verilog for `or`/`xor` casts follows the same pattern.

**Root cause:** operator-lowering for `as` and bitwise `&` / `|` / `^` / `~` in `gen_verilog_expr` already masks the operand to the target width; the W376 work formalized the regression spec and added an in-runner yosys smoke gate so the behavior stays correct.

**Verification:**
- `specs/scratch/w376_cast_width.t27` exercises narrowing `u16 -> u8` and `i16 -> i8` casts followed by `&`, `|`, and `^`, with `test` assertions covering both high-byte truncation and low-byte preservation.
- `t27c gen-verilog specs/scratch/w376_cast_width.t27` + `yosys read_verilog -sv` pass.
- The W376 CI smoke gate in `bootstrap/src/suite.rs` now runs `yosys read_verilog -sv` on every scratch spec automatically when `yosys` is on `PATH`.

**Status:** Closed as verified-correct. No compiler change was required; the existing ExprCast lowering already emits width-safe masks.

---

### Defect 5 — Struct-field reg name mismatch (FIXED in W377)

**Repro:**
```t27
struct Pt { x : u8; y : u8; }
fn get_x(p : Pt) -> u8 {
    return p.x;
}
```

**Observed Verilog (before W377):** field access on a struct-typed parameter was emitted using the parameter-variable name as a prefix, e.g. `p_x`, while the struct declaration emitted module-level registers named after the struct type, e.g. `pt_x`. This mismatch caused unresolved identifiers in simulation/synthesis.

**Observed Verilog (after W377):** the codegen now tracks parameter types and emitted struct-field register names. When a function parameter has a struct type, field access resolves to the struct-type register name (`pt_x`) instead of the variable-qualified name (`p_x`).

**Root cause:** `gen_verilog_expr` lowered `ExprFieldAccess` as `{base}_{field}` without knowing whether `base` was a struct-typed parameter and without a registry of the struct-type register names emitted by `gen_verilog_struct`.

**Fix (W377):**
- Added `param_types: HashMap<String, String>` to `VerilogCodegen` to record the declared type of each function parameter.
- Added `struct_field_regs: HashSet<String>` to record the flattened register names emitted for each struct field (e.g. `word_data`).
- In `gen_verilog_fn`, populate `param_types` from `node.params` before emitting the function body.
- In `gen_verilog_struct`, insert each emitted register name into `struct_field_regs`.
- In `ExprFieldAccess` lowering, if the base identifier's declared type is a struct, build the candidate struct-type register name (`{type}_{field}`). If it exists in `struct_field_regs`, use it; otherwise fall back to the original `{base}_{field}` behavior.

**Verification:** `specs/scratch/w377_struct_field_mapping.t27` exercises field reads on a struct-typed parameter. `t27c gen-verilog` emits `word_data` / `word_tag` references, and `yosys read_verilog -sv` + `synth_xilinx` pass.

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

1. **Defect 6** — `let` destructuring; blocked by the deeper absence of tuple-return function generation in the Verilog backend. A pure syntax-level workaround is possible but semantic correctness requires parser/codegen work for tuple return types and tuple literals. Once fixed, add `specs/igla/race/cordic.t27` and `specs/igla/race/cordic_top.t27` to the smoke gate.
2. **CI smoke gate expansion** — with Defect 5 resolved, the W377 gate now covers all 25 yosys-clean IGLA specs plus all scratch specs. Remaining expansion is gated on Defect 6.

---

## Verification Checklist

- [x] `0x` scalar width padding (`const`, `var`, `let`, `return`)
- [x] `0b` scalar width padding (`const`, `var`, `let`, `return`)
- [x] Multiple `const` declarations
- [x] Verilog keyword identifier collision (exact and underscore-delimited component matches)
- [x] Module-level const/var keyword-safe emission
- [x] Early `return` if-else chaining (FIXED in W375)
- [x] `as` / bitwise operator width correctness (FIXED/VERIFIED in W376)
- [x] Struct-field reg naming (keyword-safe, full-token escape)
- [x] Local variable keyword-safe emission
- [ ] `let` destructuring lowering (blocked by missing tuple-return function generation)
- [x] CI smoke gate for `gen-verilog` + `yosys read_verilog` on scratch specs (W376)
- [x] CI smoke gate expanded to 25 yosys-clean IGLA specs (W377)
- [x] Struct-field reg mapping from struct-type registers (`pt_x`) instead of parameter-variable registers (`p_x`) (W377)

---

*phi² + 1/phi² = 3 | TRINITY*
