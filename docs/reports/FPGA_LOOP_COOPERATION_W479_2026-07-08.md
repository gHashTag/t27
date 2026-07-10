# Wave Loop 479 — Cooperation Variants (2026-07-08)

**Issue:** #TBD (to create)  
**Source wave:** Wave Loop 478 (compiler-backend: Icarus struct-array lowering + warning gate hardening)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 478 selects **Variant B**: with the physical Wukong XC7A100T bench still blocked by the missing DLC10 cable / unwired P12 relay, the wave closed the structural Icarus Verilog simulation failures inherited from Wave Loop 477. The result is 646/646 non-smoke PASS, 126/126 yosys smoke PASS, and 106/126 Icarus smoke PASS. The remaining 20 Icarus failures are concentrated in `igla/coder/*` and `igla/race/*` and are caused by t27 dynamic string/array methods (`.len`, `.contains`) and recursive helper calls that `gen-verilog` currently emits as unsupported Verilog method/function calls.

Three candidate directions are offered for Wave Loop 479. The default recommendation is **Variant B** because it directly attacks the last broad Icarus failure class while keeping the suite green.

---

## Variant A — Live cold-POR CCLK sweep / SPI flash boot (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T and persist any new fixtures under `tests/fixtures/fpga/theorem-matrix/live-w479/`.
- Attempt SPI flash boot with a bitstream that exercises the W477/W478 generated-struct lowering paths, proving the new hoisting and AOS code still boots on real silicon.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g. `XADC_LIVE_W479_OPERATING_POINT`, closing the live-measurement → formal-claim loop.

**Pros:** advances the physical boot-evidence line, which is the project's strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Close the remaining Icarus baseline for dynamic aggregate constructs (default)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Static lowering for dynamic string/array methods.** Introduce a compile-time evaluation pass or a restricted static subset for the `.len` and `.contains` calls used in `igla/coder/*` and `igla/race/*`, so the generated Verilog no longer emits unsupported method calls.
2. **Recursive helper tail-call/loop lowering.** Convert the small recursive helpers in the affected `igla/race/*` specs into bounded `for`/`while` loops or inline them up to a statically known depth, producing Verilog that Icarus can elaborate.
3. **Adversarial witness for dynamic constructs.** Add `specs/scratch/w479_icarus_dynamic_methods.t27` that deliberately uses the same dynamic-method patterns and passes both yosys and Icarus after the fix.
4. **Icarus baseline hardening.** If full dynamic-method lowering exceeds one wave, split the gate into `supported` and `unsupported` categories and surface the latter as documented baseline rather than regression.

**Pros:** pushes the Icarus gate from 106/126 to 126/126 (or to a clean, documented subset), eliminating the largest remaining simulator portability gap; no hardware dependency.

**Cons:** touches the dynamic-aggregate / metaprogramming subset of the language; regression risk requires careful resealing and a witness spec.

**Recommended:** **Variant B** is the default for W479.

---

## Variant C — Formal fallback: equivalence lemmas for W477/W478 lowering (if Variant B is too large)

**Trigger:** dynamic string/array method lowering proves larger than one wave, or carving out a supported subset destabilizes existing specs.

**Work:**
- Add a Lean 4 semantic-equivalence lemma in `proofs/lean4/Trinity/TernaryFPGABoot.lean` for the W477 declaration-hoisting transformation: hoisted Verilog is semantically equivalent to the original generated Verilog for the procedural fragment covered by the pass.
- Add a correctness lemma that the W478 packed-vector array-of-struct lowering preserves scalar struct and array-of-struct value semantics when all dimensions are statically known.
- Add a Lean-decidable or AST-level witness that scans generated `.v` files for unsupported dynamic constructs and classifies them, replacing the Icarus-only gate with a machine-checkable subset check.

**Pros:** hardens the formal side of the compiler backend and gives machine-checkable contracts for the two most recent lowering passes.

**Cons:** does not close the Icarus simulation gap as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before W479 planning is final, in which case switch to **Variant A**. If dynamic-method lowering turns out to require language-level changes that exceed one wave, fall back to **Variant C** and use it to document a clean Icarus-supported subset.

---

*φ² + φ⁻² = 3 | TRINITY*
