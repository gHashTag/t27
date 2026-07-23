# FPGA Loop Cooperation Variants — Wave Loop 532

**Date:** 2026-07-07  
**From wave:** 531  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 531 extended the Icarus simulation gate to primitive-array witnesses:

- `./scripts/tri test --icarus-simulate --icarus-lowerable` now runs **24**
  lowerable scratch witnesses (W493–W529 + lowerable W3xx) with **0 failures**.
- Function-local and module-level arrays of primitive scalars are now lowered as
  unpacked Verilog arrays (`reg [W-1:0] arr [0:N-1];`), fixing signed-element
  widths and variable-index writes that the old packed scalar-reg fallback
  broke.
- 23 specs were resealed after the unpacked-array lowering change.
- The 16 pre-existing yosys smoke failures remain documented and unchanged.

The next wave should either broaden the lowerable subset or harden the gate.

---

## Variant A — Extend Icarus simulation to signed scalar-array struct fields (recommended)

**Goal:** Broaden the lowerable subset to scalar structs whose fields are small
signed fixed-size arrays, then add them to the simulation gate.

**Scope:**
1. Allow scalar-struct fields of the form `[N]i8`, `[N]i16`, `[N]i32` in the
   packed-vector layout already used for `[N]u8/u16/u32` fields.
2. Emit signed packed vectors where needed and preserve sign extension on slice
   reads.
3. Add positive scratch witnesses for signed array-field read, copy, param, and
   return paths.
4. Add negative witnesses for non-lowerable cases (string/enum fields, dynamic
   sizes).
5. Reseal affected specs and keep `./scripts/tri test --icarus-simulate` at 0
   simulation failures.

**Why recommended:** Many real t27 specs use signed fixed-size arrays inside
structs. Closing this gap removes a major source of `UNSUPPORTED_ICARUS`
placeholders while staying inside the existing packed-vector machinery.

---

## Variant B — Harden the lowerability boundary with adversarial non-lowerability proofs

**Goal:** Make the classifier falsifiable and document the exact lowerability
boundary in both Rust and Lean 4.

**Scope:**
1. Add negative witnesses for constructs that must remain non-lowerable:
   unresolved imports, host-only helpers, casts (`as`), enum/string fields in
   packed arrays, and unbounded dynamic loops.
2. State and prove `¬ Module.isLowerable env m` for each negative witness in
   Lean 4 using `native_decide` or the classifier predicate.
3. Add a Rust integration test that checks the classifier rejects exactly the
   same specs the Lean predicate rejects.
4. Document the boundary so future compiler changes cannot silently expand the
   lowerable subset.

**Why valuable:** A soundness proof is only as strong as the gate it protects.
Adversarial witnesses make classifier regressions detectable in both code and
proof.

---

## Variant C — Add reference-model cosimulation with cocotb

**Goal:** Replace the `$display`-only simulation gate with a Python cocotb test
that compares the generated Verilog against a reference t27 interpreter for
every lowerable spec.

**Scope:**
1. Generate a cocotb-compatible testbench wrapper for lowerable t27 specs.
2. Implement a minimal Python reference model that mirrors the t27 semantics for
   the lowerable subset.
3. Drive the DUT with pseudo-random inputs and compare outputs.
4. Keep the existing Icarus gate as the fast first line, and run the cocotb gate
   in CI on a scheduled cadence.

**Why valuable:** Reference-model cosimulation is the standard way to catch
value-level semantic drift; it also produces artifacts that external reviewers
can run independently.

---

## Recommended variant

**Variant A** is recommended. Wave Loops 530 and 531 proved that the simulation
gate catches real semantic bugs and grows safely when the lowerable subset is
extended one shape at a time. Signed scalar-array struct fields are the largest
remaining hole in the packed-vector path; closing them first keeps the risk
low. Variants B and C should follow once the subset stabilizes.

---

*φ² + φ⁻² = 3 | TRINITY*
