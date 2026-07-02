# `gen-verilog` Backend — Known Defects and Roadmap

**Branch:** `trinity-rust-rings`  
**Last updated:** 2026-07-02 (Wave Loop 370)  

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

---

## Remaining Defects

### Defect 3 — Early `return` inside bare `if` drops the rest of the function body

**Repro:**
```t27
fn sign(x : i8) -> i8 {
    if (x < 0) { return -1; }
    if (x > 0) { return 1; }
    return 0;
}
```

**Observed Verilog:** only the first `return` is generated; subsequent statements are missing or commented out.

**Root cause:** the bare `if` (no `else`) path in `gen_verilog_if_stmt` does not preserve fall-through control flow.

**Wave-safe fix order:** medium priority; affects functions with multiple early exits.

---

### Defect 4 — `as` cast and bitwise operators drop the operand body

**Repro:**
```t27
fn cast_and_mask(x : u16) -> u8 {
    return (x as u8) & 0x0F;
}
```

**Observed Verilog:** the assignment omits the inner expression, producing a placeholder or truncated result.

**Root cause:** operator-lowering for `as` and bitwise `&` / `|` / `^` / `~` is incomplete in `gen_verilog_expr`.

**Wave-safe fix order:** medium-high priority; blocks many numeric idioms.

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

---

## Recommended Triage Order

1. **Defect 3** — early return fall-through; needed for control-flow-heavy specs. Small scope, but must not break existing if-statement emission.
2. **Defect 4** — cast/bitwise operators; needed for numeric idioms. Larger lowering change; needs a scratch spec and `yosys` verification.
3. **Defect 5** — struct fields; defer until struct usage grows.
4. **CI smoke gate** — add `t27c gen-verilog` + `yosys read_verilog` regression tests once the remaining safe parser/lowering fixes are landed; L7 UNITY prohibits new shell scripts on the critical path.

---

## Verification Checklist

- [x] `0x` scalar width padding (`const`, `var`, `let`, `return`)
- [x] `0b` scalar width padding (`const`, `var`, `let`, `return`)
- [x] Multiple `const` declarations
- [ ] Early `return` fall-through
- [ ] `as` / bitwise operator body preservation
- [ ] Struct-field reg naming

---

*phi² + 1/phi² = 3 | TRINITY*
