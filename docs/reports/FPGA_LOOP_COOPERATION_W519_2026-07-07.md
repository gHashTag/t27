# FPGA Loop Cooperation Variants — Wave 519

**Date:** 2026-07-07  
**Source wave:** 518  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Status after Wave Loop 518

Wave Loop 18 closed all remaining gen-verilog smoke boundaries:

- W508 `break`/`continue` yosys/Icarus baselines cleared via a flag-based loop
  encoding.
- Function-local pragma attribute suppression cleared the final two Icarus
  baseline failures (`w468_local_ram_style.t27`,
  `w514_function_local_packed_aos_ram_style.t27`).
- `cargo test -p t27c --bin t27c`: 1525/0/2.
- `./scripts/tri test --icarus-lowerable --fast`: 0 failures, 0 baseline
  failures.
- `./scripts/tri verify --lean-lowerable`: passed with 251 lowerable specs.

---

## Proposed Wave 519 cooperation variants

### Variant A — Packed scalar struct equality / inequality (recommended)

**Goal:** add `==` and `!=` operators for packed scalar structs in the
Icarus-lowerable Verilog path.

**Why now:**

- Scalar structs are already lowered as single packed bit-vectors whenever all
  fields are lowerable scalar/array-of-scalar types (W509/W511).
- Equality of two packed vectors is a single Verilog `==` comparison.
- This is the largest remaining functional gap in the lowerable subset that has
  no adversarial witness yet.

**Deliverables:**

1. Parser/typechecker support for struct-typed `==` and `!=` in lowerable
   contexts (already accepted in host contexts).
2. Verilog backend emission that compares the two packed vectors directly.
3. Three scratch witnesses: local-to-local, module-to-local, and param-to-local
   packed scalar struct equality.
4. Icarus-lowerability predicate update in Lean 4 (if needed).
5. Reseal affected specs, update baselines, closeout report, and three W520
   variants.

**Risk:** low. The packed-vector layout is deterministic; equality is a direct
bit-vector operation. The main work is wiring the operator through the backend
and adding value-preservation theorems.

---

### Variant B — Multi-dimensional packed AOS parameters

**Goal:** extend W517 to allow arrays-of-structs with array-typed fields as
module/bench parameters in the Icarus-lowerable subset.

**Why now:**

- W517 cleared whole-array-field reads from packed scalar structs and packed AOS
  elements, plus scalar array function returns.
- The next natural boundary is passing a 2-D or deeper packed AOS across a
  module/bench parameter boundary.
- Requires extending argument packing to handle outer array dimensions and inner
  packed slices.

**Deliverables:**

1. Scratch witnesses for 2-D packed AOS parameters and nested AOS parameters with
   array-typed fields deeper than one level.
2. Update `gen_verilog_packed_struct_array_*` helpers to pack/unpack across
   parameter ports.
3. Lowerability and sequential value-preservation theorems in Lean 4.
4. Reseal affected specs and baselines.

**Risk:** medium. Parameter packing changes cross the module port boundary and
must preserve declaration order and bit-vector layout exactly.

---

### Variant C — Formal gap analysis and Icarus completeness audit

**Goal:** pause feature delivery and perform a formal/engineering audit of the
Icarus-lowerable subset.

**Why now:**

- The subset has grown rapidly across W507–W518.
- A systematic audit can identify latent disagreements between the Rust
  classifier and the Lean 4 predicate before they become baselines.
- Produces a ranked backlog and test-coverage map for subsequent waves.

**Deliverables:**

1. Generate the set of all specs that pass Icarus smoke but are classified
   `not lowerable`, and vice versa.
2. Add adversarial witnesses for any disagreements found.
3. Review every `UNSUPPORTED_ICARUS` placeholder in the corpus for
   re-classification.
4. Update `docs/reports/ICARUS_LOWERABLE_AUDIT_W519.md` with findings and a
   prioritized backlog.

**Risk:** low in terms of code churn, but may surface higher-risk follow-up work.

---

## Recommendation

**Select Variant A** for Wave 519. It is the smallest, most self-contained
feature with the highest user-facing value, and it closes an obvious gap in the
lowerable scalar-struct operator set. Variants B and C can follow in W520/W521
or be picked up in parallel by cooperating agents.

---

*φ² + φ⁻² = 3 | TRINITY*
