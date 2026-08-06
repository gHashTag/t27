# Wave Loop 535 Closeout — Align the Lean 4 lowerability predicate with the Rust structural classifier

**Issue:** #1506  
**Branch:** `wave-loop-535`  
**Closed:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What was delivered

### 1.1 Tightened Lean 4 lowerability predicate

Updated `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` to close three
known divergence points between the Rust structural classifier and the Lean
model:

- **Recursive type lowerability for structs.**  Added fuel-threaded
  `Ty.isLowerableFuel` that rejects a struct type when any declared field is
  non-lowerable (`f32`, `string`, enum, nested struct, etc.).  Undefined struct
  names are treated leniently so the simplified corpus model in
  `Completeness.lean` stays valid.
- **Rejection of unbounded `while (true)`.**  `Stmt.isLowerableFuel` now returns
  `false` for `whileLoop (.boolLit true) body`, matching the Rust classifier.
- **Rejection of imported-function calls.**  `Expr.isLowerableFuel` now rejects
  calls to any name that appears in the environment's `imports` list, because
  the Icarus backend cannot resolve cross-module imports in synthesizable code.

Call sites in `Expr.isLowerableFuel` (`.arrayLit`) and `Stmt.isLowerableFuel`
(`.varDecl`, `.constDecl`) were updated to use the new `Ty.isLowerable` wrapper.

### 1.2 Negative theorems for W534 adversarial witnesses

Added six `¬ Module.isLowerable` theorems in
`proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`, one for each W534 scratch
witness:

| Theorem | Witness construct |
|---|---|
| `w535_cast_to_string_not_lowerable` | cast to `String` |
| `w535_f32_field_not_lowerable` | scalar struct with an `f32` field |
| `w535_host_only_helper_not_lowerable` | call to a host-only helper |
| `w535_nonlowerable_struct_assign_not_lowerable` | non-lowerable struct type with `String` field |
| `w535_unbounded_while_not_lowerable` | `while (true)` |
| `w535_unresolved_import_not_lowerable` | call to an imported function |

All six are proved by `native_decide`.

### 1.3 Positive bounded-while corpus witness

- Created `specs/igla/w535_bounded_while_module.t27` with a bounded
  `while (i < n)` loop and three `assert_eq` tests.
- Added the corresponding environment, module, and positive lowerability theorem
  `igla_w535_bounded_while_module_lowerable` to
  `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean`.
- Sealed the new corpus spec in
  `.trinity/seals/igla_w535_bounded_while_module.json`.

### 1.4 Removed obsolete positive theorem

The imported-constructor expression-context witness was no longer lowerable
after the import-rejection rule was added.  Removed `imported_ctor_sound` from
`proofs/lean4/Trinity/IcarusLowerable/Soundness.lean` and the supporting
`importedCtorEnv` / `importedCtorModule` definitions from `Lemmas.lean`.

### 1.5 Documentation

- Updated `docs/ICARUS_LOWERABLE_BOUNDARY.md` to document the W535 predicate
  tightening, the six matching negative theorems, and the positive bounded-while
  corpus witness.

---

## 2. Validation gates

| Gate | Result |
|---|---|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 2 passed; 0 failed |
| `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` | Icarus Simulation: 35 passed, 0 failed; Seal Verify: 610 passed, 0 failed |
| `lake build Trinity.IcarusLowerable.Lemmas` | 4 jobs green |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs green |
| `lake build Trinity.IcarusLowerable.Completeness` | 8573 jobs green |

Yosys smoke gate still reports 24 pre-existing baseline failures in legacy
`w3xx` scratch specs. Those specs are outside the Icarus-lowerable subset and
were not touched in this wave.

`cargo test -p t27c --tests` shows one pre-existing failure in
`bitnet_pipeline::sequencer_idle_arms_on_start`, unrelated to the Icarus
lowerability work.

---

## 3. Residual risks / next-wave seeds

- The simplified Lean corpus model treats undefined struct names as lowerable.
  A future wave could close this leniency by generating struct declarations for
  every name referenced in `Completeness.lean`.
- Rust/Lean equivalence is still checked manually per witness.  A future wave
  could automate the extraction of the simplified Lean AST from the Rust parser
  and run a single classifier-equality regression test.

---

*φ² + φ⁻² = 3 | TRINITY*
