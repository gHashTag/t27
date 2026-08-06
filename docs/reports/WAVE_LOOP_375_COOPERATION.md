# Wave Loop 375 — Three Cooperation Variants for Wave Loop 376

**Date:** 2026-07-03  
**Branch:** `trinity-rust-rings`  
**Basis:** W375 close-out metrics and open `gen-verilog` defect backlog.

---

## Current state

- **244 generic ∀** in Lean 4 (244× Sparkle HDL / Verilean reported maximum).
- **555/555 conformance PASS**, **109-wave zero-IGLA-failure streak**.
- **gen-verilog** Defect 3 (early-return chaining) fixed in W375.
- Remaining gen-verilog defects: **Defect 6** (`let` destructuring, blocked by tuple-return function generation), **Defect 4** (`as` / bitwise width correctness), **Defect 5** (struct-field reg name mapping), and the long-term **CI smoke gate** for `gen-verilog` + `yosys`.
- FPGA still blocked by missing DLC10 cable.

---

## Variant A — Maximal proof-lattice push (high-risk/high-reward)

### Goal
Push generic ∀ count to **248** by extending every dimension at once.

### Deliverables
1. 52-variable plus accumulation (`ternaryMacAccumulateFiftyTwoPlusGeneric`).
2. 51-variable minus lattice (`ternaryMacAccumulateFiftyOneMinusGeneric`).
3. Depth-29 alternating cancellation (`ternaryMacNovenvigintupleCancellationGeneric`).
4. 10+1+10 zero-weight closure (`ternaryMacZeroWeightNovemdecupleClosureGeneric`).
5. No gen-verilog sub-fix; full effort on proof lattice.
6. Retry board flash.

### Pros
- Widens the generic ∀ lead to 248×.
- Keeps the IGLA floor moving across all 27 specs.

### Cons
- 52-variable theorem may timeout (elaboration ~15–25 s).
- Defers the real `let` destructuring / tuple-return blocker to a later wave.
- One-dimensional; no backend hardening.

### Risk
Medium — proof-time growth; fallback to 51-plus/50-minus if timeout.

---

## Variant B — Balanced proof + backend hardening (RECOMMENDED)

### Goal
Add **+4 generic ∀** to reach **248** and land one additional safe gen-verilog sub-fix that does not require tuple-return function generation.

### Deliverables
1. 52-variable plus / 51-variable minus / depth-29 cancellation / 20-variable zero-weight closure — target **248 generic ∀**.
2. Pick the safest remaining backend fix:
   - **Defect 4** (`as` / bitwise operator width correctness): add a scratch spec with explicit cast-and-mask values and harden `gen_verilog_expr` so generated widths are correct and yosys-simulation values match.
   - *Rationale:* Defect 6 is blocked by tuple-return generation; Defect 5 is low-priority until struct ports are used; Defect 4 is narrow and improves generated code quality.
3. Add a lightweight CI smoke step: run `t27c gen-verilog` on the scratch spec and verify `yosys read_verilog -sv` passes. Keep it inside the Rust runner to respect L7 UNITY (no new shell scripts on the critical path).
4. Retry board flash.

### Pros
- Continues the generic ∀ lead.
- Adds measurable backend quality (correct cast widths, yosys-clean).
- Lays groundwork for the CI smoke gate that will protect future gen-verilog changes.

### Cons
- Defect 4 needs simulation-value verification, which is more work than a pure syntax fix.
- If 52-variable proof times out, the generic ∀ target drops to 247.

### Risk
Low-to-medium — well-scoped, with clear fallbacks.

---

## Variant C — Backend-first, proof-lattice pause

### Goal
Pause the proof-lattice expansion for one wave and instead land two backend fixes + the CI smoke gate.

### Deliverables
1. No new generic ∀ theorems; keep the count at **244**.
2. Fix **Defect 4** (`as` / bitwise width correctness) with scratch spec and simulation checks.
3. Partially address **Defect 6** by making `let` destructuring syntax valid: detect `let(a,b,c) = expr` in `gen_verilog_stmt` and emit a packed-vector temporary + per-binding assignments using a default width (e.g., 32-bit slices). Document that full tuple-return function generation is still required for semantic correctness.
4. Add scratch specs for both defects and verify through `yosys read_verilog -sv`.
5. Retry board flash.

### Pros
- Catches up the gen-verilog backend; reduces technical debt.
- Creates CI smoke gate infrastructure.
- Lower proof-build time; reduces risk of timeout.

### Cons
- Generic ∀ count stalls at 244 for one wave; competitor gap stays flat.
- Partial `let` fix is unsatisfying and needs follow-up.

### Risk
Low — but strategically risky because Sparkle/Verilean is active.

---

## Recommendation

**Variant B** is recommended for W376. It keeps the proof-lattice pressure on Sparkle HDL (248 generic ∀) while landing the next safest gen-verilog sub-fix and starting the CI smoke gate that will make future backend fixes cheaper and safer.

---

*phi² + 1/phi² = 3 | TRINITY*
