# Wave Loop 553 Closeout Report

**Issue #1524** — Signed/unsigned mixed deterministic bench probes.  
**Branch:** `wave-loop-553`  
**Next branch:** `wave-loop-554`  
**Date:** 2026-07-07

```
Closes #1524
```

---

## Summary

Wave Loop 553 verified that `$signed(...)` wrappers and two's-complement VCD
reconstruction work end-to-end inside deterministic `bench` blocks for signed
scalar returns, signed primitive scalar array elements, and signed packed
scalar-struct fields. The loop exposed and fixed a pre-existing compiler
lowering gap: Verilog does not allow direct bit-select on a function-call
expression, so a packed primitive scalar array returned by a function call
must first be materialized into a temporary packed `reg` before indexing.

Three new scratch witnesses now exercise the signed/unsigned mixed path in
both `test` and `bench` blocks, and all three pass the Icarus simulation gate,
the cocotb reference-model cross-check, and the seal ceremony.

---

## What changed

### 1. New scratch witnesses

* `specs/scratch/w553_bench_signed_scalar_return.t27`
  * Function `neg()` returns `-42` as `i8`.
  * Both `test signed_scalar_test` and `bench "signed_scalar_bench"` assert
    `neg() == -42`.
* `specs/scratch/w553_bench_signed_array_element.t27`
  * Function `seq()` returns `[3]i8{ -1, -2, -3 }`.
  * Both `test signed_element_test` and `bench "signed_element_bench"` assert
    `seq()[0] == -1`.
* `specs/scratch/w553_bench_signed_struct_field.t27`
  * Packed scalar struct `Pt { data: [3]i16 }` is returned by `make()` with a
    negative element.
  * Both `test` and `bench` assert equality on the whole struct value.

### 2. Compiler fixes (`bootstrap/src/compiler.rs`)

* **Signed scalar probes:** the W539 probe pre-declaration now emits
  `reg signed [N:0] _probe` when the actual expression is signed; previously
  the probe register was always unsigned, so `$display("%0d", ...)` printed
  the unsigned bit pattern of negative values.
* **Call-return packed-array indexing:** `ExprIndex` whose base is an
  `ExprCall` returning a packed primitive scalar array is now lowered by
  materializing the call result into a block-local packed `reg` and then
  applying the usual row-major bit-slice access. This fixes the iverilog
  "Malformed statement" caused by `seq(1'b0)[0]`.
* **Width/signedness inference:** `expr_width_signed` now resolves the element
  width and signedness for `f()[i]` when `f` returns a primitive scalar array.

### 3. Python reference-model fix (`scripts/cocotb_ref_model.py`)

* `_cross_check` now uses the physical VCD signal width (not the expected
  literal width) when interpreting a single-signal probe value. An untyped
  literal like `-1` is typed as 32-bit in the AST, but the VCD probe is only
  8 bits wide; sign-extension must happen at width 8, not 32.

### 4. Icarus baselines

* `.trinity/icarus-baselines/specs/scratch/w553_bench_signed_scalar_return.json`
* `.trinity/icarus-baselines/specs/scratch/w553_bench_signed_array_element.json`
* `.trinity/icarus-baselines/specs/scratch/w553_bench_signed_struct_field.json`

### 5. t27 seals

* `.trinity/seals/scratch_w553_bench_signed_scalar_return.json`
* `.trinity/seals/scratch_w553_bench_signed_array_element.json`
* `.trinity/seals/scratch_w553_bench_signed_struct_field.json`

### 6. Rust integration test

* `bootstrap/tests/icarus_lowerable.rs::accepts_w553_bench_signed_cross_check`
  verifies that all three new witnesses are structurally Icarus-lowerable.

---

## Validation matrix

| Command | Result |
|---|---|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p t27c --test icarus_lowerable` | 13/0 |
| `cargo test -p tri` | 78/0 |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | Icarus 65/0, cocotb 65/0, seal mismatches 0, 24 pre-existing Yosys smoke baseline failures |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs, 0 sorry |

Spot checks:

```text
$ ./target/release/t27c icarus-cocotb specs/scratch/w553_bench_signed_scalar_return.t27
cocotb reference-model OK: 1 test block(s) / 1 bench block(s) passed (+ VCD probe check)

$ ./target/release/t27c icarus-cocotb specs/scratch/w553_bench_signed_array_element.t27
cocotb reference-model OK: 1 test block(s) / 1 bench block(s) passed (+ VCD probe check)

$ ./target/release/t27c icarus-cocotb specs/scratch/w553_bench_signed_struct_field.t27
cocotb reference-model OK: 1 test block(s) / 1 bench block(s) passed (+ VCD probe check)
```

The generated Verilog for `w553_bench_signed_array_element.t27` now shows a
packed temporary for the call-return array and a signed scalar probe:

```verilog
reg signed [7:0]  _t27_probe_signed_element_test_0; // W539 typed probe w=8 signed=true
reg signed [23:0] _t27_call_arr_tmp_signed_element_test_0; // W553 packed call-return array tmp w=24 signed=true
...
_t27_call_arr_tmp_signed_element_test_0 = seq(1'b0);
_t27_probe_signed_element_test_0 = ($signed(_t27_call_arr_tmp_signed_element_test_0[7:0]));
if ((($signed(_t27_call_arr_tmp_signed_element_test_0[7:0])) != (-1))) begin ...
```

---

## Weak points addressed

1. **No signed bench witnesses.** Three new specs cover signed scalar returns,
   signed array elements, and signed packed scalar-struct fields inside benches.
2. **Direct indexing of function-call packed arrays was invalid Verilog.** The
   compiler now materializes the packed return value into a temporary `reg`
   before indexing.
3. **Signed scalar probes were emitted as unsigned registers.** The probe
   declaration now carries the signed keyword, so VCD/log display matches the
   t27 signed interpretation.
4. **VCD signed reconstruction used the AST literal width instead of the probe
   width.** The Python reference model now sign-extends from the physical VCD
   signal width.

---

## Three cooperation variants for Wave Loop 554

### Variant A — Recommended: bench-local primitive scalar arrays
Allow `let tmp : [N]T = f();` inside a `bench` block where `f` returns a packed
primitive scalar array, and cross-check element reads against the reference
model. This extends the W553 temporary mechanism from expression contexts to a
named local binding.

### Variant B: whole-array bench assignments
Support `assert_eq` on a complete 2-D primitive scalar array value (not just a
scalarized sum) inside a bench, exercising the W540 multi-slice probe path for
wide signed packed arrays.

### Variant C: timed/non-deterministic bench classifier
Introduce an AST classifier that rejects (or skips) `bench` blocks containing
`#` delays or unbounded loops from the deterministic cocotb gate, and document
the boundary.

---

## Skills saved

Pattern: *"Signed values in deterministic bench blocks share the same
`$signed(...)` wrapping and two's-complement VCD reconstruction as test blocks;
when a bench indexes a function call returning a packed array, materialize the
call result into a temporary packed reg first."*

Saved via `/experience-save`:
* `.trinity/experience.md` updated with the W553 pattern.
* `MEMORY.md` index entry: `wave-loop-553.md`.

---

*φ² + 1/φ² = 3 | TRINITY*
