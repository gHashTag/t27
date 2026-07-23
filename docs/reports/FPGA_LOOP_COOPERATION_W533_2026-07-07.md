# FPGA Loop Cooperation Variants — Wave Loop 533

**Date:** 2026-07-07  
**From wave:** 532  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 532 closed the largest remaining gap in the packed-vector Icarus-lowerable subset:

- `./scripts/tri test --icarus-simulate --icarus-lowerable` now runs **28**
  lowerable scratch witnesses (W493–W529 + lowerable W3xx + W532) with **0 failures**.
- Scalar structs whose fields are fixed-size signed scalar arrays (`[N]i8/i16/i32`)
  now lower correctly as packed vectors, including 2-D arrays of such structs.
- Negative witnesses for non-lowerable struct fields (enum/string) emit an
  `// UNSUPPORTED_ICARUS` marker so the classifier rejects them.
- All affected specs were resealed and the live compiler hash was refrozen.
- The 23 pre-existing yosys smoke failures remain documented and unchanged.

The packed-vector path is now complete for numeric scalar-array struct fields.
The next wave should either harden the boundary or broaden the subset to
module-level / cross-boundary shapes.

---

## Variant A — Module-level packed scalar structs with array fields (recommended)

**Goal:** Bring the W527/W528 packed-vector AoS machinery to scalar structs with
fixed-size scalar array fields at module scope (constants, variables, and
parameters), and support whole-struct assignment across module boundaries.

**Scope:**
1. Lower module-level `const`/`var` scalar structs with `[N]i8/i16/i32` or
   `[N]u8/u16/u32` fields as a single packed `localparam`/`reg`.
2. Allow module-level parameters of scalar-struct type when all fields are
   lowerable scalar arrays or primitive scalars.
3. Support assignment of one scalar struct to another and from a struct-returning
   function call at module scope.
4. Add positive scratch witnesses for module const, module var, module parameter,
   and whole-struct assignment.
5. Keep `./scripts/tri test --icarus-simulate --icarus-lowerable` at 0 failures
   and reseal affected specs.

**Why recommended:** Many t27 specs define module-level configuration structs
with small signed/unsigned arrays. Closing the module-scope gap removes the
remaining `UNSUPPORTED_ICARUS` placeholders for scalar structs and unifies the
function-local and module-level lowering paths.

---

## Variant B — Adversarial lowerability boundary proofs

**Goal:** Make the lowerability classifier falsifiable and document the exact
boundary in both Rust and Lean 4.

**Scope:**
1. Add negative witnesses for constructs that must remain non-lowerable:
   unresolved imports, host-only helpers, casts (`as`), enum/string/float fields
   in packed arrays, unbounded dynamic loops, and whole-struct assignment of
   non-lowerable structs at module scope.
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

## Variant C — cocotb reference-model cosimulation

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

**Variant A** is recommended. Wave Loops 530–532 proved that the simulation
gate catches real semantic bugs and grows safely when the lowerable subset is
extended one shape at a time. Module-level scalar structs with array fields are
the last major gap in the packed-vector path; closing them first keeps the risk
low and unifies function-local and module-level lowering. Variants B and C
should follow once the subset stabilizes.

---

*φ² + φ⁻² = 3 | TRINITY*
