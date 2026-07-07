## 2026-07-07 — Wave Loop 469 (gen-verilog struct/array hardening)

**Issue:** #1447  
**Branch:** `wave-loop-469`

### Insight
Scalar structs, arrays of structs, and multi-dimensional arrays of structs can all be lowered through the same per-field register abstraction. The key is to keep expression-context packing (concatenation of field values) separate from declaration-context flattening (per-field regs/memories).

### Pattern
- Use `struct_return_width` and `flatten_struct_fields` to derive the exact packed width and field layout of any declared struct.
- For keyword-sensitive names, concatenate the raw base with the suffix before calling `verilog_safe_identifier`; otherwise escaped names like `\task _id` break Verilog syntax.
- For 2D struct arrays, compute total leaf count recursively and flatten the `ExprFieldAccess` → `ExprIndex` chain before generating the per-field mux/read.

### Anti-pattern
- Do not assume an array-parameter function call site inside another function will be detected; only module-level/test/bench sites and inner calls whose arguments are outer array-parameter identifiers are propagated.
- Do not skip resealing `FROZEN_HASH` after touching `bootstrap/src/compiler.rs`; the NMSE SSOT must be recertified in the same wave.
