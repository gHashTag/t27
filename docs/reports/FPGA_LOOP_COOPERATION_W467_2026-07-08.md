# Wave Loop 467 — Cooperation Variants (2026-07-08)

**Issue:** #1445 (to create)  
**Source wave:** Wave Loop 466 (#1444)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 466 selected **Variant B** from the W466 cooperation plan: with the
physical bench still blocked by the missing DLC10 cable / unwired P12 relay,
the wave continued the `gen-verilog` compiler-backend hardening line. W466
landed nested struct-array flattening, variable-index read/write on local
struct arrays, and a regression spec for mixed direct/indirect struct-literal
array arguments. The suite is green: **602/602 non-smoke PASS**, **82/82 yosys
smoke PASS**, `cargo test -p t27c --bin t27c` **1524 passed, 0 failed**.

Three candidate directions are offered for Wave Loop 467. The default
recommendation is **Variant B** because the physical bench remains unavailable.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong
XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w467/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g.
  `XADC_LIVE_W467_OPERATING_POINT`, closing the live-measurement → formal-claim
  loop.

**Pros:** advances the physical boot-evidence line, which is the project's
strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Continue compiler-backend hardening (default if bench blocked)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Multi-dimensional struct arrays.** Extend the flattening/lower machinery to
   handle arrays of arrays of structs (`[M][N]Pt`) and arrays of structs whose
   fields are themselves arrays (`Pt { coords : [3]u8 }`).
2. **Whole-struct assignment by value.** Lower statements such as
   `a = b` or `pts[idx] = entry` where both sides are structs, by decomposing
   into per-field scalar assignments.
3. **Keyword-safe generated clone memory names.** When a struct-array parameter
   is cloned, ensure that derived memory names like
   `_lit_3_Pt_struct_x_1_y_2_..._reg` remain single escaped tokens if a field
   happens to be named `reg`, `wire`, or another Verilog keyword.

**Pros:** directly extends the struct-array line that W455–W466 have been
hardening; no hardware dependency; maintains the zero-IGLA-failure streak.

**Cons:** does not produce new physical evidence.

**Recommended:** **Variant B** is the default for W467.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** multi-dimensional struct arrays prove too large for one wave, or a
regression-free implementation cannot be found quickly.

**Work:**
- Add a synthesizability theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  for variable-index struct arrays (priority-mux / if-else form).
- Add a mixed-call-site struct-literal correctness lemma that relates the
  `array_param_anon_roms` signature to the clone-function argument list.
- Add an adversarial nested-struct witness for keyword/memory-name escape.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native
assurance even when the bench is blocked.

**Cons:** does not close a user-facing compiler gap as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before
W467 planning is final, in which case switch to **Variant A**. If the
multi-dimensional struct-array refactor turns out to be larger than one wave,
fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
