# Wave Loop 377 — Three Cooperation Variants for Wave Loop 378

**Date:** 2026-07-03  
**Branch:** `trinity-rust-rings`  
**Basis:** W377 close-out metrics and open `gen-verilog` defect backlog.

---

## Current state

- **252 generic ∀** in Lean 4 (252× Sparkle HDL / Verilean reported maximum).
- **557/557 conformance PASS**, **111-wave zero-IGLA-failure streak**.
- **gen-verilog** Defect 5 (struct-field reg name mapping) fixed in W377; CI smoke gate expanded to 36 targets (11 scratch + 25 clean IGLA specs).
- Remaining gen-verilog defects:
  - **Defect 6** — `let` destructuring (blocked by missing tuple-return function generation). This is the only remaining defect preventing `cordic.t27` and `cordic_top.t27` from being yosys-clean.
- FPGA still blocked by missing DLC10 cable.

---

## Variant A — Maximal proof-lattice push (high-risk/high-reward)

### Goal
Push generic ∀ count to **256** by extending every lattice dimension.

### Deliverables
1. 54-variable plus accumulation (`ternaryMacAccumulateFiftyFourPlusGeneric`).
2. 53-variable minus lattice (`ternaryMacAccumulateFiftyThreeMinusGeneric`).
3. Depth-31 alternating cancellation (`ternaryMacUntrigintupleCancellationGeneric`).
4. 12+1+12 zero-weight closure (`ternaryMacZeroWeightDuovigintupleClosureGeneric`).
5. No gen-verilog sub-fix; full effort on proof lattice.
6. Retry board flash.

### Pros
- Widens the generic ∀ lead to 256×.
- Keeps the IGLA floor moving across all 27 specs.

### Cons
- 54-variable theorem may push Lean elaboration time past 30 s.
- Defers backend hardening for another wave.
- One-dimensional; no progress on gen-verilog quality.

### Risk
Medium — proof-time growth; fallback to 53-plus/52-minus if timeout.

---

## Variant B — Balanced proof + backend hardening (RECOMMENDED)

### Goal
Add **+4 generic ∀** to target **256** and attack the last safe gen-verilog item, plus expand the CI smoke gate to full IGLA coverage.

### Deliverables
1. 54-variable plus / 53-variable minus / depth-31 cancellation / 12+1+12 zero-weight closure — target **256 generic ∀**.
2. Address **Defect 6** (`let` destructuring):
   - Detect `let(a, b, c) = expr` in `gen_verilog_stmt`.
   - Emit a packed-vector temporary for the call result and per-binding scalar `reg` assignments.
   - Document that full tuple-return function generation is still required for semantic completeness; the W378 fix targets syntax-level yosys cleanliness.
   - Add `cordic.t27` and `cordic_top.t27` to the smoke gate once they parse cleanly.
3. Expand the CI smoke gate: keep it inside `bootstrap/src/suite.rs` (L7 UNITY) and cover all IGLA specs after Defect 6 is resolved.
4. Retry board flash.

### Pros
- Continues the generic ∀ lead.
- Closes the final tracked gen-verilog defect (at least syntactically).
- Achieves 100% IGLA yosys cleanliness under the smoke gate.

### Cons
- 54-variable proof may timeout; generic ∀ target could drop to 255.
- Defect 6 is partially blocked by missing tuple-return generation, so a full semantic fix may require parser/codegen work beyond one wave.

### Risk
Medium — Defect 6 is the hardest remaining backend item, but a syntax-level workaround is feasible.

---

## Variant C — Backend-first, proof-lattice pause

### Goal
Pause the proof-lattice expansion for one wave and fully close Defect 6, including the deeper tuple-return work, while expanding the smoke gate.

### Deliverables
1. No new generic ∀ theorems; keep the count at **252**.
2. Implement tuple-return function generation in the Verilog backend so `fn f(...) -> (u32, u32, u32)` emits a packed result and `let(a, b, c) = f(...)` can assign slices correctly.
3. Add full semantic regression tests for `let` destructuring.
4. Expand the smoke gate to cover all 27 IGLA specs.
5. Retry board flash.

### Pros
- Catches up the gen-verilog backend; removes the largest remaining technical debt.
- Creates a complete CI smoke gate for future backend changes.
- Lower proof-build time; reduces risk of timeout.

### Cons
- Generic ∀ count stalls for one wave; competitor gap stays flat.
- Tuple-return work may take more than one wave.

### Risk
Low-to-medium — strategically risky because Sparkle/Verilean is active, but technically well-scoped.

---

## Recommendation

**Variant B** is recommended for W378. It keeps the proof-lattice pressure on Sparkle HDL (256 generic ∀ target) while closing the last gen-verilog defect to the extent practical in one wave and expanding the smoke gate to full IGLA coverage. A syntax-level Defect 6 fix is achievable and unblocks the two remaining IGLA specs from the smoke gate.

---

*phi² + 1/phi² = 3 | TRINITY*
