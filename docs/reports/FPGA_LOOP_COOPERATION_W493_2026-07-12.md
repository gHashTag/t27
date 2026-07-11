# Wave Loop 493 — Cooperation Variants

**Date:** 2026-07-12  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Previous wave:** W492 closes in `docs/reports/WAVE_LOOP_492_CLOSEOUT.md`.

---

## Common gate

Whatever variant is chosen, the next wave must keep:

- 693 / 693 non-smoke PASS (681 base specs + 6 W490 + 4 W491 + 2 W492 scratch witnesses).
- 172 / 173 yosys smoke PASS, 1 documented baseline failure
  (`specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`).
- 171 / 173 Icarus smoke PASS, 2 documented baseline failures
  (`specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`,
  `specs/scratch/w492_predicate_rejects_nested_return_field.t27`).
- 693 / 693 seal matches.
- `cargo test -p t27c --bin t27c` green (1525 / 0 / 2).
- Zero `UNSUPPORTED_ICARUS` placeholders outside the documented adversarial witnesses.
- `t27c suite --repo-root . --fast --icarus-lowerable` green (zero disagreements).
- `tri verify --lean-lowerable` green (253 modeled specs proved lowerable).

The W492 branch must land before W493 work begins.

---

## Variant A (default) — Machine-checked semantic equivalence for the Icarus-lowerable scalar subset

**Theme:** push the W492 soundness proof from "no placeholders" to "same
results" for the fully-modeled scalar subset.

### Work

1. **Operational semantics for the simplified t27 AST.**
   - Define a small-step or denotational interpreter for the `IcarusLowerable`
     expression and statement language over concrete bit-vector values.
2. **Verilog evaluation relation.**
   - Give the shallow `VExpr` / `VStmt` AST a synchronous-read/asynchronous-assign
     semantics that matches the subset Icarus actually simulates.
3. **Value-preservation theorem.**
   - Prove that for any lowerable t27 module and any vector of input values, the
     emitted Verilog produces the same outputs as the t27 interpreter.
   - Start with scalar arithmetic, array indexing, and struct field access; leave
     multi-clock / memory initialization for a follow-up wave.
4. **Witness-driven proof.**
   - Use the 253 specs already imported into `Completeness.lean` as regression
     fuel; prove the theorem for the W492 positive witness first, then generalize.
5. **Optional Yosys/SMT bridge.**
   - If the manual proof path is too heavy, emit the shallow Verilog modules to
     SMT via Yosys `smt2` and compare against the t27 interpreter on a finite set
     of input vectors as a translation-validation fallback.

### Target outcome

- A machine-checked guarantee that the Icarus-lowerable subset is not only
  placeholder-free but bit-value preserving on scalar expressions.
- 693 / 693 non-smoke PASS, baseline failures unchanged, seals green,
  lowerability gate green.

### Pros / cons

- **+** Locks the full t27 → Icarus contract, not just the predicate.
- **+** Creates a reusable correctness argument for future backends.
- **−** Large proof-engineering effort; may need to narrow the subset further.
- **−** Does not remove any of the current baseline failures.

### Risk

High.  Going from syntactic soundness to semantic equivalence is a significant
step up in proof complexity.

---

## Variant B — Close the next layer of gen-verilog struct/call gaps

**Theme:** make the two documented adversarial baseline witnesses lowerable,
plus the module-scope/import-boundary gaps W492 left out of scope.

### Work

1. **Nested struct-return field access.**
   - Support `make_outer().inner.v` where `make_outer` returns a struct whose
     field is itself a scalar struct.  The emitter currently emits an
     `UNSUPPORTED_ICARUS` placeholder.
2. **Nested struct-literal field from a struct-typed parameter.**
   - Fix `specs/scratch/w491_nested_struct_return_field_not_lowerable.t27` so
     that a struct-literal field initialized from a struct-typed parameter lowers
     to legal Verilog instead of the current malformed syntax.
3. **Module-scope AOS constants from imported calls.**
   - Make `const pts : [N]Pt = make_pts();` work when `make_pts` is imported from
     another module.
4. **Host-only propagation across import boundaries.**
   - Ensure a function classified as host-only in its home module is also skipped
     in downstream modules that import it, even without a direct reference.
5. **Adversarial witnesses** for each new path in `specs/scratch/`.

### Target outcome

- The two documented baseline failures are removed or replaced by smaller,
  explicitly scoped adversarial witnesses.
- 693 / 693 non-smoke PASS, 0 yosys failures, 0 Icarus failures, seals green,
  lowerability gate green.

### Pros / cons

- **+** Directly expands functional coverage and removes baseline debt.
- **+** Each sub-fix is bounded and independently testable.
- **−** Touches import-resolution reachability and nested-call lowering.
- **−** Removing a baseline requires reseal and may perturb the lowerability gate.

### Risk

High-medium.  The fixes are bounded but span import resolution, nested call
lowering, and module-level constant initialization.

---

## Variant C — FPGA live cold-POR / SPI flash boot evidence

**Theme:** collect physical evidence on the QMTech Wukong V1 / XC7A100T-FGG676
using the in-repo `dlc10` driver.

### Work

1. Bring the DLC10 cable to the board and verify IDCODE `0x13631093` with
   `dlc10 idcode`.
2. Program a freshly generated smoke bitstream into SPI flash with `dlc10 flash`.
3. Run a cold-POR boot sweep across OSCFSEL variants and record boot-log JSON.
4. Compare measured CCLK period against the formal PVT envelope theorems in
   `lake/` / `proofs/lean4/`.
5. Update `fpga/HARDWARE_SSOT.md` with the observed boot signature and any
   cable/board notes.

### Target outcome

- A new `docs/reports/FPGA_EVIDENCE_W493.md` with measured boot traces and a
  comparison to the formal envelope.
- 693 / 693 non-smoke PASS, baseline failures unchanged (FPGA work does not
  touch spec generation), lowerability gate green.

### Pros / cons

- **+** Provides irreplaceable physical validation.
- **+** Directly feeds the FPGA evidence backlog.
- **−** Blocked on hardware availability (DLC10 cable and board).
- **−** If the cable is missing, the wave cannot complete.

### Risk

High because of external dependency, but technically low if hardware is present.

---

## Recommendation

Choose **Variant A** as the default.  W492 proved that the lowerability subset is
placeholder-free; W493 should take the next logical step and prove that it is
also value-preserving on the scalar subset.  If the proof turns out to be too
heavy, fold the easier cases into the existing `Soundness.lean` and pivot to
**Variant B** to remove baseline debt.  If the DLC10 cable arrives mid-wave, run a
short **Variant C** hardware sprint on a side branch without blocking the formal
work.

---

*φ² + φ⁻² = 3 | TRINITY*
