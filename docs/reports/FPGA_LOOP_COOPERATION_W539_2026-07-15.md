# FPGA Loop Cooperation Variants — Wave 539

**Date:** 2026-07-15  
**Current wave:** W538 (closed)  
**Next wave:** W539  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposes three cooperation variants for the next Wave Loop.

---

## Variant A (recommended): Typed 64-bit probe + full Python expression evaluator

### Motivation

W538 added a fixed 64-bit scalar probe and an independent VCD cross-check, but
it skips wide values and only evaluates literal expected expressions.  The
next step is to make the reference model truly cover the Icarus-lowerable
expression subset by:
1. Knowing the expected bit width and signedness of each probe so the VCD
   comparison can be width-correct instead of assuming 64-bit signed/unsigned.
2. Extending the Python evaluator to handle variable reads, parameterless
   function calls, struct field access, and scalar array indexing.
3. Emitting width-typed probes (`reg [W-1:0]`) for expressions whose width is
   statically known, and skipping only genuinely non-scalar assertions.

### Work breakdown

1. Add a lightweight type-inference helper in `scripts/cocotb_ref_model.py` that
   resolves the type/width/sign of any expression in the lowerable subset using
   the AST and declared types.
2. Pass the expected width/sign to the VCD comparison and mask/sign-extend the
   probe value accordingly.
3. Implement a recursive Python interpreter for the lowerable expression subset
   (literals, arithmetic, variable reads, function calls, field access, indexing).
4. Seed with W5xx/W3xx witnesses that currently skip due to non-literal
   expecteds and verify they now get an independent VCD cross-check.

### Estimated complexity

Medium-High.  Requires a real expression interpreter and type metadata plumbing,
but stays within the existing scalar lowerable subset.

---

## Variant B: Formalize VCD-time value preservation in Lean

### Motivation

The W538 VCD check compares a Verilog signal value at simulation time against a
Python-computed expected value.  A natural formal counterpart is a theorem that
says: for every lowerable expression `e`, the Verilog value of the emitted signal
for `e` equals the t27 source semantics of `e` at every reachable simulation
time.  This would connect the cocotb reference model to the existing
`module_value_equiv` framework.

### Work breakdown

1. Define a source-level denotation `Expr.eval` for the lowerable expression
   subset in `Trinity.IcarusLowerable.SemanticsTotal`.
2. Define a relation between the Verilog VCD value of an emitted expression and
   its source denotation.
3. Prove the relation for literals, arithmetic, variable reads, and function
   calls, building on existing sequential/combinational theorems.
4. Add a non-scratch corpus spec that exercises the relation end-to-end.

### Estimated complexity

High.  Proof engineering on top of the existing value-preservation scaffold.

---

## Variant C: Multi-signal VCD probes for wide packed structs and arrays

### Motivation

The fixed 64-bit probe cannot capture wide packed-struct arrays.  Instead of
skipping those assertions, emit a sequence of 64-bit slices (or per-field
probes) that together represent the whole value.  The Python model can
concatenate the slices and compare against a reference value computed from a
struct/array literal.

### Work breakdown

1. Extend `VerilogCodegen` to emit per-field or per-slice probes for wide
   struct/array actual expressions, and record the slice→field mapping.
2. Extend the Python VCD parser to read multiple probes and reconstruct the
   full value.
3. Extend the Python evaluator to compute whole-struct/array literal values as
   bit-vectors using the declared field widths.
4. Add a scratch witness with a wide struct-array assert and verify the slice
   reconstruction matches the self-checking testbench.

### Estimated complexity

Medium.  Backend lowering work plus bit-vector plumbing, but no new proof
machinery.

---

## Recommendation

**Choose Variant A.**  It directly removes the two largest limitations left by
W538 (fixed 64-bit assumption and literal-only expected evaluator) and gives
more assertions an independent reference-model cross-check.  Variants B and C
are strong follow-ups once the evaluator is complete and the probe width is
known.

---

*φ² + φ⁻² = 3 | TRINITY*
