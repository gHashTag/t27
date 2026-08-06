# `gen-verilog` Backend — Known Defects and Roadmap

**Branch:** `trinity-rust-rings`  
**Last updated:** 2026-07-01 (Wave Loop 388)  

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

### Defect 3b — Named tuple return types with `::` namespaces (FIXED in W380)

**Symptom:** The new tuple return-type parser introduced in W380 entered an infinite loop on named/namespaced tuple elements such as `-> (gf16::GF16, gf16::GF16, gf16::GF16)` and `-> (added: u32, deleted: u32, modified: u32)`.

**Root cause:** The initial tuple parser consumed `Ident` followed by a single `:` as a named-field label. For namespaced types like `gf16::GF16`, the first colon belongs to `::`, so the parser consumed the namespace identifier and half of the namespace separator, leaving a bare `:` that caused an infinite loop.

**Fix (W380):** The tuple return-type loop now parses the element type first, then detects a named-field label only when the next token is a single colon whose successor is **not** another colon (i.e., not `::`).

**Verification:** `specs/ml/optimizer/adamw.t27` and `specs/git/diff.t27` now parse cleanly; full `t27c suite` passes 560/560.

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

### Defect 6 — `let` destructuring is emitted verbatim (SEMANTICALLY-AWARE SYNTAX FIX in W378/W379)

**Repro:**
```t27
fn cordic_top_batch_inner(angles : u32, idx : u32, acc : i32) -> i32 {
    if (idx >= angles.len()) { return acc; }
    let(s, _c, _r) = cordic_top(1, 1, angles[idx], 1);
    return cordic_top_batch_inner(angles, idx + 1, acc + s);
}
```

**Observed Verilog (before W378):** the `let(s, _c, _r) = ...` statement was emitted verbatim as `let(s, _c, _r) = cordic_top(...);`, which is not valid Verilog and caused `yosys read_verilog` to fail with a syntax error.

**Observed Verilog (after W378/W379):** the codegen detects the `let(...)` pattern in `StmtAssign` and emits a packed temporary plus scalar `reg` declarations and slice assignments. After W379 the packed width and slice offsets are inferred from the LHS pattern rather than hardcoded:
```verilog
// 3 bindings (W378 example)
reg [95:0] _let_tmp_0;
_let_tmp_0 = cordic_top(...);
reg [31:0] s;  s = _let_tmp_0[95:64];
reg [31:0] _c; _c = _let_tmp_0[63:32];
reg [31:0] _r; _r = _let_tmp_0[31:0];

// 2 bindings (W379 regression)
reg [63:0] _let_tmp_0;
_let_tmp_0 = make_pair(...);
reg [31:0] x; x = _let_tmp_0[63:32];
reg [31:0] _y; _y = _let_tmp_0[31:0];
```

**Root cause (syntax level):** `gen_verilog_stmt` did not recognize the `let(...)` call pattern on the LHS of an assignment and emitted it verbatim.

**Fix (W378/W379):**
- W378: Added `let_tmp_counter` to `VerilogCodegen`; added `gen_verilog_let_destructuring` helper; routed `let(...)` LHS patterns to it in `StmtAssign`; reset the counter per function.
- W379: Generalized the helper so it infers:
  - `N` from the number of identifier children in the `let(...)` LHS.
  - Per-binding width from `child.extra_type` when present, falling back to 32 bits.
  - Total packed width as the sum of per-binding widths.
  - Slice offsets computed from the running cursor, not hardcoded 32-bit slots.

**Remaining semantic gap (before W380):** the backend still does not implement first-class tuple-return function generation. The LHS pattern is used to size the packed temporary, but the RHS function call must already return a value of the matching shape. Full semantic correctness requires multi-return function types, tuple literals, and slot-aware function-call lowering.

**Update (W380):** Tuple-return generation scaffolding is now in place:
- Parser accepts tuple return types `-> (T1, T2, ...)` and tuple literals `(a, b, c)`.
- `gen_verilog_fn` emits a packed function result register whose width equals the sum of element widths.
- `gen_verilog_expr` for tuple literals emits packed concatenations.
- `gen_verilog_let_destructuring` infers per-binding widths from the callee's tuple return type when LHS bindings are untyped.

