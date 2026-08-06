# Wave Loop 534 Closeout — Harden the Icarus lowerability boundary

**Issue:** #1505  
**Branch:** `wave-loop-534`  
**Closed:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What was delivered

### 1.1 Structural Icarus lowerability classifier

- Added `Compiler::is_icarus_lowerable` and `Compiler::icarus_lowerability_reason`
  in `bootstrap/src/compiler.rs`.
- The classifier walks the parsed t27 AST and rejects:
  - non-lowerable types (`f32`, `string`, enum, non-scalar structs, casts to them),
  - calls to unresolved / host-only / qualified functions,
  - iterator-style `for`,
  - unbounded `while (true)`,
  - `break` / `continue` outside loops.
- Fixed `Ok(false)` propagation in the recursive AST walk so rejections actually
  surface at the module level.

### 1.2 New CLI subcommand

```bash
t27c icarus-lowerable [--json] <file.t27>
```

Wired in `bootstrap/src/main.rs`.

### 1.3 Test-suite integration

- `bootstrap/src/suite.rs::is_icarus_lowerable` now uses the structural
  classifier as the authoritative gate.
- Generated Verilog is still compiled with `iverilog -g2012 -o /dev/null` as a
  backend sanity cross-check.
- Icarus simulation regression gate is green with 0 simulation failures and
  0 seal mismatches.

### 1.4 Adversarial negative witnesses

Created under `specs/scratch/`:

| Spec | Rejected construct |
|---|---|
| `w534_negative_cast_to_string.t27` | cast to `String` |
| `w534_negative_f32_field.t27` | scalar struct with an `f32` field |
| `w534_negative_host_only_helper.t27` | call to `host::print` |
| `w534_negative_nonlowerable_struct_assign.t27` | non-lowerable struct assignment |
| `w534_negative_unbounded_while.t27` | `while (true)` |
| `w534_negative_unresolved_import.t27` | unresolved imported function call |

All six are rejected by the structural classifier and sealed.

### 1.5 Rust integration test

`bootstrap/tests/icarus_lowerable.rs` shells out to `t27c icarus-lowerable`
via `CARGO_BIN_EXE_t27c` and asserts:
- every `w534_negative_*.t27` is rejected;
- three known-lowerable witnesses (`w532_signed_struct_array_field_2d_copy.t27`,
  `w533_module_scalar_struct_return.t27`, `w528_function_2d_struct_array_param.t27`)
  are accepted.

### 1.6 Documentation

- `docs/ICARUS_LOWERABLE_BOUNDARY.md` defines the lowerability contract, the
  structural rules, the CLI gate, and the relationship with the Lean 4 model.

### 1.7 FROZEN_HASH

Updated `bootstrap/stage0/FROZEN_HASH` after changing `bootstrap/src/compiler.rs`.

---

## 2. Validation gates

| Gate | Result |
|---|---|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 2 passed; 0 failed |
| `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` | Icarus Simulation: 35 passed, 0 failed; Seal Verify: 609 passed, 0 failed |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs green |

Yosys smoke gate still reports 24 pre-existing baseline failures in legacy
`w3xx` scratch specs. Those specs are outside the Icarus-lowerable subset and
were not touched in this wave.

---

## 3. Residual risks / next-wave seeds

- The Lean 4 `Predicate.lean` does not yet reject every construct that the Rust
  structural classifier rejects (e.g. `f32` struct fields and `while (true)`).
  A follow-up wave can tighten the Lean predicate and add matching
  `¬ Module.isLowerable` theorems.
- Cross-check between the Rust classifier and Lean predicate is currently manual.
  A future wave could generate the simplified Lean AST from the Rust parser and
  run the classifier equality as a single end-to-end test.

---

*φ² + φ⁻² = 3 | TRINITY*
