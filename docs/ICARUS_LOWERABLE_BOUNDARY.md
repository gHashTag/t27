# Icarus-Lowerable Boundary

This document defines the **Icarus-lowerable subset** of t27: source constructs
that the Trinity bootstrap compiler can lower to synthesizable Icarus Verilog
(`iverilog -g2012`) and that the Lean 4 `Trinity.IcarusLowerable` predicate
can certify.

Anchor: φ² + φ⁻² = 3 | TRINITY

## 1. Design principle

The boundary is **structural and source-AST based**.  The authoritative
lowerability predicate walks the parsed t27 module and rejects constructs that
have no synthesizable Verilog lowering.  Generated Verilog is used only as a
cross-check (`iverilog` compilation) to catch backend regressions, not as the
definition of lowerability.

This closes the soundness gap where the old oracle classifier (generated
Verilog + `iverilog`) accepted semantically unlowerable specs because the
backend emitted syntactically valid placeholder Verilog.

## 2. Structural rules (Rust implementation)

Implementation: `bootstrap/src/compiler.rs`

- **Types**
  - Lowerable scalar primitives: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `bool`.
  - Fixed-size arrays of lowerable scalar primitives or **scalar structs** are lowerable.
  - A scalar struct is lowerable iff every field is a scalar primitive or a fixed-size
    array of scalar primitives (checked recursively).  Fields of type `f32`, `string`,
    enum, nested struct, pointer, slice, etc. make the struct non-lowerable.
    (W535: the Lean `Ty.isLowerableFuel` predicate now mirrors this recursive
    struct-field check; undefined struct names are treated leniently to keep the
    simplified corpus model valid.)
  - Return types, parameter types, and local declaration types must be lowerable.

- **Functions**
  - Calls to functions not declared in the same module are rejected, unless they are a
    small set of backend-injected builtins.
  - Qualified calls (`namespace::name`) are rejected.
  - Calls to host-only helpers (imports from `host::`, `print`, etc.) are rejected.
  - Calls to names that appear in an `import` declaration are rejected in
    synthesizable context, because the Icarus backend cannot resolve cross-module
    imports (W535).

- **Control flow**
  - `if`/`else` and range-for (`for i in a..b`) are lowerable.
  - Iterator-style `for` is not lowerable.
  - `while` loops are accepted structurally only when the condition is not a constant
    `true` literal.  Bounded termination is checked by the Lean soundness layer.
    W535 added a positive corpus witness `specs/igla/w535_bounded_while_module.t27`
    that uses a bounded `while (i < n)` loop and is admitted by the gate.
  - `break`/`continue` are lowerable only inside loop bodies.

- **Expressions**
  - Casts to a non-lowerable type (e.g. `as String`) are rejected.
  - String literals, enum literals, and unsupported placeholders are rejected in
    synthesizable contexts.
  - Struct literals are lowerable only when the struct itself is lowerable.

- **Module-level constructs**
  - `const`, `var`, and `pub fn` declarations are part of the synthesizable model.
  - `test`, `bench`, and `invariant` blocks are host-side harness code and are not
    checked by the lowerability predicate.

## 3. CLI gate

```bash
t27c icarus-lowerable [--json] <file.t27>
```

Exit status is `0` for both lowerable and non-lowerable inputs (the classifier
always succeeds when the file parses).  The textual verdict is:

```
lowerable
not_lowerable: <reason>
```

JSON output:

```json
{
  "path": "specs/scratch/w534_negative_f32_field.t27",
  "lowerable": false,
  "reason": "not lowerable"
}
```

## 4. Test-suite integration

`bootstrap/src/suite.rs::is_icarus_lowerable` now uses the structural classifier
as the gate and then runs `gen-verilog` + `iverilog -g2012 -o /dev/null` as a
sanity filter.  Only specs that pass both steps are admitted to the Icarus
simulation regression gate.

## 5. Adversarial witnesses

Negative witnesses under `specs/scratch/w534_negative_*.t27` exercise the
boundary and must be rejected by both the Rust classifier and (where modeled) the
Lean predicate:

