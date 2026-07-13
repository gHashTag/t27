# FPGA/Wave Loop Cooperation Variants — W518

**Date:** 2026-07-07  
**From:** Wave Loop 517 Closeout  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 517 closed the packed-array-of-structs parameter boundary: an AOS
parameter whose element has an array-typed field can now be passed as a single
packed vector, and the whole field (`arr[i].coords`) can be returned as a packed
vector. The next loop should attack one of the three remaining concrete
boundaries below. Each variant is independent, reviewable in isolation, and
leaves the repo in a green state.

---

## Variant A — Clear the remaining W508 `break`/`continue` smoke baselines (recommended)

**Goal:** Remove the last 2 yosys and 3 Icarus smoke failures inherited from
Wave Loop 508's bounded-loop early-exit work.

**Why this is the natural next step:**

- The W508 feature is otherwise complete: the Lean model, predicate, Rust
  classifier, and Verilog emitter all support `break`/`continue` in bounded
  loops.
- The residual failures are isolated syntax/scoping issues in generated
  Verilog:
  - yosys rejects `disable` label usage in `w508_break_nested` and
    `w508_break_search`.
  - Icarus rejects/simulation-mismatches `w508_continue_sum`, while
    `w468_local_ram_style` and `w514_function_local_packed_aos_ram_style` are
    documented Icarus limitations around function-local attributes.
- A zero-baseline smoke gate simplifies every future wave and removes the need
  to maintain a growing baseline file.

**Deliverables:**

- Re-examine the generated Verilog for the 5 baseline specs and tighten the
  loop/attribute emitter for yosys/Icarus compatibility.
- Update the baseline file to zero documented failures.
- Add one adversarial witness mixing `break`/`continue` with W517 packed-vector
  return paths.
- `cargo test`, `tri test --icarus-lowerable`, and `tri verify --lean-lowerable`
  green.

**Risk:** low-to-medium — narrowly scoped, but may require touching the loop
emitter, the Icarus baseline loader, and possibly the Lean loop model.

---

## Variant B — Packed scalar struct equality / comparison operators

**Goal:** Add `==` and `!=` for packed scalar structs in the Icarus-lowerable
subset, lowering to a bit-vector equality of the packed representation.

**Why this matters:**

- Struct equality is a frequent request and currently falls back to host-only
  helpers or `UNSUPPORTED_ICARUS` placeholders.
- The packed representation already makes equality a single Verilog `==` over
  the concatenated leaf fields, so the emitter change is small once the
  type-system path accepts the operator.
- It pairs naturally with the W515/W516/W517 packed-struct work and is
  synthesizable per IEEE Std 1800-2017 §11.4 and Sutherland's packed-array
  guidance (SNUG 2013).

**Deliverables:**

- Allow `a == b` and `a != b` where `a` and `b` are packed scalar struct
  values in Icarus-lowerable contexts.
- Generate equality as packed-vector comparison (or leaf-wise equality if the
  structural layout requires it).
- 2–3 scratch witnesses: module-level struct equality, function-return struct
  equality, and an invariant using struct equality.
- Update the Lean Icarus-lowerability predicate if the operator needs to be
  modeled explicitly.
- Reseal affected specs.

**Risk:** medium — requires parser/typecheck and Verilog backend alignment,
but the proof surface is limited.

---

## Variant C — Deeper / multi-dimensional packed AOS with array-typed fields

**Goal:** Extend W517 to AOS parameters whose element struct contains a
multi-dimensional array field or a nested struct whose own field is an array,
and return those sub-fields as packed vectors.

**Why this matters:**

- W517 proved the one-level case (`arr[i].coords` where `coords : [3]u32`).
- Multi-level cases (`arr[i].inner.coords`, `arr[i].m[2]`) exercise the
  recursive packed layout and index arithmetic that the current helpers
  already encode, but no witness currently covers them.
- IEEE Std 1800-2017 §7.4 treats packed arrays as contiguous vectors, so
  multi-dimensional packed slices are legal; the remaining work is mostly in
  the field-offset calculation and the `array_of_struct_field_slice` helper.

**Deliverables:**

- 2 scratch witnesses covering 2-D/3-D scalar array fields inside AOS
  parameters and nested struct-array fields.
- Generalize `array_of_struct_field_slice` and the packed-vector call-return
  index path to arbitrary inner dimensions.
- Update the Lean lowerability predicate if new expression shapes appear.
- Reseal affected specs.

**Risk:** medium — touches the same packed-vector helpers as W517 but with
more corner cases in index arithmetic.

---

## Recommendation

Select **Variant A** for Wave Loop 518. It is the remaining cleanup item from
W508, it unblocks a fully green smoke gate, and it is a strict prerequisite
for declaring the Icarus-lowerable subset baseline-complete before adding new
language operators (Variant B) or deeper AOS shapes (Variant C).

---

*φ² + φ⁻² = 3 | TRINITY*