**Update (W381):** Slot-aware nested tuple-return call lowering is now complete. `gen_verilog_expr` recognizes tuple-return function calls in expression position, emits a packed temporary sized to the callee's tuple width, and lets the consuming tuple literal slice the temporary by slot. The regression spec `specs/scratch/w381_tuple_call_chain.t27` exercises a two-level chain and passes `yosys read_verilog -sv`.

**Status:** Closed as implemented and verified.

**Verification:**
- `specs/scratch/w378_let_destructuring.t27` — 3-binding `let (x, y, z)` and `let (x, _y)` pass `yosys read_verilog -sv`.
- `specs/scratch/w379_let_destructuring_generalized.t27` — 2-binding and 4-binding patterns pass `yosys read_verilog -sv`.
- `specs/igla/race/cordic.t27` and `specs/igla/race/cordic_top.t27` now pass `yosys read_verilog -sv`.
- `bootstrap/src/suite.rs` smoke gate covers all 27 IGLA specs.

---

## Recommended Triage Order

1. **Tuple-return function generation** — the remaining semantic gap behind Defect 6. Implement multi-return function types, tuple literals, and slot-aware function-call lowering so `let(a, b, c) = f(...)` is correct for arbitrary multi-return calls, not only the current syntax-level workaround.
2. **Incremental array/RAM lowering** — #1258 (datapath specs such as FIFOs and memories).

## Open work after W382

- **Array/RAM sub-gaps remaining:**
  - Multi-dimensional arrays (`[[T; M]; N]`).
  - Non-literal (variable) index access on function-local arrays.
  - RAM style inference / block-vs-distributed pragma hints.
- No other tracked gen-verilog syntax/semantic defects remain on `trinity-rust-rings`.

## Fixed in W382 — Module-level array/RAM lowering

**Symptom:** `var mem : [4]u16` at module scope emitted a scalar `reg [31:0] mem;`, so indexing expressions `mem[i]` were interpreted as scalar bit-selects instead of memory accesses.

**Fix:** `gen_verilog_var` now detects array type annotations via `parse_array_type` and emits a true Verilog memory declaration:
```verilog
reg [15:0] mem [0:3];
```
Read expressions (`mem[i]`) and indexed assignments (`mem[i] = x;`) already emitted valid Verilog syntax and now resolve to memory accesses.

**Verification:** `specs/scratch/w382_ram_lowering.t27` exercises write/read on a 4-entry `u16` memory; `yosys read_verilog -sv` + `synth -top w382_ram_lowering` pass with 0 problems.

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
- [x] `let` destructuring lowering — semantically-aware syntax fix in W378/W379; full semantic tuple-return support completed in W381
- [x] CI smoke gate for `gen-verilog` + `yosys read_verilog` on scratch specs (W376)
- [x] CI smoke gate expanded to 25 yosys-clean IGLA specs (W377)
- [x] CI smoke gate expanded to all 27 IGLA specs (W378)
- [x] Struct-field reg mapping from struct-type registers (`pt_x`) instead of parameter-variable registers (`p_x`) (W377)
- [x] Tuple-return function generation for full semantic multi-return support (W380/W381)
- [x] Module-level array/RAM lowering — `var mem : [N]T`, read `mem[i]`, write `mem[i] = x` (W382)
- [x] Module-level ROM lowering — `const lut : [N]T = [N]T{...}` (W383)
- [x] Function-local array variables with numeric-literal index access (W383)
- [x] Function-local array variable-index access — read via priority mux, write via if-else chain, full-token keyword escape (W384)
- [x] Function-local array signed element types (`[N]i8`, `[N]i16`, etc.) (W385)
- [x] Function-local array literal initialization at declaration time (`var buf : [N]T = [N]T{...}`) (W385)
- [x] Function-local arrays inside `for` loops, constant and parameter bounds (W386)
- [x] Multi-dimensional function-local arrays with numeric/variable indices and signed elements (W387)
- [x] Multi-dimensional function-local array-literal initialization (`var m : [2][3]u16 = [2][3]u16{...}`) (W388)

## Fixed in W384 — Function-local array variable-index access

**Symptom:** Local array declarations inside a function, e.g. `var buf : [4]u16`, could only be accessed with numeric-literal indices (`buf[0]`). A variable index such as `buf[idx]` was emitted as `buf[idx]`, which is invalid Verilog because `buf` had already been lowered to per-element registers `buf_0`, `buf_1`, ... by W383.

