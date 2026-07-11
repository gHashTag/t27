# FPGA / Wave Loop Cooperation Variants — W495

**Date:** 2026-07-13  
**From:** Wave Loop 494 close-out  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

W494 defined the first scalar bit-vector semantics for the t27 AST and the
shallow Verilog AST, and proved value preservation for a representative
scalar-struct-literal witness. The next wave should extend that equivalence
result to the constructs that W493 made lowerable: function calls,
struct-return field access, and struct-literal fields from scalar-struct
identifiers. Three variants are proposed below.

**Recommendation:** Select **Variant A** to close the function-call
equivalence gap, which is the natural continuation of W494. Select **Variant B**
if the priority is to keep shrinking the emitter's conservative gaps before
proving more theorems. Select **Variant C** only if the FPGA hardware is
available for live evidence.

---

## Variant A — Extend semantic equivalence to function calls and W493 witnesses

**Goal:** Prove value preservation for the W493 positive witnesses that
involve function calls (`make_outer(make_inner(5)).x.y` and struct-literal
fields from identifiers).

**Why now:** W494 built the scalar semantics and proved one base case. The
remaining gap is function calls: the shallow Verilog AST stores `.call` nodes
but no function bodies. Once calls are modeled, the W493 witnesses become
straightforward equivalence targets.

**Scope:**
- Add function definitions to the shallow Verilog AST (`VFunction`) and extend
  `emitModule` to emit them.
- Extend `evalVExpr` to look up and inline Verilog function bodies, matching
  the t27 evaluator's inlining semantics.
- Prove equivalence for:
  - `w493_nested_struct_field_from_identifier_lowerable.t27`
  - `w493_local_scalar_struct_field_lowerable.t27`
  - `w493_module_scalar_struct_field_lowerable.t27`
  - `w493_module_aos_element_field_lowerable.t27`
- State (and partially prove) the generic theorem:
  `Module.isLowerable env m → evalModule env m = evalVModule env (emitModule env m)`.

**Deliverables:**
- Updated `Semantics.lean` with Verilog function inlining.
- New theorems in `Soundness.lean` for each W493 positive witness.
- A generic equivalence theorem statement with at least the base case proved.

**Risk:** Medium. Requires changing the shallow Verilog AST, the emitter, the
predicate, and the exporter/model. The `Completeness.lean` count may shift.

---

## Variant B — Continue gen-verilog backend hardening (local AOS element boundary)

**Goal:** Close the remaining Icarus baseline:
`w493_local_aos_element_field_not_lowerable.t27`. Local non-memory-mode arrays
of structs are unpacked into per-element per-field registers, which prevents
packing an indexed element into a struct-literal concatenation.

**Why now:** The W494 gate is green with exactly one documented Icarus
baseline. Removing it grows the lowerable corpus and gives the equivalence work
more concrete targets.

**Scope:**
- Track local non-memory-mode arrays of structs so that an indexed element can
  be packed via a priority mux or per-element register selection.
- Add positive witnesses for local AOS element used as a whole struct value.
- Add a new adversarial witness for the next boundary (e.g., variable-index
  local AOS element as a struct-literal field, or an array-struct field used as
  a whole value inside another struct literal).
- Update baselines and reseal if the compiler changes.

**Deliverables:**
- 1–2 new positive scratch specs.
- 1 new adversarial scratch spec.
- Updated `gen_verilog_iverilog_smoke_baseline.json`.
- Green `./scripts/tri test --fast --icarus-lowerable` with zero disagreements.

**Risk:** Low to medium. The fix is localized to struct-literal/AOS lowering,
but may require careful tracking of local AOS metadata.

---

## Variant C — FPGA live cold-POR / SPI flash boot evidence

**Goal:** Collect live evidence that synthesized bitstreams boot correctly from
SPI flash across a sweep of OSCFSEL variants, using the in-repo `dlc10` driver.

**Why now:** The formal track has made strong progress; a live run would ground
that progress in hardware reality and exercise the PVT worst-case bounds in
`TernaryFPGABoot.lean`.

**Scope:**
- Use `cli/dlc10` on the QMTech Wukong V1 / XC7A100T-FGG676 (IDCODE
  `0x13631093`).
- Flash a recent bitstream and perform cold-POR boot.
- Sweep OSCFSEL 0–7 and record `boot-log.json` / `smoke_gate_report.json`.
- Compare measured CCLK timing against the Lean PVT worst-case bounds.

**Deliverables:**
- `docs/reports/FPGA_W495_LIVE_EVIDENCE_2026-07-*.md`.
- Updated `fpga/HARDWARE_SSOT.md` if any deviations are found.
- Regression fixture if a reproducible failure is discovered.

**Risk:** High availability risk. Requires the DLC10 cable and Wukong board.
Defer if hardware is unavailable.

---

## Recommended default: Variant A

W494's representative equivalence theorem is intentionally a base case.
Extending it to function calls and the W493 witnesses is the highest-leverage
follow-through and keeps the formalization and backend tracks aligned.

If Variant A uncovers new emitter bugs, fold them in as small Variant B fixes
and document any residual boundaries.

---

*φ² + φ⁻² = 3 | TRINITY*
