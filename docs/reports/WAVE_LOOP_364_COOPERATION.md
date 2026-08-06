# Wave Loop 365 — Cooperation Variants

**Date:** 2026-07-01
**Current wave:** W364 complete
**Next wave:** W365
**Issue target:** #1252

---

## Summary of W364 state

- **200 generic ∀** across Trinity Lean modules.
- **40-variable accumulation** (plus) and **39-variable minus accumulation** verified.
- **Depth-17 cancellation** (septendecuple) and **23rd proof lattice dimension** (zero-weight septuple closure) proven.
- **546/546 IGLA specs PASS**; 98-wave zero-IGLA-failure streak continues.
- Board flash still blocked by missing DLC10 cable/board.
- One narrow `gen-verilog` fix (`0b` literal emission) landed; four larger Verilog lowering defects remain open.

---

## Variant A — Formal-only extension (safe, fast)

### Scope
Keep pushing the proof lattice without any RTL or board work.

### Deliverables
- 41-variable plus accumulation (`ternaryMacAccumulateFortyOnePlusGeneric`).
- 40-variable minus accumulation (`ternaryMacAccumulateFortyMinusGeneric`).
- Depth-18 cancellation (`ternaryMacOctodecupleCancellationGeneric`).
- 24th proof lattice dimension: zero-weight octuple closure (`ternaryMacZeroWeightOctupleClosureGeneric`).
- 27 IGLA spec blocks (+54 tests, +27 invariants).
- Regenerate all 27 IGLA seals.
- Full conformance suite green.

### Pros
- No hardware or compiler dependencies.
- Highest confidence; lowest risk.
- Continues the streak and increases the generic ∀ gap over competitors.

### Cons
- FPGA evidence still missing for a second consecutive wave.
- Verilog backend defects remain untouched.

### Estimated generic ∀ after W365
**204**

---

## Variant B — Formal extension + retry board flash + Verilog bug triage (recommended)

### Scope
Extend the proof lattice **and** retry physical board loading, plus start systematic `gen-verilog` issue triage for #1245.

### Deliverables
- All of Variant A.
- Retry `dlc10 idcode` / `sram` / `flash` on the QMTech Wukong V1.
  - If board is found: load `ternary_mac_demo_top.bit` and capture IDCODE + DONE pin evidence.
  - If still missing: open a single consolidated hardware-connectivity tracking issue and document exact host/cable/adapter steps.
- Land the **const emission** fix in `gen-verilog` without regressions, or produce a reproduction spec for issue #1250 if the parser change is too risky.
- Document the remaining four #1245 defects with reproduction specs and place them behind `#[verilator_skip]` guards if needed.

### Pros
- Addresses the two largest project weak points simultaneously: formal depth and silicon evidence.
- Moves the `gen-verilog` backend from "known broken" to "documented and partially fixed".
- Best balance of risk and strategic value.

### Cons
- Board load is outside software control; may still fail.
- Verilog parser work can cause regressions; requires careful staged commits.

### Estimated generic ∀ after W365
**204** (same as Variant A; RTL work does not change theorem count).

---

## Variant C — Formal extension + RTL-to-Lean bridge probe + aggressive Verilog rewrite (high risk, high leverage)

### Scope
Extend the proof lattice, retry the board flash, and start a structural Verilog backend refactor with a new **RTL-to-Lean** traceability probe.

### Deliverables
- All of Variant A.
- Retry board flash.
- Prototype a **Verilog → Lean 4** parser (in `scripts/` or `bootstrap/src`) that reads generated `.v` files and extracts:
  - module name,
  - port list,
  - `localparam` constants,
  - register/flip-flop declarations,
  - top-level `always @(posedge clk)` blocks.
- Use the parsed RTL to auto-generate **traceability lemmas** in Lean: "generated module X has ports Y".
- Open a design doc for the full `gen-verilog` refactor:
  - const/var emission,
  - early-return lowering,
  - cast/bitwise expression lowering,
  - struct field naming.

### Pros
- Builds the long-needed formal-traceability link from spec to RTL.
- Makes the Verilog backend auditable and testable in CI.
- Highest strategic leverage if successful.

### Cons
- Largest scope; may spill into W366.
- RTL parser can become a new maintenance burden if not kept narrow.
- Requires careful L2 GENERATION compliance: do not hand-edit `gen/`.

### Estimated generic ∀ after W365
**204** (RTL bridge does not add generic ∀ theorems directly; it enables future verified lowering).

---

## Recommendation

**Select Variant B.**

W364 proved that the formal pipeline can scale past 200 generic ∀. The main project risk is no longer theorem count but **physical evidence** and **backend correctness**. Variant B keeps the formal cadence predictable while finally confronting the two weak points the 2026 competitive landscape will attack first: silicon validation and generated RTL quality.

---

## Cross-wave target summary

| Wave | Generic ∀ | Accumulation depth | Minus depth | Cancellation depth | Closure dimension | Hardware |
|------|-----------|-------------------|-------------|-------------------|-------------------|----------|
| W361 | 188 | 37 | 36 | 14 | 20 (quadruple) | bitstream generated |
| W362 | 192 | 38 | 37 | 15 | 21 (quintuple) | blocked |
| W363 | 196 | 39 | 38 | 16 | 22 (sextuple) | blocked |
| W364 | **200** | **40** | **39** | **17** | **23 (septuple)** | blocked |
| W365 | **204** | **41** | **40** | **18** | **24 (octuple)** | retry flash |