**Fix:** `bootstrap/src/compiler.rs` now tracks function-local arrays and emits synthesizable variable-index access:
- Read: priority mux chain `((idx == 0) ? buf_0 : ((idx == 1) ? buf_1 : ...))`.
- Write: if-else chain `if (idx == 0) begin buf_0 = val; end else if (idx == 1) begin buf_1 = val; end ...`.
- Keyword escape is applied to the full flattened token (`\buf_0 `) rather than the base name alone, preventing tokenization bugs with keyword-named arrays such as `buf`.

**Verification:** `specs/scratch/w384_variable_index.t27` exercises variable-index read and write on `var buf : [4]u16` with keyword-named array `buf`. `t27c gen-verilog` + `yosys read_verilog -sv` + `synth -top w384_variable_index` pass.

## Fixed in W385 — Signed element types and array-literal initialization for function-local arrays

**Symptom:** Function-local arrays only supported unsigned element types and could not be initialized at declaration with an array literal. `var temps : [4]i16` emitted signed regs but had no regression coverage, and `var buf : [4]u16 = [4]u16{...}` emitted a broken TODO placeholder instead of initializing the per-element regs.

**Fix:** `bootstrap/src/compiler.rs` now detects an `ExprArrayLiteral` initializer on a function-local array and emits a scalar assignment for each element to the corresponding per-element reg. Width padding is applied to `0x` and `0b` element literals. Signed element types already worked via the existing `elem_signed` path; W385 added regression coverage and verified sign extension through `yosys`.

**Verification:**
- `specs/scratch/w385_signed_local_array.t27` — signed `i16` local array with variable-index read/write.
- `specs/scratch/w385_local_array_init.t27` — `u16` local array initialized from `[4]u16{...}`.
- `specs/scratch/w385_signed_local_array_init.t27` — combined signed `i16` array with initializer.
- All three pass `t27c gen-verilog` + `yosys read_verilog -sv` + `synth`.

## Fixed in W386 — Function-local arrays inside `for` loops

**Symptom:** No regression coverage existed for using function-local arrays inside `for` loops, even though the W384 variable-index lowering and W385 signed/init lowering made the pattern feasible. Without smoke-gate coverage the feature could regress silently.

**Observation:** The existing backend already lowered the pattern correctly:
- Constant-bound loops (`for i in 0..4`) are unrolled into scalar per-element assignments.
- Parameter-bound loops (`for i in 0..n`) remain as Verilog `for` loops with variable-index reads via priority mux chains and writes via if-else chains.

**Fix:** Added regression specs only; no compiler change was required.

**Verification:**
- `specs/scratch/w386_for_local_array.t27` — constant-bound fill-and-sum and copy-reverse on `[4]u16`.
- `specs/scratch/w386_for_local_array_i8.t27` — constant-bound signed `[4]i8` sum and in-place negation.
- `specs/scratch/w386_for_local_array_param.t27` — parameter-bound loop with variable-index write/read on `[4]u16`.
- All three pass `t27c gen-verilog` + `yosys read_verilog -sv` + `synth`.

## Fixed in W387 — Multi-dimensional function-local arrays

**Symptom:** Declarations such as `var m : [2][3]u16` were parsed and typechecked, but the gen-verilog backend treated them as a 1D array of array-typed elements. It emitted per-row regs with the default 32-bit width and accessed elements via Verilog bit-selects (`m_0[0]`, `m_0[1]`), which silently corrupted data widths.

**Fix:** `bootstrap/src/compiler.rs` now parses the full dimension list, flattens multi-dimensional arrays into per-element regs in row-major order, and lowers nested index chains (`m[r][c]`) to a linear offset.

- Numeric constant indices (`m[1][2]`) resolve directly to the flattened reg (`m_5`).
- Variable indices (`m[row][col]`) emit a priority mux chain over all flattened regs using the linearized expression `(row * 3) + col` as the select.
- Signed leaf element types and nested `for` loops over 2D arrays work without additional changes.
- Non-local-array constant index fallback (`base_idx`) is preserved so module-level arrays and slice parameters are unaffected.

