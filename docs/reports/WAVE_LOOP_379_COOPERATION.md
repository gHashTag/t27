# Wave Loop 379 — Three Cooperation Variants for Wave Loop 380

**Date:** 2026-07-03  
**Branch:** `trinity-rust-rings`  
**Basis:** W379 close-out metrics and open backend semantic gap.

---

## Current state

- **260 generic ∀** in Lean 4 (260× Sparkle HDL / Verilean reported maximum).
- **559/559 conformance PASS**, **113-wave zero-IGLA-failure streak**.
- `gen-verilog` `let` destructuring is now **semantically aware** at the syntax level: binding count and per-binding width are inferred from the LHS pattern.
- Remaining backend work:
  - **Full tuple-return function generation** — multi-return types, tuple literals, and slot-aware function-call lowering remain a deeper project.
  - **Incremental array/RAM lowering** — #1258 (datapath specs such as FIFOs and memories).
- FPGA still blocked by missing DLC10 cable.

---

## Variant A — Maximal proof-lattice push (high-risk/high-reward)

### Goal
Push generic ∀ count to **264** by extending every lattice dimension.

### Deliverables
1. 56-variable plus accumulation (`ternaryMacAccumulateFiftySixPlusGeneric`).
2. 55-variable minus lattice (`ternaryMacAccumulateFiftyFiveMinusGeneric`).
3. Depth-33 alternating cancellation (`ternaryMacTritrigintupleCancellationGeneric`).
4. 14+1+14 zero-weight closure (`ternaryMacZeroWeightQuattuorvigintupleClosureGeneric`).
5. No backend sub-fix; defer tuple-return semantics and RAM lowering.
6. Retry board flash.

### Pros
- Widens the generic ∀ lead to 264×.
- Keeps the IGLA floor moving across all 27 specs.

### Cons
- 56-variable theorem may push Lean elaboration time past 35 s.
- Defers the remaining backend hardening for another wave.
- One-dimensional; no progress on gen-verilog semantic completeness.

### Risk
Medium — proof-time growth; fallback to 55-plus/54-minus if timeout, accepting **263 generic ∀**.

---

## Variant B — Balanced proof + backend hardening (RECOMMENDED)

### Goal
Add **+4 generic ∀** to target **264** and begin the deeper **tuple-return function generation** work in the Verilog backend, while keeping the CI smoke gate green.

### Deliverables
1. 56-variable plus / 55-variable minus / depth-33 cancellation / 14+1+14 zero-weight closure — target **264 generic ∀**.
2. Implement **tuple-return function generation**:
   - Lower function return types of the form `(T1, T2, ...)` to a packed result vector.
   - Lower tuple literals `(a, b, c)` to packed assignments.
   - Make `let(a, b, c) = f(...)` semantically correct for arbitrary multi-return calls, removing the current syntax-level workaround.
3. Add semantic regression tests for tuple-return destructuring (mixed widths, unused slots, nested calls).
4. Keep the CI smoke gate covering all 27 IGLA specs.
5. Retry board flash.

### Pros
- Continues the generic ∀ lead.
- Closes the largest remaining semantic gap in the Verilog backend.
- Makes future datapath and integration specs much easier to generate cleanly.

### Cons
- 56-variable proof may timeout; generic ∀ target could drop to 263.
- Tuple-return work is larger than one prior wave-safe sub-fix and may need two waves.

### Risk
Medium-to-high — touches parser, typechecker, and codegen; schedule risk, but strategically valuable.

---

## Variant C — Backend-first, proof-lattice pause

### Goal
Pause the proof-lattice expansion for one wave and fully implement **tuple-return function generation**, plus attack **incremental array/RAM lowering** (#1258) for datapath specs.

### Deliverables
1. No new generic ∀ theorems; keep the count at **260**.
2. Implement full tuple-return function generation in the Verilog backend.
3. Land initial array/RAM lowering support for memory-style specs (`[]T` arrays, `len`, indexing) so that FIFO/datapath specs begin to emit synthesizable Verilog.
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
Medium — technically well-scoped, but strategically risky because Sparkle/Verilean is active and continues shipping verified IP.

---

## Recommendation

**Variant B** is recommended for W380. It keeps the proof-lattice pressure on Sparkle HDL (264 generic ∀ target) while starting the semantic tuple-return backend work that the W379 generalized fix needs to become fully correct. The work should be staged: first land the 4 new theorems and the smoke-gate baseline, then attack tuple-return generation in a dedicated sub-task so that a partial result does not block the formal milestone.

---

*phi² + 1/phi² = 3 | TRINITY*
