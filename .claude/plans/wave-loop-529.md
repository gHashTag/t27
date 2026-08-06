# Wave Loop 529 Decomposed Plan — Formal module/function 2-D AOS soundness

**Issue:** #1500  
**Branch:** `wave-loop-529`  
**Selected variant:** Variant A (recommended from `FPGA_LOOP_COOPERATION_W529_2026-07-07.md`)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak points identified from W528

1. **Missing IcarusLowerable source modules in the main worktree.**
   - Only `Completeness.lean` exists in `/Users/playra/t27/proofs/lean4/Trinity/IcarusLowerable/`.
   - The core modules (`Ast.lean`, `Predicate.lean`, `Emitter.lean`, `Soundness.lean`, etc.) are present in agent worktrees but not on `wave-loop-528`.
   - `lake build Trinity.IcarusLowerable.Soundness` fails immediately with missing files.

2. **No formal proof for the new W528 cross-boundary lowering.**
   - Module-level 2-D scalar-struct `const`/`var` and function param/return lowering is implemented in the Rust backend but not covered by the Lean 4 model.
   - `Completeness.lean` (248 theorems) cannot be rebuilt without the source modules.

3. **The `ExprCast` exporter gap.**
   - W528 scratch witnesses use `(field as u32)` casts, which the current Lean exporter maps to `.unsupportedIcarus`.
   - Formal witnesses must avoid casts or the exporter/model must be extended.

4. **Gate fragility.**
   - The `t27c verify` subcommand is not currently registered, so `./scripts/tri verify --lean-lowerable` needs to be checked/repaired.

---

## Scientific background

| Work | Relevance to W529 |
|------|-------------------|
| **Herklotz et al., *Formal Verification of High-Level Synthesis* (OOPSLA 2021)** | Vericert gives a CompCert-style forward-simulation proof for C→Verilog HLS. We reuse the same idea: a shallow t27 evaluator and a shallow Verilog evaluator, related by `module_value_equiv`. |
| **Leroy, *A formally verified compiler back-end* (JAR 2009)** | CompCert’s simulation diagrams justify our use of `module_value_equiv_proved` / `module_value_equiv_proved_sequential` to compose per-pass/whole-module correctness. |
| **Choi et al., *Kami* (ICFP 2017)** | Kami shows modular hardware verification in a proof assistant using LTS/refinement; our packed-vector module/var model is a much smaller instance of the same philosophy. |

Sources: [Vericert paper](https://johnwickerson.github.io/papers/vericert_oopsla21.pdf), [CompCert backend](https://xavierleroy.org/publi/compcert-backend.pdf), [Kami](http://plv.csail.mit.edu/kami/papers/icfp17.pdf).

---

## Decomposed implementation plan

### Phase 1 — Restore IcarusLowerable source modules
- Extract the 10 core `IcarusLowerable/*.lean` files from the last commit that touched them (`33276d818`, W524) into `/Users/playra/t27/proofs/lean4/Trinity/IcarusLowerable/`.
- Keep the existing `Completeness.lean` as a generated artifact to be regenerated later.
- Verify `lake build Trinity.IcarusLowerable.Soundness` reaches at least the compile phase.

### Phase 2 — Establish a clean baseline
- Run `lake build Trinity.IcarusLowerable.Soundness` and confirm it is green.
- `grep -n "sorry\|admit" proofs/lean4/Trinity/IcarusLowerable/*.lean` must return no code matches.
- Run `cargo test -p t27c --bin t27c` and `cargo test -p tri` to ensure the Rust side is still stable.

### Phase 3 — Add W529 witnesses in `Lemmas.lean`
Add four positive witnesses (module const, module var, function param, function return), each using a scalar struct with `u32` fields to avoid the `ExprCast` gap:

1. `w529_module_2d_const_env` / `w529_module_2d_const_module` — module `const grid : [2][3]Pt` read by two functions.
2. `w529_module_2d_var_env` / `w529_module_2d_var_module` — module `var grid : [2][3]Pt` read by two functions.
3. `w529_function_2d_param_env` / `w529_function_2d_param_module` — function taking `m : [2][3]Pt`.
4. `w529_function_2d_return_env` / `w529_function_2d_return_module` — function returning `[2][3]Pt`.

For each witness:
- Populate `Env.vars` with globals, parameters, and locals that are indexed/field-accessed.
- Use `.constDecl` / `.varDecl` in `globals`.
- Use `.array 2 (.array 3 (.struct "Pt"))` for the 2-D AOS type.
- Add a lowerability theorem proved by `native_decide`.

### Phase 4 — Add value-preservation theorems in `Soundness.lean`
For each witness function:
1. Prove `Module.isLowerable` by `native_decide`.
2. Prove `Module.hasUniqueFunctionNames` by `simp`.
3. Prove `Module.isCombinational` by `native_decide`.
4. Prove `Module.callContext` by `simp` + `native_decide`.
5. Prove `findFunction` equality and non-host-only.
6. Apply `module_value_equiv_proved` with concrete argument bit-vectors and check `evalModuleFunctionTotal = evalVModuleTotal`.

### Phase 5 — Regenerate `Completeness.lean`
- Use the Rust lowerability-completeness generator (reachable via `./scripts/tri verify --lean-lowerable` or the underlying `t27c` command) to regenerate `Completeness.lean` after the source modules are restored.
- Confirm the regenerated file imports the source modules and still proves 248 lowerability theorems.

### Phase 6 — Reseal and Rust-side checks
- If any exporter fix is needed to avoid `.unsupportedIcarus` for the new witnesses, apply it carefully (avoid changing primitive-array signatures).
- Reseal only specs whose generated output actually changes.
- Run `./scripts/tri test` and confirm 0 seal mismatches and the same 16 pre-existing yosys smoke baseline failures.

### Phase 7 — Closeout and W530 cooperation variants
- Write `docs/reports/WAVE_LOOP_529_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W530_*.md` with three variants.
- Update `.trinity/current-issue.md` to Wave Loop 530.
- Append learnings to `.trinity/experience.md` and update `.claude/skills/t27-wave-loop.md`.

### Phase 8 — Final verification gates
- `lake build Trinity.IcarusLowerable.Soundness`: green, zero `sorry`.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `./scripts/tri test`: 582/582 seal matches, 16 pre-existing yosys smoke baselines.

---

## Open decision: Variant A vs. B vs. C

The cooperation document recommends **Variant A** (formal soundness). If you prefer Variant B (Icarus simulation gate) or Variant C (harden packing for signed / array-field structs), this plan can be swapped before implementation begins. Variant A is selected here because it closes the soundness gap opened by W528 and is the highest-value next step for the verified compilation path.

---

*φ² + φ⁻² = 3 | TRINITY*
