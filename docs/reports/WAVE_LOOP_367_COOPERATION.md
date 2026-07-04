# Wave Loop 368 — Cooperation Variants

**Date:** 2026-07-01
**Current wave:** W367 complete
**Next wave:** W368
**Issue target:** #1254

---

## Summary of W367 state

- **212 generic ∀** across Trinity Lean modules.
- **43-variable accumulation** (plus) and **42-variable minus accumulation** verified.
- **Depth-20 cancellation** (vigintuple) and **26th proof lattice dimension** (zero-weight decuple closure) proven.
- **546/546 IGLA specs PASS**; 101-wave zero-IGLA-failure streak continues.
- Board flash still blocked by missing DLC10 cable/board.
- `gen-verilog` defect 2 (`0x` literal width padding) fixed for scalar consts; defects 1/3/4/5 remain tracked in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

---

## Variant A — Formal-only extension (safe, fast)

### Scope
Keep pushing the proof lattice without any RTL or board work.

### Deliverables
- 44-variable plus accumulation (`ternaryMacAccumulateFortyFourPlusGeneric`).
- 43-variable minus accumulation (`ternaryMacAccumulateFortyThreeMinusGeneric`).
- Depth-21 residual cancellation (`ternaryMacVigintiunupleCancellationGeneric`) — odd-depth residual `mac(x,a,.plus)`.
- 27th proof lattice dimension: zero-weight undecuple closure (`ternaryMacZeroWeightUndecupleClosureGeneric`).
- 27 IGLA spec blocks (+54 tests, +27 invariants).
- Regenerate all 27 IGLA seals.
- Full conformance suite green.

### Pros
- No hardware or compiler dependencies.
- Highest confidence; lowest risk.
- Continues the streak and increases the generic ∀ gap over competitors.

### Cons
- FPGA evidence still missing for a fifth consecutive wave.
- Verilog backend defects 1/3/4/5 remain untouched.

### Estimated generic ∀ after W368
**216**

---

## Variant B — Formal extension + retry board flash + one safe gen-verilog fix (recommended)

### Scope
Extend the proof lattice **and** retry physical board loading, plus land **one** additional safe `gen-verilog` sub-fix from #1245.

### Deliverables
- All of Variant A.
- Retry `dlc10 idcode` / `sram` / `flash` on the QMTech Wukong V1.
  - If board is found: load `ternary_mac_demo_top.bit` and capture IDCODE + DONE pin evidence.
  - If still missing: keep the consolidated hardware-connectivity tracking issue open and update `FPGA_EVIDENCE_W368.md`.
- Land **one** regression-free `gen-verilog` fix, chosen from:
  - Extend `0x`/`0b` literal width padding to **non-const contexts** (assignments, expressions) by threading expected width into `gen_verilog_expr`,
  - or add a **narrow parser context flag** that lets `is_top_level_start()` recognize `KwConst`/`KwVar` at true top level without breaking nested-block error recovery (attack defect 1),
  - or a `#[verilog_skip]`-style guard for specs that trigger defects 3–5.
- If no safe fix is found, promote the reproduction doc to a full `docs/reports/GEN_VERILOG_BACKEND_ROADMAP.md`.

### Pros
- Addresses the two largest project weak points while keeping risk bounded.
- Moves the Verilog backend from "two defects smaller" to "three defects smaller".
- Best balance of risk and strategic value.

### Cons
- Board load is outside software control; may still fail.
- Even narrow compiler changes can cause regressions; requires staged validation.

### Estimated generic ∀ after W368
**216** (RTL work does not change theorem count).

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
- Largest scope; may spill into W369.
- RTL parser can become a new maintenance burden if not kept narrow.
- Requires careful L2 GENERATION compliance: do not hand-edit `gen/`.

### Estimated generic ∀ after W368
**216** (RTL bridge does not add generic ∀ theorems directly).

---

## Recommendation

**Select Variant B.**

W367 proved that the formal pipeline scales past 200 generic ∀, that a second safe `gen-verilog` sub-fix can land without mass seal regeneration, and that the board/cable remain unavailable. The next wave should extend the proof lattice, retry the board, and attempt the most impactful remaining safe backend fix — ideally a narrow attack on defect 1 (only first const emits) or width-context extension for non-const expressions. Variant B is the only option that simultaneously protects the zero-failure streak, keeps pressure on the silicon-evidence gap, and makes measurable progress on backend quality.

---

## Cross-wave target summary

| Wave | Generic ∀ | Accumulation depth | Minus depth | Cancellation depth | Closure dimension | Hardware |
|------|-----------|-------------------|-------------|-------------------|-------------------|----------|
| W363 | 196 | 39 | 38 | 16 | 22 (sextuple) | blocked |
| W364 | 200 | 40 | 39 | 17 | 23 (septuple) | blocked |
| W365 | **204** | **41** | **40** | **18** | **24 (octuple)** | blocked |
| W366 | **208** | **42** | **41** | **19** | **25 (nonuple)** | blocked |
| W367 | **212** | **43** | **42** | **20** | **26 (decuple)** | blocked |
| W368 | **216** | **44** | **43** | **21** | **27 (undecuple)** | retry flash |
