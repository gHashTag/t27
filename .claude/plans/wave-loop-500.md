# Wave Loop 500 — Decomposed Plan

**Goal:** make `specs/scratch/w493_local_aos_element_field_not_lowerable.t27`
Icarus-lowerable, closing the last documented Icarus baseline.

**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak-point analysis

The failing spec uses a local array of scalar structs:

```t27
pub fn make_outer(i : u32) -> Outer {
    let choices : [2]Inner = make_choices();
    return Outer { x: choices[i] };
}
```

The generated Verilog contains:

```verilog
function [31:0] make_outer;
    input [31:0] i;
    reg [31:0]  choices_0_y;
    reg [31:0]  choices_1_y;
    reg [63:0] _aos_ret_tmp_0;
    begin : make_outer_body
        _aos_ret_tmp_0 = make_choices(0);
        choices_0_y = _aos_ret_tmp_0[63:32];
        choices_1_y = _aos_ret_tmp_0[31:0];
        make_outer = 32'd0 /* UNSUPPORTED_ICARUS: Outer struct literal not lowered */;
    end
endfunction
```

The local array `choices` is lowered in **register mode**: each element's fields
are unpacked into per-field registers (`choices_0_y`, `choices_1_y`). When the
indexed element `choices[i]` is used inside a struct literal (`Outer { x:
choices[i] }`), the struct-literal leaf emitter `emit_struct_literal_leaf`
recognizes only:

- scalar struct identifiers (`is_packed_struct_identifier`)
- module-level AOS elements (`is_module_aos_element`)
- struct-return calls
- nested struct literals

It does **not** recognize a local register-mode array-of-struct element, so it
falls back to `gen_verilog_expr`, which for a variable-indexed element does not
produce a packed scalar struct vector; the outer `try_emit_struct_literal_packed`
fails and emits an `UNSUPPORTED_ICARUS` placeholder.

---

## Scientific / engineering grounding

1. **Icarus Verilog limitation**: unpacked arrays of packed structs with indexed
   member access hit a known assertion in `elab_expr.cc`
   ([steveicarus/iverilog#1134](https://github.com/steveicarus/iverilog/issues/1134)).
   The t27 emitter already avoids this by flattening local AOS variables into
   per-field registers. The remaining gap is *re-packing* an indexed element into
   a packed vector when it becomes a struct-literal operand.

2. **CompCert-style emission (W499)**: by emitting every function unconditionally,
   the generic equivalence theorem no longer needs a static reachability proof.
   W500 keeps that invariant and closes a concrete lowering gap instead of adding
   a new theorem assumption.

3. **Packed-vector lowering pattern**: both yosys and the t27 emitter already
   represent scalar structs as flat bit vectors. Re-packing a local AOS element
   is a pure local transformation (per-field register → concatenation) and
   preserves the bit-vector semantics.

---

## Decomposed work plan

### Step 1 — Rust emitter: detect local register-mode AOS element in struct literal
**File:** `bootstrap/src/compiler.rs`
**Target:** `emit_struct_literal_leaf` around line 6297.
**Change:** add a fourth recognized shape, `is_local_register_aos_element`,
which is true when `inner_val` is an `ExprIndex` whose base is a local array of
scalar structs stored in register mode (i.e. in `local_arrays`, element type is a
scalar struct, and `local_struct_array_has_array_field` is false or the entry
is absent).

### Step 2 — Rust emitter: implement the re-packer
**File:** `bootstrap/src/compiler.rs`
**Target:** new helper `gen_verilog_pack_local_register_aos_element`.
**Behavior:**
- For all-literal indices, emit a concatenation of the per-field registers in
  declaration order, exactly like the existing literal-index path of
  `gen_verilog_pack_struct_array_element`.
- For variable indices, emit a priority mux over every possible element,
  selecting the per-field registers that correspond to the current index.
  Example for `[2]Inner` with one field `y`:
  ```verilog
  ((i == 0) ? {choices_0_y} :
   (i == 1) ? {choices_1_y} : 0)
  ```

### Step 3 — Rust emitter: hook the re-packer into struct-literal leaf emission
**File:** `bootstrap/src/compiler.rs`
**Target:** `emit_struct_literal_leaf`.
**Change:** when `is_local_register_aos_element` holds, write the packed vector
instead of calling `gen_verilog_expr`.

### Step 4 — Rust classifier / smoke gate alignment
**File:** `bootstrap/src/compiler.rs` (ICARUS classification), `scripts/tri`.
**Change:** ensure that the new shape no longer produces an `UNSUPPORTED_ICARUS`
placeholder and that the Icarus smoke gate reports the spec as lowerable.

### Step 5 — Lean model alignment (if needed)
**File:** `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`,
`proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`.
**Check:** the Lean predicate already classifies this spec by checking that
`Expr.isLowerable` holds for every expression. If the Rust predicate changes,
verify the Lean model still agrees. Most likely no Lean change is required
because the predicate is expression-agnostic and only checks structural
lowerability/combinationality.

### Step 6 — Witness and seal
**File:** `specs/scratch/w493_local_aos_element_field_not_lowerable.t27`
**Change:** rename it to drop `_not_lowerable` once it passes.
**Seal:** regenerate seal after the rename.

### Step 7 — Verification gates
**Commands:**
1. `lake build Trinity.IcarusLowerable.Soundness`
2. `./scripts/tri verify --lean-lowerable`
3. `./scripts/tri test`
4. `cargo test -p t27c --bin t27c`

**Acceptance:**
- `./scripts/tri test` reports 698 / 698 non-smoke PASS.
- 178 / 178 yosys smoke PASS.
- 178 / 178 Icarus smoke PASS (0 baseline failures).
- 698 / 698 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Risk assessment

| Risk | Mitigation |
|------|------------|
| Variable-index priority mux blows up on large arrays | Keep the existing array-size limits; this is the same explosion already accepted for local scalar arrays. |
| Memory-mode AOS vs register-mode confusion | Only handle register mode (element struct has no array-typed fields). Memory mode already has a separate, working path. |
| Lean predicate/classifier drift | Re-run `tri verify --lean-lowerable` after the Rust change. |
| Breaking existing struct-literal cases | Add the new shape only after the existing identifier/module-AOS/call/literal checks fail, so existing behavior is unchanged. |

---

## Execution status

Implemented and verified on 2026-07-13.  
See `docs/reports/WAVE_LOOP_500_CLOSEOUT.md` for final results.

---

*φ² + φ⁻² = 3 | TRINITY*
