# FPGA Loop Cooperation Variants — Wave Loop 530

**Date:** 2026-07-07  
**From wave:** 529  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 529 formalized the W528 packed-vector 2-D array-of-scalar-struct
cross-boundary lowering in Lean 4. Four positive witnesses now have
machine-checked value-preservation proofs:

- module-level `const` read
- module-level `var` read
- 2-D AoS function parameter
- 2-D AoS function return bound to a local variable

The 16 pre-existing yosys smoke failures are unchanged, and `tri test` does
not yet invoke Icarus Verilog simulation automatically.

---

## Variant A — Icarus simulation gate in `tri test` (recommended)

**Goal:** Turn the static Icarus-lowerability classifier into a dynamic
simulation contract.

**Scope:**
1. Extend `./scripts/tri` to accept `--icarus-lowerable` / `--icarus-simulate`
   flags and pass them through to `t27c suite` (or a new `t27c` subcommand).
2. For every spec that the classifier marks lowerable, generate Verilog,
   compile it with `iverilog`, run it with `vvp`, and capture `$display`
   output.
3. Add JSON baselines for expected simulation output under
   `.trinity/icarus-baselines/`.
4. Promote the existing lowerable scratch witnesses (W493–W529) into the first
   regression suite.
5. Keep the 16 yosys smoke failures as documented baselines until they are
   fixed separately.

**Why recommended:** The W529 soundness proof shows that the emitted Verilog
computes the same values as t27 for the lowerable subset. A simulation gate
makes that contract executable and prevents silent semantic regressions.

---

## Variant B — Harden parameter/return packing for larger structs

**Goal:** Broaden the lowerable subset to cover more realistic struct shapes.

**Scope:**
1. Support 2-D AoS parameters/returns whose scalar-struct fields are
   themselves fixed-size scalar arrays (packed-element AoS crossing the
   function boundary).
2. Support mixed signed/unsigned scalar fields in packed structs, emitting
   signed packed vectors where needed.
3. Add negative witnesses for non-lowerable mixed cases (e.g., struct fields
   containing strings or enums).
4. Extend `Trinity.IcarusLowerable` with the new shapes and prove value
   preservation.
5. Reseal affected specs and keep smoke baselines flat.

**Why valuable:** Many real t27 specs use structs with small scalar-array
fields. Closing this gap removes a major reason specs fall back to
`UNSUPPORTED_ICARUS` placeholder code.

---

## Variant C — Negative/ adversarial lowerability proofs

**Goal:** Strengthen the formal gate by proving that the classifier rejects
non-lowerable constructs cleanly.

**Scope:**
1. Add negative witnesses for constructs that must remain non-lowerable:
   casts (`as`), unresolved imports, host-only helpers, enum/string fields in
   packed arrays, and dynamic loops without bounded ranges.
2. Prove `¬ Module.isLowerable env m` for each negative witness using
   `native_decide`.
3. Add a Lean theorem connecting the classifier's rejection to the absence of
   emitted placeholders.
4. Document the exact boundary so future compiler changes cannot silently
   expand the lowerable subset.

**Why valuable:** A soundness proof is only as strong as the gate it protects.
Adversarial witnesses make the lowerability predicate falsifiable and protect
against classifier regressions.

---

## Recommended variant

**Variant A** is recommended. The W529 soundness proof established the
static contract; the highest-value next step is to make it executable by
running Icarus on the lowerable subset inside `./scripts/tri test`. This
directly hardens the Verilog backend against semantic drift and gives the
project a repeatable simulation regression suite.

---

*φ² + φ⁻² = 3 | TRINITY*
