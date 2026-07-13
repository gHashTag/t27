# FPGA/Wave Loop Cooperation Variants — W517

**Date:** 2026-07-07  
**From:** Wave Loop 516 Closeout  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 16 closed the whole-array-field read boundary for **packed scalar
structs** and **function-local packed arrays-of-structs**. The next loop should
attack one of the three remaining concrete boundaries below. Each variant is
independent, reviewable in isolation, and leaves the repo in a green state.

---

## Variant A — Packed AOS parameter whole-array-field reads (recommended)

**Goal:** Extend W516 so that a whole array-typed field can be read from a
packed array-of-structs **parameter**, not only from a function-local AOS.

**Why this is the natural next step:**

- W516 already proved the local-AOS slice path works for `arr[i].coords` where
  `arr` is declared inside the function.
- The current backend decomposes some AOS parameters into per-field memories
  (`arr_coords`) while the call site passes a single packed vector, producing
  an unsound binding when the returned field is itself an array.
- Fixing this closes the last remaining shape of whole-array-field read and
  makes AOS parameters first-class.

**Deliverables:**

- 1–2 scratch witnesses in `specs/scratch/` covering `fn f(a : [N]S, i : u8) -> [M]u32 { return a[i].coords; }`.
- Update the array-parameter binding path in `bootstrap/src/compiler.rs` to keep
  the parameter as a packed vector and slice the full field width.
- Reseal affected specs.
- `cargo test`, `tri test --icarus-lowerable --fast`, and `tri verify --lean-lowerable` green.

**Risk:** medium — touches the AOS parameter clone/binding pass, which has
multiple call-site shapes.

---

## Variant B — Clear the remaining W508 `break`/`continue` smoke baselines

**Goal:** Remove the last 2 yosys and 3 Icarus smoke failures inherited from
Wave Loop 508's bounded-loop early-exit work.

**Why now:**

- The failures are documented but they are the only remaining smoke baselines.
- The Lean model, predicate, and emitter already support `break`/`continue`;
  the residual issue is usually a Verilog syntax quirk (e.g. placement of
  `disable` labels, loop-body scoping, or `initial` vs. function context).
- A clean smoke gate simplifies all future waves.

**Deliverables:**

- Re-examine the generated Verilog for the baseline specs and tighten the
  emitter to produce yosys/Icarus-compatible syntax.
- Update baselines to zero.
- Add one adversarial witness that mixes `break`/`continue` with the W516
  packed-vector return paths.

**Risk:** low-to-medium — narrowly scoped, but may require touching the loop
emitter and the Icarus baseline loader.

---

## Variant C — Packed scalar struct equality / comparison operators

**Goal:** Add `==` and `!=` for packed scalar structs in the Icarus-lowerable
subset, lowering to a bit-vector equality of the packed vector representation.

**Why this matters:**

- Struct equality is a common request and currently falls back to host-only
  helpers or unresolved placeholders.
- The packed representation already makes equality a single `==` over the
  concatenated leaf fields, so the emitter change is small once the type system
  accepts the operator.
- It pairs naturally with the W515/W516 packed-struct work.

**Deliverables:**

- Allow `a == b` where `a` and `b` are packed scalar struct values in
  Icarus-lowerable contexts.
- Generate `a == b` as Verilog equality of the packed vectors (or leaf-wise
  equality if widths differ).
- 2–3 scratch witnesses: module-level struct equality, function-return struct
  equality, and an invariant using struct equality.
- Update Lean predicate if the operator needs to be modeled explicitly.

**Risk:** medium — requires parser/typecheck and Verilog backend alignment.

---

## Recommendation

Select **Variant A** for Wave Loop 517. It is the direct continuation of W516,
the remaining code surface is well understood after this loop, and it keeps the
packed-vector/AOS feature coherent before moving to broader cleanup (Variant B)
or language-level operators (Variant C).

---

*φ² + φ⁻² = 3 | TRINITY*
