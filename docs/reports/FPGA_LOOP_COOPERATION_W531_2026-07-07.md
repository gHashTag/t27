# FPGA Loop Cooperation Variants — Wave Loop 531

**Date:** 2026-07-07  
**From wave:** 530  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 530 made the Icarus-lowerability classifier executable:

- `./scripts/tri test --icarus-simulate --icarus-lowerable` now runs the first
  simulation regression suite (10 W493–W529 witnesses) with **0 failures**.
- A latent 2-D packed-vector layout bug was fixed by reversing concatenation
  order so t27 index `[0]` maps to the LSB.
- 125 specs were resealed after the Verilog layout change.
- 16 pre-existing yosys smoke failures remain as documented baselines.

The next wave should either broaden the lowerable subset or harden the gate.

---

## Variant A — Extend Icarus simulation to W493–W531 new witnesses (recommended)

**Goal:** Keep the simulation gate green while adding the next lowerable
witnesses produced in W531.

**Scope:**
1. Add any new W531 lowerable scratch specs to the regression whitelist.
2. Record JSON baselines under `.trinity/icarus-baselines/` for the new
   witnesses.
3. Refine the classifier so it accepts the new shapes without letting older
   non-lowerable scratch specs leak in.
4. Maintain the invariant: `./scripts/tri test --icarus-simulate` reports 0
   Icarus simulation failures.

**Why recommended:** The gate is now the project's strongest protection against
Verilog semantic drift. Growing it incrementally is lower risk than changing the
subset semantics.

---

## Variant B — Support signed scalar-array fields in packed scalar structs

**Goal:** Broaden the lowerable subset to structs whose fields are small signed
scalar arrays.

**Scope:**
1. Allow scalar-struct fields of the form `[N]i8`, `[N]i16`, etc. in the packed
   vector layout.
2. Emit signed packed vectors where needed and preserve sign extension on slice
   reads.
3. Add positive scratch witnesses for signed array-field read, copy, param, and
   return.
4. Add negative witnesses for non-lowerable cases (string/enum fields).
5. Extend `Trinity.IcarusLowerable` value-preservation theorems and reseal.

**Why valuable:** Many real t27 specs use signed fixed-size arrays inside
structs. Closing this gap removes a major cause of `UNSUPPORTED_ICARUS`
placeholder fallback.

---

## Variant C — Harden the lowerability boundary with adversarial proofs

**Goal:** Make the classifier falsifiable and document the exact lowerability
boundary.

**Scope:**
1. Add negative witnesses for constructs that must remain non-lowerable:
   unresolved imports, host-only helpers, casts (`as`), enum/string fields in
   packed arrays, and unbounded dynamic loops.
2. Prove `¬ Module.isLowerable env m` for each negative witness in Lean 4 using
   `native_decide` or the classifier predicate.
3. Add a Rust integration test that checks the classifier rejects exactly the
   same specs the Lean predicate rejects.
4. Document the boundary so future compiler changes cannot silently expand the
   lowerable subset.

**Why valuable:** A soundness proof is only as strong as the gate it protects.
Adversarial witnesses make classifier regressions detectable in both code and
proof.

---

## Recommended variant

**Variant A** is recommended. W530 proved the simulation gate catches real
semantic bugs. The safest next step is to keep that gate green while folding in
W531's new lowerable witnesses, then alternate with Variants B and C to broaden
and harden the subset.

---

*φ² + φ⁻² = 3 | TRINITY*
