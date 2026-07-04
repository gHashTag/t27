# Wave Loop 376 — Three Cooperation Variants for Wave Loop 377

**Date:** 2026-07-01  
**Branch:** `trinity-rust-rings`  
**Basis:** W376 close-out metrics and open `gen-verilog` defect backlog.

---

## Current state

- **248 generic ∀** in Lean 4 (248× Sparkle HDL / Verilean reported maximum).
- **556/556 conformance PASS**, **110-wave zero-IGLA-failure streak**.
- **gen-verilog** Defect 4 (`as`/bitwise width correctness) verified fixed in W376; in-runner CI smoke gate now covers `specs/scratch/*.t27`.
- Remaining gen-verilog defects:
  - **Defect 6** — `let` destructuring (blocked by missing tuple-return function generation).
  - **Defect 5** — struct-field reg name mapping (low priority until struct ports are widely used).
  - **CI smoke gate expansion** — extend coverage from scratch specs to all synthesizable IGLA specs once Defects 5/6 are resolved.
- FPGA still blocked by missing DLC10 cable.

---

## Variant A — Maximal proof-lattice push (high-risk/high-reward)

### Goal
Push generic ∀ count to **252** by extending every lattice dimension.

### Deliverables
1. 53-variable plus accumulation (`ternaryMacAccumulateFiftyThreePlusGeneric`).
2. 52-variable minus lattice (`ternaryMacAccumulateFiftyTwoMinusGeneric`).
3. Depth-30 alternating cancellation (`ternaryMacTrigintupleCancellationGeneric`).
4. 11+1+11 zero-weight closure (`ternaryMacZeroWeightVigintupleClosureGeneric`).
5. No gen-verilog sub-fix; full effort on proof lattice.
6. Retry board flash.

### Pros
- Widens the generic ∀ lead to 252×.
- Keeps the IGLA floor moving across all 27 specs.

### Cons
- 53-variable theorem may push Lean elaboration time to 20–30 s.
- Defers backend hardening for another wave.
- One-dimensional; no progress on gen-verilog quality.

### Risk
Medium — proof-time growth; fallback to 52-plus/51-minus if timeout.

---

## Variant B — Balanced proof + backend hardening (RECOMMENDED)

### Goal
Add **+4 generic ∀** to target **252** and land one additional safe gen-verilog sub-fix, plus expand the CI smoke gate.

### Deliverables
1. 53-variable plus / 52-variable minus / depth-30 cancellation / 11+1+11 zero-weight closure — target **252 generic ∀**.
2. Pick the safest remaining backend fix:
   - **Defect 5** (struct-field reg name mapping): unify struct-type register names (`pt_x`) and variable-based field access (`p_x`) so simulation no longer sees unresolved names. Lower risk than Defect 6 because no tuple-return work is required.
   - *Rationale:* Defect 6 remains blocked by tuple-return function generation; Defect 5 is narrow and improves generated code quality for the struct specs already in the repo.
3. Expand the CI smoke gate: keep it inside `bootstrap/src/suite.rs` (L7 UNITY) and add an opt-in or per-spec list that exercises more synthesizable IGLA specs under `yosys read_verilog -sv`.
4. Retry board flash.

### Pros
- Continues the generic ∀ lead.
- Closes another gen-verilog defect without opening the tuple-return blocker.
- Makes the smoke gate more useful for catching regressions in IGLA specs.

### Cons
- 53-variable proof may timeout; generic ∀ target could drop to 251.
- Defect 5 affects fewer specs than Defect 6, so backend impact is smaller.

### Risk
Low-to-medium — well-scoped, with clear fallbacks.

---

## Variant C — Backend-first, proof-lattice pause

### Goal
Pause the proof-lattice expansion for one wave and instead attack Defect 5 and a syntax-level workaround for Defect 6, while expanding the smoke gate.

### Deliverables
1. No new generic ∀ theorems; keep the count at **248**.
2. Fix **Defect 5** (struct-field reg name mapping) and add a scratch spec with explicit struct-field read/write tests.
3. Partially address **Defect 6** by making `let` destructuring syntax valid: detect `let(a,b,c) = expr` in `gen_verilog_stmt` and emit a packed-vector temporary + per-binding assignments using a default width (e.g., 32-bit slices). Document that full tuple-return function generation is still required for semantic correctness.
4. Expand the smoke gate to cover the new scratch specs and any IGLA specs that become yosys-clean after these fixes.
5. Retry board flash.

### Pros
- Catches up the gen-verilog backend; reduces technical debt.
- Creates a larger CI smoke gate for future backend changes.
- Lower proof-build time; reduces risk of timeout.

### Cons
- Generic ∀ count stalls for one wave; competitor gap stays flat.
- Partial `let` fix is unsatisfying and needs follow-up.

### Risk
Low — but strategically risky because Sparkle/Verilean is active.

---

## Recommendation

**Variant B** is recommended for W377. It keeps the proof-lattice pressure on Sparkle HDL (252 generic ∀ target) while closing the next safest gen-verilog defect and expanding the smoke gate. Defect 5 is narrower and less risky than Defect 6, and the smoke gate expansion protects the work.

---

*phi² + 1/phi² = 3 | TRINITY*
