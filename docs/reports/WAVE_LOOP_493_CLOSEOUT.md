# Wave Loop 493 Close-Out Report

**Issue:** #1463 (closed by this wave)  
**Branch:** `wave-loop-493`  
**Variant selected:** B — gen-verilog struct/call lowering hardening with formal follow-through  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What was attempted

Wave Loop 493 continued the gen-verilog backend hardening track (Variant B from
the W493 cooperation plan). The concrete goals were:

1. Close the two documented adversarial baseline witnesses left over from W491/W492.
2. Add a new adversarial witness that exercises the *next* unsupported boundary.
3. Keep the Icarus-lowerability classifier and the Lean 4 completeness gate in sync.
4. Produce three cooperation variants for the next wave.

---

## 2. What actually happened

### 2.1 The W492 "failure" was a false alarm

The file `specs/scratch/w492_predicate_rejects_nested_return_field.t27` was
supposed to be an adversarial witness for nested-struct-return field access
(`make_outer().inner.v`). It was emitted with an empty function body and a
`// TODO: implement` comment.

Root cause: the spec used indented `;` comments. The t27 lexer only treats `;`
as a line comment when it appears in column 1; an indented `;` is tokenized as a
`Semicolon` token. The parser recovery in `parse_fn_body` repeatedly failed on
the unexpected semicolons and dropped the entire body, leaving it empty.

Fix: rename the file to `w492_nested_return_field_positive.t27`, rewrite all
comments to `//`, and confirm that `make_outer().inner.v` already lowers to a
packed-vector slice and passes Icarus smoke.

### 2.2 The W491 struct-literal-field bug was real

The file `specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`
lowered `Outer { x: inner }` (where `inner : Inner` is a scalar-struct
parameter) to malformed Verilog:

```verilog
make_outer = {0 /* Outer {...} */;
```

Root cause: `emit_struct_literal_leaf` only accepted nested struct field values
that were either a nested `ExprStructLit` or a struct-return `ExprCall`. It did
not accept a bare `ExprIdentifier` whose type is a scalar struct.

Fix: extend `emit_struct_literal_leaf` in `bootstrap/src/compiler.rs` to emit
scalar-struct identifiers (function parameters, packed local variables,
module-level constants/variables) as a single packed-vector concatenation
operand. The same path also accepts literal-index elements of module-level
arrays of structs, because those are lowered to flat memories and can be packed
by `gen_verilog_pack_scalar_struct_expr`.

The fixed witness was renamed to
`w493_nested_struct_field_from_identifier_lowerable.t27`.

### 2.3 Placeholder cleanup

The struct-literal fallback (`ExprStructLit` that `try_emit_struct_literal_packed`
cannot pack) used to emit `0 /* Name {...} */`, which left an unclosed `{` in the
output and was invisible to the Icarus-lowerability classifier.

Refactored `try_emit_struct_literal_packed` to build the concatenation in a
temporary buffer and only commit it on success. The fallback now emits a sized
zero with an `UNSUPPORTED_ICARUS:` marker, so the classifier and smoke gate
agree on the boundary.

### 2.4 New adversarial and positive witnesses

Added four new scratch specs:

| Spec | Role | Smoke |
|------|------|-------|
| `w493_nested_struct_field_from_identifier_lowerable.t27` | Positive: parameter as struct-literal field | PASS |
| `w493_local_scalar_struct_field_lowerable.t27` | Positive: local packed struct var as struct-literal field | PASS |
| `w493_module_scalar_struct_field_lowerable.t27` | Positive: module const as struct-literal field | PASS |
| `w493_module_aos_element_field_lowerable.t27` | Positive: module-level AOS literal-index element as struct-literal field | PASS |
| `w493_local_aos_element_field_not_lowerable.t27` | Adversarial: indexed *local* AOS element as struct-literal field | documented baseline |

The remaining boundary is local non-memory-mode arrays of structs: they are
unpacked into per-element per-field registers, and packing an indexed element
inside a struct-literal concatenation is not yet supported.

### 2.5 Baselines updated

- `docs/reports/gen_verilog_smoke_baseline.json` — yosys baseline reduced from 1
to 0 (the old W491 malformed-syntax witness is now lowerable).
- `docs/reports/gen_verilog_iverilog_smoke_baseline.json` — Icarus baseline
reduced from 2 to 1, now tracking only
`w493_local_aos_element_field_not_lowerable.t27`.

---

## 3. Verification numbers

| Gate | Result |
|------|--------|
| `./scripts/tri test --fast --icarus-lowerable` | **697 / 697 non-smoke PASS**, 0 seal mismatches, 0 Icarus disagreements |
| Icarus smoke | **176 / 177 PASS** (1 documented baseline failure) |
| Yosys smoke | **177 / 177 PASS** (0 baseline failures) |
| `cargo test -p t27c --bin t27c` | **1525 / 0 / 2** |
| `tri verify --lean-lowerable` | **green**, 253 specs in `Completeness.lean` |

The `Completeness.lean` count stayed at 253: the newly-lowerable witnesses were
offset by specs that were previously misclassified as lowerable because the old
struct-literal fallback had no `UNSUPPORTED_ICARUS` marker. The cleanup makes
the modeled set strictly more honest.

---

## 4. Files changed

- `bootstrap/src/compiler.rs` — struct-literal lowering, packed-vector operand
  handling, placeholder cleanup.
- `docs/reports/gen_verilog_smoke_baseline.json` — yosys baseline updated.
- `docs/reports/gen_verilog_iverilog_smoke_baseline.json` — Icarus baseline updated.
- `specs/scratch/w492_nested_return_field_positive.t27` — renamed and fixed.
- `specs/scratch/w493_nested_struct_field_from_identifier_lowerable.t27` — renamed from W491, fixed.
- `specs/scratch/w493_local_scalar_struct_field_lowerable.t27` — new positive witness.
- `specs/scratch/w493_module_scalar_struct_field_lowerable.t27` — new positive witness.
- `specs/scratch/w493_module_aos_element_field_lowerable.t27` — new positive witness.
- `specs/scratch/w493_local_aos_element_field_not_lowerable.t27` — new adversarial witness.
- `.trinity/seals/` — resealed after compiler change.
- `.claude/plans/wave-loop-493.md` — updated with completion markers.

---

## 5. Lessons for the next wave

1. **Comment syntax is part of the language surface.** A witness that uses
   comment characters in the wrong column can mislead diagnosis for hours.
2. **Classifier-visible markers must be emitted on every unsupported path.**
   Silent placeholders create disagreement between the predicate and the smoke
   gate.
3. **Scalar-struct lowering has three shapes:** packed-vector parameters, packed
   local regs, and per-field module constants. Each shape needs its own
   identifier-emitting rule when used as a whole value.
4. **AOS lowering has a memory/register mode split.** Module-level AOS uses flat
   memories; local AOS without array-typed fields is unpacked into per-element
   per-field registers. Crossing that boundary inside a struct literal is the
   next concrete gap.

---

*φ² + φ⁻² = 3 | TRINITY*
