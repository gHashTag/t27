# Wave Loop 563 Plan — Array-of-struct return call deduplication

**Issue:** #1534  
**Branch:** `wave-loop-563` (created from `wave-loop-562`)  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Weak points addressed by this wave

1. **1-D array-of-struct local declaration is missing.** `var tmp : [2]Pt = make_pts(...)`
   falls through the generic local-emission path, producing a `reg [31:0] tmp`
   declaration of the wrong width and no packed-vector assignment.

2. **Field access on a 1-D array-of-struct element is emitted as an unpacked
   struct name.** `tmp[0].x` currently becomes `tmp_x`, which is not declared.

3. **Field access on a call returning a 1-D array-of-struct loses the base.**
   `make_pts(...)[0].x` currently becomes `_x`.

4. **Call-CSE does not yet cover array-of-struct returns.** Even after the
   three gaps above are fixed, a call like `make_pts(...)` used at two sites in
   the same block will be re-evaluated unless `call_returning_cse_value_info`
   recognizes `[N]Pt` returns.

---

## 2. Scientific / engineering background

Packed array-of-struct lowering is a standard aggregate flattening step in
hardware compilers. CIRCT's `HWLegalizeModules` pass lowers `hw::ArrayGetOp` and
`hw::ArrayCreateOp` into per-element wires and `casez` lookups when
`disallowPackedArrays` is set, because many downstream tools (Icarus, Yosys
native frontend) do not support packed arrays of structs. t27 takes a simpler
but equivalent approach: the whole `[N]Pt` value is stored as one unsigned packed
vector, and every element/field access is emitted as a constant or dynamic
part-select.

Common-subexpression elimination for the simulation assertion harness is the
same idea used in CIRCT's `createCSEPass()` before legalization: evaluate a pure
function call once per block and reuse the packed result. W556–W560 extended
this to scalar, scalar-array, and scalar-struct returns; W563 completes the
series for arrays of scalar structs.

Sources:
- [CIRCT Verilog Generation / LoweringOptions](https://circt.llvm.org/docs/VerilogGeneration/)
- [CIRCT HWLegalizeModules source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html)
- [Yosys packed-struct array support gap](https://github.com/YosysHQ/yosys/issues/4653)
- [CIRCT CSE pass documentation](https://circt.llvm.org/docs/Passes/#hw-cse)

---

## 3. Decomposed implementation plan

### Phase 1 — Spec / TDD

Create `specs/scratch/w563_bench_array_of_struct_call_dedup.t27`:

```t27
module w563_bench_array_of_struct_call_dedup;

pub struct Pt { x : i16, y : i16 }

pub fn make_pts(a : i16, b : i16, c : i16, d : i16) -> [2]Pt {
    return [2]Pt{ Pt{ .x = a, .y = b }, Pt{ .x = c, .y = d } };
}

test aos_test {
    var tmp : [2]Pt = make_pts(1, 2, 3, 4);
    assert_eq(tmp[0].x, 1);
    assert_eq(tmp[1].y, 4);
    assert_eq(make_pts(1, 2, 3, 4)[0].y, 2);
    assert_eq(make_pts(1, 2, 3, 4)[1].x, 3);
}

bench aos_bench {
    var t : [2]Pt = make_pts(5, 6, 7, 8);
    assert_eq(t[0].y, 6);
    assert_eq(make_pts(5, 6, 7, 8)[1].x, 7);
}

endmodule
```

The test verifies:
- local 1-D AoS initialized by a call,
- field access on a local 1-D AoS element,
- field access on a call-return 1-D AoS element,
- multi-site reuse of the same call in one block (CSE).

### Phase 2 — Compiler fixes

In `bootstrap/src/compiler.rs`:

1. **1-D AoS local declaration.** In `emit_local`, add a branch between the
   bare-scalar-struct branch and the multi-D AoS branch for `parse_array_type`
   returning `dims.len() == 1` with a scalar-struct element type. Declare a
   packed-vector `reg` of total width `packed_width(ty)`. If the initializer is
   an `ExprArrayLiteral`, call `emit_packed_array_literal_concat`; if it is a
   call or other packed-vector expression, assign it directly.

2. **Field access on 1-D AoS element / call-return element.** Generalize
   `try_emit_struct_array_access` to:
   - accept `dims.len() >= 1` (remove the `dims.len() < 2` guard),
   - accept a base that is an `ExprCall` returning a 1-D array of scalar structs,
   - look up a predeclared call temporary when `use_call_array_temps` is active,
   - fall back to a parenthesized raw call expression otherwise,
   - compute the linear element index and optional field offset exactly as the
     existing multi-D path does.

3. **Call-CSE for `[N]Pt` returns.** In `call_returning_cse_value_info`, add a
   branch after the primitive-array branch for arrays whose element type is a
   lowerable scalar struct. Return `(key, dims, elem_type, packed_width(ret_ty),
   false)`.

4. Update `bootstrap/stage0/FROZEN_HASH` to the new SHA-256 of
   `bootstrap/src/compiler.rs`.

### Phase 3 — Reference-model alignment

In `scripts/cocotb_ref_model.py`:

- Verify that field access on an array-of-struct value evaluates correctly. The
  existing Python evaluator likely handles `arr[i]` and `.x` independently; if
  a cocotb mismatch appears, add an array-of-struct case that reconstructs the
  packed element before extracting the field.

### Phase 4 — Gen / Seal / Baseline / Test

- Save t27 seal for `w563_bench_array_of_struct_call_dedup.t27`.
- Record Icarus baseline.
- Add `accepts_w563_bench_array_of_struct_call_dedup` integration test to
  `bootstrap/tests/icarus_lowerable.rs`.

### Phase 5 — Verify

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- Direct `t27c icarus-simulate` / `icarus-cocotb` on W563 witness.
- `lake build Trinity.IcarusLowerable.Soundness`

### Phase 6 — Closeout / next variants

- Commit on `wave-loop-563` with `Closes #1534`.
- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W563_2026-07-07.md`.
- Update `.trinity/current-issue.md` with three W564 variants.
- Save skills to `.trinity/experience.md` and project memory.

---

## 4. Acceptance criteria

- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- W563 witness passes direct Icarus simulation and cocotb cross-check.
- Generated Verilog for the bench block contains exactly one
  `_t27_call_tmp_*` assignment for `make_pts(...)`.
- New integration test passes.
- Closeout report and three W564 variants recorded.

---

## 5. Three cooperation variants for Wave Loop 564

1. **Variant A — Recommended: whole-array comparison for 1-D arrays of scalar
   structs.** Extend the W555/W562 whole-array `assert_eq` probe path to
   packed 1-D AoS values, enabling `assert_eq(make_pts(...), [2]Pt{...})` in
   bench blocks.

2. **Variant B: 2-D array-of-struct return call deduplication.** Generalize
   W563 to function calls returning 2-D arrays of scalar structs (`[N][M]Pt`).
   The existing multi-D local/field access paths should already cover most of
   this; the CSE descriptor and call-temporary slice emission need to be
   verified.

3. **Variant C: negative / boundary witnesses for non-lowerable array-of-struct
   returns.** Add witnesses where a function returns `[N]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field, proving the structural
   classifier rejects the whole return type before CSE can apply.
