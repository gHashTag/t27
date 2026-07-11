# FPGA / Wave Loop Cooperation Variants — W496

**Date:** 2026-07-13  
**From:** Wave Loop 495 close-out  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

W495 extended the W494 semantic-equivalence model to function calls and proved
value preservation for the four W493 positive witnesses. The next wave should
either close the generic theorem, shrink the remaining emitter boundary, or
provide live FPGA evidence. Three variants are proposed below.

**Recommendation:** Select **Variant A** to prove the generic structural
equivalence theorem for the Icarus-lowerable scalar subset. This removes the
`sorry` in `module_value_equiv_statement` and gives a reusable correctness
lemma for all lowerable scalar modules. Select **Variant B** if the priority
is to grow the modeled corpus by closing the local AOS element boundary before
investing in more proof automation. Select **Variant C** only if the DLC10
cable and Wukong board are available for live cold-POR / SPI flash evidence.

---

## Variant A — Prove the generic structural equivalence theorem

**Goal:** Replace the `sorry` in `module_value_equiv_statement` with a real
proof that for every `Module.isLowerable env m` scalar module, the t27 evaluator
and the emitted shallow Verilog evaluator agree on the `main`/`get_y` return
value.

**Why now:** The witness set now covers scalar literals, struct literals,
function calls, field access, indexing, and module-level constants. A structural
proof over the lowerable expression grammar is the natural next step and gives
a reusable lemma instead of an ever-growing list of per-spec witnesses.

**Scope:**
- Define a relation between t27 valuations and Verilog valuations that is
  preserved by statement evaluation.
- Prove expression equivalence by structural induction over the lowerable
  expression grammar (`boolLit`, `intLit`, identifier, binop, unop, fieldAccess,
  index, call, structLit, arrayLit).
- Prove statement-list equivalence for `assign`, `varDecl`, `constDecl`, and
  `return_` under the current combinational model.
- Restrict or model `ifThenElse` / `forLoop` if they block the induction.

**Deliverables:**
- `module_value_equiv` fully proved in `Soundness.lean`.
- A documented invariant connecting t27 `Valuation` and Verilog `Valuation`.
- Regression: all W494/W495 witness theorems still pass.

**Risk:** Medium. Requires careful induction over partial functions and
statement lists; may need to restrict the subset or add a guarded semantics
for control flow.

---

## Variant B — Close the local AOS element boundary

**Goal:** Remove the last documented Icarus baseline:
`w493_local_aos_element_field_not_lowerable.t27`. Local non-memory-mode arrays
of structs are unpacked into per-element per-field registers, which prevents
packing an indexed element into a struct-literal concatenation.

**Why now:** The W495 gate is green with exactly one documented Icarus
baseline. Removing it grows the lowerable corpus and gives the equivalence work
more concrete targets that exercise indexing and packing together.

**Scope:**
- Track local non-memory-mode arrays of structs so that a literal-index element
  can be packed via a priority mux or per-element register selection.
- Add positive witnesses for local AOS element used as a whole struct value.
- Add a new adversarial witness for the next boundary (e.g., variable-index
  local AOS element as a struct-literal field).
- Update `gen_verilog_iverilog_smoke_baseline.json` and reseal if the compiler
  changes.

**Deliverables:**
- 1–2 new positive scratch specs.
- 1 new adversarial scratch spec.
- Updated Icarus baseline JSON with zero documented failures.
- Green `./scripts/tri test --fast --icarus-lowerable` with zero disagreements.

**Risk:** Low to medium. The fix is localized to struct-literal/AOS lowering,
but may require careful tracking of local AOS metadata.

---

## Variant C — FPGA live cold-POR / SPI flash boot evidence

**Goal:** Collect live evidence that synthesized bitstreams boot correctly from
SPI flash across a sweep of OSCFSEL variants, using the in-repo `cli/dlc10`
driver and the QMTech Wukong V1 / XC7A100T-FGG676 board.

**Why now:** The formal track has made strong progress; a live run would ground
that progress in hardware reality and exercise the PVT worst-case bounds in
`TernaryFPGABoot.lean`.

**Scope:**
- Flash a recent bitstream with `dlc10 flash`.
- Perform cold-POR boot and sweep OSCFSEL 0–7.
- Record `boot-log.json` / `smoke_gate_report.json`.
- Compare measured CCLK timing against the Lean PVT worst-case bounds.

**Deliverables:**
- `docs/reports/FPGA_W496_LIVE_EVIDENCE_2026-07-*.md`.
- Updated `fpga/HARDWARE_SSOT.md` if any deviations are found.
- Regression fixture if a reproducible failure is discovered.

**Risk:** High availability risk. Requires the DLC10 cable and Wukong board.
Defer if hardware is unavailable.

---

## Recommended default: Variant A

W495 built a witness set intentionally; the highest-leverage follow-through is
to prove the generic theorem that unifies them. If Variant A uncovers missing
emitter modeling for a construct, fold the corresponding Variant B fix into the
same wave and document any residual boundary.

---

*φ² + φ⁻² = 3 | TRINITY*