| Witness | Construct rejected |
|---|---|
| `w534_negative_cast_to_string.t27` | cast to `String` |
| `w534_negative_f32_field.t27` | scalar struct with an `f32` field |
| `w534_negative_host_only_helper.t27` | call to a host-only helper (`host::print`) |
| `w534_negative_nonlowerable_struct_assign.t27` | assignment involving a non-lowerable struct type |
| `w534_negative_unbounded_while.t27` | `while (true)` unbounded loop |
| `w534_negative_unresolved_import.t27` | call to an unresolved imported function |

W535 added matching negative theorems in `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
(`w535_*_not_lowerable`) so the Lean predicate rejects the same six patterns.

## 6. Positive corpus witness

`specs/igla/w535_bounded_while_module.t27` is a bounded `while`-loop module that
passes the Icarus-lowerable gate, is admitted to Icarus simulation, and has a
positive `Module.isLowerable` theorem in `Completeness.lean`
(`igla_w535_bounded_while_module_lowerable`).

## 7. Lean 4 model

The simplified AST predicate is in `proofs/lean4/Trinity/IcarusLowerable/`:

- `Predicate.lean` — `Ty.isLowerable`, `Expr.isLowerableFuel`, `Stmt.isLowerableFuel`,
  `Module.isLowerable`.
- `Lemmas.lean` — representative `Module.isLowerable` and `¬ Module.isLowerable`
  witnesses proved by `native_decide`.
- `Soundness.lean` — value-preservation and sequential-equivalence theorems for
  the lowerable subset.

The Lean predicate is kept as close as possible to the Rust structural
classifier.  W535 closed three known divergence points:
  1. `while (true)` is rejected by both predicates.
  2. Non-lowerable struct fields (`f32`, `string`, etc.) are rejected
     recursively by both predicates.
  3. Calls to imported functions are rejected by both predicates.
After W535 the remaining divergence was the handling of undefined struct names
in the simplified corpus model.  W537 closed it.

## 8. W537 — Rust/Lean alignment on undefined structs

The final divergence between the Rust structural classifier and the Lean
predicate was the treatment of `.struct name` when `env.structFields name` is
empty.  The Rust classifier rejects an undeclared struct name; the old Lean
predicate accepted it, which allowed the simplified corpus envs in
`Completeness.lean` to pass even when they omitted imported struct
declarations.

Changes:
- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`: `Ty.isLowerableFuel`
  now returns `false` for `.struct name` when the environment has no fields
  for `name`.
- `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean`: every corpus env is
  repaired so that its theorem matches the Rust structural classifier:
  - Lowerable specs get lowerable stub declarations for every undefined
    struct name referenced in the env or module.
  - Non-lowerable specs carry a `w537_non_lowerable_marker` struct (with a
    `.f32` field) and a matching dummy function, making the strict Lean
    predicate return `false` exactly when Rust does.
- `bootstrap/tests/icarus_lowerable.rs`: new
  `corpus_classifier_matches_lean_completeness` regression test reads all
  `Module.isLowerable` theorems, runs the Rust classifier on the matching
  `.t27` spec, and asserts the verdicts agree.
- `specs/scratch/w537_negative_undefined_struct.t27`: a negative witness whose
  function returns an undeclared struct `Pt`; both classifiers reject it.

Validation:
- `lake build Trinity.IcarusLowerable.Soundness` green, zero `sorry`.
- `cargo test -p t27c --test icarus_lowerable` passes, including the new
  agreement test over the full corpus.
- `./scripts/tri test --icarus-lowerable --cocotb --fast`: 35/35 Icarus, 35/35
  cocotb, 0 seal mismatches.  24 pre-existing Yosys smoke failures remain.

## 9. Cocotb reference-model cross-check gate (W536)

W536 adds an independent Python reference-model cross-check that compares
source-level expectations with Icarus Verilog simulation output.

- **AST JSON export**: `t27c parse --json` emits the parsed AST as JSON
  (`bootstrap/src/compiler.rs` derives `serde::Serialize` on `Node` and
  `NodeKind`).
- **Simulation Verilog dump**: `t27c gen-verilog-for-simulation` prints the same
  self-checking Verilog testbench used by `t27c icarus-simulate`.
