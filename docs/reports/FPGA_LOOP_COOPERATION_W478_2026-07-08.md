# Wave Loop 478 — Cooperation Variants (2026-07-08)

**Issue:** #TBD (to create)  
**Source wave:** Wave Loop 477 (compiler-backend hygiene + Icarus simulation gate)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 477 selects **Variant B**: with the physical Wukong XC7A100T bench still blocked by the missing DLC10 cable / unwired P12 relay, the wave hardens the `gen-verilog` backend for strict Verilog-2001 / Icarus Verilog compliance. The result is 645/645 non-smoke PASS, 125/125 yosys smoke PASS, and a new Icarus simulation gate with 92/125 clean targets (the remaining 33 failures are pre-existing packed-vector struct-array lowering gaps inherited from W475/W476).

Three candidate directions are offered for Wave Loop 478. The default recommendation is **Variant B** because it directly closes the largest remaining Icarus failure class while keeping the suite green.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist any new fixtures under `tests/fixtures/fpga/theorem-matrix/live-w478/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g. `XADC_LIVE_W478_OPERATING_POINT`, closing the live-measurement → formal-claim loop.
- If the W477 hoisting work is stable, include a generated-struct test bitstream and verify it still boots after hoisting.

**Pros:** advances the physical boot-evidence line, which is the project's strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Close Icarus failures in struct-array / packed-vector lowering (default)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Packed-vector whole-array / whole-struct assignment.** Rewrite array-of-struct and struct-with-array-field copy lowering so Icarus no longer sees assignment to an entire array or indefinite-width concatenation operands. This is the dominant failure mode in the 33 baseline Icarus smoke failures.
2. **Icarus-warning hygiene gate.** Extend the Icarus phase to also surface elaboration warnings (e.g., port coercion, implicit nets) so the gate is stricter than "errors only".
3. **Adversarial Icarus witness spec.** Add `specs/scratch/w478_icarus_struct_array.t27` that deliberately exercises the previously failing patterns and passes both yosys and Icarus.
4. **Whole-struct return writeback re-audit.** Revisit the W474/W475 AOS return path with Icarus in the loop; ensure packed temporaries have known widths and no array-slice assignments.

**Pros:** turns the Icarus gate from 92/125 to 125/125, eliminating a major simulator portability gap; no hardware dependency; maintains the zero-IGLA-failure streak.

**Cons:** touches the most fragile lowering path (packed-vector memory); regression risk requires careful resealing.

**Recommended:** **Variant B** is the default for W478.

---

## Variant C — Formal fallback (if Variant B is too large for one wave)

**Trigger:** packed-vector array-of-struct lowering proves larger than one wave, or an Icarus-correct rewrite would destabilize yosys.

**Work:**
- Add a Lean 4 synthesizability lemma in `proofs/lean4/Trinity/TernaryFPGABoot.lean` for the declaration-hoisting transformation: hoisted Verilog is semantically equivalent to the original generated Verilog.
- Add a correctness lemma that the `assert(cond) else $fatal(...)` emission preserves test semantics under Icarus.
- Add an adversarial Verilog-AST witness that scans generated `.v` files for declarations after statements, unhoisted attributes, or indefinite-width concatenations and reports them without needing Icarus installed.

**Pros:** hardens the formal side of the compiler backend and gives a machine-checkable contract for the hoisting pass.

**Cons:** does not close the Icarus simulation gap as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before W478 planning is final, in which case switch to **Variant A**. If the packed-vector struct-array Icarus rewrite turns out to be larger than one wave, fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
