# FPGA Loop Cooperation Variants — Wave 540

**Date:** 2026-07-08  
**Current wave:** W539 (closed)  
**Next wave:** W540  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposes three cooperation variants for the next Wave Loop.

---

## Variant A (recommended): Multi-signal VCD probes for wide packed structs and arrays

### Motivation

W539 made every scalar assertion independently cross-checkable against a typed
VCD probe. The remaining gap is whole-struct, whole-array, and multi-dimensional
packed-struct assertions whose bit width exceeds 64 bits. Instead of skipping
these assertions, emit a deterministic sequence of 64-bit (or smaller) slices
and reconstruct the full value in the Python reference model.

### Work breakdown

1. Extend `VerilogCodegen` to detect when an `assert_eq` actual expression
   width exceeds 64 bits and emit multiple probe registers, each annotated with
   its slice offset and width.
2. Record slice metadata in `probe_specs` so the Python side can reconstruct
   the packed vector in source order.
3. Extend `_VcdParser` to read slice probes and concatenate them into a single
   bit-vector.
4. Add a scratch witness with a wide packed-struct-array assertion and verify
   the slice reconstruction matches the self-checking testbench.

### Estimated complexity

Medium. Backend-only change plus bit-vector plumbing; no formal proof work.

---

## Variant B: Reference-model coverage of scalar function-call arguments

### Motivation

The W539 Python evaluator only handles parameterless function calls because
argument binding and per-parameter width/signedness must be threaded through the
call context. The next natural increment is to support scalar arguments in
function calls so that assertions like `assert_eq(add(-3, 4), 1)` and
`assert_eq(read_signed(i, j, k), expected)` get an independent VCD cross-check.

### Work breakdown

1. Extend `EvalContext` with parameter type information from the function
   declaration and evaluate each argument as a `Bv` before binding.
2. In `expr_width_signed`, resolve scalar call results when arguments are
   scalar.
3. Seed with W5xx/W3xx witnesses that use scalar function-call actuals and
   verify they no longer fall back to the log-based self-check.
4. Add a negative witness showing that non-scalar / non-lowerable call
   arguments remain skipped.

### Estimated complexity

Medium. Builds directly on the W539 evaluator architecture.

---

## Variant C: Formalize VCD-time expression equivalence in Lean

### Motivation

The cocotb reference model now independently checks many expressions by
comparing a Verilog VCD value against a Python-computed expected value. A
formal counterpart would strengthen confidence: prove that for every lowerable
expression `e`, the bit-vector value emitted by the Verilog backend equals the
t27 source semantics of `e` at every reachable simulation time. This connects
W539's runtime cross-check to the existing `module_value_equiv` framework.

### Work breakdown

1. Define a source-level denotation `Expr.eval` for the lowerable expression
   subset in `Trinity.IcarusLowerable.SemanticsTotal`, including literals,
   identifiers, binary/unary operators, casts, and function calls.
2. Define a relation between a Verilog VCD signal value and the source
   denotation, parameterized by the emitted width and signedness.
3. Prove the relation for the combinational subset first, then extend to
   sequential identifiers under the existing `module_value_equiv` invariant.
4. Add a non-scratch corpus spec that exercises end-to-end expression
   equivalence.

### Estimated complexity

High. Proof engineering extending the existing value-preservation scaffold.

---

## Recommendation

**Choose Variant A.** It closes the largest remaining hole in the cocotb
reference-model coverage (wide packed values) and keeps the work purely in the
backend and Python model, matching the W539 momentum. Variant B is a strong
follow-up once the evaluator fully covers function-call arguments, and Variant C
becomes tractable once the VCD comparison surface is complete.

---

*φ² + φ⁻² = 3 | TRINITY*
