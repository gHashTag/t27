# Wave Loop 519 — Closeout Report

**Issue:** #1488
**Branch:** `wave-loop-519`
**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Selected variant

**Variant A (recommended):** add packed scalar struct equality / inequality
operators in the Icarus-lowerable subset.

### Actual focus discovered during implementation

Equality and inequality for scalar structs (`==`, `!=`) were already lowered
correctly by the W470 path. The real remaining gap was the **ordering
operators** (`<`, `<=`, `>`, `>=`) on local scalar struct variables: they fell
through to the generic `ExprBinary` path and produced illegal Verilog such as
`(a < b)`, where `a` and `b` are emitted as per-field registers, not as a
single net. W519 therefore broadened the W470 scalar-struct special case to
all six relational operators.

---

## What changed

### 1. Broadened scalar-struct / AOS comparison lowering

In `bootstrap/src/compiler.rs`, the `NodeKind::ExprBinary` branch previously
special-cased only `==` and `!=`. It now triggers for every relational operator
(`==`, `!=`, `<`, `<=`, `>`, `>=`) via a new `is_comparison_op` helper.

For scalar structs, both operands are passed through
`gen_verilog_pack_scalar_struct_expr`, producing a contiguous packed-vector
comparison such as:

```verilog
{a_x, a_y} < {b_x, b_y}
```

For small arrays-of-structs, the existing W474/W475 packed-array path is
reused with the operator taken directly from the source expression.

This change is valid under IEEE Std 1800-2017: a packed structure is treated
as a single vector (§7.2.1), aggregate expressions may be compared with
relational operators (§11.2.2, §11.4.4, §11.4.5), and equivalent bit-vector
types compare bit-for-bit (§6.22.2).

Files touched: `bootstrap/src/compiler.rs`.

### 2. Regression / integration test

Added `bootstrap/tests/w519_struct_order_verilog.rs`, which compiles a tiny
probe spec and asserts that:

- the local-variable scalar struct function emits packed-vector comparisons
  for all six relational operators;
- the generated Verilog contains no `UNSUPPORTED_ICARUS` or `TODO` markers.

### 3. Witness specs

Added three scratch witnesses in `specs/scratch/`:

- `w519_struct_order_local.t27` — local scalar struct variables compared
  against literals and other locals, including nested structs.
- `w519_struct_order_param.t27` — scalar struct parameters and return values
  compared in function bodies.
- `w519_struct_order_module.t27` — module-level scalar struct `const`/`var`
  comparisons, including a W509-style scalar struct with a fixed-size scalar
  array field (`Buf.data : [4]u8`).

All three specs were sealed under `.trinity/seals/`.

---

## Validation

| Gate | Result |
|------|--------|
| `cargo build --release` | ✅ |
| `cargo test -p t27c --bin t27c` | 1525 passed, 0 failed, 2 ignored |
| `cargo test -p t27c --tests` | ✅ all integration tests pass |
| `./scripts/tri test --icarus-lowerable --fast` | 0 failures, 0 seal mismatches, 0 yosys/Icarus baseline failures |
| `./scripts/tri verify --lean-lowerable` | ✅ passed (251 lowerable specs) |
| Manual Icarus smoke for the three new witnesses | ✅ all pass |

Suite summary:

```
Parse failures:           0
Typecheck fails:          0
Gen Verilog fails:        0
Gen Verilog smoke fails:  0
Gen Verilog Icarus fails: 0
Seal mismatches:          0
Icarus lowerable:         227
Icarus not lowerable:     0
Icarus disagreements:     0
TOTAL FAILURES:           0
```

---

## Scientific background

- IEEE Std 1800-2017, §7.2.1: packed structures are treated as a single
  vector when used as a primary.
- IEEE Std 1800-2017, §11.2.2 / §11.4.4 / §11.4.5: aggregate expressions may
  be copied and compared with relational and equality operators.
- IEEE Std 1800-2017, §6.22.2: packed arrays, packed structures, packed
  unions, and built-in integral types are equivalent if they share the same
  total bit width, state model, and signedness.
- Sutherland / DVCon, *"Can My Synthesis Compiler Do That?"*: packed structs
  can be assigned, passed through ports, and compared with `==` / `!=` in
  synthesizable RTL.
- AMD Vivado Synthesis UG901 confirms packed/unpacked structures and operator
  support on aggregate expressions.

Sources:
- [IEEE 1800-2017 standard (MIT mirror)](https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf)
- [DVCon "Can My Synthesis Compiler Do That?"](https://lcdm-eng.com/papers/2014-DVCon_ASIC-FPGA_SV_Synthesis_paper.pdf)
- [AMD Vivado Synthesis UG901 — SystemVerilog Constructs](https://docs.amd.com/r/2022.1-English/ug901-vivado-synthesis/SystemVerilog-Constructs)
- [SystemVerilog.dev relational operators reference](https://systemverilog.dev/2.html)

---

## Residual boundaries carried forward

- Multi-dimensional packed AOS parameters with array-typed fields deeper than
  one struct level still lack dedicated witness coverage.
- Formal proofs for scalar struct ordering comparisons in the Lean 4
  IcarusLowerable soundness module are not yet written; the existing
  `module_value_equiv_proved_sequential` machinery should cover them once a
  witness is added to the completeness set.

---

*φ² + φ⁻² = 3 | TRINITY*