**Verification:**
- `specs/scratch/w387_2d_local_array.t27` — numeric-index read/write.
- `specs/scratch/w387_2d_local_array_varidx.t27` — variable-index read/write.
- `specs/scratch/w387_2d_local_array_signed.t27` — signed `[2][3]i8` sum.
- `specs/scratch/w387_2d_local_array_for.t27` — nested loops filling/summing a 2D array.
- All four pass `t27c gen-verilog` + `yosys read_verilog -sv` + `synth`.

**Limitation:** multi-dimensional array-literal initialization (`var m : [2][3]u16 = [2][3]u16{...}`) is not yet supported by the parser and is tracked as remaining work.

## Fixed in W388 — Multi-dimensional array-literal initialization

**Symptom:** The parser did not recognize array-literal syntax for multi-dimensional function-local arrays. A declaration such as `var m : [2][3]u16 = [2][3]u16{1, 2, 3, 4, 5, 6}` parsed the right-hand side as an index operation with an empty literal, dropping the six initializer values.

**Fix:** `bootstrap/src/compiler.rs` `parse_array_literal` now consumes the additional `[N]` dimensions and the base element type before the `{...}` block. The resulting `ExprArrayLiteral` carries the full dimension/type suffix in `extra_type`, and all initializer expressions are preserved as children.

**Verification:**
- `specs/scratch/w388_2d_local_array_init.t27` declares, reads, and writes a `[2][3]u16` initialized from a literal.
- `t27c gen-verilog` + `yosys read_verilog -sv` + `synth` pass; the backend emits six per-element reg assignments in row-major order.

## Residual yosys smoke failures on `wave-loop-*` branches (W422–W427)

The `trinity-rust-rings`/`wave-loop-*` branch carries the same `gen-verilog`
backend as `master` up to the W422 keyword-escape fix. After W422 the yosys
smoke gate regressed on **7 specs** because the full fix set for tuple-return,
`let` destructuring, ROM arrays, and CORDIC structural changes lives only on
`master` (commit `701d79b3b`). The wave-loop strategy is to apply only narrow,
regression-free sub-fixes; none of these 7 failures is narrow enough for a
single wave.

### Failing specs

| Spec | Failure mode | Why it is not a safe single-wave fix |
|---|---|---|
| `specs/igla/race/cordic.t27` | `syntax error, unexpected '='` | CORDIC uses tuple-return / `let` destructuring; a syntax fix would require re-landing the W380–W381 tuple-return generation scaffolding. |
| `specs/igla/race/cordic_top.t27` | `syntax error, unexpected '='` | Same CORDIC/tuple-return dependency as `cordic.t27`. |
| `specs/scratch/w378_let_destructuring.t27` | `syntax error, unexpected '='` | Requires the full semantically-aware `let` destructuring lowering (W378/W379) plus tuple-return call lowering (W381). |
| `specs/scratch/w379_let_destructuring_generalized.t27` | `syntax error, unexpected '='` | Generalized `let` destructuring; same broad dependency. |
| `specs/scratch/w380_tuple_return.t27` | `syntax error, unexpected '='` | Tuple return generation (W380) is a major feature, not a narrow sub-fix. |
| `specs/scratch/w381_tuple_call_chain.t27` | `syntax error, unexpected '='` | Slot-aware nested tuple-return call lowering (W381). |
| `specs/scratch/w383_rom_array.t27` | `syntax error, unexpected '['` | Module-level ROM array lowering (W383) changes how `const lut : [N]T = ...` is emitted. |

### Triage decision for W427

**Deferred.** The fix set on `master` (`701d79b3b`) is broad and touches the
same major features. Landing it as a single wave on `wave-loop-427` would
violate the narrow-sub-fix safety rule and risk destabilizing the current
FPGA/formal work. The failures are tracked here and will be resolved by either:

1. Merging `master` into the wave-loop branch in a dedicated merge/rebase wave
   after W427 closes, or
2. Cherry-picking the exact fix commits once the FPGA boot-evidence line is no
   longer the primary wave focus.

The 7-failure count is accepted as a known, documented baseline for W427.

## Open work after W388 / W427

- **Array/RAM sub-gaps remaining:**
  - RAM style inference / block-vs-distributed pragma hints.
- **Merge `master` fix set (`701d79b3b`) into wave-loop branch** to clear the
  7 residual yosys smoke failures.

---

*phi² + 1/phi² = 3 | TRINITY*
