# Wave Loop 562 Plan — Whole-struct comparison for structs with array-typed fields

**Issue:** #1533  
**Branch:** `wave-loop-562` (created from `wave-loop-561`)  
**Date:** 2026-07-16  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Weak points addressed by this wave

1. **Struct-array field access on a call temporary is malformed.** A
   `make_packet(...).data[1]` expression currently emits
   `$signed(tmp[0 +: 32])[1]`, which Icarus rejects because the `$signed(...)`
   wrapper is applied to a part-select before indexing. The correct form is
   `$signed(tmp[(0 + (1 * 8)) +: 8])`.

2. **No end-to-end bench cross-check for whole-struct comparison with array-typed
   fields.** The compiler already emits the whole packed struct value for
   `assert_eq(make_packet(...), Packet{...})` and already handles bench-local
   scalar-struct variables. Array-typed scalar fields are the next shape that
   needs a witness to lock the behavior.

3. **Python reference model may mismatch on struct-literal array-field packing.**
   W560 fixed scalar field width masking; array-typed fields inside a struct
   literal must also be packed at their declared element width so the cocotb
   cross-check agrees with the generated Verilog.

---

## 2. Scientific / engineering background

The wave continues the packed-vector lowering of scalar structs with scalar-array
fields (W532). In hardware design, this is equivalent to flattening a
SystemVerilog packed struct whose members include packed arrays:

```systemverilog
typedef struct packed {
    logic signed [7:0] data [0:3];
    logic signed [15:0] sum;
} packet_t;
```

Icarus Verilog does not support packed struct member arrays, so t27 lowers the
whole struct to a single packed vector and computes every field/element slice
as a constant or dynamic part-select. The same approach is used by CIRCT's
HWArith-to-HW lowering and by Yosys' `read_verilog` front-end when it flattens
packed structs.

Sources:
- [IEEE 1800-2017 packed arrays / packed structs](https://ieeexplore.ieee.org/document/8299595)
- [CIRCT lowering of aggregate constant arrays](https://circt.llvm.org/docs/Dialects/HW/)
- [Yosys packed struct support notes](https://yosyshq.net/yosys/documentation.html)

---

## 3. Decomposed implementation plan

### Phase 1 — Spec/TDD
Create one primary witness `specs/scratch/w562_bench_struct_array_field.t27`:
- `struct Packet { data: [4]i8, sum: i16 }`
- `pub fn make_packet(...) -> Packet`
- test: whole-struct `assert_eq(make_packet(...), Packet{...})`, element
  access `make_packet(...).data[1]`, scalar field access `.sum`.
- bench: local `tmp : Packet = make_packet(...)`, same checks on `tmp`.

### Phase 2 — Compiler fix
In `bootstrap/src/compiler.rs`:
- Extend `try_emit_struct_array_field_element_access` to recognize when the
  base of the field access is a function call (`ExprCall`) and the field is a
  fixed-size scalar array. When `use_call_array_temps` is active and a
  temporary exists for the call, emit a single correct dynamic part-select:
  `$signed(tmp[(field_offset + inner_idx * inner_w) +: inner_w])`.
- If the base is a bare call without a temporary, fall back to emitting the
  original call text in the slice base (with parentheses because it is a
  function-call expression).
- Update `FROZEN_HASH` to the new SHA-256 of `bootstrap/src/compiler.rs`.

### Phase 3 — Reference-model alignment
In `scripts/cocotb_ref_model.py`:
- Ensure `_eval_struct_lit_bv` packs scalar-array fields by calling
  `_eval_array_lit_bv` / masking to the declared element width. W560 already
  fixed scalar fields; array-typed fields need the same discipline.

### Phase 4 — Gen / Seal / Baseline / Test
- Save t27 seal for `w562_bench_struct_array_field.t27`.
- Record Icarus baseline.
- Add `accepts_w562_bench_struct_array_field` integration test to
  `bootstrap/tests/icarus_lowerable.rs`.

### Phase 5 — Verify
- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- Direct `t27c icarus-simulate` / `icarus-cocotb` on W562 witness.
- `lake build Trinity.IcarusLowerable.Soundness`

### Phase 6 — Closeout / next variants
- Commit on `wave-loop-562` with `Closes #1533`.
- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W562_2026-07-16.md`.
- Update `.trinity/current-issue.md` with three W563 variants.
- Save skills to `.trinity/experience.md` and project memory.

---

## 4. Acceptance criteria

- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- W562 witness passes direct Icarus simulation and cocotb cross-check.
- New integration test passes.
- Closeout report and three W563 variants recorded.

---

## 5. Three cooperation variants for Wave Loop 563

1. **Variant A — Recommended: array-of-struct return call deduplication.**
   Extend the W556–W558 / W560 block-scoped call temporary machinery to
   function calls that return fixed-size arrays of lowerable packed scalar
   structs (`[N]Pt`). Requires prerequisite fixes already identified in W561:
   `ExprArrayLiteral` lowering for `[N]Pt`, bench-local 1-D AoS variables, and
   1-D AoS element field access.

2. **Variant B: whole-struct comparison for structs with multi-dimensional
   array-typed fields.**  
   Generalize W562 to scalar struct fields that are 2-D fixed-size scalar
   arrays, e.g. `struct Tile { m: [2][3]i8, tag: u8 }`.

3. **Variant C: negative / boundary witnesses for non-lowerable scalar-array
   fields.**  
   Add witnesses where a scalar struct field is an array of `f32`, `string`,
   `enum`, or unresolved-import type, proving the classifier rejects the whole
   struct and the W562/W560 optimization cannot fire.

---

φ² + 1/φ² = 3 | TRINITY
