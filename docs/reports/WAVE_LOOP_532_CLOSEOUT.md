# Wave Loop 532 Close-Out Report — Signed scalar-array struct fields

**Date:** 2026-07-07  
**Issue:** #1503  
**Branch:** `wave-loop-532`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend the Icarus-lowerable packed-vector subset to scalar structs whose fields
are fixed-size **signed** scalar arrays (`[N]i8`, `[N]i16`, `[N]i32`), and to
2-D arrays of such structs. Close the largest remaining source of
`UNSUPPORTED_ICARUS` placeholders in real t27 specs while keeping the existing
packed-vector architecture intact.

---

## What changed

### `bootstrap/src/compiler.rs`

- Added `scalar_field_width`, `scalar_field_is_signed`, and `scalar_array_info`
  helpers so that scalar-struct fields of the form `[N]<scalar>` are sized and
  signed correctly for packed-vector layout.
- Updated `ExprStructLit` emission to render array-typed fields as nested
  concatenations of per-element sized values in Verilog MSB-first order.
- Added `emit_packed_scalar_value`, `emit_packed_struct_field_value`, and
  `emit_packed_array_element_value` so that primitive scalars, signed array
  fields, and scalar-struct array elements all lower to width-correct
  concatenation pieces.
- Added `try_emit_struct_array_field_element_access` to lower `p.data[k]` and
  `grid[i][j].data[k]` as a single dynamic part-select:
  `base[(linear*elem_width + field_offset + inner_index*inner_elem_width) +: inner_width]`.
- Preserved 1-D array-of-scalar-struct flattening for bare scalar fields by
  keeping `try_emit_struct_array_access` on the multi-dimensional path.
- Fixed signed negative literals in packed concatenations: they now emit as
  `-{w}'sd{abs}` instead of the unsupported `{w}'sd-{value}` or the
  width-ambiguous `$signed(-value)` fallback.
- Allowed colon (`field: value`) as well as equals in on-demand re-parsing of
  stored array-literal text, so module-level `const` initializers with the
  t27-native colon syntax lower correctly.
- Added `VerilogCodegen::is_lowerable_scalar_struct` and used it in
  `detect_unsupported_verilog_locals` and in `gen_verilog_struct` to emit an
  `// UNSUPPORTED_ICARUS` marker for structs that contain string, enum, float, or
  other non-scalar fields. This keeps the Icarus classifier aligned with what
  the backend can actually pack.

### New scratch witnesses

| Spec | Purpose |
|------|---------|
| `specs/scratch/w532_unsigned_struct_array_field_2d_read.t27` | Regression for unsigned `[2][3]Pt{ data: [3]u16 }` read |
| `specs/scratch/w532_signed_struct_array_field_2d_read.t27` | Read an element from a 2-D signed array-field |
| `specs/scratch/w532_signed_struct_array_field_2d_copy.t27` | Copy module const into local 2-D signed array field |
| `specs/scratch/w532_signed_struct_array_field_param.t27` | Pass a scalar struct with signed array field as parameter |
| `specs/scratch/w532_signed_struct_array_field_return.t27` | Return a scalar struct with signed array field by value |
| `specs/scratch/w532_negative_enum_field.t27` | Struct with enum field must stay non-lowerable |
| `specs/scratch/w532_negative_string_field.t27` | Struct with string field must stay non-lowerable |

### Seals and baselines

- Resealed all affected specs (the `gen_hash_verilog` changed for the broad set
  of specs that contain struct declarations or signed literal emission).
- Recorded Icarus JSON baselines for the five lowerable W532 witnesses under
  `.trinity/icarus-baselines/`.
- Updated `bootstrap/stage0/FROZEN_HASH` to the live compiler hash.

---

## Verification

- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: **1494 passed; 0 failed; 2 ignored**.
- `cargo test -p tri`: **78 passed; 0 failed**.
- `./scripts/tri test --icarus-simulate --icarus-lowerable`:
  - Icarus simulation: **28 passed, 0 failed**.
  - Seal verify: **593 passed, 0 mismatches**.
  - Gen Verilog smoke: **50 passed, 23 failed** (pre-existing baseline failures
    in old scratch and keyword-collision igla specs; none introduced by W532).

All five positive W532 witnesses pass Icarus simulation; the two negative
witnesses carry `// UNSUPPORTED_ICARUS` markers and are correctly classified as
non-lowerable.

---

## Patterns to reuse

- When a packed-vector element can be either a primitive scalar or a
  fixed-size scalar array, compute width/sign per field and emit each field as
  its own concatenation sub-tree rather than trying to reuse the primitive
  scalar path.
- For signed values inside packed concatenations, emit a sized signed literal
  (`-16'sd2`) so the result occupies exactly the declared element width and
  avoids Icarus's rejection of `16'sd-2` or the width ambiguity of
  `$signed(-2)`.
- Dynamic part-selects for array-field elements must scale the inner index by
  the inner element width; adding the index directly selects a bit, not a word.
- Keep 1-D array-of-struct flattening for bare scalar fields; introduce a
  separate helper only for the new shape (inner index into an array field)
  to avoid regressing existing HIR parity tests.
- Reject non-lowerable struct fields explicitly with an `UNSUPPORTED_ICARUS`
  marker so the classifier stays honest even when the generated Verilog
  degrades gracefully for host-only use.

---

## Anti-patterns to avoid

- Do not emit signed negative values as `{width}'d{negative}`; Icarus rejects
  the syntax and the value is silently truncated or misinterpreted.
- Do not wrap signed expressions in `$signed(...)` inside a packed concatenation
  without an explicit width; it returns a 32-bit signed value and corrupts the
  packed layout.
- Do not change the array-of-struct access helper to handle 1-D arrays unless
  you also update the HIR flattening tests; the existing path intentionally
  flattens 1-D arrays into per-field registers.
- Do not reseal only the new specs; any change to generated-code shape usually
  changes `gen_hash_verilog` for a large fraction of the corpus.

---

## Next wave setup

- Created `docs/reports/FPGA_LOOP_COOPERATION_W533_2026-07-07.md` with three
  cooperation variants.
- Updated `.trinity/current-issue.md` to Wave Loop 533.

---

*φ² + φ⁻² = 3 | TRINITY*
