# Wave Loop 367 — Cooperation Variants

**Date:** 2026-07-01
**Current wave:** W366 complete
**Next wave:** W367
**Issue target:** #1253

---

## Summary of W366 state

- **208 generic ∀** across Trinity Lean modules.
- **42-variable accumulation** (plus) and **41-variable minus accumulation** verified.
- **Depth-19 cancellation** (novemdecuple) and **25th proof lattice dimension** (zero-weight nonuple closure) proven.
- **546/546 IGLA specs PASS**; 100-wave zero-IGLA-failure streak continues.
- Board flash still blocked by missing DLC10 cable/board.
- `gen-verilog` #1245 remains open with four reproducible defects; no risky fix was landed.

---

## Variant A — Formal-only extension (safe, fast)

### Scope
Keep pushing the proof lattice without any RTL or board work.

### Deliverables
- 43-variable plus accumulation (`ternaryMacAccumulateFortyThreePlusGeneric`).
- 42-variable minus accumulation (`ternaryMacAccumulateFortyTwoMinusGeneric`).
- Depth-20 identity cancellation (`ternaryMacVigintupleCancellationGeneric`) — even-depth identity `= x`.
- 26th proof lattice dimension: zero-weight decuple closure (`ternaryMacZeroWeightDecupleClosureGeneric`).
- 27 IGLA spec blocks (+54 tests, +27 invariants).
- Regenerate all 27 IGLA seals.
- Full conformance suite green.

### Pros
- No hardware or compiler dependencies.
- Highest confidence; lowest risk.
- Continues the streak and increases the generic ∀ gap over competitors.

### Cons
- FPGA evidence still missing for a fourth consecutive wave.
- Verilog backend defects remain untouched.

### Estimated generic ∀ after W367
**212**

---

## Variant B — Formal extension + retry board flash + one safe gen-verilog fix (recommended)

### Scope
Extend the proof lattice **and** retry physical board loading, plus land **one** additional safe `gen-verilog` sub-fix from #1245.

### Deliverables
- All of Variant A.
- Retry `dlc10 idcode` / `sram` / `flash` on the QMTech Wukong V1.
  - If board is found: load `ternary_mac_demo_top.bit` and capture IDCODE + DONE pin evidence.
  - If still missing: keep the consolidated hardware-connectivity tracking issue open and update `FPGA_EVIDENCE_W367.md`.
- Land **one** regression-free `gen-verilog` fix, chosen from:
  - Pad `0x` literals to the declared target width in assignments/declarations (requires threading expected width into `gen_verilog_expr`),
  - or a `#[verilog_skip]`-style guard for specs that trigger defects 3–5,
  - or a narrow test-time lint that rejects early `return` and mismatched `as` patterns in specs marked for Verilog generation.
- If no safe fix is found, promote the reproduction doc to a full `docs/reports/GEN_VERILOG_BACKEND_ROADMAP.md`.

### Pros
- Addresses the two largest project weak points while keeping risk bounded.
- Moves the Verilog backend from "documented" to "one defect smaller".
- Best balance of risk and strategic value.

### Cons
- Board load is outside software control; may still fail.
- Even narrow compiler changes can cause regressions; requires staged validation.

### Estimated generic ∀ after W367
**212** (RTL work does not change theorem count).

---

## Variant C — Formal extension + RTL-to-Lean traceability prototype (high risk, high leverage)

### Scope
Extend the proof lattice, retry the board flash, and start a structural Verilog traceability prototype.

### Deliverables
- All of Variant A.
- Retry board flash.
- Prototype `scripts/verilog_to_lean.py` that reads `t27c gen-verilog` output and extracts:
  - module name,
  - port list,
  - `localparam` / `parameter` declarations,
  - register / flip-flop declarations,
  - top-level `always @(posedge clk)` block signatures.
- Use the parsed RTL to auto-generate **traceability lemmas** in Lean: "generated module X has ports Y and parameters Z".
- Open a design doc for the full `gen-verilog` refactor with phased fixes for defects 1, 3, 4, 5.

### Pros
- Builds the long-needed formal-traceability link from spec to RTL.
- Makes future Verilog backend changes auditable in CI.
- Highest strategic leverage if successful.

### Cons
- Largest scope; may spill into W368.
- RTL parser can become a new maintenance burden if not kept narrow.
- Requires careful L2 GENERATION compliance: do not hand-edit `gen/`.

### Estimated generic ∀ after W367
**212** (RTL bridge does not add generic ∀ theorems directly).

---

## Recommendation

**Select Variant B.**

W366 proved that the formal pipeline scales past 200 generic ∀ and that the remaining `gen-verilog` defects can be reproduced without regressions. The next wave should finally land a second safe backend improvement while keeping the formal cadence predictable. Variant B is the only option that simultaneously protects the zero-failure streak, confronts the silicon-evidence gap, and makes measurable progress on backend quality.

---

## Cross-wave target summary

| Wave | Generic ∀ | Accumulation depth | Minus depth | Cancellation depth | Closure dimension | Hardware |
|------|-----------|-------------------|-------------|-------------------|-------------------|----------|
| W362 | 192 | 38 | 37 | 15 | 21 (quintuple) | blocked |
| W363 | 196 | 39 | 38 | 16 | 22 (sextuple) | blocked |
| W364 | 200 | 40 | 39 | 17 | 23 (septuple) | blocked |
| W365 | **204** | **41** | **40** | **18** | **24 (octuple)** | blocked |
| W366 | **208** | **42** | **41** | **19** | **25 (nonuple)** | blocked |
| W367 | **212** | **43** | **42** | **20** | **26 (decuple)** | retry flash |
