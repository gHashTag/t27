# Wave Loop 468 — Cooperation Variants (2026-07-08)

**Issue:** #1446 (to create)  
**Source wave:** Wave Loop 467 (#1445)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 467 selected **Variant B** from the W467 cooperation plan: with the
physical bench still blocked by the missing DLC10 cable / unwired P12 relay,
the wave continued the `gen-verilog` compiler-backend hardening line. W467
landed whole-struct assignment by value, whole-element assignment into struct
arrays, struct fields that are fixed-size arrays, and a keyword-field clone-path
regression spec. The suite is green: **606/606 non-smoke PASS**, **86/86 yosys
smoke PASS**, `cargo test -p t27c --bin t27c` **1524 passed, 0 failed**.

Three candidate directions are offered for Wave Loop 468. The default
recommendation is **Variant B** because the physical bench remains unavailable.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong
XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w468/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g.
  `XADC_LIVE_W468_OPERATING_POINT`, closing the live-measurement → formal-claim
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
2. **Struct-return function call assignment.** Lower statements such as
   `let p : Pt = make_pt()` where `make_pt` returns a struct, by destructuring
   the packed tuple return into per-field registers.
3. **RAM style pragma support for local and parameter arrays.** Honor the
   existing `style` pragma for function-local and array-parameter memories,
   emitting the appropriate `(* ram_style = "..." *)` attribute in Verilog.

**Pros:** directly extends the struct-array line that W455–W467 have been
hardening; no hardware dependency; maintains the zero-IGLA-failure streak.

**Cons:** does not produce new physical evidence.

**Recommended:** **Variant B** is the default for W468.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** multi-dimensional struct arrays or struct-return assignment prove
too large for one wave, or a regression-free implementation cannot be found
quickly.

**Work:**
- Add a synthesizability theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  for whole-struct assignment decomposition (per-field scalar assignment is
  semantically equivalent to the source struct assignment).
- Add an array-field flattening correctness lemma that relates a source struct
  field `coords : [3]u8` to its flattened scalar leaf registers/memories.
- Add an adversarial keyword-field clone-memory witness that checks the
  W461/W463 clone path never emits a bare Verilog keyword token.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native
assurance even when the bench is blocked.

**Cons:** does not close a user-facing compiler gap as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before
W468 planning is final, in which case switch to **Variant A**. If the
multi-dimensional struct-array / struct-return refactor turns out to be larger
than one wave, fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