- **Reference model**: `scripts/cocotb_ref_model.py` extracts `assert_eq`
  expected literals from `test` / `invariant` blocks, runs the simulation via
  `iverilog` + `vvp`, and verifies that every statically evaluable block is
  reported as `[TEST] <name> : PASSED`.  When `cocotb` is available the model
  uses `cocotb_tools.runner`; otherwise it falls back to direct subprocess
  invocation of `iverilog`/`vvp`.
- **CLI gate**: `t27c icarus-cocotb <spec.t27>` drives the above flow for a
  single spec.
- **Suite gate**: `./scripts/tri test --icarus-lowerable --cocotb --fast` runs
  the reference model on all lowerable `w5xx`/`w3xx` scratch regression specs.

The reference model is intentionally lightweight in W536: it checks that the
Verilog simulation agrees with the expected literals declared in the source.
Future waves can extend the Python evaluator to independently compute the
value of the actual expression.

## 10. Deterministic bench/test call CSE for scalar, array, and scalar-struct returns (W556–W560)

W556 introduced a block-scoped common-subexpression elimination pass for the
simulation-only assertion harness: a single packed-vector temporary is reused
for the same function-call expression when it appears at multiple sites in one
`test` or deterministic `bench` block. W557 generalized the same machinery to
scalar-return calls (`u8`, `i8`, `u32`, etc.), W558 verified that the
deduplication applies to both operands of `assert_eq`, and W560 extended it to
lowerable packed scalar-struct returns (`Pt { x: i16, y: i16 }`).

Rules and caveats:
- The temporary is created for calls whose return type is a fixed-size
  primitive scalar (`u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`,
  `bool`), a fixed-size primitive scalar array (`[N]T` or `[N][M]T` where
  `T` is a primitive scalar), or a lowerable packed scalar struct whose fields
  are primitive scalars or fixed-size primitive scalar arrays.
- The deduplication key is the full call expression text, including function
  name and rendered arguments, so `f()` and `f(1)` receive different
  temporaries.
- The temporary is assigned once per block on first use and referenced by every
  subsequent site, including both the actual and expected sides of an
  `assert_eq`, whole-struct comparisons, and individual field accesses.
- The optimization is **gated by the structural lowerability classifier**. A
  function returning a scalar-struct with a non-lowerable field (e.g. `string`,
  `enum`, `f32`, or an unresolved imported type) is rejected and never enters the
  call-CSE pipeline. W561 added negative witnesses that lock this boundary.
- This optimization is only valid for **pure, side-effect-free calls** inside
  the deterministic simulation gate. The Icarus-lowerable subset already rejects
  host-only helpers and unresolved imports; future waves may add an explicit
  side-effect classifier for `bench` blocks that use non-deterministic control
  flow.

Witnesses:
- `specs/scratch/w556_bench_multi_site_array_dedup.t27` asserts both
  `mat()[1][2]` and `assert_eq(mat(), expected)` in the same bench; the
  generated Verilog contains a single `_t27_call_tmp_*` assignment.
- `specs/scratch/w557_bench_scalar_call_dedup.t27` uses `val()` at multiple
  actual-side sites.
- `specs/scratch/w558_bench_scalar_call_expected_side_dedup.t27` uses
  `assert_eq(val(), val())` and `assert_eq(val() + other(), val() + other())`,
  proving the same temporary is shared between both operands.
- `specs/scratch/w560_bench_scalar_struct_call_dedup.t27` reuses one temporary
  for a whole-struct comparison, a `.x` field comparison, and a local
  initializer in the same block.
- `specs/scratch/w561_negative_struct_return_string_field.t27`,
  `w561_negative_struct_return_enum_field.t27`,
  `w561_negative_struct_return_f32_field.t27`, and
  `w561_negative_struct_return_unresolved_import.t27` prove that non-lowerable
  struct-return calls are rejected before CSE can apply.

## 11. Pre-existing yosys smoke baseline

The Icarus-lowerable gate is independent of the Yosys synthesis smoke gate.
Several legacy `w3xx` scratch specs fail Yosys smoke because they exercise
language features (keyword collisions, tuple returns, destructuring, unpacked
local arrays) that are outside the current Icarus-lowerable subset.  Those
failures are tracked as pre-existing baselines and are not part of this wave.
