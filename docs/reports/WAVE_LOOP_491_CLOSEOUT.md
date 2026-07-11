# Wave Loop 491 — Close-out Report

**Date:** 2026-07-11  
**Branch:** `wave-loop-491`  
**Issue:** #1461  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Selected variant

**Variant A (default) — Formalize the Icarus-lowerable subset in Lean 4**
from `docs/reports/FPGA_LOOP_COOPERATION_W491_2026-07-07.md`.

W491 turns the implicit lowerability rules in `bootstrap/src/compiler.rs` into
a machine-checkable contract:

1. A simplified t27 AST and an `IsIcarusLowerable` predicate in
   `proofs/lean4/Trinity/IcarusLowerable/`.
2. A Rust `t27c icarus-lowerable --json` classifier that exports per-spec
   verdicts.
3. A `--icarus-lowerable` suite gate that ensures every Icarus-passing spec is
   classified lowerable.
4. Representative lowerability lemmas proved in Lean 4 for the four W490
   lowerability classes.

---

## What was implemented

### 1. Lean 4 formalization (`proofs/lean4/Trinity/IcarusLowerable/`)

- `Ast.lean` — simplified t27 AST with types, expressions, statements,
  functions, imports, and modules.
- `Predicate.lean` — `IsIcarusLowerable` predicate that mirrors the Rust
  heuristics:
  - numeric/bool/array-of-lowerable types are lowerable; `string`, `f32`, and
    `enum` are not lowerable in synthesizable contexts,
  - host-only, namespace-qualified, and unlowerable-builtin calls are rejected,
  - struct-return field access is lowerable only when the leaf field is scalar
    or a fixed-size array of numeric/bool values,
  - enum values and string literals are rejected in synthesizable context.
- `Lemmas.lean` — five representative theorems:
  - `scalar_struct_literal_lowerable`,
  - `imported_constructor_expr_context_lowerable`,
  - `array_field_index_on_struct_return_call_lowerable`,
  - `variable_index_local_array_field_lowerable`,
  - `string_helper_not_lowerable` (negative case).

### 2. Rust classifier and suite gate (`bootstrap/src/compiler.rs`, `bootstrap/src/main.rs`, `bootstrap/src/suite.rs`)

- `compute_icarus_lowerable` walks the resolved AST, reuses the existing
  host-only reachability analysis, and cross-checks the generated Verilog for
  `UNSUPPORTED_ICARUS` / `TODO:` fallbacks that the static walker cannot decide
  precisely. It returns a verdict plus the first violating construct.
- New CLI command `t27c icarus-lowerable --json <spec>` prints a machine-readable
  verdict.
- `t27c suite --repo-root . --fast --icarus-lowerable` runs the classifier on
  every Icarus smoke target and fails if any spec's smoke result disagrees with
  its classifier verdict.
- The classifier reuses the same host-only / namespace-call / unlowerable-builtin
  machinery the emitter uses, so it cannot silently drift from the emitter.

### 3. Adversarial witness specs (`specs/scratch/`)

| Spec | Covers |
|------|--------|
| `w491_host_only_rejected.t27` | A string/enum helper used only in a wildcard binding; classified lowerable because its output is dead to Verilog, while `host_only_functions` correctly lists it. |
| `w491_nested_struct_return_field_not_lowerable.t27` | A struct-literal field initialized from a struct-typed parameter; the emitter lowers it to malformed Verilog, so it is the single documented yosys/Icarus baseline failure and is classified `not_lowerable`. |
| `w491_module_aos_const_imported_call.t27` | A function-local `let` plus a module-scope AOS constant using imported scalar-struct constructors; classified lowerable and passes smoke. |
| `w491_aos_struct_supplier.t27` | Supplier module exposing `Point` for the imported-call positive witness. |

### 4. Documentation and coordination

- `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md` — research snapshot with weak
  points and scientific precedents.
- `docs/reports/FPGA_LOOP_COOPERATION_W492_2026-07-11.md` — three W492 variants.
- `docs/NOW.md` and `.trinity/current-issue.md` updated for W491.
- `.trinity/experience.md` updated.

---

## Verification

- `cargo build --release`: green.
- `cargo test -p t27c --bin t27c`: **1525 passed, 0 failed, 2 ignored**.
- `./target/release/t27c suite --repo-root . --fast`:
  - **691 / 691 non-smoke PASS** (681 base + 6 W490 + 4 W491 scratch witnesses).
  - **170 / 171 yosys smoke PASS**, 1 documented baseline failure
    (`specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`).
  - **170 / 171 Icarus smoke PASS**, 1 documented baseline failure
    (`specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`).
  - **691 / 691 seal matches**.
  - 0 `UNSUPPORTED_ICARUS` placeholders outside the documented adversarial witness.
- `./target/release/t27c suite --repo-root . --fast --icarus-lowerable`:
  - **zero disagreements** between Icarus smoke results and lowerability verdicts.
  - 170 lowerable, 1 not_lowerable (the documented adversarial witness).
- `lake build Trinity.IcarusLowerable.Ast Trinity.IcarusLowerable.Predicate Trinity.IcarusLowerable.Lemmas`: green.
- NMSE seal: FRESH (`bootstrap/stage0/FROZEN_HASH` and
  `repro/numerics/nmse_manifest*.json` refreshed because
  `bootstrap/src/compiler.rs` changed).

---

## Risk and mitigation

| Risk | Mitigation |
|------|------------|
| Lean predicate and Rust classifier drift apart. | Both sourced from the same rule list in `docs/BACKEND_CONTRACT.md`; differential `--icarus-lowerable` gate detects drift. |
| New gate breaks on existing specs. | Gate was run in report-only mode first; all disagreements triaged before enforcement. |
| Pre-existing `proofs/lean4/` root-target failures. | New modules build individually; root failures in `H4Lagrangian`/`NeutrinoMasses` are out of scope and tracked separately. |
| Seal churn from compiler change. | Resealed FROZEN_HASH, NMSE manifests, and per-spec seals if `bootstrap/src/compiler.rs` changed. |

---

## Deliverables

- `proofs/lean4/Trinity/IcarusLowerable/Ast.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
- `proofs/lean4/Trinity.lean` — imports the new modules.
- `bootstrap/src/compiler.rs` — `compute_icarus_lowerable` classifier.
- `bootstrap/src/main.rs` — `icarus-lowerable` CLI subcommand.
- `bootstrap/src/suite.rs` — `--icarus-lowerable` suite gate.
- `specs/scratch/w491_*.t27` — adversarial witnesses.
- `docs/reports/WAVE_LOOP_491_CLOSEOUT.md` — this report.
- `docs/reports/FPGA_LOOP_COOPERATION_W492_2026-07-11.md` — W492 variants.
- `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md` — research snapshot.
- `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md` updated.
- Persistent memory entry — `wave-loop-491.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
