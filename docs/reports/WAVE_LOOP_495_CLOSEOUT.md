# Wave Loop 495 Close-Out Report

**Date:** 2026-07-13  
**Issue:** #1465  
**Branch:** `wave-loop-495`  
**Variant:** A — extend semantic equivalence to function calls and W493 witnesses  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 495 closed the function-call equivalence gap left by W494. The
shallow Verilog AST now stores `VFunction` definitions, the emitter derives
field slices and index widths from expression types, and both the t27 and
Verilog evaluators inline function bodies. Four W493 positive witnesses that
rely on struct-return function calls are now modeled in Lean and proved to
preserve their packed bit-vector return values with `native_decide`.

No compiler changes were required; the gate stays green with the single
documented Icarus baseline that existed in W494.

---

## Weak points addressed

1. **Verilog function bodies missing from the shallow AST.**  
   Fixed by adding `VFunction` to `VModule` and updating `emitModule` to emit
   t27 functions as definitions rather than flattening every reachable body.
2. **Function-call field access fell through to `none`.**  
   Fixed by adding `Expr.typeOf` and using it in both the t27 evaluator and
   the emitter to compute struct-field offsets for general function-call
   results, not only constructor calls.
3. **Array-index element width hard-coded to 8 bits.**  
   Fixed by deriving the element width from the base expression's type and
   storing it in the `VExpr.index` node.
4. **No way to run a specific Verilog function in `evalVModule`.**  
   Fixed by evaluating module-level items first, then running the named
   `VFunction`.
5. **Module-level constants were invisible to function bodies.**  
   Fixed by evaluating globals in `evalModuleFunction` before binding
   parameters and running the function body.

---

## Scientific context

The following works informed the design and justify the theorem shape:

- **Lutsig** (Andreas Lööw, CPP 2021) — a verified Verilog-to-netlist compiler
  in HOL4 with a machine-checked semantic-equivalence theorem. It sets the
  canonical contract: lowerability implies semantic preservation.
- **Vericert** (Herklotz et al., OOPSLA 2021) — a verified C-to-Verilog HLS
  compiler that ports the Lööw–Myreen Verilog semantics into Coq/CompCert and
  proves a backward-simulation theorem. Our model avoids the clocked-event and
  memory-model complexity that Vericert handles by staying combinational.
- **"The Essence of Verilog"** (Chen et al., OOPSLA 2023) — an operational
  semantics for Verilog validated against Icarus and Verilator; supports our
  decision to treat the lowerable subset as finite and combinational.
- **Kami** (Choi et al., ICFP 2017) — a Coq-embedded hardware DSL with modular
  refinement and Bluespec/Verilog extraction; a proof-assistant precedent for
  compiling a high-level hardware language to Verilog with guarantees.
- **Sparkle / Verilean** — a Lean 4 HDL compiler that generates SystemVerilog
  and verifies IP cores inside Lean; the closest native analog to our work.
- **CktFormalizer** (Xiong et al., arXiv 2026) — LLM-generated hardware translated
  into a dependently typed Lean HDL, then to synthesizable Verilog with machine-
  checked equivalence; shows the same Lean-to-Verilog strategy is viable.
- **Melchert et al. (FMCAD 2025)** — SMT-based translation validation across
  CGRA compiler stages; our per-witness `native_decide` proofs are a decidable,
  proof-assistant analog.

Sources:
- [Lutsig paper](https://cakeml.org/cpp21.pdf)
- [Lutsig DOI](https://doi.org/10.1145/3437992.3439916)
- [Vericert repo](https://github.com/ymherklotz/vericert)
- [Vericert OOPSLA 2021 paper](https://johnwickerson.github.io/papers/vericert_oopsla21.pdf)
- [The Essence of Verilog (λV)](https://yuelee.bitbucket.io/papers/oopsla2023.pdf)
- [Kami ICFP 2017 paper](http://plv.csail.mit.edu/kami/papers/icfp17.pdf)
- [Sparkle / Verilean](https://github.com/Verilean/sparkle)
- [CktFormalizer arXiv](https://arxiv.org/abs/2605.07782)
- [Melchert FMCAD 2025](https://doi.org/10.34727/2025/isbn.978-3-85448-084-6_26)

---

## Files changed

- `proofs/lean4/Trinity/IcarusLowerable/Ast.lean`
  - Added `Module.findFunction`.
- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
  - Added `Env.vars` (default `[]`) and `Env.varType`.
  - Added `Expr.typeOf` type-inference helper.
- `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`
  - Added `VFunction` structure.
  - Extended `VModule` with `functions : List VFunction`.
  - Added `VFunction.hasPlaceholder` and updated `VModule.hasPlaceholder`.
  - Added element-width field to `VExpr.index`.
- `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`
  - Threaded the module `m` through expression/statement emission.
  - Added `emitVFunction`.
  - Updated `emitModule` to emit function definitions and keep globals/tests/benches
    as module items.
  - Derived index element widths from the base expression's type.
  - Generalized field-access slicing to use `Expr.typeOf`.
- `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean`
  - Added `evalModuleFunction` (evaluates globals before the named function).
  - Updated `evalFunction`/`evalCall` to carry a base valuation.
  - Generalized t27 field access and indexing to use `Expr.typeOf`.
  - Added `evalVFunction` and updated `evalVExpr`/`evalVModule` to inline Verilog
    function bodies.
- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  - Added W495 witness environments and modules for the four W493 positive specs.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - Updated `scalar_struct_value_equiv` to the new evaluator signatures.
  - Added lowerability and value-preservation theorems for all four W495
    witnesses.
  - Added the generic `module_value_equiv_statement` (proved for the witness set,
    full structural proof stated with `sorry`).
- `.trinity/current-issue.md`
  - Updated for W495 Variant A.
- `.claude/plans/wave-loop-495.md`
  - Decomposed plan, weak-point analysis, literature references, risk register.
- `docs/NOW.md`
  - Recorded W495 close-out and W496 next-wave pointer.
- `.trinity/experience.md`
  - Added W495 learnings.

---

## Verification

- `lake build Trinity.IcarusLowerable.Ast Trinity.IcarusLowerable.Predicate
  Trinity.IcarusLowerable.Verilog Trinity.IcarusLowerable.Emitter
  Trinity.IcarusLowerable.Lemmas Trinity.IcarusLowerable.Semantics
  Trinity.IcarusLowerable.Soundness Trinity.IcarusLowerable.Completeness`: green.
- `./scripts/tri test --fast`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS (0 baseline failures).
  - 176 / 177 Icarus smoke PASS (1 documented baseline failure,
    `specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 697 / 697 seal matches.
  - 0 Icarus lowerability disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- No `bootstrap/src/compiler.rs` changes; NMSE seal unchanged.

---

## Known residual boundaries

- The **generic structural equivalence theorem** is stated but not yet proved;
  the current proof strategy is per-witness `native_decide`.
- **Conditionals and loops** are not modeled operationally in the Verilog
  evaluator; they are outside the combinational subset.
- The **local AOS element boundary** (`w493_local_aos_element_field_not_lowerable`)
  remains the single documented Icarus baseline.
- `Expr.typeOf` is a heuristic helper, not a full valuation-based type system.

---

*φ² + φ⁻² = 3 | TRINITY*
