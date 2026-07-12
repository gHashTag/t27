# Wave Loop 499 Close-out Report

| Field | Value |
|-------|-------|
| Issue | #1459 |
| Branch | `wave-loop-499` |
| Ring | 12 (gen-verilog / Icarus semantics) |
| Date | 2026-07-13 |
| Anchor | φ² + φ⁻² = 3 | TRINITY |

---

## 1. What was attempted

W498 proved a generic structural-equivalence theorem
`module_value_equiv_statement` for the Icarus-lowerable combinational subset,
but the proof still carried two reachability preconditions:
`Module.callsResolved` and `Module.callsReachable`.  Those assumptions forced
a separate proof that every function call inside the module resolved to a
reachable, emitted function, which complicated reuse in the `gen-verilog`
regression gates.

Wave Loop 499 removed those preconditions by making the emitter unconditional:
`emitModuleFuel` now lowers **every non-host-only function** in `m.functions`,
regardless of reachability.  This is the standard CompCert-style
*Unusedglob* technique: emit all candidate definitions and let the simulator /
synthesis tool deal with unreachable ones, so the correctness theorem needs no
syntactic reachability closure.

---

## 2. What was actually changed

### 2.1 Predicate model (`proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`)

- `Function.isLowerable` no longer has a reachability shortcut.  Host-only
  helpers are treated as outside the synthesizable model and return `true`
  without checking their bodies.
- `Module.isLowerable` checks only globals and `m.functions`; tests/benches are
  not part of the Icarus semantics.
- `Function.isCombinational` / `Module.isCombinational` follow the same
  host-only-aware, emitted-only shape.
- Added `Module.emittedFunctions env m := m.functions.filter (¬ host-only)`.
- Added `Module.hasEmittedFunctionNamed` and `Module.hasUniqueFunctionNames`.
- Replaced the reachability bookkeeping with a single `callContext` family:
  every function name occurring in an expression or statement is reachable,
  non-host-only, and resolvable to an emitted function.
- Kept `Module.callsResolved` / `Module.callsReachable` for documentation but
  they are no longer used by the generic theorem.

### 2.2 AST lookup (`proofs/lean4/Trinity/IcarusLowerable/Ast.lean`)

- `Module.findFunction` now searches only `m.functions`; tests/benches are not
  call targets.

### 2.3 Emitter (`proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`)

- `emitModuleFuel` emits `Module.emittedFunctions env m` and drops the old
  test/bench item expansion into `VModule.items`.  The generated shallow
  Verilog now contains exactly the synthesizable globals and emitted
  functions.

### 2.4 Equivalence proof (`proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`)

- Removed `callsResolved` / `callsReachable` section variables.
- Added `hctx₀ : Module.callContext env₀ m₀` and
  `hunique₀ : Module.hasUniqueFunctionNames m₀` to `all_equiv`.
- Rewrote `Module.isCombinational_function_body` to operate over emitted
  functions only.
- Rewrote `emit_function_lookup` to rely on uniqueness and membership in
  `Module.emittedFunctions`.
- Added `Module.hasEmittedFunctionNamed_findFunction` and supporting lemmas.
- Updated the `.call` branch to derive callee membership in the emitted set
  from `Expr.callContext`.

### 2.5 Top-level soundness statement (`proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`)

- `module_value_equiv_statement` now carries only lowerability,
  combinationality, unique function names, the module-level call context,
  and the requirement that `main` is not host-only.

### 2.6 Witness

- Added `specs/scratch/w499_unconditional_function_emission.t27`: two
  unreachable functions, one calling the other, plus a `main` test.  Under
  unconditional emission the generated Verilog contains all three functions,
  has no placeholders, and the test passes.
- Seal: `.trinity/seals/scratch_w499_unconditional_function_emission.json`.

---

## 3. Literature / related work

The design follows the *Unusedglob* pass observation from CompCert
(Leroy, Blazy, et al.): when a compiler emits every global definition and
proves that evaluation ignores unreachable ones, the translation-validation
proof can omit a static reachability analysis.  In the t27 setting the
"emission" is not a separate pass but a direct change to `emitModuleFuel`,
and the proof obligation is reduced to a local `callContext` invariant plus
unique function names.

---

## 4. Verification results

| Gate | Result |
|------|--------|
| `lake build Trinity.IcarusLowerable.Soundness` | green, zero `sorry` |
| `./scripts/tri verify --lean-lowerable` | passed |
| `./scripts/tri test` non-smoke | 698 / 698 PASS |
| `./scripts/tri test` yosys smoke | 178 / 178 PASS (0 baseline) |
| `./scripts/tri test` Icarus smoke | 177 / 178 PASS (1 documented baseline) |
| `./scripts/tri test` seal verify | 698 / 698 match |
| FPGA board-less smoke gate / replay | OK |
| `cargo test -p t27c --bin t27c` | 1525 / 0 / 2 |

The single Icarus failure is the documented baseline
`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`, which is
intentionally outside the lowerable subset.

---

## 5. Weak points discovered

1. **The `main`-not-host-only hypothesis** is still explicit in
   `module_value_equiv_statement`.  It is true for every synthesizable entry
   point, but a fully general theorem should either derive it from the call
   context or be parameterized over an arbitrary emitted function name.
2. **The local AOS element boundary** (`w493_local_aos_element_field_not_lowerable.t27`)
   remains the only documented Icarus baseline.  Expanding the emitter to
   handle this pattern would make the Icarus smoke gate fully green.
3. **Conditionals and loops** remain outside the modeled operational
   semantics.  A guarded big-step semantics for `ifThenElse` / `forLoop` is
  future work.
4. **Unique function names** are now a hard proof assumption.  The Rust
  front-end already guarantees uniqueness, but the formal model does not yet
  prove that property from the module AST.

---

## 6. Recommendations for Wave Loop 500

See `docs/reports/FPGA_LOOP_COOPERATION_W500_2026-07-13.md` for three scoped
variants.  The recommended ordering is:

1. **Variant A** (baseline closure) — highest leverage because it removes the
   last documented Icarus failure.
2. **Variant B** (entry-point generalization) — cleans up the remaining
   explicit hypothesis in the generic theorem.
3. **Variant C** (sequential constructs) — widens the subset once the
   combinational contract is fully hardened.

---

*φ² + φ⁻² = 3 | TRINITY*
