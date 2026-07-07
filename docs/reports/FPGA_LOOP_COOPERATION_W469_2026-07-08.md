# Wave Loop 469 — Cooperation Variants (2026-07-08)

**Issue:** #1447 (to create)  
**Source wave:** Wave Loop 468 (#1446)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 468 selected **Variant B** from the W468 cooperation plan: with the
physical bench still blocked by the missing DLC10 cable / unwired P12 relay, the
wave continued the `gen-verilog` compiler-backend hardening line. W468 landed
struct-return function call assignment, 2D scalar local arrays, and local
RAM-style pragma propagation. The suite is green: **610/610 non-smoke PASS**,
**90/90 yosys smoke PASS**, `cargo test -p t27c --bin t27c` **1524 passed,
0 failed**.

Three candidate directions are offered for Wave Loop 469. The default
recommendation is **Variant B** because the physical bench remains unavailable.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong
XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w469/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g.
  `XADC_LIVE_W469_OPERATING_POINT`, closing the live-measurement → formal-claim
  loop.

**Pros:** advances the physical boot-evidence line, which is the project's
strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Continue compiler-backend hardening (default if bench blocked)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Multi-dimensional arrays of structs.** Extend the flattening/lower
   machinery to handle arrays of arrays of structs (`[M][N]Pt`) and arrays of
   structs whose fields are themselves arrays (`Pt { coords : [3]u8 }` at
   module level / in array parameters).
2. **Module-level scalar struct variables / consts.** Lower module-level
   `var state : Pt = Pt{...}` or scalar struct `const` into per-field
   registers/memories (`state_x`, `state_y`).
3. **Scalar struct parameters.** Lower `fn f(p : Pt)` into multiple Verilog
   inputs and bind field access at call sites.
4. **Whole-struct comparison.** Lower `a == b` for flattened struct variables
   into a field-wise equality expression ANDed together.

**Pros:** directly extends the struct-array line that W455–W468 have been
hardening; no hardware dependency; maintains the zero-IGLA-failure streak.

**Cons:** does not produce new physical evidence.

**Recommended:** **Variant B** is the default for W469.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** multi-dimensional struct arrays, scalar struct parameters, or
whole-struct comparison prove too large for one wave, or a regression-free
implementation cannot be found quickly.

**Work:**
- Add a synthesizability theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  for struct-return packing/unpacking (the packed concatenation is equivalent to
  per-field assignment).
- Add a 2D scalar array indexing correctness lemma that relates source
  `m[i][j]` accesses to the flattened per-leaf register names.
- Add an adversarial local-RAM-style pragma witness that checks the emitted
  attribute is syntactically valid and survives yosys elaboration.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native
assurance even when the bench is blocked.

**Cons:** does not close user-facing compiler gaps as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before
W469 planning is final, in which case switch to **Variant A**. If the
multi-dimensional struct-array / scalar struct parameter refactor turns out to
be larger than one wave, fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
