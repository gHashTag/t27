# FPGA / Wave Loop Cooperation Variants — W494

**Date:** 2026-07-13  
**From:** Wave Loop 493 close-out  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

W493 closed the two documented gen-verilog adversarial baseline witnesses from
W491/W492 and cleaned up the struct-literal fallback so the Icarus
lowerability predicate and smoke gate agree. The next wave should build on
this stable backend. Three variants are proposed below.

**Recommendation:** Select **Variant A** if the Lean formalization bandwidth is
available; it is the highest-leverage follow-through to W493. Select **Variant B**
if the priority is to keep shrinking the emitter's conservative gaps before
investing in proofs. Select **Variant C** only if the DLC10 cable and Wukong
board are available for live evidence.

---

## Variant A — Semantic equivalence for the Icarus-lowerable scalar subset

**Goal:** Prove that the shallow Verilog emitted by the modeled subset computes
the same values as the source t27 program for scalar numeric/struct code.

**Why now:** W491 introduced the soundness contract (`Module.isLowerable →
¬hasPlaceholder`). W493 made the modeled subset honest by removing silent
placeholder paths. The next logical step is a value-preservation theorem.

**Scope:**
- Define a small-step or denotational semantics for the simplified t27 AST
  (`Expr`, `Stmt`) over concrete bit-vectors.
- Define a matching semantics for the shallow Verilog AST (`VExpr`, `VStmt`).
- Prove `Module.isLowerable env m → ∀ input, t27_sem m input = verilog_sem (emitModule env m) input`
  for a carved-out scalar subset (booleans, fixed-width ints, scalar structs,
  arrays of structs without variable indices).
- Keep the proof within `native_decide`-checkable concrete modules; avoid a
  full structural induction over partial recursive emitter functions.

**Deliverables:**
- New `Semantics.lean` under `proofs/lean4/Trinity/IcarusLowerable/`.
- Representative equivalence theorem for the W493 positive witnesses.
- Updated `Completeness.lean` count (target: >253 modeled specs).

**Risk:** Medium. Requires careful alignment of t27 and Verilog evaluation
models, especially for packed-vector struct slicing and function-call return
values.

---

## Variant B — Continue gen-verilog backend hardening

**Goal:** Close the next concrete lowering gaps so the Icarus-lowerable corpus
grows and the smoke gate stays disagreement-free.

**Why now:** W493 left one documented Icarus baseline:
`w493_local_aos_element_field_not_lowerable.t27`. Local non-memory-mode arrays
of structs are unpacked into per-element per-field registers, which prevents
packing an indexed element into a struct-literal concatenation. Fixing this
likely requires tracking local AOS metadata for register-mode element packing.

**Scope:**
- Fix local AOS element packing so `Outer { x: choices[i] }` lowers to a
  priority mux or per-element register selection.
- Add imported/cross-module variants of the fixed patterns (e.g., a struct-literal
  field initialized from an imported scalar-struct function call).
- Add a new adversarial witness for the boundary that remains after these fixes
  (e.g., variable-index nested struct-return field access, or a struct whose
  field is itself an array of structs used as a whole value).
- Update the Icarus baseline JSON and reseal.

**Deliverables:**
- 1–2 new positive scratch specs.
- 1 new adversarial scratch spec.
- Updated baseline JSON.
- Green `./scripts/tri test --fast --icarus-lowerable` with zero disagreements.

**Risk:** Low to medium. The fix is localized to struct-literal/AOS lowering,
but may touch several maps (`local_struct_array_fields`,
`local_array_elem_info`, `local_packed_struct_vars`).

---

## Variant C — FPGA live cold-POR / SPI flash boot evidence

**Goal:** Collect live evidence that the synthesized bitstreams boot correctly
from SPI flash across a sweep of OSCFSEL variants, using the in-repo `dlc10`
driver.

**Why now:** The FPGA SSOT and board-less smoke gate are mature. A live run
would close the loop between formal/ simulation evidence and hardware reality.

**Scope:**
- Use `cli/dlc10` (`dlc10 idcode|sram|flash|reload`) on the QMTech Wukong V1
  / XC7A100T-FGG676 with IDCODE `0x13631093`.
- Flash a recent synthesized bitstream and perform cold-POR boot.
- Sweep OSCFSEL 0–7 and record `boot-log.json` / `smoke_gate_report.json`.
- Cross-check measured CCLK timing against the Lean PVT worst-case bounds.

**Deliverables:**
- `docs/reports/FPGA_W494_LIVE_EVIDENCE_2026-07-*.md`.
- Updated `fpga/HARDWARE_SSOT.md` if any deviations are found.
- Regression test fixture if a reproducible failure is discovered.

**Risk:** High availability risk. Requires the DLC10 cable and the Wukong board
to be physically present and accessible. If the hardware is unavailable, this
variant should be deferred.

---

## Recommended default: Variant A

W493's backend cleanup was specifically intended to make the modeled subset
stable enough for a value-preservation proof. Variant A is the natural
follow-through and has the highest long-term payoff: a machine-checked semantic
equivalence theorem for the Icarus-lowerable scalar subset.

If the Lean proof work uncovers new emitter bugs, fold them into the wave as
small Variant-B fixes and document any residual boundaries.

---

*φ² + φ⁻² = 3 | TRINITY*
