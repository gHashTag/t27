# Wave Loop 378 — Three Cooperation Variants for Wave Loop 379

**Date:** 2026-07-03  
**Branch:** `trinity-rust-rings`  
**Basis:** W378 close-out metrics and open backend semantic gap.

---

## Current state

- **256 generic ∀** in Lean 4 (256× Sparkle HDL / Verilean reported maximum).
- **558/558 conformance PASS**, **112-wave zero-IGLA-failure streak**.
- **gen-verilog** Defect 6 (`let` destructuring) fixed at the syntax level in W378; all 27 IGLA specs are now yosys-clean under the CI smoke gate (38 targets total).
- The deeper **tuple-return function generation** work is still open: multi-return functions are not yet semantically complete in the Verilog backend.
- FPGA still blocked by missing DLC10 cable.

---

## Variant A — Maximal proof-lattice push (high-risk/high-reward)

### Goal
Push generic ∀ count to **260** by extending every lattice dimension.

### Deliverables
1. 55-variable plus accumulation (`ternaryMacAccumulateFiftyFivePlusGeneric`).
2. 54-variable minus lattice (`ternaryMacAccumulateFiftyFourMinusGeneric`).
3. Depth-32 alternating cancellation (`ternaryMacDuotrigintupleCancellationGeneric`).
4. 13+1+13 zero-weight closure (`ternaryMacZeroWeightTrevigintupleClosureGeneric`).
5. No backend sub-fix; defer tuple-return semantics to a later wave.
6. Retry board flash.

### Pros
- Widens the generic ∀ lead to 260×.
- Keeps the IGLA floor moving across all 27 specs.

### Cons
- 55-variable theorem may push Lean elaboration time past 30 s.
- Defers the deeper backend hardening for another wave.
- One-dimensional; no progress on tuple-return semantics.

### Risk
Medium — proof-time growth; fallback to 54-plus/53-minus if timeout, accepting **259 generic ∀**.

---

## Variant B — Balanced proof + semantic backend completion (RECOMMENDED)

### Goal
Add **+4 generic ∀** to target **260** and begin the deeper tuple-return function generation work in the Verilog backend, while keeping the CI smoke gate green.

### Deliverables
1. 55-variable plus / 54-variable minus / depth-32 cancellation / 13+1+13 zero-weight closure — target **260 generic ∀**.
2. Implement **tuple-return function generation**:
   - Parse/lower function return types of the form `(u32, u32, u32)` or similar packed tuples.
   - Emit a packed result vector for multi-return functions.
   - Make `let(a, b, c) = f(...)` assignments semantically correct for arbitrary tuple-return calls, not only for the current syntax-level workaround.
3. Add semantic regression tests for tuple-return destructuring.
4. Keep the CI smoke gate covering all 27 IGLA specs.
5. Retry board flash.

### Pros
- Continues the generic ∀ lead.
- Closes the largest remaining semantic gap in the Verilog backend.
- Makes the W378 syntax-level fix fully correct for future specs.

### Cons
- 55-variable proof may timeout; generic ∀ target could drop to 259.
- Tuple-return work is larger than one prior wave-safe sub-fix and may need two waves.

### Risk
Medium-to-high — tuple-return generation touches parser, typechecker, and codegen; schedule risk, but strategically valuable.

---

## Variant C — Backend-first, proof-lattice pause

### Goal
Pause the proof-lattice expansion for one wave and fully implement tuple-return function generation, plus attack the next broad backend item (#1258: incremental array/RAM lowering for datapath specs).

### Deliverables
1. No new generic ∀ theorems; keep the count at **256**.
2. Implement tuple-return function generation in the Verilog backend.
3. Land initial incremental array/RAM lowering support for `fifo`/`memory`-style specs (#1258).
4. Expand the smoke gate to include new datapath specs as they become yosys-clean.
5. Retry board flash.

### Pros
- Catches up the gen-verilog backend; removes the largest remaining technical debt.
- Opens the door to formal datapath specs (FIFOs, memories) in future waves.
- Lower proof-build time; reduces risk of timeout.

### Cons
- Generic ∀ count stalls for one wave; competitor gap stays flat.
- Tuple-return + RAM work may take more than one wave.

### Risk
Medium — technically well-scoped, but strategically risky because Sparkle/Verilean is active.

---

## Recommendation

**Variant B** is recommended for W379. It keeps the proof-lattice pressure on Sparkle HDL (260 generic ∀ target) while starting the semantic tuple-return backend work that the W378 syntax-level fix needs to become fully correct. The work should be staged: first land the 4 new theorems and the smoke-gate baseline, then attack tuple-return generation in a separate sub-task so that a partial result does not block the formal milestone.

---

*phi² + 1/phi² = 3 | TRINITY*
